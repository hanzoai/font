//! Fit Zen to the drawn LUX on the FEATURES, not on the pixel blob.
//!
//! `fit` scores a whole-mark coverage residual, and for this job that metric is
//! actively misleading: it is dominated by overall width and position, so a cut
//! that lands the width scores 17% while its bars run 21-29% thin. Minimising it
//! chose NO shaping at all. Bar thickness is the one thing a person notices, so
//! it is what this scores.
//!
//! The measurements are the numbers a type designer reads off the drawing — stem
//! width, bar thickness — each divided by CAP HEIGHT so the two marks compare at
//! any scale.
//!
//! PROBES SIT AT FIXED FRACTIONS OF WIDTH, not at letter boundaries, because the
//! drawn mark has no letter boundaries: its X tucks under the U's right shoulder
//! (the U spans x 18-38.7 of 63 and the X starts at 33.5), so no column between
//! them is empty and splitting on blank columns returns one letter, not three.

use crate::outline::Face;
use crate::shape::{inside, mark, Mark};

/// Fractions of total width to probe. Read off the drawn geometry.
const P_L_FOOT: f32 = 0.19;  // the L's arm, clear of the stem
const P_U_MID: f32 = 0.45;   // the centre of the U's bowl
const L_SPAN: f32 = 0.12;    // scan this far in for the L's stem — the arm starts after
const U_SPAN: (f32, f32) = (0.26, 0.62);  // the U, clear of both neighbours
// Read the stems at 0.65 cap, ABOVE the U's bowl. At 0.25 a horizontal row cuts
// the curving side at an angle and reports a run wider than the stem — which is a
// measurement artefact, and it was reporting the U 7 points fatter than the L on a
// mark whose two stems differ by half a point.
const HI: f32 = 0.65;



#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Features {
    pub l_stem: f32,
    pub l_foot: f32,
    pub l_arm: f32,
    pub u_stem: f32,
    pub u_bottom: f32,
    pub x_stroke: f32,
    pub width: f32,
}

