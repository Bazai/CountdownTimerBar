#!/usr/bin/env bash
# Zips the bundle from `make build` as releases/CountdownTimerBar-v<version>.zip,
# ready to attach to the GitHub release.
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
bundle_path="$project_dir/releases/CountdownTimerBar.app"
[[ -d "$bundle_path" ]] || { printf 'dist: build the bundle first (make build)\n' >&2; exit 1; }

version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$project_dir/Cargo.toml" | head -n 1)"
archive_path="$project_dir/releases/CountdownTimerBar-v$version.zip"
rm -f "$archive_path"
ditto -c -k --sequesterRsrc --keepParent "$bundle_path" "$archive_path"
printf 'dist: %s\n' "$archive_path"
