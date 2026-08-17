#!/usr/bin/env bash
#
# Build PaperStack and install it into /Applications.
#
#   scripts/install-local.sh              # this Mac's architecture, fastest
#   scripts/install-local.sh --universal  # x86_64 + arm64, as shipped
#   scripts/install-local.sh --no-launch  # install but don't reopen it
#
# Only the .app is built — the DMG is skipped, which saves the whole hdiutil and
# Finder-scripting round trip. Use `npm run tauri build` when you want something
# to hand out; that path also needs scripts/finalize-dmg.py afterwards.
#
# Locally built apps are never quarantined, so nothing here trips Gatekeeper.
# That is only a problem for a copy that has been downloaded from the web.

set -euo pipefail

cd "$(dirname "$0")/.."

universal=false
launch=true
for arg in "$@"; do
  case "$arg" in
    --universal) universal=true ;;
    --no-launch) launch=false ;;
    -h|--help) sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

echo "==> Building"
# Spelled out per branch rather than assembling an args array: macOS ships bash
# 3.2, where expanding an empty array under `set -u` is an unbound-variable error.
if $universal; then
  built="src-tauri/target/universal-apple-darwin/release/bundle/macos/PaperStack.app"
  npm run tauri build -- --target universal-apple-darwin --bundles app
else
  built="src-tauri/target/release/bundle/macos/PaperStack.app"
  npm run tauri build -- --bundles app
fi

[ -d "$built" ] || { echo "expected an app bundle at $built" >&2; exit 1; }

# Quit a running copy first: replacing the bundle underneath a live process
# leaves it running from a directory that no longer exists, and its next window
# save writes to nowhere. Ask politely, then insist.
if pgrep -x paperstack >/dev/null 2>&1; then
  echo "==> Quitting the running copy"
  osascript -e 'tell application "PaperStack" to quit' >/dev/null 2>&1 || true
  for _ in $(seq 1 20); do
    pgrep -x paperstack >/dev/null 2>&1 || break
    sleep 0.5
  done
  pgrep -x paperstack >/dev/null 2>&1 && pkill -x paperstack || true
fi

destination="/Applications"
[ -w "$destination" ] || destination="$HOME/Applications"
mkdir -p "$destination"
installed="$destination/PaperStack.app"

echo "==> Installing to $installed"
rm -rf "$installed"
# ditto, not cp: it keeps the bundle's symlinks, resource forks and the ad-hoc
# signature intact, and cp -R can quietly break a code signature.
ditto "$built" "$installed"

arches=$(lipo -archs "$installed/Contents/MacOS/paperstack" 2>/dev/null || echo "unknown")
version=$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" \
  "$installed/Contents/Info.plist" 2>/dev/null || echo "?")
echo "==> Installed PaperStack $version ($arches)"

if $launch; then
  echo "==> Launching"
  open "$installed"
fi
