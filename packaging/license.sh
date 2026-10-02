#!/usr/bin/env bash
# Sets the years in the LICENSE notice to "<first year>-<current year>" (or just
# the year when they are equal). The first year and the holder are kept.
# The About card reads the same line, so a rebuild picks the change up.
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
license="$project_dir/LICENSE"
current="$(date +%Y)"

line="$(grep -m1 -E '^Copyright \(c\) [0-9]{4}(-[0-9]{4})? ' "$license")" \
    || { printf 'license: no "Copyright (c) <year> <holder>" line in LICENSE\n' >&2; exit 1; }
first="$(printf '%s' "$line" | sed -E 's/^Copyright \(c\) ([0-9]{4}).*/\1/')"
holder="$(printf '%s' "$line" | sed -E 's/^Copyright \(c\) [0-9]{4}(-[0-9]{4})? //')"

if [[ "$first" == "$current" ]]; then
    years="$current"
else
    years="$first-$current"
fi

updated="Copyright (c) $years $holder"
if [[ "$line" == "$updated" ]]; then
    printf 'license: already current: %s\n' "$updated"
    exit 0
fi

tmp="$(mktemp)"
sed -E "s|^Copyright \(c\) [0-9]{4}(-[0-9]{4})? .*|$updated|" "$license" > "$tmp"
cat "$tmp" > "$license"
rm "$tmp"
printf 'license: %s\n' "$updated"
