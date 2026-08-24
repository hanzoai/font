//! Reshape Zen outlines toward a drawn mark. Two operations, each a measured defect.
//!
//! The premise this replaces: "no setting of this font reaches the drawn LUX."
//! True of SETTINGS — `wght` moves stems and bars together, `scaleX` moves stems
//! and not bars, and there is no width axis — but a wordmark is outlines, and
//! outlines can be reshaped. Measured against the drawn mark, normalised to cap:
//!
//! ```text
//!            drawn     Zen 845 x 1.40
//!   L stem   0.2967    0.3625   +22%   too fat
//!   L foot   0.2667    0.2100   -21%   too thin   <- the complaint
//!   U bottom 0.2633    0.1867   -29%   too thin   <- the complaint
//! ```
//!
//! bar/stem is 0.90 on the drawn mark against 0.58 on Zen: it is nearly MONOLINE,
//! and that is a shape no weight reaches.
//!
//! NEITHER OPERATION NEEDS A POLYGON BOOLEAN, which is why this file has no such
//! dependency and no rasterise-and-trace step. `flatten` clamps y. `thicken` keeps
//! each OUTER contour and a copy shifted up by d — under nonzero winding that pair
//! already IS their union, so the renderer does the boolean — and raises each
//! COUNTER's floor by d, which is that hole's half of the same union. A
//! vertical stem's edges are at the same x in both copies, so its WIDTH is
//! untouched and so is the mark's advance. That decouples the three controls:
//!
//! ```text
//!   wght      overall weight — bars and stems together
//!   thicken   the horizontal bars ONLY
//!   scaleX    the stems and the width — no vertical measurement at all
//! ```
//!
//! Measured after: every feature within 3.2%, from +22% / -21% / -29%.

use crate::outline::Glyph;

/// Is (x, y) inside the glyph? NONZERO winding, counting signed crossings to the
/// left — the rule a font is drawn for and the rule the SVG is rendered with.
///
/// Odd-even would be the shorter code and it is wrong here: `thicken` overlaps two
/// copies of the same contour, and under odd-even their intersection cancels to a
/// hole. The same rule has to hold in the measurement and in the renderer or the
/// numbers describe a shape nobody sees.
pub fn inside(g: &Glyph, x: f32, y: f32) -> bool {
    let mut w = 0i32;
    for c in &g.contours {
        let n = c.len();
        for i in 0..n {
            let (a, b) = (c[i], c[(i + 1) % n]);
            let up = a.1 <= y && y < b.1;
            let down = b.1 <= y && y < a.1;
            if !(up || down) {
                continue;
            }
            if a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1) < x {
                w += if up { 1 } else { -1 };
            }
        }
    }
    w != 0
}

/// Clamp every point into the baseline/cap band.
///
/// Zen's `U` drops 16 units below the baseline. That is correct type design — a
/// round form must overshoot or it reads short beside a flat one — and it is
/// exactly wrong here, because the drawn U is geometric with its bottom dead flat
/// on the L's line. Clamping turns the overshoot into that flat.
pub fn flatten(g: &Glyph, lo: f32, hi: f32) -> Glyph {
    Glyph {
        contours: g
            .contours
            .iter()
            .map(|c| c.iter().map(|&(x, y)| (x, y.clamp(lo, hi))).collect())
            .collect(),
        advance: g.advance,
    }
}

/// Signed area. Positive and negative windings are outer contours and counters;
/// which sign means which depends on the font, so the rule is "the majority sign
/// is outer" rather than a constant that is right for TrueType and wrong for CFF.
fn area(c: &[(f32, f32)]) -> f32 {
    let n = c.len();
    (0..n).map(|i| {
        let (a, b) = (c[i], c[(i + 1) % n]);
        a.0 * b.1 - b.0 * a.1
    }).sum::<f32>() / 2.0
}

