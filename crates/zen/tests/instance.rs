//! A cut of the variable font must draw what the variable font draws.

use skrifa::{
    outline::{DrawSettings, OutlinePen},
    prelude::{LocationRef, Size},
    raw::{types::Tag, FontRef, TableProvider},
    GlyphId, MetadataProvider,
};

const NOTICE: &str = "Copyright 2026 Hanzo AI, Inc.";

fn source() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fonts/Zen/variable/Zen[wght].ttf"))
        .expect("run `make build` first")
}

/// Every on-curve point a face draws for one character, at one weight.
#[derive(Default, PartialEq, Debug)]
struct Points(Vec<(i32, i32)>);

impl OutlinePen for Points {
    fn move_to(&mut self, x: f32, y: f32) { self.0.push((x.round() as i32, y.round() as i32)) }
    fn line_to(&mut self, x: f32, y: f32) { self.0.push((x.round() as i32, y.round() as i32)) }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0.push((cx.round() as i32, cy.round() as i32));
        self.0.push((x.round() as i32, y.round() as i32));
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, x: f32, y: f32) {
        self.0.push((x.round() as i32, y.round() as i32))
    }
    fn close(&mut self) {}
}

fn trace(font: &FontRef, ch: char, wght: f32) -> Points {
    let location = font.axes().location(&[("wght", wght)]);
    let gid = font.charmap().map(ch).expect("no such character");
    let mut pen = Points::default();
    font.outline_glyphs()
        .get(gid)
        .expect("no outline")
        .draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::from(&location)), &mut pen)
        .unwrap();
    pen
}

#[test]
fn a_cut_draws_what_the_variable_font_draws() {
    let src = source();
    let variable = FontRef::new(&src).unwrap();
    for wght in [220.0, 400.0, 606.0, 900.0] {
        let cut = zen::instance::instance(&src, &[("wght", wght)], 1.0, 0.0, &[], "Zen Cut", "Regular", NOTICE).unwrap();
        let cut = FontRef::new(&cut).unwrap();
        for ch in "HanzoZen@017".chars() {
            assert_eq!(trace(&cut, ch, 400.0), trace(&variable, ch, wght), "{ch} at {wght}");
        }
    }
}

#[test]
fn a_cut_carries_the_advances_of_its_weight() {
    let src = source();
    let variable = FontRef::new(&src).unwrap();
    let cut_bytes = zen::instance::instance(&src, &[("wght", 900.0)], 1.0, 0.0, &[], "Zen Cut", "Black", NOTICE).unwrap();
    let cut = FontRef::new(&cut_bytes).unwrap();
    let heavy = variable.axes().location(&[("wght", 900.0)]);
    let want = variable.glyph_metrics(Size::unscaled(), LocationRef::from(&heavy));
    let got = cut.glyph_metrics(Size::unscaled(), LocationRef::default());
    for ch in "Hanzo".chars() {
        let gid = variable.charmap().map(ch).unwrap();
        assert_eq!(got.advance_width(gid), want.advance_width(gid), "advance of {ch}");
    }
}

#[test]
fn a_cut_is_static() {
    let src = source();
    let cut = zen::instance::instance(&src, &[("wght", 500.0)], 1.0, 0.0, &[], "Zen Cut", "Regular", NOTICE).unwrap();
    let cut = FontRef::new(&cut).unwrap();
    for tag in ["fvar", "gvar", "avar", "HVAR", "MVAR", "STAT"] {
        assert!(cut.table_data(Tag::new(tag.as_bytes().try_into().unwrap())).is_none(), "{tag} survived");
    }
    assert!(cut.fvar().is_err());
    assert_eq!(cut.maxp().unwrap().num_glyphs(), FontRef::new(&source()).unwrap().maxp().unwrap().num_glyphs());
}

#[test]
fn a_feature_bakes_the_alternate_in() {
    let src = source();
    let variable = FontRef::new(&src).unwrap();
    let plain = zen::instance::instance(&src, &[("wght", 400.0)], 1.0, 0.0, &[], "Zen", "Regular", NOTICE).unwrap();
    let single = zen::instance::instance(&src, &[("wght", 400.0)], 1.0, 0.0, &["ss01"], "Zen", "Regular", NOTICE).unwrap();
    let plain = FontRef::new(&plain).unwrap();
    let single = FontRef::new(&single).unwrap();

    // ss01 is the single-storey a. The letter must change and the file must not.
    assert_ne!(trace(&single, 'a', 400.0), trace(&plain, 'a', 400.0), "ss01 did not reach the a");
    assert_eq!(trace(&single, 'H', 400.0), trace(&plain, 'H', 400.0), "ss01 moved a letter it does not name");
    assert_eq!(
        single.charmap().map('a').map(GlyphId::to_u32),
        variable.charmap().map('a').map(GlyphId::to_u32),
        "the character map moved"
    );
}