impl Features {
    pub fn named(&self) -> [(&'static str, f32); 7] {
        [
            ("L stem", self.l_stem),
            ("L foot", self.l_foot),
            ("L arm", self.l_arm),
            ("U stem", self.u_stem),
            ("U bottom", self.u_bottom),
            ("X stroke", self.x_stroke),
            ("width", self.width),
        ]
    }

    /// How even the strokes are: widest over narrowest. 1.00 is monoline, which
    /// is what the drawn mark nearly is and what "keep every stroke the same
    /// width" asks for.
    pub fn evenness(&self) -> f32 {
        let s = [self.l_stem, self.l_foot, self.u_stem, self.u_bottom, self.x_stroke];
        let lo = s.iter().cloned().fold(f32::MAX, f32::min);
        let hi = s.iter().cloned().fold(f32::MIN, f32::max);
        if lo > 0.0 { hi / lo } else { 9.9 }
    }
}

/// Ink extents along a vertical line, in cap units.
fn column(m: &Mark, x: f32, cap: f32) -> f32 {
    // Walk the band in fine steps and count where ink is. Sampling rather than
    // solving because a bar's boundary can be a curve, and the count is what the
    // ratio needs — not the exact edge.
    let n = 400;
    let mut hits = 0;
    for i in 0..n {
        let y = cap * (i as f32 + 0.5) / n as f32;
        if m.glyphs.iter().any(|(g, off)| inside(g, x - off, y)) {
            hits += 1;
        }
    }
    hits as f32 / n as f32
}

/// The first ink run down a column, in cap units — the vertical chord of whatever
/// stroke that column crosses.
fn column_runs(m: &Mark, x: f32, cap: f32) -> f32 {
    let n = 600;
    let mut run = 0.0f32;
    for i in 0..n {
        let y = cap * (i as f32 + 0.5) / n as f32;
        if m.glyphs.iter().any(|(g, off)| inside(g, x - off, y)) {
            run += cap / n as f32;
        } else if run > 0.0 {
            return run / cap;
        }
    }
    run / cap
}

/// Widths of the ink runs along a horizontal line, in cap units.
fn row(m: &Mark, y: f32, x0: f32, x1: f32, cap: f32) -> Vec<f32> {
    let n = 1200;
    let (mut runs, mut run) = (Vec::new(), 0.0_f32);
    let step = (x1 - x0) / n as f32;
    for i in 0..n {
        let x = x0 + step * (i as f32 + 0.5);
        if m.glyphs.iter().any(|(g, off)| inside(g, x - off, y)) {
            run += step;
        } else if run > 0.0 {
            runs.push(run / cap);
            run = 0.0;
        }
    }
    if run > 0.0 {
        runs.push(run / cap);
    }
    runs
}

/// Measure a shaped mark. `scale_x` is applied to the horizontal readings only —
/// it provably cannot change a vertical measurement, which is what lets the fit
/// solve for it instead of searching it.
pub fn features(m: &Mark, scale_x: f32) -> Features {
    let cap = m.cap;
    let (x0, x1) = m.bounds().unwrap_or((0.0, 0.0));
    let w = x1 - x0;
    let at = |f: f32| x0 + w * f;

    let hi = cap * HI;
    let ls = row(m, hi, x0, x0 + w * L_SPAN, cap);
    let us = row(m, hi, at(U_SPAN.0), at(U_SPAN.1), cap);

    // The L's arm, measured LOW where only the foot is present: its right end is
    // where the bar stops, and "how close the bar reaches to the U" is exactly
    // that number. Read as a fraction of total width so it is a proportion of the
    // lockup, not a length that moves with scaleX.
    let lo = cap * 0.10;
    let arm = row(m, lo, x0, at(0.34), cap);
    let l_arm = arm.first().copied().unwrap_or(0.0) / (w / cap);

    // The X's diagonal, read BELOW the crossing where the two arms are separate.
    // At mid height they merge into one run and the "stroke" measures the width
    // of the crossing instead — a number that tracks the letter, not the pen.
    //
    // PERPENDICULAR, not horizontal. A horizontal run across a diagonal is
    // w / sin(theta), so it grows as the diagonal lays down — and `scaleX` lays it
    // down. Measured that way, 845 x 1.40 and 625 x 1.545 both scored within 1% of
    // the drawn X while visibly differing, because the shallower one needs a
    // thinner pen to cut the same horizontal chord. For a straight stroke,
    // h = w/sin, v = w/cos, so 1/h^2 + 1/v^2 = 1/w^2 — the perpendicular falls out
    // of one horizontal reading and one vertical, with no angle to estimate.
    let xs = row(m, cap * 0.22, at(0.62), x1, cap);
    let h = if xs.is_empty() { 0.0 } else { xs.iter().sum::<f32>() / xs.len() as f32 * scale_x };
    let v = column_runs(m, at(0.72), cap);
    let x_stroke = if h > 0.0 && v > 0.0 { h * v / (h * h + v * v).sqrt() } else { 0.0 };

    Features {
        l_stem: ls.first().copied().unwrap_or(0.0) * scale_x,
        l_foot: column(m, at(P_L_FOOT), cap),
        l_arm,
        u_stem: us.first().copied().unwrap_or(0.0) * scale_x,
        u_bottom: column(m, at(P_U_MID), cap),
        x_stroke,
        width: w / cap * scale_x,
    }
}

/// Measure the drawn mark from its own SVG.
///
/// These were five hardcoded floats with a comment defending the choice. The
/// defence did not survive needing a sixth: deriving `x_stroke` by hand out of
/// 633 bytes of path data is the step that quietly produces a wrong target, and
/// a wrong target looks exactly like a bad fit. `svg.rs` costs seventy lines and
/// makes every number here re-derivable from the artwork.
pub fn drawn(src: &str, cap: f32) -> Option<Features> {
    crate::svg::as_mark(src, cap).map(|m| features(&m, 1.0))
}

/// What each feature is worth. The two bars and the X carry the complaint —
/// "the bottom of the L", "the U", "the X is thinner than before" — and the arm
/// is how far the bar reaches toward the U. Width is not scored: `scale_x`
/// satisfies it exactly by construction.
fn deviation(got: &Features, want: &Features) -> f32 {
    let d = |a: f32, b: f32, w: f32| if a <= 0.0 || b <= 0.0 { 9.9 } else { (a / b).ln().abs() * w };
    [
        (got.l_foot, want.l_foot, 1.4),
        (got.u_bottom, want.u_bottom, 1.4),
        // The heaviest weight in the set. A max-deviation objective will trade
        // any one feature to lower the worst, and the X is what it kept trading:
        // twice reported as reading thin, twice measured as within tolerance.
        // Reported beats tolerated.
        (got.x_stroke, want.x_stroke, 1.8),
        (got.l_arm, want.l_arm, 1.2),
        (got.l_stem, want.l_stem, 1.0),
        (got.u_stem, want.u_stem, 1.0),
    ]
    .iter()
    .fold(0.0f32, |acc, &(a, b, w)| acc.max(d(a, b, w)))
}

#[derive(Clone, Copy, Debug)]
pub struct Cut {
    pub wght: f32,
    pub scale_x: f32,
    pub track: f32,
    pub thicken: f32,
    /// The weight diagonal letters are cut at.
    pub wght_diag: f32,
    pub deviation: f32,
    pub got: Features,
}

/// Search (wght, thicken, diag, track) and SOLVE scale_x, against a target read
/// from `src`.
///
/// `scale_x` is never searched: the mark must land on the drawn width and it is
/// the only control that changes width, so it is determined by the others.
///
/// Tracking IS searched, because it moves a feature nothing else reaches. The
/// L's bar has to end near the U, and how near is the L-to-U gap; tightening it
/// shrinks the raw width, `scale_x` grows to compensate, and the arm's share of
/// the whole rises. Fixed at -0.04 the arm came out 7.6% short with no control
/// able to answer for it.
pub fn fit(bytes: &[u8], src: &str) -> Option<(Cut, Features)> {
    let probe = Face::new(bytes, 700.0).ok()?;
    let want = drawn(src, probe.cap)?;

    let mut best: Option<Cut> = None;
    for wi in 0..=14 {
        let wght = 475.0 + wi as f32 * 25.0;
        let face = Face::new(bytes, wght).ok()?;
        for ti in 0..=18 {
            let thicken = ti as f32 * 6.0;
            for di in 0..=8 {
                let wght_diag = 550.0 + di as f32 * 50.0;
                for tk in 0..=5 {
                    let track = -0.02 - tk as f32 * 0.015;
                            let dface = Face::new(bytes, wght_diag).ok()?;
                    let m = mark(&face, "LUX", track, true, thicken, Some(&dface)).ok()?;
                    let raw = features(&m, 1.0);
                    if raw.width <= 0.0 {
                        continue;
                    }
                    let scale_x = want.width / raw.width;
                    if !(0.9..=2.2).contains(&scale_x) {
                        continue;
                    }
                    let got = features(&m, scale_x);
                    let dev = deviation(&got, &want);
                    if best.map_or(true, |b: Cut| dev < b.deviation) {
                        best = Some(Cut { wght, scale_x, track, thicken, wght_diag, deviation: dev, got });
                    }
                }
            }
        }
    }
    best.map(|c| (c, want))
}

/// Stroke widths for a whole alphabet at one cut, so a shaping fitted to three
/// letters can be checked against the twenty-three it did not see.
///
/// `zen-wide` sets words, not just the wordmark — LUX CREDIT, SOVEREIGN — so a
/// `thicken` that makes LUX monoline and leaves E or S lumpy has moved the
/// problem rather than solved it. Two numbers per letter, both in cap units:
///
///   stem   the median horizontal ink run across the middle band
///   bar    the median vertical ink run down the letter
///
/// Median, not mean: a counter splits a row into several runs and an O's two
/// sides are the same stroke twice, while the mean is dragged by whichever
/// crossing is widest.
///
/// A STROKE IS THE SHORT DIMENSION, and the filter that enforces it is the whole
/// difference between a measurement and a number. Without it, E's crossbar is a
/// horizontal run spanning the letter and reads as a 1.02-cap "stem"; I is solid
/// down every column and reads as a 1.00-cap "bar"; A, M, V and W report their
/// diagonals as half-cap bars. The alphabet then scores 7.5 on a spread where the
/// real answer is near 1, and the number says nothing about the type.
pub struct Stroke {
    pub ch: char,
    pub stem: f32,
    pub bar: f32,
}

fn median(v: &mut Vec<f32>) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    v[v.len() / 2]
}