/// Add `d` to every horizontal bar; leave every vertical stem and the advance alone.
///
/// The union of the glyph with a copy of itself shifted UP by d — the Minkowski
/// sum with a vertical segment. A horizontal bar spanning [y, y+t] becomes
/// [y, y+t+d]; a vertical stem's edges are at the same x in both, so its width
/// and the advance do not move.
///
/// OUTER contours and COUNTERS are handled differently. Under nonzero winding two
/// overlapping outers already read as their union, so the outer half is free.
/// Duplicating a COUNTER is not: the shifted hole would punch ink out of the
/// original shape. The union's hole is `H ∩ (H + d)` — the part surviving in both
/// copies — which for a counter that does not double back is the counter with its
/// floor raised by d. A clamp, no boolean.
///
/// WHAT THIS OPERATION IS FOR, and what it is not:
///
/// It is exact on a HORIZONTAL BAR, which is the whole point. It DAMAGES AN
/// ANGLED OR CURVED TERMINAL: a terminal cut on the diagonal, unioned with itself
/// shifted up, ends in a step, because the two copies' ends do not align. So it
/// is a wordmark operation on chosen letters, not a face-wide one.
///
/// LUX is clean because L, U and X terminate flat or vertically. Measured on the
/// rest of the alphabet at the amount the wide transform would need (d=90): G, S,
/// 2, 3, 5 and 9 all step at their terminals. Counters were the first suspect and
/// were a real bug, fixed above — but fixing them changed nothing, because the
/// defect was never the holes. Do not reach for this to even out running text.
pub fn thicken(g: &Glyph, d: f32) -> Glyph {
    if d <= 0.0 {
        return Glyph { contours: g.contours.clone(), advance: g.advance };
    }
    let areas: Vec<f32> = g.contours.iter().map(|c| area(c)).collect();
    let outer_sign = {
        let pos: f32 = areas.iter().filter(|a| **a > 0.0).sum();
        let neg: f32 = areas.iter().filter(|a| **a < 0.0).map(|a| -a).sum();
        if pos >= neg { 1.0 } else { -1.0 }
    };

    let mut contours = Vec::with_capacity(g.contours.len() * 2);
    for (c, a) in g.contours.iter().zip(&areas) {
        if a * outer_sign > 0.0 {
            contours.push(c.clone());
            contours.push(c.iter().map(|&(x, y)| (x, y + d)).collect());
        } else {
            let floor = c.iter().fold(f32::MAX, |m, p| m.min(p.1)) + d;
            contours.push(c.iter().map(|&(x, y)| (x, y.max(floor))).collect());
        }
    }
    Glyph { contours, advance: g.advance }
}

/// One shaped, positioned run of glyphs.
pub struct Mark {
    pub glyphs: Vec<(Glyph, f32)>,
    pub cap: f32,
}

/// Cut `text` from a face and apply the shaping.
///
/// `d` is the bar amount; `diag` scales it for diagonal letters (see DIAGONAL).
/// Letters built from diagonals rather than stems and bars.
///
/// A diagonal at angle θ to the horizontal gains `d·cot θ` in its horizontal
/// measurement, not `d`, so one amount cannot serve both. Feeding the X the
/// bars' `d` overshot its stroke by 6.3%; excluding it entirely — the version
/// before that — left it THINNER than the unshaped cut once the fit dropped the
/// weight to thin the stems. `diag` scales `d` for these, and is searched.
pub const DIAGONAL: &str = "XVWAKZxvwy/";

pub fn mark(
    face: &crate::outline::Face,
    text: &str,
    track: f32,
    flat: bool,
    d: f32,
    diag: f32,
) -> Result<Mark, String> {
    let mut glyphs = Vec::new();
    let mut pen = 0.0_f32;
    for ch in text.chars() {
        let g = face.glyph(ch).ok_or_else(|| format!("no glyph for {ch:?}"))?;
        let adv = g.advance;
        let amount = if DIAGONAL.contains(ch) { d * diag } else { d };
        let g = if amount > 0.0 { thicken(&g, amount) } else { g };
        // AFTER thickening, never before: raising a bar also raises the top of the
        // letter, and this is what puts it back on the cap line.
        let g = if flat { flatten(&g, 0.0, face.cap) } else { g };
        glyphs.push((g, pen));
        pen += adv + track * face.upem;
    }
    Ok(Mark { glyphs, cap: face.cap })
}

