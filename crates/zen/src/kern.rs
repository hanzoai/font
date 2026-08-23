//! Optical kerning, computed from the outlines.
//!
//! A font's kern table is cut for TEXT. Set the same pairs at display size and the
//! round-to-round joins open up, because the eye judges the AREA of white between
//! two shapes and area grows with the square of the size while the kern grows
//! linearly. Uniform tracking cannot fix that: tightening enough to close S·O
//! crushes H·I.
//!
//! So measure the area. Across the band where two glyphs both have ink, the white
//! between them is `(advance − right edge of A) + left edge of B + kern`. Average
//! it, clip how deep into a bay we look, compare to a target.
//!
//! The target is LEARNED from the font's own kern table. Pairs a designer has
//! already tuned sit at the white THEY chose; the median of those is the goal for
//! pairs nobody tuned. Picking a constant instead would be substituting taste for
//! the designer's, wearing the font's name.

use crate::outline::{profile, Face, Glyph};
use std::collections::HashMap;

/// Scanline spacing, font units.
const STEP: i32 = 10;
/// How far into a bay we look, as a fraction of cap height. Without a ceiling the
/// open shoulder of a `T` swamps every pair it appears in.
const DEPTH: f32 = 0.34;

pub struct Pair {
    pub left: char,
    pub right: char,
    pub have: f32,
    pub white: f32,
    pub want: f32,
    /// Correction in em. Negative tightens.
    pub kern: f32,
}

struct Cache<'a> {
    face: &'a Face<'a>,
    glyphs: HashMap<char, Option<(Glyph, Vec<(i32, f32, f32)>)>>,
}

impl<'a> Cache<'a> {
    fn new(face: &'a Face<'a>) -> Self {
        Self { face, glyphs: HashMap::new() }
    }
    fn get(&mut self, ch: char) -> Option<&(Glyph, Vec<(i32, f32, f32)>)> {
        self.glyphs
            .entry(ch)
            .or_insert_with(|| self.face.glyph(ch).map(|g| {
                let p = profile(&g, STEP);
                (g, p)
            }))
            .as_ref()
    }

    /// Mean white between `a` and `b` in font units, or None if either is absent
    /// or they share too little vertical band to judge.
    fn white(&mut self, a: char, b: char, kern: f32, cap: f32) -> Option<f32> {
        let (ga, pa) = self.get(a)?;
        let (adv, pa) = (ga.advance, pa.clone());
        let (_, pb) = self.get(b)?;
        let right: HashMap<i32, f32> = pa.iter().map(|&(y, _, r)| (y, r)).collect();
        let limit = cap * DEPTH;
        let (mut total, mut n) = (0.0f32, 0usize);
        for &(y, l, _) in pb {
            if let Some(&r) = right.get(&y) {
                total += ((adv - r) + l + kern).min(limit);
                n += 1;
            }
        }
        (n >= 3).then(|| total / n as f32)
    }
}

/// Median white of the cap pairs the designer already kerned.
///
/// Takes the tuned pairs as `(left, right, adjustment)`; the caller supplies them
/// because reading GPOS is the font format's problem and this is the typography.
pub fn learn(face: &Face, tuned: &[(char, char, f32)]) -> Option<f32> {
    let mut cache = Cache::new(face);
    let mut samples: Vec<f32> = tuned
        .iter()
        .filter(|(a, b, _)| a.is_uppercase() && b.is_uppercase())
        .filter_map(|&(a, b, k)| cache.white(a, b, k, face.cap))
        .collect();
    if samples.is_empty() {
        return None;
    }
    samples.sort_by(|p, q| p.partial_cmp(q).unwrap());
    Some(samples[samples.len() / 2])
}

/// Per-pair corrections for a string, in em.
pub fn kern(face: &Face, text: &str, target: f32, tuned: &[(char, char, f32)]) -> Vec<Pair> {
    let have: HashMap<(char, char), f32> =
        tuned.iter().map(|&(a, b, k)| ((a, b), k)).collect();
    let mut cache = Cache::new(face);
    let chars: Vec<char> = text.chars().collect();
    chars
        .windows(2)
        .filter_map(|w| {
            let (a, b) = (w[0], w[1]);
            let h = have.get(&(a, b)).copied().unwrap_or(0.0);
            let white = cache.white(a, b, h, face.cap)?;
            Some(Pair {
                left: a,
                right: b,
                have: h,
                white,
                want: target,
                kern: (target - white) / face.upem,
            })
        })
        .collect()
}
