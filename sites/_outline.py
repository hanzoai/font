"""Emit a string of Zen as SVG path data. Called by wordmark.mjs.

A logotype must render where the font is absent — an email client, a partner's
deck, a favicon, a PDF someone prints. So this writes real outlines rather than a
`<text font-family="Zen">` element: the shapes ARE Zen, and nothing has to resolve
at draw time for them to be right.

Coordinates come out in a viewBox whose height is the ink, so the mark drops into
a layout by setting one dimension and never needs a magic offset to sit on a
baseline it cannot see.
"""
import sys
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.misc.transform import Transform

FONT = __file__.rsplit('/', 2)[0] + '/packages/zen/dist/fonts/zen-sans/Zen-Variable.ttf'


def main():
    text = sys.argv[1]
    wght = float(sys.argv[2])
    sx = float(sys.argv[3])
    track = float(sys.argv[4])

    font = instancer.instantiateVariableFont(TTFont(FONT), {'wght': wght}, inplace=False)
    upem = font['head'].unitsPerEm
    gs = font.getGlyphSet()
    cmap = font.getBestCmap()
    metrics = font['hmtx']

    # Lay the string out once, in font units, so tracking is a real advance rather
    # than a CSS property the SVG cannot carry.
    pen_x = 0.0
    placed = []
    for ch in text:
        gid = cmap.get(ord(ch))
        if gid is None:
            sys.exit(f'no glyph for {ch!r}')
        placed.append((gid, pen_x))
        pen_x += metrics[gid][0] + track * upem

    # Ink bounds, so the viewBox is the mark and not the em box. A logo aligned to
    # its em box carries invisible padding that differs per string, which is how a
    # lockup ends up needing a hand-tuned nudge at every size.
    from fontTools.pens.boundsPen import BoundsPen
    x0 = y0 = 1e9
    x1 = y1 = -1e9
    for gid, off in placed:
        bp = BoundsPen(gs)
        gs[gid].draw(bp)
        if bp.bounds is None:
            continue
        a, b, c, d = bp.bounds
        x0 = min(x0, a + off); x1 = max(x1, c + off)
        y0 = min(y0, b);       y1 = max(y1, d)
    if x0 > x1:
        sys.exit('no ink')

    w = (x1 - x0) * sx
    h = y1 - y0
    paths = []
    for gid, off in placed:
        pen = SVGPathPen(gs)
        # y flips because SVG grows downward; the translate lands ink at the origin.
        t = Transform(sx, 0, 0, -1, -x0 * sx, y1).translate(off, 0)
        gs[gid].draw(TransformPen(pen, t))
        d = pen.getCommands()
        if d:
            paths.append(f'  <path d="{d}"/>')

    body = '\n'.join(paths)
    print(f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.2f} {h:.2f}" \
role="img" aria-label="{text}">
  <!-- {text}, cut from Zen at wght {wght:.0f}{f", {sx}x wide" if sx != 1 else ""}, \
tracking {track}em. Outlines, not a <text> element, so the mark is right where the
       font is absent. Regenerate with sites/wordmark.mjs — do not hand-edit. -->
<g fill="currentColor" fill-rule="nonzero">
{body}
</g>
</svg>''')


main()
