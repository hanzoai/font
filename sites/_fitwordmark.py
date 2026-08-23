"""Search the SHAPED parameter space for the closest Zen cut of the drawn LUX.

The earlier fit searched (wght, scaleX, track) and stalled at 12.5% residual,
because bar-to-cap is invariant under every one of them: weight is capped at the
heaviest master and scaling multiplies bar and cap together. That was a true
statement about SETTINGS and a false conclusion about OUTLINES — a wordmark is
path data, and path data can be reshaped.

So this searches two more axes, both from `_shape.py`:

  flatten   cut the round-form overshoot. Zen's U drops 16 units below the
            baseline (correct type design — a curve must overshoot or it reads
            short); the drawn U is dead flat on the same line as the L. That
            overshoot is what made the U and the L foot look "too low".
  r, s      anisotropic dilation. Squash by s, dilate by r, unsquash: verticals
            gain 2r, horizontals gain 2r/s. At s<1 the bar grows FASTER than the
            stem, which is the axis Zen does not have and the one the drawn mark
            needs (bar/cap 0.266 against Zen's 0.221 maximum).

Scoring renders both marks to a bitmap at ONE cap height and takes the residual
as a fraction of target ink — the same measure as the earlier fit, so the numbers
are comparable. Both are normalised baseline-to-cap, never ink bounds: ink bounds
include the overshoot, so the old comparison scaled the two marks differently and
then blamed the font for the difference.
"""
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
SHAPE = HERE / '_shape.py'
DRAWN = pathlib.Path('/home/z/work/lux/logo/svg/lux-wordmark-white.svg')

H = 200          # raster cap height, px
STEPS = 40       # curve flattening for the drawn mark


def shaped(wght, sx, track, flatten, r, s):
    out = subprocess.run(
        ['python3', str(SHAPE), 'LUX', str(wght), str(sx), str(track),
         '1' if flatten else '0', str(r), str(s)],
        capture_output=True, text=True, timeout=120)
    return out.stdout if out.returncode == 0 else None


# ── a tiny SVG path rasteriser ───────────────────────────────────────────────
# Only the commands these two files use (M/L/H/V/Q/C/Z, absolute and relative),
# which is why this is 60 lines rather than a dependency.

def tokens(d):
    return re.findall(r'([MmLlHhVvQqCcZz])|(-?\d*\.?\d+(?:[eE][-+]?\d+)?)', d)


def polys(d):
    out, cur, pen, start, cmd = [], [], (0.0, 0.0), (0.0, 0.0), None
    nums, toks = [], tokens(d)
    i = 0

    def bez(p0, pts, n=16):
        r = []
        for k in range(1, n + 1):
            t = k / n
            q = [p0] + list(pts)
            while len(q) > 1:
                q = [(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
                     for a, b in zip(q, q[1:])]
            r.append(q[0])
        return r

    def take(k):
        nonlocal i
        vals = []
        while len(vals) < k and i < len(toks):
            c, n = toks[i]
            if c:
                break
            vals.append(float(n)); i += 1
        return vals

    while i < len(toks):
        c, n = toks[i]
        if c:
            cmd = c; i += 1
        rel = cmd.islower()
        up = cmd.upper()
        if up == 'Z':
            if len(cur) > 2: out.append(cur)
            cur, pen = [], start
            continue
        if up == 'M':
            v = take(2)
            if len(v) < 2: break
            pen = (pen[0] + v[0], pen[1] + v[1]) if rel else (v[0], v[1])
            if len(cur) > 2: out.append(cur)
            cur, start = [pen], pen
            cmd = 'l' if rel else 'L'
        elif up == 'L':
            v = take(2)
            if len(v) < 2: break
            pen = (pen[0] + v[0], pen[1] + v[1]) if rel else (v[0], v[1])
            cur.append(pen)
        elif up == 'H':
            v = take(1)
            if not v: break
            pen = (pen[0] + v[0], pen[1]) if rel else (v[0], pen[1])
            cur.append(pen)
        elif up == 'V':
            v = take(1)
            if not v: break
            pen = (pen[0], pen[1] + v[0]) if rel else (pen[0], v[0])
            cur.append(pen)
        elif up == 'Q':
            v = take(4)
            if len(v) < 4: break
            p = [(pen[0] + v[0], pen[1] + v[1]), (pen[0] + v[2], pen[1] + v[3])] if rel \
                else [(v[0], v[1]), (v[2], v[3])]
            cur += bez(pen, p); pen = p[-1]
        elif up == 'C':
            v = take(6)
            if len(v) < 6: break
            p = [(pen[0] + v[0], pen[1] + v[1]), (pen[0] + v[2], pen[1] + v[3]),
                 (pen[0] + v[4], pen[1] + v[5])] if rel else \
                [(v[0], v[1]), (v[2], v[3]), (v[4], v[5])]
            cur += bez(pen, p); pen = p[-1]
        else:
            i += 1
    if len(cur) > 2: out.append(cur)
    return out


def paths_of(svg):
    out = []
    for d in re.findall(r'<path[^>]*\sd="([^"]+)"', svg):
        out += polys(d)
    for pts in re.findall(r'<polygon[^>]*\spoints="([^"]+)"', svg):
        v = [float(x) for x in re.findall(r'-?\d*\.?\d+', pts)]
        out.append(list(zip(v[0::2], v[1::2])))
    return out


def viewbox(svg):
    m = re.search(r'viewBox="([^"]+)"', svg)
    return [float(x) for x in m.group(1).split()]


def raster(svg, h=H):
    """Even-odd scanline fill, normalised so the viewBox height maps to h."""
    cs = paths_of(svg)
    if not cs: return None
    vb = viewbox(svg)
    k = h / vb[3]
    w = max(1, int(round(vb[2] * k)))
    grid = [bytearray(w) for _ in range(h)]
    for row in range(h):
        y = (row + 0.5) / k
        xs = []
        for c in cs:
            for a, b in zip(c, c[1:] + [c[0]]):
                if (a[1] <= y < b[1]) or (b[1] <= y < a[1]):
                    xs.append(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]))
        xs.sort()
        for j in range(0, len(xs) - 1, 2):
            lo = max(0, int(round(xs[j] * k)))
            hi = min(w, int(round(xs[j + 1] * k)))
            for x in range(lo, hi):
                grid[row][x] = 1
    return grid


