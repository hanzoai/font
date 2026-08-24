//! Cut a static font out of the variable one, at a place you choose.
//!
//! A variable font carries every weight and picks one at render time, which is
//! what a browser wants and what a font menu, a design tool or a terminal often
//! cannot use. This walks the outlines at one location, writes them into a plain
//! `glyf`, and hands back a font that needs no variation settings to look right.
//!
//! Stylistic sets are applied the same way — by DRAWING the alternate. `ss01`
//! swaps the double-storey `a` for the single-storey one, so a glyph whose id the
//! feature substitutes is written with the substitute's outline and advance under
//! the original's id. The character map never moves, so the file stays a drop-in
//! replacement while carrying the letterforms you chose.
//!
//! Width and tracking are baked rather than left to CSS. `transform:scaleX()`
//! stretches what is painted and leaves the layout box alone; a font that is
//! actually wider carries the width in its advances too, or its letters overlap.
//! Tracking is the same story — `letter-spacing` is space a layout engine adds,
//! and here it is space the glyph owns. So a cut taken at a preset needs no CSS
//! to look like that preset.

use skrifa::{
    outline::{DrawSettings, OutlinePen},
    prelude::{LocationRef, Size},
    raw::{
        tables::gsub::{SingleSubst, SubstitutionLookup},
        types::Tag,
        FontRef, TableProvider,
    },
    GlyphId, MetadataProvider,
};
use std::collections::HashMap;
use write_fonts::read::tables::glyf::CurvePoint;
use write_fonts::{
    tables::{
        glyf::{Bbox, Contour, GlyfLocaBuilder, SimpleGlyph},
        name::{Name, NameRecord},
    },
    types::{NameId, Tag as OutTag},
    FontBuilder,
};

/// The same four bytes, on the other side of the boundary. skrifa and
/// write-fonts each carry their own `font-types`, so a tag crosses as bytes.
fn out_tag(tag: &[u8; 4]) -> OutTag {
    OutTag::new(tag)
}

/// Tables that come across untouched: the character map and the glyph names.
/// Nothing else is copied, so fvar, gvar, avar, HVAR, MVAR and STAT are absent
/// by construction rather than by deletion.
const VERBATIM: [&[u8; 4]; 2] = [b"cmap", b"post"];

#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

fn err(m: impl std::fmt::Display) -> Error {
    Error(m.to_string())
}

/// Collects a drawn outline as glyf contours.
///
/// A TrueType source draws in lines and quadratics, so a cubic can only arrive
/// from a CFF face — it is flattened to its endpoint rather than silently
/// dropped, which keeps the contour closed.
#[derive(Default)]
struct Pen {
    width: f32,
    contours: Vec<Contour>,
    open: Vec<CurvePoint>,
    min: (i16, i16),
    max: (i16, i16),
    any: bool,
}

impl Pen {
    fn round(&mut self, x: f32, y: f32) -> (i16, i16) {
        let p = ((x * self.width).round() as i16, y.round() as i16);
        if self.any {
            self.min = (self.min.0.min(p.0), self.min.1.min(p.1));
            self.max = (self.max.0.max(p.0), self.max.1.max(p.1));
        } else {
            self.min = p;
            self.max = p;
            self.any = true;
        }
        p
    }

    fn end(&mut self) {
        if self.open.len() > 1 {
            self.contours.push(Contour::from(std::mem::take(&mut self.open)));
        } else {
            self.open.clear();
        }
    }

    fn finish(mut self) -> SimpleGlyph {
        self.end();
        SimpleGlyph {
            bbox: Bbox { x_min: self.min.0, y_min: self.min.1, x_max: self.max.0, y_max: self.max.1 },
            contours: self.contours,
            ..Default::default()
        }
    }
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.end();
        let p = self.round(x, y);
        self.open.push(CurvePoint::on_curve(p.0, p.1));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.round(x, y);
        self.open.push(CurvePoint::on_curve(p.0, p.1));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let c = self.round(cx, cy);
        let p = self.round(x, y);
        self.open.push(CurvePoint::off_curve(c.0, c.1));
        self.open.push(CurvePoint::on_curve(p.0, p.1));
    }

    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, x: f32, y: f32) {
        self.line_to(x, y);
    }

    fn close(&mut self) {
        self.end();
    }
}

