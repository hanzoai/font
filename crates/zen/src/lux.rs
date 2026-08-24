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

pub const NO_THICKEN: &str = "XVWAKZ/";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Features {
    pub l_stem: f32,
    pub l_foot: f32,
    pub u_stem: f32,
    pub u_bottom: f32,
    pub width: f32,
}

impl Features {
    pub fn named(&self) -> [(&'static str, f32); 5] {
        [
            ("L stem", self.l_stem),
            ("L foot", self.l_foot),
            ("U stem", self.u_stem),
            ("U bottom", self.u_bottom),
            ("width", self.width),
        ]
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

    Features {
        l_stem: ls.first().copied().unwrap_or(0.0) * scale_x,
        l_foot: column(m, at(P_L_FOOT), cap),
        u_stem: us.first().copied().unwrap_or(0.0) * scale_x,
        u_bottom: column(m, at(P_U_MID), cap),
        width: w / cap * scale_x,
    }
}

/// The drawn LUX, measured once. Hard numbers rather than a re-read of the SVG:
/// the artwork is 633 bytes of geometry in another repo, and a fit that silently
/// changes when someone edits that file is worse than one that fails to compile.
pub const DRAWN: Features = Features {
    l_stem: 0.2967,
    l_foot: 0.2667,
    u_stem: 0.3033,
    u_bottom: 0.2633,
    width: 3.7067,
};

/// The bars carry the complaint, so they outweigh the stems. Width is not scored —
/// `scale_x` satisfies it exactly by construction.
const W_BAR: f32 = 1.4;
const W_STEM: f32 = 1.0;

fn deviation(got: &Features) -> f32 {
    let d = |a: f32, b: f32, w: f32| if a <= 0.0 || b <= 0.0 { 9.9 } else { (a / b).ln().abs() * w };
    d(got.l_foot, DRAWN.l_foot, W_BAR)
        .max(d(got.u_bottom, DRAWN.u_bottom, W_BAR))
        .max(d(got.l_stem, DRAWN.l_stem, W_STEM))
        .max(d(got.u_stem, DRAWN.u_stem, W_STEM))
}

#[derive(Clone, Copy, Debug)]
pub struct Cut {
    pub wght: f32,
    pub scale_x: f32,
    pub track: f32,
    pub thicken: f32,
    pub deviation: f32,
    pub got: Features,
}

/// Search (wght, thicken) and SOLVE scale_x. Two nested loops, not three: the
/// mark must land on the drawn width, and `scale_x` is the only control that
/// changes width — so it is determined, never searched.
pub fn fit(bytes: &[u8], track: f32) -> Option<Cut> {
    let mut best: Option<Cut> = None;
    for wi in 0..=14 {
        let wght = 500.0 + wi as f32 * 25.0;
        let face = Face::new(bytes, wght).ok()?;
        for ti in 0..=16 {
            let thicken = ti as f32 * 6.0;
            let m = mark(&face, "LUX", track, true, thicken, NO_THICKEN).ok()?;
            let raw = features(&m, 1.0);
            if raw.width <= 0.0 {
                continue;
            }
            let scale_x = DRAWN.width / raw.width;
            if !(0.9..=2.2).contains(&scale_x) {
                continue;
            }
            let got = features(&m, scale_x);
            let dev = deviation(&got);
            if best.map_or(true, |b| dev < b.deviation) {
                best = Some(Cut { wght, scale_x, track, thicken, deviation: dev, got });
            }
        }
    }
    best
}
