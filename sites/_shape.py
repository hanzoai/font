"""Reshape Zen outlines toward a drawn mark. Three controls, each a real defect it fixes.

The premise this replaces: "no setting of this font reaches the drawn LUX." True of
SETTINGS — weight, width and tracking are the only knobs a font exposes, and
bar-to-cap is invariant under all of them. But a wordmark is outlines, and outlines
can be reshaped. These are the three operations that close the measured gaps:

FLATTEN — Zen's `U` drops 16 units below the baseline. That is correct type design:
  a round form must overshoot or it reads short beside a flat one. The drawn LUX has
  no overshoot at all — its U is geometric, bottom dead flat on the same line as the
  L. So the round-form convention is exactly wrong here, and it is why the U and the
  L foot looked "too low". Intersecting with the baseline/cap band cuts it clean.

EMBOLDEN — bar-to-cap is 0.221 at Zen's heaviest against the drawn 0.266, and no
  scale changes it: scaling multiplies bar and cap together. A DILATION does not.
  Squash vertically by s, dilate uniformly by r, unsquash: a vertical stem gains 2r
  while a horizontal bar gains 2r/s. One knob for weight, one for how much of it
  lands on the horizontals — which is the axis Zen does not have.

BASELINE — normalise to baseline-and-cap, never to ink bounds. Ink bounds include
  the overshoot, so the earlier comparison scaled the two marks by different amounts
  and then blamed the difference on the font.

Run: python3 _shape.py <text> <wght> <scaleX> <track> <flatten> <r> <s>
"""
import sys
import pathops
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.misc.transform import Transform

FONT = __file__.rsplit('/', 2)[0] + '/packages/zen/dist/fonts/zen-sans/Zen-Variable.ttf'


def glyph_path(gs, name):
    p = pathops.Path()
    gs[name].draw(p.getPen(glyphSet=gs))
    return p


def scaled(p, sx, sy):
    q = pathops.Path()
    p.draw(pathops.PathPen(q))
    q.transform(sx, 0.0, 0.0, sy, 0.0, 0.0)
    return q


def dilate(p, r, s):
    """Anisotropic outward offset: verticals gain 2r, horizontals gain 2r/s."""
    if r <= 0:
        return p
    q = scaled(p, 1.0, s)                      # squash
    edge = pathops.Path()
    q.draw(pathops.PathPen(edge))
    # A stroke centred on the contour reaches r outward and r inward; union with the
    # fill keeps only the outward half, which is the offset we want.
    edge.stroke(2 * r, pathops.LineCap.BUTT_CAP, pathops.LineJoin.MITER_JOIN, 4.0)
    out = pathops.Path()
    pathops.union([q, edge], out.getPen())
    return scaled(out, 1.0, 1.0 / s)           # unsquash


def band(p, lo, hi, wide):
    """Cut everything outside the baseline/cap band — kills round-form overshoot."""
    rect = pathops.Path()
    pen = rect.getPen()
    pen.moveTo((-wide, lo)); pen.lineTo((wide, lo))
    pen.lineTo((wide, hi)); pen.lineTo((-wide, hi)); pen.closePath()
    return pathops.op(p, rect, pathops.PathOp.INTERSECTION)


def main():
    text, wght, sx, track = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
    flatten = sys.argv[5] == '1'
    r, s = float(sys.argv[6]), float(sys.argv[7])

    font = instancer.instantiateVariableFont(TTFont(FONT), {'wght': wght}, inplace=False)
    upem = font['head'].unitsPerEm
    cap = font['OS/2'].sCapHeight
    gs = font.getGlyphSet(); cm = font.getBestCmap(); hm = font['hmtx']

    pen_x, parts = 0.0, []
    for ch in text:
        gid = cm.get(ord(ch))
        if gid is None:
            sys.exit(f'no glyph for {ch!r}')
        p = glyph_path(gs, gid)
        if r > 0:
            p = dilate(p, r, s)
        if flatten:
            p = band(p, 0, cap, upem * 4)
        parts.append((p, pen_x))
        pen_x += hm[gid][0] + track * upem

    # widest ink, for the viewBox only — vertical framing is baseline-and-cap so the
    # two marks are normalised identically
    x0, x1 = 1e9, -1e9
    for p, off in parts:
        b = p.bounds
        if b is None:
            continue
        x0 = min(x0, b[0] + off); x1 = max(x1, b[2] + off)
    if x0 > x1:
        sys.exit('no ink')

    w = (x1 - x0) * sx
    paths = []
    for p, off in parts:
        out = SVGPathPen(None)
        t = Transform(sx, 0, 0, -1, -x0 * sx, cap).translate(off, 0)
        p.draw(TransformPen(out, t))
        d = out.getCommands()
        if d:
            paths.append(f'  <path d="{d}"/>')

    print(f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.2f} {cap:.2f}" \
role="img" aria-label="{text}">
<g fill="currentColor" fill-rule="nonzero">
{chr(10).join(paths)}
</g>
</svg>''')


main()
