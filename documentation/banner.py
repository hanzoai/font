"""The repository banner, drawn with the fonts it is about.

Every letter is an outline rather than text, so the file needs no font and
renders the same on a repository page, in an editor and in a README that a
mirror is serving. Light and dark are the same geometry with the ink and the
ground swapped.

Run: python3 documentation/banner.py   (writes documentation/img/zen-banner--{light,dark}.svg)
"""
import pathlib
import re
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "documentation" / "img"

W, H = 1600, 560
INK = {"light": ("#fdfdfc", "#111113", "#8b8b86"), "dark": ("#08080a", "#ededea", "#6f6f6a")}

# One stop on the axis that holds all five cuts. Every pixel letter is a few
# hundred contours, so the strip of all five was a megabyte of path data for a
# picture — one cut says the same thing at a twentieth the size.
CUT = ("Triangle", 60)


def face(family, axis, value):
    path = ROOT / "fonts" / family / "variable" / f"{family}[{axis}].ttf"
    return instantiateVariableFont(TTFont(path), {axis: value}, inplace=False)


def round_path(d):
    """Whole units. The banner is 1600px wide; a hundredth of a font unit is
    not a thing anyone can see, and it is most of the file."""
    return re.sub(r"-?\d+\.\d+", lambda m: str(round(float(m.group()))), d)


def run(font, text, size, x, y, track=0.0):
    """One line of outlines, laid out on the font's own advances."""
    glyphs, cmap, hmtx = font.getGlyphSet(), font.getBestCmap(), font["hmtx"]
    upem = font["head"].unitsPerEm
    scale = size / upem
    out, pen_x = [], 0.0
    for ch in text:
        name = cmap.get(ord(ch))
        if name:
            pen = SVGPathPen(glyphs)
            glyphs[name].draw(pen)
            d = round_path(pen.getCommands())
            if d:
                out.append(f'<path transform="translate({x + pen_x * scale:.2f} {y}) '
                           f'scale({scale:.5f} {-scale:.5f})" d="{d}"/>')
            pen_x += hmtx[name][0]
        else:
            pen_x += upem * 0.5
        pen_x += track * upem
    return out, pen_x * scale


def banner(mode):
    ground, ink, dim = INK[mode]
    sans, mono = face("Zen", "wght", 220), face("ZenMono", "wght", 400)
    body = face("Zen", "wght", 500)

    parts = [f'<rect width="{W}" height="{H}" fill="{ground}"/>', f'<g fill="{ink}">']
    parts += run(sans, "Zen", 300, 110, 330)[0]
    parts += [f'</g><g fill="{dim}">']
    parts += run(body, "Sans · Mono · Pixel", 40, 118, 400)[0]
    parts += run(mono, "one variable file · 100–900 · font.hanzo.ai", 30, 118, 462)[0]
    parts += ['</g>', f'<g fill="{ink}">']
    parts += run(face("ZenPixel", "ELSH", CUT[1]), "Zen", 210, 900, 330)[0]
    parts += ['</g>']
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" '
            f'viewBox="0 0 {W} {H}" role="img">\n' + "\n".join(parts) + "\n</svg>\n")


OUT.mkdir(parents=True, exist_ok=True)
for mode in INK:
    p = OUT / f"zen-banner--{mode}.svg"
    p.write_text(banner(mode))
    print(f"wrote {p.relative_to(ROOT)}  {p.stat().st_size // 1024} KB")