/// Every single substitution the named features perform, as old id -> new id.
///
/// Only single substitution is honoured, because that is what a stylistic set
/// on this family is: one letterform for another. A ligature or a contextual
/// rule depends on what is beside it and cannot be baked into a glyph.
fn substitutions(font: &FontRef, features: &[&str]) -> HashMap<GlyphId, GlyphId> {
    let mut map = HashMap::new();
    let Ok(gsub) = font.gsub() else { return map };
    let (Ok(features_list), Ok(lookups)) = (gsub.feature_list(), gsub.lookup_list()) else {
        return map;
    };
    let wanted: Vec<Tag> = features.iter().filter_map(|f| Tag::new_checked(f.as_bytes()).ok()).collect();
    for record in features_list.feature_records() {
        if !wanted.contains(&record.feature_tag()) {
            continue;
        }
        let Ok(feature) = record.feature(features_list.offset_data()) else { continue };
        for index in feature.lookup_list_indices() {
            let Ok(SubstitutionLookup::Single(single)) = lookups.lookups().get(index.get() as usize)
            else {
                continue;
            };
            for table in single.subtables().iter().flatten() {
                match table {
                    SingleSubst::Format1(t) => {
                        let delta = t.delta_glyph_id();
                        if let Ok(coverage) = t.coverage() {
                            for gid in coverage.iter() {
                                let to = (gid.to_u32() as i32 + delta as i32) as u32;
                                map.insert(GlyphId::new(gid.to_u32()), GlyphId::new(to));
                            }
                        }
                    }
                    SingleSubst::Format2(t) => {
                        if let Ok(coverage) = t.coverage() {
                            for (gid, to) in coverage.iter().zip(t.substitute_glyph_ids()) {
                                map.insert(GlyphId::new(gid.to_u32()), GlyphId::new(to.get().to_u32()));
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/// Overwrite a big-endian u16 at `offset`, if the table is long enough.
fn patch(table: &mut [u8], offset: usize, value: u16) {
    if let Some(slot) = table.get_mut(offset..offset + 2) {
        slot.copy_from_slice(&value.to_be_bytes());
    }
}

/// A full name table: the required notice, the family, and the machine names.
fn names(family: &str, style: &str, notice: &str) -> Name {
    let full = if style == "Regular" { family.to_string() } else { format!("{family} {style}") };
    let postscript: String = full.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let rows = [
        (NameId::COPYRIGHT_NOTICE, notice.to_string()),
        (NameId::FAMILY_NAME, family.to_string()),
        (NameId::SUBFAMILY_NAME, style.to_string()),
        (NameId::UNIQUE_ID, format!("{postscript};HNZO")),
        (NameId::FULL_NAME, full),
        (NameId::VERSION_STRING, "Version 1.000".to_string()),
        (NameId::POSTSCRIPT_NAME, postscript),
    ];
    Name::new(
        rows.into_iter()
            .map(|(id, s)| NameRecord::new(3, 1, 0x409, id, s.into()))
            .collect(),
    )
}

/// Cut a static font at one setting, with the named features applied.
///
/// `at` is where on the axes to cut — `[("wght", 497.0)]` for the sans, or
/// `[("ELSH", 40.0)]` for the pixel element. `width` scales horizontally and
/// `track` adds space after every glyph, in em. `family` names the result.
pub fn instance(
    source: &[u8],
    at: &[(&str, f32)],
    width: f32,
    track: f32,
    features: &[&str],
    family: &str,
    style: &str,
    notice: &str,
) -> Result<Vec<u8>, Error> {
    let font = FontRef::new(source).map_err(err)?;
    let location = font.axes().location(at.iter().copied());
    let location = LocationRef::from(&location);
    let outlines = font.outline_glyphs();
    let metrics = font.glyph_metrics(Size::unscaled(), location);
    let count = font.maxp().map_err(err)?.num_glyphs();
    let swap = substitutions(&font, features);
    let em = font.head().map_err(err)?.units_per_em() as f32;
    let extra = (track * em).round() as i32;

    let mut glyphs = GlyfLocaBuilder::new();
    let mut advances: Vec<(u16, i16)> = Vec::with_capacity(count as usize);
    for id in 0..count as u32 {
        let gid = GlyphId::new(id);
        let drawn = *swap.get(&gid).unwrap_or(&gid);
        let mut pen = Pen { width, ..Pen::default() };
        if let Some(outline) = outlines.get(drawn) {
            outline
                .draw(DrawSettings::unhinted(Size::unscaled(), location), &mut pen)
                .map_err(err)?;
        }
        let glyph = pen.finish();
        let left = glyph.bbox.x_min;
        glyphs.add_glyph(&glyph).map_err(err)?;
        let advance = metrics.advance_width(drawn).unwrap_or(0.0) * width;
        advances.push(((advance.round() as i32 + extra).clamp(0, u16::MAX as i32) as u16, left));
    }
    let (glyf, loca, loca_format) = glyphs.build();

    let mut hmtx = Vec::with_capacity(advances.len() * 4);
    for (advance, left) in &advances {
        hmtx.extend_from_slice(&advance.to_be_bytes());
        hmtx.extend_from_slice(&left.to_be_bytes());
    }

    let mut out = FontBuilder::new();
    out.add_table(&glyf).map_err(err)?;
    out.add_table(&loca).map_err(err)?;
    out.add_table(&names(family, style, notice)).map_err(err)?;
    out.add_raw(out_tag(b"hmtx"), hmtx);

    for tag in VERBATIM {
        if let Some(data) = font.table_data(Tag::new(tag)) {
            out.add_raw(out_tag(tag), data.as_bytes().to_vec());
        }
    }

    // head, hhea, maxp and OS/2 survive with three numbers moved: where loca
    // switched format, how many advances hmtx now holds, and the weight this cut
    // reports to a font menu.
    let mut head = table(&font, b"head")?;
    patch(&mut head, 50, loca_format as u16);
    out.add_raw(out_tag(b"head"), head);

    let mut hhea = table(&font, b"hhea")?;
    patch(&mut hhea, 34, advances.len() as u16);
    out.add_raw(out_tag(b"hhea"), hhea);

    out.add_raw(out_tag(b"maxp"), table(&font, b"maxp")?);

    let mut os2 = table(&font, b"OS/2")?;
    if let Some((_, wght)) = at.iter().find(|(tag, _)| *tag == "wght") {
        patch(&mut os2, 4, wght.round().clamp(1.0, 1000.0) as u16);
    }
    out.add_raw(out_tag(b"OS/2"), os2);

    Ok(out.build())
}

fn table(font: &FontRef, tag: &[u8; 4]) -> Result<Vec<u8>, Error> {
    font.table_data(Tag::new(tag))
        .map(|d| d.as_bytes().to_vec())
        .ok_or_else(|| err(format!("source has no {}", String::from_utf8_lossy(tag))))
}