pub fn strokes(
    bytes: &[u8],
    text: &str,
    wght: f32,
    scale_x: f32,
    thicken: f32,
    diag: f32,
) -> Vec<Stroke> {
    let mut out = Vec::new();
    let Ok(face) = Face::new(bytes, wght) else { return out };
    let dface = (diag > 0.0).then(|| Face::new(bytes, diag).ok()).flatten();
    for ch in text.chars() {
        let from = if crate::shape::DIAGONAL.contains(ch) {
            dface.as_ref().unwrap_or(&face)
        } else {
            &face
        };
        let Ok(m) = mark(from, &ch.to_string(), 0.0, true, thicken, None) else { continue };
        // scale_x FIRST. It lays a diagonal down, so measuring before it and
        // multiplying after is only right for a stroke that is exactly vertical
        // — which is the one case that needed no correction. Applying it to the
        // geometry measures the letter that actually gets drawn.
        let m = Mark {
            glyphs: m.glyphs.iter()
                .map(|(g, off)| (crate::outline::Glyph {
                    contours: g.contours.iter()
                        .map(|c| c.iter().map(|&(x, y)| (x * scale_x, y)).collect())
                        .collect(),
                    advance: g.advance * scale_x,
                }, off * scale_x))
                .collect(),
            cap: m.cap,
        };
        let Some((x0, y0, x1, y1)) = m.box2() else { continue };
        let cap = m.cap;
        let (w, h) = (x1 - x0, y1 - y0);
        if w <= 0.0 || h <= 0.0 {
            continue;
        }
        let (mut stems, mut bars) = (Vec::new(), Vec::new());
        let n = 41;
        for i in 1..n {
            let y = y0 + h * i as f32 / n as f32;
            for j in 1..n {
                let x = x0 + w * j as f32 / n as f32;
                if !m.glyphs.iter().any(|(g, off)| inside(g, x - off, y)) {
                    continue;
                }
                let (pw, ang) = thinnest(&m, x, y);
                if pw <= 0.0 {
                    continue;
                }
                // The chord's own direction says which way the stroke runs: the
                // thinnest chord crosses the stroke, so the stroke is at right
                // angles to it. Near-horizontal chord -> upright stroke.
                if ang.cos().abs() >= std::f32::consts::FRAC_1_SQRT_2 {
                    stems.push(pw / cap)
                } else {
                    bars.push(pw / cap)
                }
            }
        }
        out.push(Stroke { ch, stem: median(&mut stems), bar: median(&mut bars) });
    }
    out
}