#[test]
fn a_cut_names_itself() {
    let src = source();
    let cut = zen::instance::instance(&src, &[("wght", 606.0)], 1.0, 0.0, &[], "Zen Medium Cut", "Medium", NOTICE).unwrap();
    let cut = FontRef::new(&cut).unwrap();
    let name = cut.name().unwrap();
    let read = |id: u16| {
        name.name_record()
            .iter()
            .find(|r| r.name_id().to_u16() == id)
            .map(|r| r.string(name.string_data()).unwrap().chars().collect::<String>())
    };
    assert_eq!(read(1).as_deref(), Some("Zen Medium Cut"));
    assert_eq!(read(2).as_deref(), Some("Medium"));
    assert_eq!(read(4).as_deref(), Some("Zen Medium Cut Medium"));
    assert_eq!(read(0).as_deref(), Some(NOTICE));
    assert_eq!(cut.os2().unwrap().us_weight_class(), 606);
}

#[test]
fn a_wide_cut_carries_its_width() {
    let src = source();
    let variable = FontRef::new(&src).unwrap();
    let wide = zen::instance::instance(&src, &[("wght", 650.0)], 1.5, 0.0, &[], "Zen Wide", "Regular", NOTICE).unwrap();
    let wide = FontRef::new(&wide).unwrap();
    let at650 = variable.axes().location(&[("wght", 650.0)]);
    let want = variable.glyph_metrics(Size::unscaled(), LocationRef::from(&at650));
    let got = wide.glyph_metrics(Size::unscaled(), LocationRef::default());
    let gid = variable.charmap().map('H').unwrap();
    let ratio = got.advance_width(gid).unwrap() / want.advance_width(gid).unwrap();
    assert!((ratio - 1.5).abs() < 0.01, "advance scaled {ratio}, wanted 1.5");

    let (narrow, broad) = (trace(&variable, 'H', 650.0), trace(&wide, 'H', 400.0));
    let far = |p: &Points| p.0.iter().map(|(x, _)| *x).max().unwrap() as f32;
    let stretch = far(&broad) / far(&narrow);
    assert!((stretch - 1.5).abs() < 0.02, "outline scaled {stretch}, wanted 1.5");
}

#[test]
fn tracking_becomes_the_glyphs_own_space() {
    let src = source();
    let plain = zen::instance::instance(&src, &[("wght", 400.0)], 1.0, 0.0, &[], "Zen", "Regular", NOTICE).unwrap();
    let tracked = zen::instance::instance(&src, &[("wght", 400.0)], 1.0, 0.05, &[], "Zen", "Regular", NOTICE).unwrap();
    let plain = FontRef::new(&plain).unwrap();
    let tracked = FontRef::new(&tracked).unwrap();
    let em = plain.head().unwrap().units_per_em() as f32;
    let gid = plain.charmap().map('H').unwrap();
    let width = |f: &FontRef| f.glyph_metrics(Size::unscaled(), LocationRef::default()).advance_width(gid).unwrap();
    assert_eq!((width(&tracked) - width(&plain)).round(), (0.05 * em).round());

    // The letter itself must not move — tracking is space beside it, not inside.
    assert_eq!(trace(&tracked, 'H', 400.0), trace(&plain, 'H', 400.0));
}

#[test]
fn the_pixel_element_is_an_axis_like_any_other() {
    let src = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"), "/../../fonts/ZenPixel/variable/ZenPixel[ELSH].ttf"))
        .expect("run `make build` first");
    let variable = FontRef::new(&src).unwrap();
    let (square, triangle) = (
        zen::instance::instance(&src, &[("ELSH", 1.0)], 1.0, 0.0, &[], "Zen Pixel Square", "Regular", NOTICE).unwrap(),
        zen::instance::instance(&src, &[("ELSH", 60.0)], 1.0, 0.0, &[], "Zen Pixel Triangle", "Regular", NOTICE).unwrap(),
    );
    let (square, triangle) = (FontRef::new(&square).unwrap(), FontRef::new(&triangle).unwrap());
    assert_ne!(trace(&square, 'A', 400.0), trace(&triangle, 'A', 400.0), "the element did not change");
    assert_eq!(trace(&square, 'A', 400.0), {
        let at = variable.axes().location(&[("ELSH", 1.0)]);
        let gid = variable.charmap().map('A').unwrap();
        let mut pen = Points::default();
        variable.outline_glyphs().get(gid).unwrap()
            .draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::from(&at)), &mut pen).unwrap();
        pen
    });
}
