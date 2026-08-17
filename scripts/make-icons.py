"""Build the PaperStack app icons from the mark in design_system/.

Flat on purpose. The app's own surfaces are flat with hard offset shadows
(--shadow-sm is `2px 2px 0`, not a blur) and there is no dark theme. The mark is
also already a drawing with light in it — its faces are four different whites —
so lighting the tile underneath sets up two light sources that disagree, and a
cast shadow under the mark lifts it off a tile it should be printed on. One
solid fill, a hairline rim, nothing else.

The single exception is the shadow the tile casts on the desktop. That is not a
style choice: every macOS icon bakes it, and the 100px margin of the 1024 canvas
exists to hold it.

The tile is a superellipse (|x|^n + |y|^n = 1, n = 5) rather than a rounded
rectangle, because macOS corners are continuous-curvature and the difference
shows at 1024. Big Sur grid: 824px body centred in 1024.

Blue is oklch(0.63 0.115 240) — phthalocyanine blue's hue, at a lightness that
keeps the near-white paper faces reading against it.

Two variants, bold and light linework, on the same tile. Bold ships as the
default because it is the one that survives a dock; light is written alongside
it as a complete set so it can be swapped in with one constant.

Two builds per variant, because one drawing cannot cover 16px to 1024px:

  > 48   the mark as drawn, linework and all.
  <= 48  linework removed and the front faces darkened. The strokes are 4.5
         units in a 1000-unit canvas, so below about 48px they land on a
         fraction of a pixel: they stop delineating anything and just scatter
         grey into the fills, and the interior paper-edge marks turn to noise.
         Dropping them and separating the faces by value instead is what keeps
         three stacks reading as three stacks all the way down to 16.

Removing the linework is also what makes the two variants converge: below 48px
they are the same image, because linework is the only thing that distinguishes
them. So the small build is rendered once and shared rather than duplicated.

Run: python3 scripts/make-icons.py
"""

import io
import re
import struct
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

import cairosvg

ROOT = Path(__file__).resolve().parent.parent
ICONS = ROOT / "src-tauri" / "icons"
VARIANTS = ICONS / "variants"
STATIC = ROOT / "static"
DESIGN = ROOT / "design_system"

MARKS = {
    "bold": DESIGN / "paperstack-mark-light-bold.svg",
    "light": DESIGN / "paperstack-mark-light.svg",
}
DEFAULT = "bold"  # the set written to src-tauri/icons and used by the bundle

SS = 4
CANVAS = 1024
BODY = 824
EXPONENT = 5.0

# PaperStack blue — oklch(0.63 0.115 240). Phthalocyanine blue's hue at a
# lightness that keeps the near-white paper faces reading against it. Mirrored
# in src/app.css as --phthalo; change it in both places or neither.
TILE = "#3c91c7"
RIM = 0.13

# Measured from the rendered alpha rather than read off the path data, because
# round joins and caps extend past the geometry. The drawing is off-centre in
# its own 1000-unit canvas, so centring the viewBox would sit the mark low left.
CONTENT = (117, 180, 882, 837)
GLYPH_W = 0.70
SMALL_GLYPH_W = 0.72

SMALL_CUTOFF = 48

# Front faces darkened, tops left white. At 16px this value separation is the
# only thing still distinguishing one stack from the next.
SMALL_FACES = {
    "front-face": "#cfccc4",
    "right-face": "#eceae5",
    "top-sheet": "#ffffff",
    "curl-underside": "#dbd9d2",
    # Between the darkened front face and the right face, so the notch under
    # each curl still reads as a plane of its own once the linework is gone.
    "sheet-below": "#dcd9d2",
}


def hexrgb(h):
    h = h.lstrip("#")
    return np.array([int(h[i : i + 2], 16) for i in (0, 2, 4)], dtype=np.float64)


def small_svg():
    """The mark with linework removed and the faces pulled apart by value.

    Either source works — stripping the strokes is exactly what makes bold and
    light identical — so this reads the default and the result is shared.
    """
    svg = MARKS[DEFAULT].read_text()
    svg = re.sub(r"stroke:\s*#[0-9a-fA-F]+;", "stroke: none;", svg)
    for cls, col in SMALL_FACES.items():
        svg = re.sub(
            rf"\.{cls} {{ fill: #[0-9a-fA-F]+; }}", f".{cls} {{ fill: {col}; }}", svg
        )
    return svg


def resize(img, size):
    """Downsample in premultiplied alpha.

    Pillow resamples the four channels independently, so the transparent margin
    keeps the tile's colour and Lanczos overshoot paints it back as a halo just
    outside the edge. Premultiplying first leaves nothing there to bleed.
    """
    if img.size == (size, size):
        return img
    a = np.asarray(img).astype(np.float64)
    alpha = a[..., 3:4] / 255.0
    a[..., :3] *= alpha
    pre = Image.fromarray(a.astype(np.uint8), "RGBA").resize((size, size), Image.LANCZOS)
    b = np.asarray(pre).astype(np.float64)
    alpha = b[..., 3:4] / 255.0
    np.divide(b[..., :3], alpha, out=b[..., :3], where=alpha > 0)
    b[..., :3] = np.clip(b[..., :3], 0, 255)
    return Image.fromarray(b.astype(np.uint8), "RGBA")