impl Mark {
    /// Ink extent in x, with each glyph's pen offset applied.
    pub fn bounds(&self) -> Option<(f32, f32)> {
        self.box2().map(|(x0, _, x1, _)| (x0, x1))
    }

    /// Ink extent in both axes: (x0, y0, x1, y1).
    pub fn box2(&self) -> Option<(f32, f32, f32, f32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (g, off) in &self.glyphs {
            for c in &g.contours {
                for p in c {
                    x0 = x0.min(p.0 + off);
                    x1 = x1.max(p.0 + off);
                    y0 = y0.min(p.1);
                    y1 = y1.max(p.1);
                }
            }
        }
        if x0 > x1 { None } else { Some((x0, y0, x1, y1)) }
    }

    /// SVG, framed to INK and scaled in x.
    ///
    /// Ink, not baseline-to-cap: a logo aligned to its em box carries invisible
    /// padding that differs per string, so a lockup ends up needing a hand-tuned
    /// nudge at every size. `zoo` is the case that proves it — all three letters
    /// are x-height, so a cap-height frame gives the file 180 units of empty air
    /// above the mark and anyone sizing it by height draws it a third too small.
    /// (The FIT normalises baseline-to-cap and does not come through here, so the
    /// two conventions do not collide.)
    ///
    /// Polylines, not curves: the outline arrives here already flattened, and
    /// re-fitting beziers to it would be a second approximation of a shape that is
    /// only approximate once. `simplify` keeps the file small instead.
    pub fn svg(&self, scale_x: f32, tol: f32) -> String {
        let (x0, y0, x1, y1) = self.box2().unwrap_or((0.0, 0.0, 0.0, 0.0));
        let w = (x1 - x0) * scale_x;
        let h = y1 - y0;
        let mut paths = String::new();
        for (g, off) in &self.glyphs {
            let mut d = String::new();
            for c in &g.contours {
                // y flips because SVG grows downward; cap becomes y = 0.
                let pts: Vec<(f32, f32)> = c
                    .iter()
                    .map(|&(x, y)| (((x + off) * scale_x) - x0 * scale_x, y1 - y))
                    .collect();
                let pts = simplify(&pts, tol);
                if pts.len() < 3 {
                    continue;
                }
                d.push_str(&format!("M{:.1} {:.1}", pts[0].0, pts[0].1));
                for p in &pts[1..] {
                    d.push_str(&format!("L{:.1} {:.1}", p.0, p.1));
                }
                d.push('Z');
            }
            if !d.is_empty() {
                paths.push_str(&format!("  <path d=\"{d}\"/>\n"));
            }
        }
        // fill-rule is load bearing, not decoration: `thicken` leaves two
        // overlapping copies of every contour, and odd-even would cancel their
        // intersection to a hole.
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w:.2} {h:.2}\" role=\"img\">\n\
             <g fill=\"currentColor\" fill-rule=\"nonzero\">\n{paths}</g>\n</svg>\n"
        )
    }
}

/// Ramer–Douglas–Peucker. A flattened curve carries 20 points per segment and a
/// logo does not need them; at 1 unit of a 1000-unit em the discarded ones are
/// below the precision the file is written at.
fn simplify(pts: &[(f32, f32)], tol: f32) -> Vec<(f32, f32)> {
    if pts.len() < 3 || tol <= 0.0 {
        return pts.to_vec();
    }
    let (a, b) = (pts[0], pts[pts.len() - 1]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    let (mut worst, mut at) = (0.0_f32, 0usize);
    for (i, p) in pts.iter().enumerate().take(pts.len() - 1).skip(1) {
        let dist = if len == 0.0 {
            ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt()
        } else {
            ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / len
        };
        if dist > worst {
            worst = dist;
            at = i;
        }
    }
    if worst <= tol {
        return vec![a, b];
    }
    let mut out = simplify(&pts[..=at], tol);
    out.pop();
    out.extend(simplify(&pts[at..], tol));
    out
}
