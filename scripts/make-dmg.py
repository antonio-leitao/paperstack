"""Build the distributable DMG without going anywhere near Finder.

    python3 scripts/make-dmg.py                       # uses the universal build
    python3 scripts/make-dmg.py path/to/PaperStack.app -o out.dmg

Needs `ds-store` and `mac_alias` (pip3 install ds-store mac_alias) and macOS's
hdiutil. Replaces Tauri's own DMG step — build the app with
`npm run tauri build -- --bundles app` and then run this.

Why this exists
---------------
Tauri bundles a DMG by mounting it and then asking Finder, over AppleScript, to
set the window size, place the icons and apply the background. That makes the
build depend on a GUI process and on macOS's Automation permissions: if the
terminal running the build has not been granted Automation → Finder, osascript
is refused, `bundle_dmg.sh` exits 64, and Tauri reports only the useless
"failed to run bundle_dmg.sh". The same build then succeeds from a terminal that
does have the grant, which makes it look intermittent when it is not.

None of that is necessary. Everything Finder is being asked to do ends up in a
single `.DS_Store` file on the volume, and that file can be written directly:

  bwsp   the window's size and position
  icvp   icon size, and the background picture as a bookmark
  Iloc   each icon's position

So this creates the image, writes those records itself, and converts. No
AppleScript, no Automation grant, no GUI. It runs identically from any terminal,
over SSH, or on CI — which the Finder route cannot do at all.
"""

import argparse
import plistlib
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

VOLUME_NAME = "PaperStack"
BACKGROUND = ROOT / "src-tauri" / "dmg" / "background.tiff"
VOLUME_ICON = ROOT / "src-tauri" / "icons" / "icon.icns"

# Must agree with scripts/make-dmg-background.py, which draws the artwork to fit.
WINDOW = (660, 400)
APP_POS = (180, 158)
APPS_POS = (480, 158)
ICON_SIZE = 128.0


def run(*args, **kw):
    return subprocess.run(args, check=True, capture_output=True, text=True, **kw)


def detach(path, quiet=True):
    subprocess.run(["hdiutil", "detach", str(path), "-quiet"],
                   capture_output=True, text=True)


def write_ds_store(mount: Path, app_name: str) -> None:
    """Write the Finder view settings that Finder would otherwise be asked for."""
    from ds_store import DSStore
    import mac_alias

    background = mount / ".background" / BACKGROUND.name
    alias = mac_alias.Bookmark.for_file(str(background)).to_bytes()

    store = mount / ".DS_Store"
    store.unlink(missing_ok=True)
    with DSStore.open(str(store), "w+") as d:
        d["."]["bwsp"] = {
            "WindowBounds": "{{10, 8}, {%d, %d}}" % WINDOW,
            "ShowStatusBar": False,
            "ShowToolbar": False,
            "ShowTabView": False,
            "ShowSidebar": False,
            "ContainerShowSidebar": False,
        }
        d["."]["icvp"] = {
            "viewOptionsVersion": 1,
            "backgroundType": 2,          # 2 = picture
            "backgroundImageAlias": alias,
            "backgroundColorRed": 1.0,
            "backgroundColorGreen": 1.0,
            "backgroundColorBlue": 1.0,
            "iconSize": ICON_SIZE,
            "textSize": 16.0,
            "arrangeBy": "none",
            "gridOffsetX": 0.0,
            "gridOffsetY": 0.0,
            "gridSpacing": 100.0,
            "labelOnBottom": True,
            "showIconPreview": True,
            "showItemInfo": False,
        }
        d[app_name]["Iloc"] = APP_POS
        d["Applications"]["Iloc"] = APPS_POS


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", nargs="?", help="path to PaperStack.app")
    parser.add_argument("-o", "--output", help="path for the finished .dmg")
    args = parser.parse_args()

    app = Path(args.app) if args.app else (
        ROOT / "src-tauri/target/universal-apple-darwin/release/bundle/macos/PaperStack.app")
    if not app.is_dir():
        print(f"no app bundle at {app}\n"
              f"build one first: npm run tauri build -- "
              f"--target universal-apple-darwin --bundles app", file=sys.stderr)
        return 1
    if not BACKGROUND.is_file():
        print(f"missing {BACKGROUND} — run scripts/make-dmg-background.py", file=sys.stderr)
        return 1

    version = plistlib.loads((app / "Contents" / "Info.plist").read_bytes())[
        "CFBundleShortVersionString"]
    out = Path(args.output) if args.output else app.parent.parent / "dmg" / \
        f"PaperStack_{version}_universal.dmg"
    out.parent.mkdir(parents=True, exist_ok=True)

    # A volume left over from an interrupted run would take the name and send
    # the background bookmark to the wrong path.
    for stale in Path("/Volumes").glob(f"{VOLUME_NAME}*"):
        print(f"  ejecting stale {stale}")
        detach(stale)

    print(f"building {out.name}  (from {app.name}, version {version})")

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        stage = tmp / "stage"
        stage.mkdir()

        print("  staging")
        # ditto, not copytree: it preserves the bundle's symlinks and the ad-hoc
        # signature, which a naive copy silently breaks.
        run("ditto", str(app), str(stage / app.name))
        (stage / "Applications").symlink_to("/Applications")
        (stage / ".background").mkdir()
        shutil.copy2(BACKGROUND, stage / ".background" / BACKGROUND.name)
        if VOLUME_ICON.is_file():
            shutil.copy2(VOLUME_ICON, stage / ".VolumeIcon.icns")

        print("  creating a writable image")
        rw = tmp / "rw.dmg"
        run("hdiutil", "create", "-srcfolder", str(stage), "-volname", VOLUME_NAME,
            "-fs", "HFS+", "-format", "UDRW", "-quiet", str(rw))

        print("  mounting (hidden — no Finder window)")
        # -nobrowse keeps it out of the sidebar, -noautoopen stops Finder opening
        # a window. Nothing here ever asks Finder for anything.
        out_txt = run("hdiutil", "attach", str(rw), "-nobrowse", "-noautoopen",
                      "-readwrite").stdout
        mount = None
        for line in out_txt.splitlines():
            if "/Volumes/" in line:
                mount = Path(line.split("\t")[-1].strip())
        if mount is None:
            print("could not determine the mount point", file=sys.stderr)
            return 1

        try:
            print("  writing Finder view settings directly")
            write_ds_store(mount, app.name)
            if (mount / ".VolumeIcon.icns").is_file():
                # Marks the volume as having a custom icon. Cosmetic, and
                # SetFile is not always installed, so failure is not fatal.
                subprocess.run(["SetFile", "-a", "C", str(mount)],
                               capture_output=True, text=True)
            # Give the writes a moment to land before the volume goes away.
            time.sleep(1)
        finally:
            print("  unmounting")
            detach(mount)

        print("  compressing")
        out.unlink(missing_ok=True)
        run("hdiutil", "convert", str(rw), "-format", "UDZO",
            "-imagekey", "zlib-level=9", "-quiet", "-o", str(out))

    size = out.stat().st_size / 1e6
    print(f"done — {out.relative_to(ROOT) if out.is_relative_to(ROOT) else out}  {size:.1f} MB")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