def superellipse_mask(size, n=EXPONENT):
    t = (np.arange(size) + 0.5) / size * 2.0 - 1.0
    return ((np.abs(t)[None, :] ** n + np.abs(t)[:, None] ** n) <= 1.0).astype(
        np.float64
    )


def _erode(mask, k):
    m = Image.fromarray((mask * 255).astype(np.uint8))
    return np.array(m.filter(ImageFilter.MinFilter(2 * k + 1))).astype(np.float64) / 255


def render_mark(width, height, svg):
    """Rasterise the mark cropped to CONTENT, scaled to width x height.

    cairosvg has no crop, so the whole canvas is rendered at the scale that puts
    CONTENT at the requested size, and the crop is cut afterwards.
    """
    sx = width / (CONTENT[2] - CONTENT[0])
    sy = height / (CONTENT[3] - CONTENT[1])
    png = cairosvg.svg2png(
        bytestring=svg.encode(),
        output_width=int(round(1000 * sx)),
        output_height=int(round(1000 * sy)),
    )
    im = Image.open(io.BytesIO(png)).convert("RGBA")
    box = tuple(int(round(v * s)) for v, s in zip(CONTENT, (sx, sy, sx, sy)))
    return im.crop(box).resize((width, height), Image.LANCZOS)


def build(variant=DEFAULT, small=False):
    S = BODY * SS
    mask = superellipse_mask(S)

    rgb = np.broadcast_to(hexrgb(TILE), (S, S, 3)).astype(np.float64).copy()
    rim = np.clip(mask - _erode(mask, max(1, int(round(1.0 * SS)))), 0.0, 1.0)
    rgb = rgb * (1.0 - (rim * RIM)[..., None])

    tile = np.dstack([np.clip(rgb, 0, 255), mask * 255.0]).astype(np.uint8)
    tile = resize(Image.fromarray(tile, "RGBA"), BODY)

    w = int(round(BODY * (SMALL_GLYPH_W if small else GLYPH_W)))
    h = max(1, int(round(w * (CONTENT[3] - CONTENT[1]) / (CONTENT[2] - CONTENT[0]))))
    glyph = render_mark(w, h, small_svg() if small else MARKS[variant].read_text())
    tile.paste(glyph, ((BODY - w) // 2, (BODY - h) // 2), glyph)

    canvas = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    off = (CANVAS - BODY) // 2
    for oy, blur_r, a in ((20, 30, 0.18), (4, 8, 0.12)):
        drop = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
        body = Image.new("RGBA", (BODY, BODY), (0, 0, 0, 255))
        body.putalpha(tile.getchannel("A"))
        drop.paste(body, (off, off + oy), body)
        drop = drop.filter(ImageFilter.GaussianBlur(blur_r))
        drop.putalpha(drop.getchannel("A").point(lambda v: int(v * a)))
        canvas = Image.alpha_composite(canvas, drop)
    canvas.paste(tile, (off, off), tile)
    return canvas


ICNS_TYPES = [
    (b"icp4", 16),
    (b"icp5", 32),
    (b"ic11", 32),
    (b"ic12", 64),
    (b"ic07", 128),
    (b"ic13", 256),
    (b"ic08", 256),
    (b"ic14", 512),
    (b"ic09", 512),
    (b"ic10", 1024),
]


def at(tiers, size):
    return resize(tiers["small" if size <= SMALL_CUTOFF else "master"], size)


def write_icns(tiers, path):
    chunks = b""
    for tag, size in ICNS_TYPES:
        buf = io.BytesIO()
        at(tiers, size).save(buf, "PNG")
        data = buf.getvalue()
        chunks += tag + struct.pack(">I", len(data) + 8) + data
    path.write_bytes(b"icns" + struct.pack(">I", len(chunks) + 8) + chunks)


def main():
    VARIANTS.mkdir(parents=True, exist_ok=True)

    # Rendered once: with the linework stripped the two variants are the same
    # image, so there is nothing to render twice.
    small = build(small=True)

    built = {}
    for name in MARKS:
        built[name] = {"master": build(name), "small": small}
        at(built[name], 1024).save(VARIANTS / f"paperstack-{name}-1024.png")
        write_icns(built[name], VARIANTS / f"paperstack-{name}.icns")
        for s in (256, 512):
            at(built[name], s).save(VARIANTS / f"paperstack-{name}-{s}.png")
        print(f"variant {name}")

    tiers = built[DEFAULT]

    for fname, size in (
        ("32x32.png", 32),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
        ("icon.png", 512),
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44),
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),
    ):
        at(tiers, size).save(ICONS / fname)

    write_icns(tiers, ICONS / "icon.icns")

    ico_sizes = [16, 32, 48, 64, 128, 256]
    frames = [at(tiers, s) for s in ico_sizes]
    frames[-1].save(
        ICONS / "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ico_sizes],
        append_images=frames[:-1],
    )

    at(tiers, 256).save(STATIC / "favicon.png")
    print(f"default set written from {DEFAULT}")


if __name__ == "__main__":
    main()