def score(a, b):
    """Residual as a fraction of target ink, left-aligned, zero-padded."""
    w = max(len(a[0]), len(b[0]))
    diff = ink = 0
    for ra, rb in zip(a, b):
        for x in range(w):
            va = ra[x] if x < len(ra) else 0
            vb = rb[x] if x < len(rb) else 0
            diff += va ^ vb
            ink += vb
    return diff / ink if ink else 9.9


def main():
    target = raster(DRAWN.read_text())
    print(f'  target rasterised: {len(target[0])}x{len(target)}')

    def fit(w, sx, tr, fl, r, s):
        svg = shaped(w, sx, tr, fl, r, s)
        if not svg: return None
        g = raster(svg)
        return score(g, target) if g else None

    base = fit(845, 1.56, -0.04, False, 0, 1)
    print(f'  BASELINE (settings only, no shaping): {base * 100:.1f}%')

    flat = fit(845, 1.56, -0.04, True, 0, 1)
    print(f'  + flatten (kill the U overshoot):     {flat * 100:.1f}%')

    best = (flat, 845, 1.56, -0.04, 8, 0.5)
    print('\n  searching r (dilation) x s (anisotropy) x scaleX …')
    for r in (0, 4, 8, 12, 16, 20, 24):
        for s in (1.0, 0.7, 0.5, 0.35, 0.25):
            if r == 0 and s != 1.0: continue
            for sx in (1.40, 1.48, 1.56, 1.64):
                v = fit(845, sx, -0.04, True, r, s)
                if v is not None and v < best[0]:
                    best = (v, 845, sx, -0.04, r, s)
    print(f'  best so far: {best[0]*100:.1f}%  r={best[4]} s={best[5]} sx={best[2]}')

    print('\n  refining weight + tracking around it …')
    v0, _, sx0, tr0, r0, s0 = best
    for w in (700, 760, 800, 845, 900):
        for tr in (-0.06, -0.05, -0.04, -0.03, -0.02):
            v = fit(w, sx0, tr, True, r0, s0)
            if v is not None and v < best[0]:
                best = (v, w, sx0, tr, r0, s0)
    v, w, sx, tr, r, s = best
    print(f'\n  BEST: {v*100:.1f}%   wght {w} · scaleX {sx} · track {tr}em · '
          f'flatten · r={r} s={s}')
    print(f'  improvement over settings-only: {base*100:.1f}% -> {v*100:.1f}% '
          f'({(1 - v/base)*100:.0f}% of the gap closed)')

    out = shaped(w, sx, tr, True, r, s)
    if out:
        (HERE / 'dist' / 'wordmark').mkdir(parents=True, exist_ok=True)
        (HERE / 'dist' / 'wordmark' / 'lux-fitted.svg').write_text(out)
        print('  wrote dist/wordmark/lux-fitted.svg')


main()
