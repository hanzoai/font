//! Glyph outlines, flattened, and the scanline profile both algorithms read.
//!
//! Everything downstream — the kerner's white-area integral and the fitter's
//! coverage grid — is a question about where ink is at a given height. So there is
//! ONE representation: a glyph is a list of closed polylines in font units, and a
//! scanline crossing it yields the x values where the contour is cut. The kerner
//! wants the outermost pair; the rasteriser wants all of them with a winding rule.
//! Two consumers, one measurement.

use skrifa::{
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},

    FontRef, GlyphId, MetadataProvider,
};

/// Curve subdivision. 20 was enough for the Python original to agree with the
/// font's own kern table on the sign of every pair it was checked against, and a
/// scanline every 10 units cannot resolve more detail than that anyway.
const STEPS: usize = 20;

pub struct Glyph {
    pub contours: Vec<Vec<(f32, f32)>>,
    pub advance: f32,
}

/// Collects pen commands into flat polylines.
struct Flatten {
    contours: Vec<Vec<(f32, f32)>>,
    cur: Vec<(f32, f32)>,
}

impl Flatten {
    fn new() -> Self {
        Self { contours: Vec::new(), cur: Vec::new() }
    }
    fn at(&self) -> (f32, f32) {
        *self.cur.last().unwrap_or(&(0.0, 0.0))
    }
    fn flush(&mut self) {
        if self.cur.len() > 2 {
            self.contours.push(core::mem::take(&mut self.cur));
        } else {
            self.cur.clear();
        }
    }
}

impl OutlinePen for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        self.cur.push((x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.cur.push((x, y));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (x0, y0) = self.at();
        for i in 1..=STEPS {
            let t = i as f32 / STEPS as f32;
            let u = 1.0 - t;
            self.cur.push((
                u * u * x0 + 2.0 * u * t * cx + t * t * x,
                u * u * y0 + 2.0 * u * t * cy + t * t * y,
            ));
        }
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (x0, y0) = self.at();
        for i in 1..=STEPS {
            let t = i as f32 / STEPS as f32;
            let u = 1.0 - t;
            let (uu, tt) = (u * u, t * t);
            self.cur.push((
                uu * u * x0 + 3.0 * uu * t * c0x + 3.0 * u * tt * c1x + tt * t * x,
                uu * u * y0 + 3.0 * uu * t * c0y + 3.0 * u * tt * c1y + tt * t * y,
            ));
        }
    }
    fn close(&mut self) {
        self.flush();
    }
}

/// A font at one variation instance.
pub struct Face<'a> {
    font: FontRef<'a>,
    loc: skrifa::instance::Location,
    pub upem: f32,
    pub cap: f32,
    pub xheight: f32,
}

impl<'a> Face<'a> {
    pub fn new(bytes: &'a [u8], wght: f32) -> Result<Self, String> {
        let font = FontRef::new(bytes).map_err(|e| format!("not a font: {e}"))?;
        let loc = font.axes().location([("wght", wght)]);
        let m = font.metrics(Size::unscaled(), LocationRef::from(&loc));
        Ok(Self {
            upem: m.units_per_em as f32,
            // A face with no declared cap height is not one we can reason about
            // proportionally, so fall back to the em rather than to zero — a zero
            // would silently make every ratio infinite instead of merely wrong.
            cap: m.cap_height.unwrap_or(m.units_per_em as f32 * 0.7),
            xheight: m.x_height.unwrap_or(m.units_per_em as f32 * 0.5),
            font,
            loc,
        })
    }

    pub fn gid(&self, ch: char) -> Option<GlyphId> {
        self.font.charmap().map(ch)
    }

    pub fn glyph(&self, ch: char) -> Option<Glyph> {
        let gid = self.gid(ch)?;
        let outlines = self.font.outline_glyphs();
        let g = outlines.get(gid)?;
        let mut pen = Flatten::new();
        g.draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::from(&self.loc)),
            &mut pen,
        )
        .ok()?;
        pen.flush();
        let advance = self
            .font
            .glyph_metrics(Size::unscaled(), LocationRef::from(&self.loc))
            .advance_width(gid)
            .unwrap_or(0.0);
        Some(Glyph { contours: pen.contours, advance })
    }
}

/// x values where a horizontal ray at `y` cuts the outline, sorted.
pub fn cut(g: &Glyph, y: f32) -> Vec<f32> {
    let mut xs = Vec::new();
    for c in &g.contours {
        for w in 0..c.len() {
            let (a, b) = (c[w], c[(w + 1) % c.len()]);
            if (a.1 <= y && y < b.1) || (b.1 <= y && y < a.1) {
                xs.push(a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1));
            }
        }
    }
    xs.sort_by(|p, q| p.partial_cmp(q).unwrap_or(core::cmp::Ordering::Equal));
    xs
}

/// Left and right ink edge per scanline, on an ABSOLUTE grid.
///
/// Absolute is load bearing, and it is the one bug the Python original shipped.
/// Sampling from each glyph's OWN lowest point gives every glyph a differently
/// phased set of scanlines, so a round `O` — which overshoots the baseline — and a
/// pointed `V` share almost no y values and their overlap band comes out empty.
/// That surfaced as "no outline" and silently dropped three of the eight pairs in
/// SOVEREIGN, while the pairs whose phase happened to agree measured fine.
pub fn profile(g: &Glyph, step: i32) -> Vec<(i32, f32, f32)> {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for c in &g.contours {
        for p in c {
            lo = lo.min(p.1);
            hi = hi.max(p.1);
        }
    }
    if lo > hi {
        return Vec::new();
    }
    let (lo, hi) = (
        (lo / step as f32).floor() as i32 * step,
        (hi / step as f32).floor() as i32 * step,
    );
    let mut out = Vec::new();
    let mut y = lo;
    while y <= hi {
        let xs = cut(g, y as f32);
        if xs.len() >= 2 {
            out.push((y, xs[0], xs[xs.len() - 1]));
        }
        y += step;
    }
    out
}
