# Mailwoman icon

One mark, used for the favicon, the web manifest, the apple-touch icon and both
Tauri shells: an envelope built from the three primary shapes on a near-black
field.

- **Blue** `#4d7cff`: the envelope body, a rectangle with a V cut from its top edge.
- **Yellow** `#f5c400`: the flap, a triangle sitting in the V.
- **Red** `#ff5a4a`: a circular seal on the point of the V.
- **Field** `#111111`: the square behind them, the same near-black as the dark theme's page.

No text, no gradients, square corners. No shape touches another, so every edge is
a shape against the field: the three colours are under 3:1 against each other
(yellow on blue is 2.3:1), and the field gap is what keeps them apart. Each colour has at least 3:1 contrast on the field
(blue 5.1:1, red 6.1:1, yellow 11.5:1); the build script refuses a colour below
that. The colours are the dark theme's accent, danger and signal values.

## Files

| file | what it is |
| --- | --- |
| `mark.json` | the geometry: colours and one shape list per size range (the source of everything below) |
| `mark.svg` | the full mark, 1024 grid |
| `mark-mono.svg` | one colour (`#000000`): the field solid, the shapes cut out; for masks and single-colour uses |
| `mark-1024.png` | the full mark as a 1024 px PNG, input to `cargo tauri icon` |
| `mark-android-fg-1024.png` | Android adaptive foreground: the shapes on transparency, scaled into the 66 dp safe circle |
| `mark-android-mono-1024.png` | Android themed-icon layer: the same shapes in one colour |
| `tauri-icon.json` | the `cargo tauri icon` manifest (master, foreground, monochrome, background `#111111`) |

## Small sizes

`mark.json` has three variants. Each output size uses the first one whose
`maxSize` it fits:

| sizes | grid | shapes |
| --- | --- | --- |
| up to 16 px | 16 | envelope body and flap only; the seal is left out because at 16 px it fills with its own gap |
| 17 to 48 px | 32 | all three shapes, edges on whole pixels at 32 and 48 px |
| above 48 px | 1024 | the full mark |

`favicon.svg` is the 16 px variant, because browsers draw it in a 16 px tab.

## Outputs and paths

Web (`apps/web/public/`, linked from `apps/web/index.html`):

- `favicon.svg`, `favicon.ico` (16, 32, 48), `apple-touch-icon.png` (180, opaque)
- `manifest.webmanifest`, with `icons/icon-192.png`, `icons/icon-512.png` and
  `icons/icon-maskable-512.png` (shapes inside the 40 % safe circle)

Tauri (`apps/desktop/src-tauri/` and `apps/mobile/src-tauri/`): `app-icon.png` (the
1024 master) and the whole `icons/` directory.

These paths are fixed. Admin branding (t32-admin §10.6) serves its own icons at
the same paths when an admin uploads an icon master, and falls back to these files
otherwise, so renaming one means changing the server override too.

## Rebuilding

The outputs are committed; CI does not run the script. After changing
`mark.json`, from the repo root:

```
py -3.12 scripts/icons/build-icons.py --preview <dir>   # contact sheet only: 16/32/180/512 on dark and light
py -3.12 scripts/icons/build-icons.py                   # branding/icon + apps/web/public
py -3.12 scripts/icons/build-icons.py --tauri           # also both Tauri icon sets
```

It needs Python 3.10 or later with Pillow. `--tauri` also needs the Tauri CLI
(`cargo tauri`, 2.x). `cargo tauri icon` scales the master down for every size; the
script then redraws the square PNGs of 48 px and less and every `icon.ico` entry
from their own variant. `icon.icns` keeps Tauri's scaled entries, because Pillow
cannot write its 16 px slot.

`apps/web/src/icons.test.ts` checks that every file `index.html` and the manifest
name exists and that each PNG and ICO has the sizes it is declared with.
