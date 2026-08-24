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
#[cfg(feature = "name")]
pub mod instance;
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
#[derive(Clone, Copy, Debug)]
pub struct Preset {
    pub name: &'static str,
    pub wght: f32,
    pub scale_x: f32,
    pub track: f32,
}

pub use presets::PRESETS;

pub fn preset(name: &str) -> Option<Preset> {
    PRESETS.iter().find(|p| p.name == name).copied()
}
