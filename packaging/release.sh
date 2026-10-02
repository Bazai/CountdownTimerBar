#!/usr/bin/env bash
# Releases the version in Cargo.toml: checks, tags `v<version>` and pushes the
# current branch and the tag to origin. DRY_RUN=1 stops before tagging.
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$project_dir"

fail() {
    printf 'release: %s\n' "$1" >&2
    exit 1
}

version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
[[ -n "$version" ]] || fail "no version found in Cargo.toml"
tag="v$version"
branch="$(git rev-parse --abbrev-ref HEAD)"
[[ "$branch" != "HEAD" ]] || fail "detached HEAD; check out a branch"
[[ -z "$(git status --porcelain)" ]] || fail "the working tree has uncommitted changes"

year="$(date +%Y)"
grep -q -E "^Copyright \(c\) ([0-9]{4}-)?$year " LICENSE || fail "the LICENSE years are not current; run make license and commit"

git fetch --tags --quiet origin
if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
    fail "tag $tag already exists; bump the version in Cargo.toml"
fi

make check

if [[ -n "${DRY_RUN:-}" ]]; then
    printf 'release: dry run; would tag %s on %s and push %s and %s to origin\n' "$tag" "$(git rev-parse --short HEAD)" "$branch" "$tag"
    exit 0
fi

git tag -a "$tag" -m "CountdownTimerBar $version"
git push origin "$branch" "$tag"
printf 'release: pushed %s\n' "$tag"
