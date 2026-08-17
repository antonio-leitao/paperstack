"""Fix the DMG window size that Tauri's bundler fails to apply.

Run after `npm run tauri build`:

    python3 scripts/finalize-dmg.py                 # newest DMG under target/
    python3 scripts/finalize-dmg.py path/to.dmg     # a specific one

Needs `ds-store` (pip install ds-store) and macOS's hdiutil.

Why this exists
---------------
`bundle.macOS.dmg.windowSize` in tauri.conf.json is passed through to
`bundle_dmg.sh`, which asks Finder to set the volume window's bounds over
AppleScript. Finder does not reliably honour it: on this machine the bundled
DMG opens at 1492x918 regardless of the configured 660x400, and restarting
Finder does not change that. Whatever size Finder decides on the *build*
machine is what gets written into the volume's .DS_Store, and that file ships
inside the DMG — so every user inherits it.

Everything else the bundler writes is correct: the icon positions, the 128pt
icon size, and the background picture alias. So rather than rebuilding the
layout, this reopens the finished DMG read-write and rewrites .DS_Store with
the same records plus the window bounds that were asked for.

The background alias is copied across as raw bytes rather than regenerated. It
is a bookmark to a path on the mounted volume, and the volume is mounted at the
same place while this runs, so the original bytes stay valid.
"""

import plistlib
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONFIG = ROOT / "src-tauri" / "tauri.conf.json"
TARGET = ROOT / "src-tauri" / "target"


def run(*args, **kw):
    return subprocess.run(args, check=True, capture_output=True, text=True, **kw)


def load_layout():
    """Window size and icon positions, read from tauri.conf.json so this can't drift."""
    import json

    cfg = json.loads(CONFIG.read_text())
    dmg = cfg["bundle"]["macOS"]["dmg"]
    return (
        (dmg["windowSize"]["width"], dmg["windowSize"]["height"]),
        (dmg["appPosition"]["x"], dmg["appPosition"]["y"]),
        (dmg["applicationFolderPosition"]["x"], dmg["applicationFolderPosition"]["y"]),
    )


def read_icon_view_options(ds_store: Path) -> dict:
    """Pull the bundler's icon-view settings back out, background alias included.

    Read by hand rather than through ds_store: the file also holds a bookmark
    record that mac_alias refuses to parse, which aborts a full traversal.
    """
    data = ds_store.read_bytes()
    for match in re.finditer(b"bplist00", data):
        start = match.start()
        for end in range(len(data), start, -1):
            try:
                parsed = plistlib.loads(data[start:end])
            except Exception:
                continue
            if isinstance(parsed, dict) and "backgroundType" in parsed:
                return parsed
            break
    raise SystemExit("could not find the icon view settings in .DS_Store")


def icon_positions(ds_store: Path) -> list[tuple[int, int]]:
    data = ds_store.read_bytes()
    found, index = [], 0
    while True:
        index = data.find(b"Iloc", index)
        if index < 0:
            return found
        found.append(struct.unpack(">II", data[index + 12:index + 20]))
        index += 4


def rewrite(volume: Path, size, app_pos, apps_pos, app_name: str) -> None:
    from ds_store import DSStore

    ds_store = volume / ".DS_Store"
    view_options = read_icon_view_options(ds_store)
    before = icon_positions(ds_store)
    ds_store.unlink()

    with DSStore.open(str(ds_store), "w+") as store:
        store["."]["bwsp"] = {
            "WindowBounds": "{{10, 8}, {%d, %d}}" % size,
            "ShowStatusBar": False,
            "ShowToolbar": False,
            "ShowTabView": False,
            "ShowSidebar": False,
            "ContainerShowSidebar": False,
        }
        store["."]["icvp"] = view_options
        store[app_name]["Iloc"] = app_pos
        store["Applications"]["Iloc"] = apps_pos

    print(f"    icon positions {before} -> {icon_positions(ds_store)}")
    print(f"    window bounds  -> {size[0]}x{size[1]}")


def main() -> int:
    if len(sys.argv) > 1:
        dmg = Path(sys.argv[1]).resolve()
        if not dmg.is_file():
            print(f"no such DMG: {dmg}", file=sys.stderr)
            return 1
    else:
        # Any target triple, so the universal build is found as readily as the
        # host-only one. Newest wins.
        found = sorted(TARGET.glob("*/release/bundle/dmg/*.dmg"),
                       key=lambda p: p.stat().st_mtime)
        found += sorted(TARGET.glob("release/bundle/dmg/*.dmg"),
                        key=lambda p: p.stat().st_mtime)
        if not found:
            print("no DMG under src-tauri/target — run `npm run tauri build` first",
                  file=sys.stderr)
            return 1
        dmg = max(found, key=lambda p: p.stat().st_mtime)
    size, app_pos, apps_pos = load_layout()
    print(f"finalizing {dmg.name}")

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        writable = tmp / "rw.dmg"
        print("  converting to read-write")
        run("hdiutil", "convert", str(dmg), "-format", "UDRW", "-o", str(writable))

        print("  mounting")
        out = run("hdiutil", "attach", str(writable), "-nobrowse", "-noautoopen").stdout
        volume = Path(re.search(r"(/Volumes/.+)$", out.strip(), re.M).group(1).strip())
        app_name = next(p.name for p in volume.glob("*.app"))
        try:
            print("  rewriting .DS_Store")
            rewrite(volume, size, app_pos, apps_pos, app_name)
        finally:
            print("  unmounting")
            run("hdiutil", "detach", str(volume), "-quiet")

        print("  recompressing")
        final = tmp / "final.dmg"
        run("hdiutil", "convert", str(writable), "-format", "UDZO",
            "-imagekey", "zlib-level=9", "-o", str(final))
        shutil.move(str(final), str(dmg))

    print(f"done — {dmg.relative_to(ROOT)} ({dmg.stat().st_size / 1e6:.1f} MB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
