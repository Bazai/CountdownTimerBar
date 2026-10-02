#!/usr/bin/env bash
# Checks that the version in Cargo.toml is ready to release. It publishes
# nothing: pushing the bump to main lets .github/workflows/release.yml tag and
# publish the version once CI passes.
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
[[ -z "$(git status --porcelain)" ]] || fail "the working tree has uncommitted changes"

year="$(date +%Y)"
grep -q -E "^Copyright \(c\) ([0-9]{4}-)?$year " LICENSE || fail "the LICENSE years are not current; run make license and commit"

git fetch --tags --quiet origin
if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
    fail "tag $tag already exists; bump the version in Cargo.toml"
fi

make check

printf 'release: %s is ready; push it to main and CI will publish it\n' "$tag"
