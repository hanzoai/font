#!/usr/bin/env python3
"""Cut the shade dither back to the character cell.

The shades are a checkerboard of circles on a 100-unit lattice: big circles on
x = 100,300,500, small ones on x = 0,200,400. Circles whose centre sits on a
cell edge are drawn whole, so ink runs past the cell -- 54 units below and 81
above the 1300-unit line box, which paints over the lines either side. The
escaping ink is exactly what the neighbouring cell contributes, so cutting each
edge circle at the edge leaves tiled output unchanged and stops the bleed.

Centres sit exactly on the edges, so every cut is a half or a quarter circle:
two arcs plus the straight edge, no arc splitting needed.
"""
import re
import sys
from pathlib import Path

CELL = (0, -340, 600, 960)          # xMin, yMin, xMax, yMax
KAPPA = 0.5523                      # circle-to-bezier constant
SOURCES = Path(__file__).resolve().parent.parent / "sources"
GLYPHS = ("shadelight", "shademedium", "shadedark")

NODE = re.compile(r"\(([-\d.]+),([-\d.]+),(\w+)\)")
BLOCK = re.compile(r"nodes = \(\n(.*?)\n\);", re.S)


def arc(cx, cy, r, quarter):
    """Two off-curve points plus the on-curve end of one quadrant arc.

    `quarter` runs 0..3 counter-clockwise from the bottom of the circle, the
    same direction the sources are drawn in.
    """
    k = round(r * KAPPA)
    ends = [(cx + r, cy), (cx, cy + r), (cx - r, cy), (cx, cy - r)]
    ctrl = [[(cx + k, cy - r), (cx + r, cy - k)],
            [(cx + r, cy + k), (cx + k, cy + r)],
            [(cx - k, cy + r), (cx - r, cy + k)],
            [(cx - r, cy - k), (cx - k, cy - r)]]
    return ctrl[quarter], ends[quarter]


def circle(cx, cy, r, quarters, close):
    """Nodes for a run of consecutive quadrants closed by straight edges.

    The list ends on an on-curve point: Glyphs treats the last node as the
    start, and the leading off-curves belong to the arc arriving at the first
    on-curve.
    """
    nodes = []
    for i, q in enumerate(quarters):
        (c1, c2), end = arc(cx, cy, r, q)
        last = i == len(quarters) - 1
        nodes += [(*c1, "o"), (*c2, "o"), (*end, "c" if last and close else "cs")]
    nodes += [(*p, "l") for p in close]
    return nodes


def clip(cx, cy, r):
    """Quadrants and corner points that keep a circle inside the cell."""
    x0, y0, _, y1 = CELL
    left, bottom, top = cx - r < x0, cy - r < y0, cy + r > y1
    if left and bottom:                       # corner: keep upper right
        return circle(cx, cy, r, [1], [(cx, cy), (cx + r, cy)])
    if left:                                  # keep right half
        return circle(cx, cy, r, [0, 1], [(cx, cy - r)])
    if bottom:                                # keep top half
        return circle(cx, cy, r, [1, 2], [(cx + r, cy)])
    if top:                                   # keep bottom half
        return circle(cx, cy, r, [3, 0], [(cx - r, cy)])
    return None


def fmt(nodes, indent):
    return ",\n".join(f"{indent}({x:g},{y:g},{t})" for x, y, t in nodes)


def process(path):
    text = path.read_text(encoding="utf-8")
    cut = 0

    def one(m):
        nonlocal cut
        pts = [(float(a), float(b), t) for a, b, t in NODE.findall(m.group(1))]
        xs, ys = [p[0] for p in pts], [p[1] for p in pts]
        cx, cy = (min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2
        r = (max(xs) - min(xs)) / 2
        nodes = clip(cx, cy, r)
        if nodes is None:
            return m.group(0)
        cut += 1
        indent = re.match(r"\s*", m.group(1)).group(0)
        return f"nodes = (\n{fmt(nodes, indent)}\n);"

    out = BLOCK.sub(one, text)
    if out != text:
        path.write_text(out, encoding="utf-8")
    return cut


def main():
    total = 0
    for pkg in sorted(SOURCES.glob("*.glyphspackage")):
        for name in GLYPHS:
            path = pkg / "glyphs" / f"{name}.glyph"
            if not path.exists():
                continue
            n = process(path)
            total += n
            print(f"{path.relative_to(SOURCES.parent)}: cut {n} circles")
    if not total:
        raise SystemExit("no circles cut -- sources already fitted?")


if __name__ == "__main__":
    sys.exit(main())
