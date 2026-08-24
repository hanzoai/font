//! Zen type tooling, from the outlines.
//!
//! Four questions, one representation. `outline` turns a glyph into flat polylines
//! and answers "where is ink at this height"; `kern` integrates the white between
//! two of them; `fit` fills a coverage grid from the same walk and scores one face
//! against another; `shape` moves the points that walk identifies and `lux` reads
//! the features off it. Nothing here rasterises twice or parses a table twice.
//!
//! Pure Rust throughout — the same source is a native binary and a wasm32 module
//! with nothing swapped out, which is the whole reason it is not Python.

pub mod fit;
pub mod kern;
pub mod lux;
pub mod outline;
pub mod presets;
pub mod shape;
pub mod svg;

#[cfg(feature = "wasm")]
pub mod web;

pub use fit::{render, score, search, Fit, Grid};
pub use kern::{kern, learn, Pair};
pub use outline::{cut, profile, Face, Glyph};
pub use shape::{flatten, mark, thicken, Mark};

/// The presets, as data — GENERATED from `packages/zen/scripts/presets.mjs`, which
/// is the one source. It was restated here by hand, so the CLI and the stylesheet
/// could disagree about what `wide` is and nothing would say so.
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
    /// Coverage: whole-mark pixel difference as a fraction of target ink.
    pub residual: Option<f32>,
    /// The worst single feature — stem, bar — as a ratio to the target's.
    /// Not comparable to `residual`; see the note in presets.mjs.
    pub within: Option<f32>,
}

pub use presets::PRESETS;

pub fn preset(name: &str) -> Option<Preset> {
    PRESETS.iter().find(|p| p.name == name).copied()
}
