#!/usr/bin/env bash
# Bumps the version in Cargo.toml and Cargo.lock and commits it.
#   packaging/bump.sh            next patch (0.0.2 -> 0.0.3)
#   packaging/bump.sh minor      next minor (0.0.2 -> 0.1.0)
#   packaging/bump.sh major      next major (0.0.2 -> 1.0.0)
#   packaging/bump.sh 0.4.0      an explicit version
# DRY_RUN=1 prints the plan and changes nothing.
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$project_dir"

fail() {
    printf 'bump: %s\n' "$1" >&2
    exit 1
}

current="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
[[ "$current" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]] || fail "the version in Cargo.toml is not X.Y.Z: $current"
major="${BASH_REMATCH[1]}"
minor="${BASH_REMATCH[2]}"
patch="${BASH_REMATCH[3]}"

case "${1:-patch}" in
    patch) next="$major.$minor.$((patch + 1))" ;;
    minor) next="$major.$((minor + 1)).0" ;;
    major) next="$((major + 1)).0.0" ;;
    *)
        [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "expected patch, minor, major or X.Y.Z, got: $1"
        next="$1"
        ;;
esac

newest="$(printf '%s\n%s\n' "$current" "$next" | sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1)"
[[ "$next" != "$current" && "$newest" == "$next" ]] || fail "$next is not newer than $current"

printf 'bump: %s -> %s\n' "$current" "$next"
if [[ -n "${DRY_RUN:-}" ]]; then
    printf 'bump: dry run, nothing changed\n'
    exit 0
fi

[[ -z "$(git status --porcelain)" ]] || fail "the working tree has uncommitted changes"

tmp="$(mktemp)"
awk -v next_version="$next" '!done && /^version = "/ { print "version = \"" next_version "\""; done = 1; next } { print }' Cargo.toml > "$tmp"
cat "$tmp" > Cargo.toml
rm "$tmp"
cargo metadata --offline --format-version 1 > /dev/null

changed="$(git diff --numstat | awk '{ added += $1; removed += $2 } END { print added "/" removed }')"
[[ "$changed" == "2/2" ]] || fail "expected one changed line in Cargo.toml and one in Cargo.lock, got +/- $changed"

git add Cargo.toml Cargo.lock
git commit --quiet -m "chore(release): Bump the version to $next"
printf 'bump: committed %s\n' "$next"
printf 'next: make release   (checks the commit), then git push\n'
