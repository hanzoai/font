//! Fit one face to another by pixel residual, and search the parameter space.
//!
//! Render the target, render the candidate at (wght, scaleX, tracking), normalise
//! both to one cap height, and score the difference. Then search. The number is
//! what makes this honest: a target the candidate genuinely cannot reach reports a
//! bad residual instead of a plausible-looking stretch that only fails once it is
//! set beside the real thing.
//!
//! The rasteriser is the kerner's scanline walk with a fill rule, not a second
//! engine. Even-odd, which agrees with non-zero on any outline whose counters wind
//! against their shell — true of every face here, and the alternative is carrying
//! winding direction through the flattener to serve one hypothetical.

use crate::outline::{cut, Face};

/// Vertical subsamples per pixel row. Four is the usual anti-aliasing floor and
/// the residual is a sum over thousands of pixels, so more buys precision the
/// comparison cannot use.
const SUB: usize = 4;

pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub a: Vec<f32>,
}

/// Rasterise `text` at a cap height of `h` pixels.
///
/// `sx` scales horizontally — advances and vertical stems together, which is what
/// CSS `scaleX` does and why it cannot reach a genuinely condensed cut: it thins
/// the verticals while leaving the horizontals untouched.
pub fn render(face: &Face, text: &str, h: usize, sx: f32, track: f32) -> Option<Grid> {
    let scale = h as f32 / face.cap;
    let mut placed = Vec::new();
    let mut pen = 0.0f32;
    for ch in text.chars() {
        let g = face.glyph(ch)?;
        placed.push((g.advance, pen, ch));
        pen += g.advance + track * face.upem;
    }
    let glyphs: Vec<_> = text.chars().filter_map(|c| face.glyph(c)).collect();
    if glyphs.len() != placed.len() {
        return None;
    }

    // Ink bounds, so the grid is tight and two renders align on content rather
    // than on whatever side bearing each face happens to carry.
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (g, &(_, off, _)) in glyphs.iter().zip(&placed) {
        for c in &g.contours {
            for p in c {
                x0 = x0.min(p.0 + off);
                x1 = x1.max(p.0 + off);
                y0 = y0.min(p.1);
                y1 = y1.max(p.1);
            }
        }
    }
    if x0 > x1 {
        return None;
    }

    let w = (((x1 - x0) * scale * sx).ceil() as usize).max(1);
    let hh = (((y1 - y0) * scale).ceil() as usize).max(1);
    let mut a = vec![0.0f32; w * hh];

    for row in 0..hh {
        for s in 0..SUB {
            // font-unit y for this subsample, top-down
            let fy = y1 - (row as f32 + (s as f32 + 0.5) / SUB as f32) / scale;
            let mut spans: Vec<f32> = Vec::new();
            for (g, &(_, off, _)) in glyphs.iter().zip(&placed) {
                for x in cut(g, fy) {
                    spans.push((x + off - x0) * scale * sx);
                }
            }
            spans.sort_by(|p, q| p.partial_cmp(q).unwrap());
            for pair in spans.chunks_exact(2) {
                let (l, r) = (pair[0].max(0.0), pair[1].min(w as f32));
                if r <= l {
                    continue;
                }
                let (li, ri) = (l.floor() as usize, (r.ceil() as usize).min(w));
                for px in li..ri {
                    // partial coverage at the two ends, full in between
                    let cov = (r.min(px as f32 + 1.0) - l.max(px as f32)).clamp(0.0, 1.0);
                    a[row * w + px] += cov / SUB as f32;
                }
            }
        }
    }
    for v in a.iter_mut() {
        *v = v.min(1.0);
    }
    Some(Grid { w, h: hh, a })
}

/// Residual as a fraction of target ink. 0 is identical.
pub fn score(cand: &Grid, target: &Grid) -> f32 {
    let w = cand.w.max(target.w);
    let h = cand.h.max(target.h);
    let (mut diff, mut ink) = (0.0f32, 0.0f32);
    for y in 0..h {
        for x in 0..w {
            let c = if y < cand.h && x < cand.w { cand.a[y * cand.w + x] } else { 0.0 };
            let t = if y < target.h && x < target.w { target.a[y * target.w + x] } else { 0.0 };
            diff += (c - t).abs();
            ink += t;
        }
    }
    if ink <= 0.0 {
        f32::MAX
    } else {
        diff / ink
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fit {
    pub wght: f32,
    pub sx: f32,
    pub track: f32,
    pub residual: f32,
}

/// Coarse grid, then refine around the winner.
///
/// `face_at` builds the candidate face at a weight — the caller owns the bytes and
/// the instancing, so this stays a search and not a font loader.
pub fn search<F>(mut face_at: F, target: &Grid, h: usize, text: &str) -> Option<Fit>
where
    F: FnMut(f32) -> Option<Face<'static>>,
{
    let mut best: Option<Fit> = None;
    let try_one = |best: &mut Option<Fit>, w: f32, sx: f32, tr: f32, face: &Face| {
        if let Some(g) = render(face, text, h, sx, tr) {
            let r = score(&g, target);
            if best.map_or(true, |b| r < b.residual) {
                *best = Some(Fit { wght: w, sx, track: tr, residual: r });
            }
        }
    };

    let mut w = 200.0;
    while w <= 900.0 {
        if let Some(face) = face_at(w) {
            let mut sx = 0.80;
            while sx <= 1.85 {
                for tr in [-0.04, -0.02, 0.0, 0.02, 0.05] {
                    try_one(&mut best, w, sx, tr, &face);
                }
                sx += 0.05;
            }
        }
        w += 50.0;
    }

    let b = best?;
    let mut w = (b.wght - 45.0).max(100.0);
    while w <= (b.wght + 45.0).min(900.0) {
        if let Some(face) = face_at(w) {
            let mut sx = (b.sx - 0.05).max(0.5);
            while sx <= b.sx + 0.051 {
                let mut tr = b.track - 0.02;
                while tr <= b.track + 0.021 {
                    try_one(&mut best, w, sx, tr, &face);
                    tr += 0.01;
                }
                sx += 0.01;
            }
        }
        w += 10.0;
    }
    best
}
