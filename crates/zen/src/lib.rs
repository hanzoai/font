//! Zen type tooling, from the outlines.
//!
//! Three questions, one representation. `outline` turns a glyph into flat polylines
//! and answers "where is ink at this height"; `kern` integrates the white between
//! two of them; `fit` fills a coverage grid from the same walk and scores one face
//! against another. Nothing here rasterises twice or parses a table twice.
//!
//! Pure Rust throughout — the same source is a native binary and a wasm32 module
//! with nothing swapped out, which is the whole reason it is not Python.

pub mod fit;
pub mod kern;
pub mod outline;

#[cfg(feature = "wasm")]
pub mod web;

pub use fit::{render, score, search, Fit, Grid};
pub use kern::{kern, learn, Pair};
pub use outline::{cut, profile, Face, Glyph};

/// The presets, as data. Kept beside the algorithms that produced them so a change
/// to the fitter and a change to the numbers it found land in one commit.
///
/// `book` and `medium` were fitted against a licensed text face and `wide` against
/// a licensed display one; `residual` records how close the winner got, because a
/// preset that is a strong resemblance and a preset that is a match should not look
/// the same in the source.
#[derive(Clone, Copy, Debug)]
pub struct Preset {
    pub name: &'static str,
    pub wght: f32,
    pub scale_x: f32,
    pub track: f32,
    pub residual: Option<f32>,
}

pub const PRESETS: [Preset; 5] = [
    Preset { name: "air",    wght: 220.0, scale_x: 1.00, track: -0.030, residual: None },
    Preset { name: "book",   wght: 497.0, scale_x: 1.00, track:  0.000, residual: Some(0.297) },
    Preset { name: "medium", wght: 606.0, scale_x: 1.00, track:  0.000, residual: Some(0.254) },
    Preset { name: "wide",   wght: 845.0, scale_x: 1.56, track: -0.040, residual: Some(0.125) },
    Preset { name: "round",  wght: 900.0, scale_x: 1.00, track: -0.018, residual: None },
];

pub fn preset(name: &str) -> Option<Preset> {
    PRESETS.iter().find(|p| p.name == name).copied()
}
