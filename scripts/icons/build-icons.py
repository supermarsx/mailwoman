"""Build the Mailwoman icon set from branding/icon/mark.json (t32-e22).

Dev-time only: the outputs are committed and CI does not run this script.
Needs Python 3.10+ and Pillow (no other dependency). From the repo root:

    py -3.12 scripts/icons/build-icons.py            # SVGs, PNGs, ICO, manifest
    py -3.12 scripts/icons/build-icons.py --tauri    # also both Tauri icon sets
    py -3.12 scripts/icons/build-icons.py --preview DIR [--spec FILE]
                                                     # contact sheet only

The geometry lives in mark.json on a square grid per variant. Each variant is a
list of shapes (rect, circle, polygon) drawn in order on the field colour. A
shape may carry `"gap": n`, which first paints the shape grown by n units in the
field colour, so it is cut free of whatever lies under it. Every size uses the
variant whose `maxSize` it fits: the small sizes have their own hand-placed
geometry instead of a blurred copy of the large one. The SVGs and the rasters are
drawn from the same numbers; the rasters are supersampled and box-reduced.
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
BRAND = ROOT / "branding" / "icon"
PUBLIC = ROOT / "apps" / "web" / "public"
SHELLS = [ROOT / "apps" / "desktop" / "src-tauri", ROOT / "apps" / "mobile" / "src-tauri"]


def supersample(size: int) -> int:
    """Canvas scale for `size`: about 2048 px wide, between 4x and 32x.

    Pillow fills a polygon including its right and bottom boundary, one canvas
    pixel too far; at 32x (the favicon sizes) that is 1/32 of an output pixel."""
    return max(4, min(32, 2048 // size))


# Safe zones, as the radius of the circle (fraction of the icon width) that must
# hold every shape. Web maskable icons: 40 % (W3C manifest spec). Android
# adaptive foreground: 33 of 108 dp.
MASKABLE_SAFE_RADIUS = 0.40
ANDROID_FG_SAFE_RADIUS = 33 / 108


# ── colour checks ────────────────────────────────────────────────────────────


def hex_rgb(value: str) -> tuple[int, int, int]:
    value = value.lstrip("#")
    return int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16)


def luminance(value: str) -> float:
    def channel(c: int) -> float:
        s = c / 255
        return s / 12.92 if s <= 0.04045 else ((s + 0.055) / 1.055) ** 2.4

    r, g, b = (channel(c) for c in hex_rgb(value))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: str, b: str) -> float:
    la, lb = sorted((luminance(a), luminance(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


# ── geometry ─────────────────────────────────────────────────────────────────


def grown(shape: dict, by: float) -> dict:
    """The shape enlarged by `by` units on every side (mitred corners)."""
    kind = shape["kind"]
    if kind == "circle":
        return {**shape, "r": shape["r"] + by}
    if kind == "rect":
        return {**shape, "x": shape["x"] - by, "y": shape["y"] - by,
                "w": shape["w"] + 2 * by, "h": shape["h"] + 2 * by}
    pts = shape["points"]
    n = len(pts)
    area = sum(pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1] for i in range(n))
    sign = 1 if area > 0 else -1  # outward normal side depends on winding
    out = []
    for i in range(n):
        p0, p1, p2 = pts[i - 1], pts[i], pts[(i + 1) % n]
        normals = []
        for a, b in ((p0, p1), (p1, p2)):
            dx, dy = b[0] - a[0], b[1] - a[1]
            length = math.hypot(dx, dy)
            normals.append((sign * dy / length, -sign * dx / length))
        (ax, ay), (bx, by_) = normals
        k = 1 + ax * bx + ay * by_
        out.append([p1[0] + by * (ax + bx) / k, p1[1] + by * (ay + by_) / k])
    return {**shape, "points": out}


def transformed(shape: dict, scale: float, cx: float, cy: float) -> dict:
    """The shape scaled by `scale` about (cx, cy)."""
    def pt(x: float, y: float) -> list[float]:
        return [cx + (x - cx) * scale, cy + (y - cy) * scale]

    kind = shape["kind"]
    out = {**shape}
    if "gap" in shape:
        out["gap"] = shape["gap"] * scale
    if kind == "circle":
        out["cx"], out["cy"] = pt(shape["cx"], shape["cy"])
        out["r"] = shape["r"] * scale
    elif kind == "rect":
        out["x"], out["y"] = pt(shape["x"], shape["y"])
        out["w"], out["h"] = shape["w"] * scale, shape["h"] * scale
    else:
        out["points"] = [pt(x, y) for x, y in shape["points"]]
    return out


def reach(shape: dict, cx: float, cy: float) -> float:
    """Distance from (cx, cy) to the farthest point of the shape."""
    kind = shape["kind"]
    if kind == "circle":
        return math.hypot(shape["cx"] - cx, shape["cy"] - cy) + shape["r"]
    if kind == "rect":
        x, y, w, h = shape["x"], shape["y"], shape["w"], shape["h"]
        corners = [(x, y), (x + w, y), (x, y + h), (x + w, y + h)]
    else:
        corners = shape["points"]
    return max(math.hypot(px - cx, py - cy) for px, py in corners)


def fitted(variant: dict, safe_radius: float) -> dict:
    """The variant with its shapes scaled down until they sit in the safe circle."""
    grid = variant["grid"]
    c = grid / 2
    far = max(reach(s, c, c) for s in variant["shapes"])
    scale = min(1.0, safe_radius * grid / far)
    return {**variant, "shapes": [transformed(s, scale, c, c) for s in variant["shapes"]]}


def variant_for(spec: dict, size: int) -> dict:
    for variant in sorted(spec["variants"], key=lambda v: v["maxSize"]):
        if size <= variant["maxSize"]:
            return variant
    raise SystemExit(f"no variant covers {size}px")


# ── raster ───────────────────────────────────────────────────────────────────


def paint(draw: ImageDraw.ImageDraw, shape: dict, fill, k: float) -> None:
    kind = shape["kind"]
    if kind == "circle":
        cx, cy, r = shape["cx"] * k, shape["cy"] * k, shape["r"] * k
        draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill)
    elif kind == "rect":
        x0, y0 = shape["x"] * k, shape["y"] * k
        x1, y1 = x0 + shape["w"] * k, y0 + shape["h"] * k
        draw.polygon([(x0, y0), (x1, y0), (x1, y1), (x0, y1)], fill=fill)
    else:
        draw.polygon([(x * k, y * k) for x, y in shape["points"]], fill=fill)


def render(spec: dict, variant: dict, size: int, *, field: bool = True, mono: bool = False) -> Image.Image:
    """Draw `variant` at `size` px.

    field=False leaves the field transparent (Android foreground). mono=True
    draws in the one `mono` colour: with the field, the field is solid and the
    shapes are cut out of it; without it, the shapes are solid and the gaps
    clear (Android monochrome layer).
    """
    colors = spec["colors"]
    ss = supersample(size)
    big = size * ss
    k = big / variant["grid"]
    if mono:
        ground, ink = (255, 0) if field else (0, 255)
        mask = Image.new("L", (big, big), ground)
        draw = ImageDraw.Draw(mask)
        for shape in variant["shapes"]:
            if shape.get("gap"):
                paint(draw, grown(shape, shape["gap"]), ground, k)
            paint(draw, shape, ink, k)
        img = Image.new("RGBA", (big, big), hex_rgb(colors["mono"]) + (0,))
        img.putalpha(mask)
        return img.reduce(ss)
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    if field:
        draw.rectangle([0, 0, big - 1, big - 1], fill=hex_rgb(colors["field"]) + (255,))
    for shape in variant["shapes"]:
        if shape.get("gap"):
            gap_fill = hex_rgb(colors["field"]) + (255,) if field else (0, 0, 0, 0)
            paint(draw, grown(shape, shape["gap"]), gap_fill, k)
        paint(draw, shape, hex_rgb(colors[shape["fill"]]) + (255,), k)
    return img.reduce(ss)


# ── SVG ──────────────────────────────────────────────────────────────────────


def num(v: float) -> str:
    return f"{v:.2f}".rstrip("0").rstrip(".")


def svg_shape(shape: dict, fill: str) -> str:
    kind = shape["kind"]
    if kind == "circle":
        return f'<circle cx="{num(shape["cx"])}" cy="{num(shape["cy"])}" r="{num(shape["r"])}" fill="{fill}"/>'
    if kind == "rect":
        return (f'<rect x="{num(shape["x"])}" y="{num(shape["y"])}" width="{num(shape["w"])}" '
                f'height="{num(shape["h"])}" fill="{fill}"/>')
    pts = " ".join(f"{num(x)},{num(y)}" for x, y in shape["points"])
    return f'<polygon points="{pts}" fill="{fill}"/>'


def svg(spec: dict, variant: dict, *, mono: bool = False) -> str:
    colors = spec["colors"]
    g = num(variant["grid"])
    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {g} {g}" width="{g}" height="{g}">']
    if mono:
        lines.append('<mask id="m">')
        lines.append(f'<rect width="{g}" height="{g}" fill="#fff"/>')
        for shape in variant["shapes"]:
            if shape.get("gap"):
                lines.append(svg_shape(grown(shape, shape["gap"]), "#fff"))
            lines.append(svg_shape(shape, "#000"))
        lines.append("</mask>")
        lines.append(f'<rect width="{g}" height="{g}" fill="{colors["mono"]}" mask="url(#m)"/>')
    else:
        lines.append(f'<rect width="{g}" height="{g}" fill="{colors["field"]}"/>')
        for shape in variant["shapes"]:
            if shape.get("gap"):
                lines.append(svg_shape(grown(shape, shape["gap"]), colors["field"]))
            lines.append(svg_shape(shape, colors[shape["fill"]]))
    lines.append("</svg>")
    return "\n".join(lines) + "\n"


# ── outputs ──────────────────────────────────────────────────────────────────


def check_colors(spec: dict) -> None:
    colors = spec["colors"]
    for name in spec["shapeColors"]:
        ratio = contrast(colors[name], colors["field"])
        if ratio < 3:
            raise SystemExit(f"{name} {colors[name]} is {ratio:.2f}:1 on the field; needs 3:1")


def preview(spec: dict, out: Path, label: str) -> Path:
    """Contact sheet: 16/32/180/512 at 1:1, on a dark and a light page, plus 8x
    blow-ups of 16 and 32 so their pixels can be inspected."""
    sizes = [16, 32, 180, 512]
    pad = 24
    zoom = 8
    width = pad + sum(s + pad for s in sizes) + 16 * zoom + pad + 32 * zoom + pad
    row_h = 512 + 2 * pad
    sheet = Image.new("RGB", (width, row_h * 2), (0, 0, 0))
    for row, bg in enumerate(("#1e1e1e", "#f4f1ea")):
        y0 = row * row_h
        ImageDraw.Draw(sheet).rectangle([0, y0, width, y0 + row_h], fill=hex_rgb(bg))
        x = pad
        for s in sizes:
            img = render(spec, variant_for(spec, s), s)
            sheet.paste(img, (x, y0 + pad), img)
            x += s + pad
        for s in (16, 32):
            img = render(spec, variant_for(spec, s), s).resize((s * zoom, s * zoom), Image.NEAREST)
            sheet.paste(img, (x, y0 + pad), img)
            x += s * zoom + pad
    out.mkdir(parents=True, exist_ok=True)
    path = out / f"{label}.png"
    sheet.save(path)
    extras = Image.new("RGB", (3 * (512 + pad) + pad, 512 + 2 * pad), hex_rgb("#808080"))
    full = variant_for(spec, 1024)
    for i, img in enumerate((
        render(spec, fitted(full, MASKABLE_SAFE_RADIUS), 512),
        render(spec, full, 512, mono=True),
        render(spec, fitted(full, ANDROID_FG_SAFE_RADIUS), 512, field=False),
    )):
        extras.paste(img, (pad + i * (512 + pad), pad), img)
    extras.save(out / f"{label}-variants.png")
    return path


def save_png(img: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, optimize=True)
    print(f"wrote {path.relative_to(ROOT)} ({img.width}x{img.height})")


def build(spec: dict) -> None:
    full = variant_for(spec, 1024)
    favicon_variant = variant_for(spec, spec["faviconSvgSize"])

    BRAND.mkdir(parents=True, exist_ok=True)
    (BRAND / "mark.svg").write_text(svg(spec, full), encoding="utf-8", newline="\n")
    (BRAND / "mark-mono.svg").write_text(svg(spec, full, mono=True), encoding="utf-8", newline="\n")
    print("wrote branding/icon/mark.svg, branding/icon/mark-mono.svg")
    save_png(render(spec, full, 1024), BRAND / "mark-1024.png")
    save_png(render(spec, fitted(full, ANDROID_FG_SAFE_RADIUS), 1024, field=False), BRAND / "mark-android-fg-1024.png")
    save_png(render(spec, fitted(full, ANDROID_FG_SAFE_RADIUS), 1024, mono=True, field=False), BRAND / "mark-android-mono-1024.png")

    (PUBLIC / "favicon.svg").write_text(svg(spec, favicon_variant), encoding="utf-8", newline="\n")
    print("wrote apps/web/public/favicon.svg")
    ico_sizes = [16, 32, 48]
    ico_images = [render(spec, variant_for(spec, s), s) for s in ico_sizes]
    ico_images[-1].save(PUBLIC / "favicon.ico", format="ICO", sizes=[(s, s) for s in ico_sizes],
                        append_images=ico_images[:-1])
    print("wrote apps/web/public/favicon.ico (16, 32, 48)")
    # iOS rounds the corners itself and shows transparency as black: keep the field.
    save_png(render(spec, variant_for(spec, 180), 180).convert("RGB"), PUBLIC / "apple-touch-icon.png")
    for s in (192, 512):
        save_png(render(spec, variant_for(spec, s), s), PUBLIC / "icons" / f"icon-{s}.png")
    save_png(render(spec, fitted(full, MASKABLE_SAFE_RADIUS), 512), PUBLIC / "icons" / "icon-maskable-512.png")

    colors = spec["colors"]
    manifest = {
        "name": "Mailwoman",
        "short_name": "Mailwoman",
        "theme_color": colors["field"],
        "background_color": colors["field"],
        "icons": [
            {"src": "icons/icon-192.png", "sizes": "192x192", "type": "image/png", "purpose": "any"},
            {"src": "icons/icon-512.png", "sizes": "512x512", "type": "image/png", "purpose": "any"},
            {"src": "icons/icon-maskable-512.png", "sizes": "512x512", "type": "image/png", "purpose": "maskable"},
        ],
    }
    (PUBLIC / "manifest.webmanifest").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print("wrote apps/web/public/manifest.webmanifest")

    tauri_manifest = {
        "default": "mark-1024.png",
        "bg_color": colors["field"],
        "android_fg": "mark-android-fg-1024.png",
        "android_fg_scale": 100,
        "android_monochrome": "mark-android-mono-1024.png",
    }
    (BRAND / "tauri-icon.json").write_text(json.dumps(tauri_manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print("wrote branding/icon/tauri-icon.json")


def build_tauri(spec: dict) -> None:
    """Both shells: `cargo tauri icon` from tauri-icon.json, then the small sizes
    redrawn from their own variant, then app-icon.png = the 1024 master.

    `cargo tauri icon` scales the 1024 master down for every size. The square
    PNGs of 48 px and less and every entry of icon.ico are drawn again here from
    the variant for that size. icon.icns keeps Tauri's entries: Pillow cannot
    write its 16 px slot.
    """
    field = spec["colors"]["field"]
    for shell in SHELLS:
        out = shell / "icons"
        cmd = ["cargo", "tauri", "icon", str(BRAND / "tauri-icon.json"), "-o", str(out)]
        print("run", " ".join(cmd))
        subprocess.run(cmd, check=True, cwd=shell)
        bg = (out / "android" / "values" / "ic_launcher_background.xml").read_text(encoding="utf-8")
        if f">{field}<" not in bg:
            raise SystemExit(f"cargo tauri icon did not set the Android background to {field}: {bg}")
        for png in sorted(out.rglob("*.png")):
            if any(part in png.name for part in ("foreground", "monochrome", "round")):
                continue
            with Image.open(png) as old:
                size, mode = old.size, old.mode
            if size[0] == size[1] and size[0] <= 48:
                save_png(render(spec, variant_for(spec, size[0]), size[0]).convert(mode), png)
        with Image.open(out / "icon.ico") as ico:
            ico_sizes = sorted(s for s, _ in ico.info["sizes"])
        ico_images = [render(spec, variant_for(spec, s), s) for s in ico_sizes]
        ico_images[-1].save(out / "icon.ico", format="ICO", sizes=[(s, s) for s in ico_sizes],
                            append_images=ico_images[:-1])
        print(f"wrote {(out / 'icon.ico').relative_to(ROOT)} {ico_sizes}")
        shutil.copyfile(BRAND / "mark-1024.png", shell / "app-icon.png")
        print(f"wrote {(shell / 'app-icon.png').relative_to(ROOT)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--spec", type=Path, default=BRAND / "mark.json")
    parser.add_argument("--preview", type=Path, help="write a contact sheet to this directory and stop")
    parser.add_argument("--tauri", action="store_true", help="also regenerate both Tauri icon sets")
    args = parser.parse_args()
    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    check_colors(spec)
    if args.preview:
        print(f"wrote {preview(spec, args.preview, args.spec.stem)}")
        return 0
    build(spec)
    if args.tauri:
        build_tauri(spec)
    return 0


if __name__ == "__main__":
    sys.exit(main())