/// The stroke width at (x, y): the SHORTEST chord through the point, and the
/// angle it lies at.
///
/// The shortest chord through an interior point crosses the stroke square-on, so
/// its length is the pen width — for a stem, a bar, a diagonal or a curve alike,
/// with no angle to know in advance and no per-letter special case.
///
/// Two wrong answers preceded this one, and both were wrong by a lot. Reading
/// the HORIZONTAL chord gives w/sin(theta), which put X at 0.43 and Z at 0.56
/// against a 0.30 stem. Combining the horizontal and vertical chords as
/// 1/w^2 = 1/h^2 + 1/v^2 is exact only when BOTH cut across the stroke — on an
/// upright stem the vertical chord runs ALONG it, so that formula was fed the
/// stroke's LENGTH and returned 0.2841 for an L stem that measures 0.2977 flat
/// at every height. Sampling the angle out is the fix: nothing has to be
/// assumed about which way the stroke runs.
fn thinnest(m: &Mark, x: f32, y: f32) -> (f32, f32) {
    let Some((x0, y0, x1, y1)) = m.box2() else { return (0.0, 0.0) };
    let reach = (x1 - x0).max(y1 - y0);
    let step = reach / 700.0;
    let hit = |px: f32, py: f32| m.glyphs.iter().any(|(g, off)| inside(g, px - off, py));
    if !hit(x, y) {
        return (0.0, 0.0);
    }
    // 24 directions is every 7.5 degrees, so the worst a straight stroke can be
    // over-read is 1/cos(3.75 deg) — 0.2%, well under the differences at issue.
    let k = 24;
    let (mut best, mut at) = (f32::MAX, 0.0);
    for i in 0..k {
        let a = std::f32::consts::PI * i as f32 / k as f32;
        let (dx, dy) = (a.cos(), a.sin());
        let mut lo = 0.0;
        while lo < reach && hit(x - dx * (lo + step), y - dy * (lo + step)) {
            lo += step;
        }
        let mut hi = 0.0;
        while hi < reach && hit(x + dx * (hi + step), y + dy * (hi + step)) {
            hi += step;
        }
        let len = lo + hi;
        if len < best {
            best = len;
            at = a;
        }
    }
    (best, at)
}
