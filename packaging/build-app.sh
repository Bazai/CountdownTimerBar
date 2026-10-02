#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
out_dir="$project_dir/releases"
bundle_path="$out_dir/CountdownTimerBar.app"
contents_path="$bundle_path/Contents"
binary_path="$project_dir/target/release/CountdownTimerBar"
info_plist="$project_dir/packaging/Info.plist"
icon_path="$project_dir/packaging/AppIcon.icns"

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        printf 'Missing required file: %s\n' "$path" >&2
        exit 1
    fi
}

require_file "$info_plist"
require_file "$icon_path"

rustc_path="$(mise exec rust@1.98.1 -- sh -c 'exec "$CARGO_HOME/bin/rustup" which rustc')"
cargo_path="$(mise exec rust@1.98.1 -- sh -c 'exec "$CARGO_HOME/bin/rustup" which cargo')"
RUSTC="$rustc_path" "$cargo_path" build --release --manifest-path "$project_dir/Cargo.toml"
require_file "$binary_path"

rm -rf "$bundle_path"
mkdir -p "$out_dir"
mkdir -p "$contents_path/MacOS" "$contents_path/Resources"
install -m 755 "$binary_path" "$contents_path/MacOS/CountdownTimerBar"
install -m 644 "$info_plist" "$contents_path/Info.plist"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$project_dir/Cargo.toml" | head -n 1)"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $version" "$contents_path/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $version" "$contents_path/Info.plist"
install -m 644 "$icon_path" "$contents_path/Resources/AppIcon.icns"

# cargo's release stripping needs a toolchain `rust-objcopy` that fails to load
# here, so the bundled binary is stripped directly. Must precede codesign.
strip "$contents_path/MacOS/CountdownTimerBar"

if [[ -n "${APPLE_SIGNING_IDENTITY:-}" && -n "${APPLE_TEAM_ID:-}" && -n "${APPLE_NOTARY_KEYCHAIN_PROFILE:-}" ]]; then
    codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$bundle_path"

    archive_path="$(mktemp "${TMPDIR:-/tmp}/CountdownTimerBar.XXXXXX.zip")"
    trap 'rm -f "$archive_path"' EXIT
    ditto -c -k --sequesterRsrc --keepParent "$bundle_path" "$archive_path"
    xcrun notarytool submit "$archive_path" \
        --keychain-profile "$APPLE_NOTARY_KEYCHAIN_PROFILE" \
        --team-id "$APPLE_TEAM_ID" \
        --wait
    xcrun stapler staple "$bundle_path"
    codesign --verify --deep --strict --verbose=2 "$bundle_path"
    xcrun stapler validate "$bundle_path"
    printf 'Built, signed, notarized, and stapled: %s\n' "$bundle_path"
else
    printf '%s\n' "Built unsigned bundle: $bundle_path"
    printf '%s\n' 'Signing and notarization require APPLE_SIGNING_IDENTITY, APPLE_TEAM_ID, and APPLE_NOTARY_KEYCHAIN_PROFILE.'
fi
