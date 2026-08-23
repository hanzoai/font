"""Area-based optical kerning for Zen. Called by kern.mjs; see that file for why.

The measurement: for every scanline across the band where two glyphs both have ink,
the white between them is (advance_A - right_edge_A) + left_edge_B + kern. Average
that, clipping how deep into a bay we look, and you have a number that tracks what
the eye actually judges.

The target comes from Zen's own kern table. Pairs a designer has already tuned are
sitting at the white THEY chose; the median of those is the goal for pairs nobody
tuned. Learning it beats picking it — a constant I invented would be my taste
wearing the font's name.
"""
import sys, statistics
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools.pens.recordingPen import RecordingPen

FONT = __file__.rsplit('/', 2)[0] + '/dist/fonts/zen-sans/Zen-Variable.ttf'
STEP = 10           # scanline sampling, font units
DEPTH = 0.34        # how far into a bay we look, as a fraction of cap height


def flatten(font, ch):
    gs = font.getGlyphSet()
    cmap = font.getBestCmap()
    if ord(ch) not in cmap:
        return None, 0
    name = cmap[ord(ch)]
    pen = RecordingPen(); gs[name].draw(pen)
    polys, cur = [], []

    def bez(p0, pts, n=20):
        out = []
        for i in range(1, n + 1):
            t = i / n; q = [p0] + list(pts)
            while len(q) > 1:
                q = [(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t) for a, b in zip(q, q[1:])]
            out.append(q[0])
        return out

    for op, args in pen.value:
        if op == 'moveTo': cur = [args[0]]
        elif op == 'lineTo': cur.append(args[0])
        elif op == 'qCurveTo':
            pts = list(args)
            if pts[-1] is None: pts = pts[:-1]
            for i in range(len(pts) - 1):
                mid = ((pts[i][0] + pts[i + 1][0]) / 2, (pts[i][1] + pts[i + 1][1]) / 2)
                cur += bez(cur[-1], [pts[i], mid])
            if len(pts) > 1: cur += bez(cur[-1], [pts[-2], pts[-1]])
        elif op == 'curveTo': cur += bez(cur[-1], list(args))
        elif op in ('closePath', 'endPath'):
            if cur: polys.append(cur); cur = []
    if cur: polys.append(cur)
    return polys, gs[name].width


def profiles(polys):
    """{y: (min x, max x)} of ink, sampled every STEP units.

    On an ABSOLUTE grid — multiples of STEP — not from each glyph's own min y.
    Sampling from the glyph's own extreme gives every glyph a differently-phased
    set of scanlines, so a round O (which overshoots the baseline) and a pointed V
    share almost no y values and their overlap band comes out empty. That reads as
    "no outline" and silently drops the pair: OV, IG and GN all vanished from
    SOVEREIGN this way while SO and VE, whose phases happened to agree, worked.
    """
    if not polys: return {}
    ys = [p[1] for poly in polys for p in poly]
    lo = int(min(ys) // STEP) * STEP
    hi = int(max(ys) // STEP) * STEP
    out = {}
    for y in range(lo, hi + 1, STEP):
        xs = []
        for poly in polys:
            for a, b in zip(poly, poly[1:] + [poly[0]]):
                if (a[1] <= y < b[1]) or (b[1] <= y < a[1]):
                    xs.append(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]))
        if xs: out[y] = (min(xs), max(xs))
    return out


def white(cache, font, a, b, cap, kern=0.0):
    """Mean white between a and b, in font units. None if either glyph is absent."""
    for ch in (a, b):
        if ch not in cache:
            polys, adv = flatten(font, ch)
            cache[ch] = (profiles(polys), adv) if polys else (None, adv)
    (pa, adva), (pb, _) = cache[a], cache[b]
    if not pa or not pb: return None
    band = set(pa) & set(pb)
    if len(band) < 3: return None
    limit = cap * DEPTH
    tot = 0.0
    for y in band:
        gap = (adva - pa[y][1]) + pb[y][0] + kern
        tot += min(gap, limit)
    return tot / len(band)


def table_pairs(font):
    """Flat (left, right) -> xAdvance from every PairPos format-1 subtable."""
    out = {}
    if 'GPOS' not in font: return out
    rev = {g: c for c, g in font.getBestCmap().items()}
    for lk in font['GPOS'].table.LookupList.Lookup:
        for st in getattr(lk, 'SubTable', []):
            if getattr(st, 'Format', None) != 1 or not hasattr(st, 'PairSet'): continue
            for gname, ps in zip(st.Coverage.glyphs, st.PairSet):
                if gname not in rev: continue
                for r in ps.PairValueRecord:
                    if r.SecondGlyph not in rev: continue
                    v = getattr(r.Value1, 'XAdvance', 0)
                    if v: out[(chr(rev[gname]), chr(rev[r.SecondGlyph]))] = v
    return out


def main():
    text = sys.argv[1] if len(sys.argv) > 1 else 'SOVEREIGN'
    wght = float(sys.argv[2]) if len(sys.argv) > 2 else 845
    font = instancer.instantiateVariableFont(TTFont(FONT), {'wght': wght}, inplace=False)
    cap = font['OS/2'].sCapHeight
    upem = font['head'].unitsPerEm
    cache = {}

    # Learn the target from pairs the designer already tuned.
    tuned = table_pairs(font)
    caps = [(a, b, k) for (a, b), k in tuned.items() if a.isupper() and b.isupper()]
    samples = []
    for a, b, k in caps:
        w = white(cache, font, a, b, cap, k)
        if w is not None: samples.append(w)
    if not samples:
        print('no kerned cap pairs to learn from', file=sys.stderr); sys.exit(1)
    target = statistics.median(samples)
    print(f'learned from {len(samples)} hand-kerned cap pairs in Zen @ wght {wght:.0f}')
    print(f'target white = {target:.0f} units ({target/upem:.4f} em), '
          f'spread {statistics.quantiles(samples, n=4)[0]:.0f}–'
          f'{statistics.quantiles(samples, n=4)[2]:.0f}\n')

    print(f'{"pair":<6}{"has":>8}{"white":>8}{"want":>8}{"kern":>9}   note')
    print('-' * 62)
    css = []
    for a, b in zip(text, text[1:]):
        have = tuned.get((a, b), 0)
        w = white(cache, font, a, b, cap, have)
        if w is None:
            print(f'{a+b:<6}{"—":>8}{"—":>8}{"—":>8}{"—":>9}   no outline'); continue
        delta = target - w                      # units to ADD (negative = tighten)
        em = delta / upem
        flag = 'TIGHTEN' if em < -0.008 else ('open' if em > 0.008 else 'ok')
        print(f'{a+b:<6}{have:>8.0f}{w:>8.0f}{target:>8.0f}{em:>+9.4f}   {flag}')
        if abs(em) > 0.004:
            css.append((a + b, em))

    if css:
        print('\nApply — one span per pair that moved:')
        for pair, em in css:
            print(f'  {pair}: letter-spacing:{em:+.4f}em on the "{pair[0]}"')


main()
