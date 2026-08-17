# PaperStack brand assets

Three stacks of paper in three-quarter view, the top sheet of each curled at the
corner.

## The marks

| File | Linework | Use |
| --- | --- | --- |
| `paperstack-mark-light-bold.svg` | `#20201e`, 4.5px structure / 4px detail | App icon, anywhere small or on a strong colour |
| `paperstack-mark-light.svg` | `#b8b6af` / `#aaa8a1`, 2.5px | In-app at comfortable sizes — empty states, about panel |

The two files are the same drawing. Their path data is byte-identical; only the
stroke colour and width differ. If you edit the geometry of one, edit the other,
or they drift apart silently — `src/lib/PaperStackMark.svelte` holds the same
geometry once with a `bold` prop, and is the better place to change it.

Both are authored on a 1000×1000 canvas, but the drawing sits at
x 117–882, y 180–837 within it. Anything placing the mark should use that
content box, not the canvas, or the mark lands low and left with dead space
around it.

### The sheet below each curl

Each stack has a `.sheet-below` fill and a short exposed edge appended to its
structure path. Without them the curled corner lifts away from the box's
top-right edge and leaves a wedge painted by nothing at all — the background
shows through and the stack reads as hollow.

The fill is drawn as the stack's whole flat top face (the parallelogram
`A, A+E−D, E, D`) and then covered by the top sheet, so the only part that
shows is the wedge. It stays correct if the curl is ever redrawn.

The exposed edge runs from the box's right corner `E` up-left until it meets the
curl's underside curve. Those endpoints are solved, not eyeballed:

| Stack | From | Meets the curl at |
| --- | --- | --- |
| Tall centre | 718,250 | 676,240 |
| Medium left | 420,410 | 384,400 |
| Short right | 880,545 | 847,536 |

Redraw a curl and these move. `scripts/make-icons.py` does not compute them —
they are baked into the path data.

## The colour

```
#3c91c7   oklch(0.63 0.115 240)
```

Phthalocyanine blue's hue, at a lightness that keeps the mark's near-white paper
faces legible on it. It lives in two places and must agree in both:

- `src/app.css` as `--phthalo`
- `scripts/make-icons.py` as `TILE`

It is currently the icon tile only. See the note in `app.css` before promoting
it to `--accent`.

## Regenerating the icons

```sh
python3 scripts/make-icons.py    # needs pillow, numpy, cairosvg
```

Writes the default set to `src-tauri/icons/`, both variants to
`src-tauri/icons/variants/`, and the favicon to `static/`. Bold is the default;
change `DEFAULT` at the top of the script to ship light instead.

Two things worth knowing before you touch it:

**The tile is a superellipse**, not a rounded rectangle. macOS corners are
continuous-curvature and a `border-radius` equivalent looks wrong at 1024. It
sits on the Big Sur grid — an 824px body centred in a 1024px canvas, the 100px
margin holding the baked drop shadow that every macOS icon carries.

**Below 48px the linework is dropped** and the faces are separated by value
instead. The strokes are 4.5 units in a 1000-unit canvas, so at 32px they land
on about a tenth of a pixel: they stop drawing edges and just scatter grey into
the fills. Thickening them instead was tried and turns to noise. This is also
why the two variants are the same image below 48px — linework is the only thing
that distinguishes them.

After installing, macOS will keep showing the old icon until its cache is
dropped:

```sh
touch /Applications/PaperStack.app
killall Dock
```
