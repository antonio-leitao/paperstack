#!/usr/bin/env bash
#
# Cut a release: bump the version, build a universal DMG, tag it, and publish it
# to GitHub Releases.
#
#   scripts/release.sh 0.2.0
#   scripts/release.sh 0.2.0 --dry-run    # build and check, change nothing
#
# Everything up to the tag is local and reversible. Nothing is pushed and no
# release is created until you confirm at the prompt.
#
# The website needs no attention here. docs/index.html links to
# /releases/latest and fills in the version, size and direct asset URL from the
# GitHub API at load time, so it is correct the moment this script finishes and
# never has to be edited on release day.

set -euo pipefail

cd "$(dirname "$0")/.."

version="${1:-}"
dry_run=false
for arg in "${@:2}"; do
  case "$arg" in
    --dry-run) dry_run=true ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: scripts/release.sh <major.minor.patch> [--dry-run]" >&2
  exit 2
fi

tag="v$version"
say() { printf '\n\033[1m==> %s\033[0m\n' "$1"; }
die() { printf '\033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }

# ── Preflight ──────────────────────────────────────────────────────────────
# All of it before anything is touched, so a failure here costs nothing.
say "Checking"

[ -z "$(git status --porcelain)" ] || die "working tree is dirty — commit or stash first"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "tag $tag already exists"
command -v gh >/dev/null || die "gh is not installed (brew install gh)"
gh auth status >/dev/null 2>&1 || die "gh is not authenticated (gh auth login)"
git remote get-url origin >/dev/null 2>&1 || die "no 'origin' remote — add one before releasing"

# scripts/make-dmg.py writes the DMG's .DS_Store and background alias itself,
# rather than asking Finder to. Failing here beats failing after a 3-minute build.
python3 -c "import ds_store, mac_alias" 2>/dev/null || die \
  "python needs ds-store: pip3 install ds-store mac_alias"

rustup target list --installed | grep -qx x86_64-apple-darwin || die \
  "missing x86_64 target for the universal build: rustup target add x86_64-apple-darwin"

[ -f src-tauri/dmg/background.tiff ] || die \
  "src-tauri/dmg/background.tiff is missing — run scripts/make-dmg-background.py"

# The three files that carry the version must already agree, or the bump below
# would paper over a drift that started earlier.
cur_pkg=$(python3 -c "import json;print(json.load(open('package.json'))['version'])")
cur_conf=$(python3 -c "import json;print(json.load(open('src-tauri/tauri.conf.json'))['version'])")
cur_cargo=$(grep -m1 '^version = ' src-tauri/Cargo.toml | cut -d'"' -f2)
if [ "$cur_pkg" != "$cur_conf" ] || [ "$cur_pkg" != "$cur_cargo" ]; then
  die "versions disagree: package.json=$cur_pkg tauri.conf.json=$cur_conf Cargo.toml=$cur_cargo"
fi
echo "  $cur_pkg -> $version"
$dry_run && echo "  (dry run — nothing will be written)"

# ── Bump ───────────────────────────────────────────────────────────────────
say "Bumping version in 3 files"
if ! $dry_run; then
  python3 - "$version" <<'PY'
import json, re, sys, pathlib
version = sys.argv[1]

for path in ("package.json", "src-tauri/tauri.conf.json"):
    p = pathlib.Path(path)
    # Edited as text, not via json.dump, so key order and formatting survive.
    s = p.read_text()
    s, n = re.subn(r'("version":\s*")[^"]+(")', rf'\g<1>{version}\g<2>', s, count=1)
    assert n == 1, path
    p.write_text(s)

p = pathlib.Path("src-tauri/Cargo.toml")
s = p.read_text()
s, n = re.subn(r'(?m)^(version = ")[^"]+(")', rf'\g<1>{version}\g<2>', s, count=1)
assert n == 1, "Cargo.toml"
p.write_text(s)
print("  package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml")
PY
fi

# ── Build ──────────────────────────────────────────────────────────────────
dmg="src-tauri/target/universal-apple-darwin/release/bundle/dmg/PaperStack_${version}_universal.dmg"

# Only the .app comes from Tauri. Its DMG step drives Finder over AppleScript,
# which needs the calling terminal to hold macOS's Automation → Finder grant;
# without it osascript is refused and the build dies with nothing useful in the
# log. scripts/make-dmg.py writes the same Finder settings into .DS_Store itself,
# so the DMG builds identically from any terminal, over SSH, or on CI.
say "Building universal (this compiles the whole tree for both architectures)"
npm run tauri build -- --target universal-apple-darwin --bundles app

app="src-tauri/target/universal-apple-darwin/release/bundle/macos/PaperStack.app"
[ -d "$app" ] || die "expected an app bundle at $app"

say "Building the DMG"
python3 scripts/make-dmg.py "$app" -o "$dmg"

[ -f "$dmg" ] || die "expected a DMG at $dmg"

size=$(python3 -c "import os;print(f'{os.path.getsize(\"$dmg\")/1e6:.1f} MB')")
sha=$(shasum -a 256 "$dmg" | cut -d' ' -f1)
echo "  $(basename "$dmg")  $size"
echo "  sha256 $sha"

if $dry_run; then
  say "Dry run complete — no commit, no tag, no release"
  git checkout -- package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml 2>/dev/null || true
  exit 0
fi

# ── Commit and tag ─────────────────────────────────────────────────────────
say "Committing and tagging"
# Cargo.lock carries the version too; the build above regenerated it.
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
# Re-releasing the version already in the files — the common case for a first
# release — stages nothing, and `git commit` treats that as an error. There is
# genuinely nothing to record, so tag the commit that is already there.
if git diff --cached --quiet; then
  echo "  files already at $version — tagging HEAD, no bump commit needed"
else
  git commit -m "release: $version"
fi
git tag -a "$tag" -m "PaperStack $version"

# ── Publish ────────────────────────────────────────────────────────────────
branch=$(git rev-parse --abbrev-ref HEAD)
cat <<EOF

  Ready to publish:
    push    $branch and $tag to $(git remote get-url origin)
    release $tag with $(basename "$dmg") ($size)

  Nothing has left this machine yet.
EOF
read -r -p "  Push and publish? [y/N] " reply
if [[ ! "$reply" =~ ^[Yy]$ ]]; then
  cat <<EOF

  Stopped. The commit and tag are local. To undo them:
    git tag -d $tag && git reset --hard HEAD~1

  Or to publish later:
    git push origin $branch && git push origin $tag
    gh release create $tag "$dmg" --generate-notes --title "PaperStack $version"
EOF
  exit 0
fi

say "Pushing"
# Branch and tag pushed separately rather than with --follow-tags: that only
# carries tags along with commits it is actually pushing, so on a release where
# the branch is already up to date the tag would silently stay local.
git push origin "$branch"
git push origin "$tag"

say "Creating the release"
# --generate-notes works out the previous release on its own, which is the
# behaviour we want and the only one that is correct for the first release.
gh release create "$tag" "$dmg" \
  --title "PaperStack $version" \
  --generate-notes

say "Done"
gh release view "$tag" --json url --jq '"  " + .url'
echo "  The download page updates itself — no edit needed."
