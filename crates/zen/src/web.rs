//! The browser surface.
//!
//! Same algorithms, same source — this file only moves values across the boundary.
//! A page hands in font bytes (an ArrayBuffer from fetch or a file input) and gets
//! JSON back, so nothing here needs a server, an upload, or a round trip. That is
//! the whole reason for the port: the fitter and the kerner become something a
//! designer runs live in a specimen page while dragging a slider.

use crate::{fit, kern, outline::Face, PRESETS};
use wasm_bindgen::prelude::*;

fn err(e: impl core::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Metrics and the proportions that decide whether one face can stand in for
/// another: stem and bar as fractions of cap, and their ratio.
#[wasm_bindgen]
pub fn measure(font: &[u8], wght: f32) -> Result<JsValue, JsValue> {
    let f = Face::new(font, wght).map_err(err)?;
    let l = f.glyph('L').ok_or_else(|| err("no L"))?;
    let xs = crate::cut(&l, f.cap * 0.62);
    let stem = if xs.len() >= 2 { xs[1] - xs[0] } else { 0.0 };

    // The bar is a VERTICAL measurement taken through the foot, which is why a
    // horizontal scale cannot change it and why widening alone never reaches a
    // heavier-looking cut.
    let mut ys: Vec<f32> = Vec::new();
    let x = l.advance * 0.72;
    for c in &l.contours {
        for w in 0..c.len() {
            let (a, b) = (c[w], c[(w + 1) % c.len()]);
            if (a.0 <= x && x < b.0) || (b.0 <= x && x < a.0) {
                ys.push(a.1 + (x - a.0) * (b.1 - a.1) / (b.0 - a.0));
            }
        }
    }
    ys.sort_by(|p, q| p.partial_cmp(q).unwrap());
    let bar = if ys.len() >= 2 { ys[ys.len() - 1] - ys[0] } else { 0.0 };

    let out = format!(
        r#"{{"upem":{},"cap":{},"xheight":{},"stem":{:.4},"bar":{:.4},"contrast":{:.4},"advance":{:.4}}}"#,
        f.upem, f.cap, f.xheight,
        stem / f.cap, bar / f.cap,
        if stem > 0.0 { bar / stem } else { 0.0 },
        l.advance / f.cap,
    );
    Ok(JsValue::from_str(&out))
}

/// Per-pair optical corrections, in em, for a string.
///
/// `target` is the white to aim each join at. Pass 0 to use the string's own
/// median, which is the right default for a wordmark: the pairs are judged against
/// each other rather than against a number from somewhere else.
#[wasm_bindgen]
pub fn kern_text(font: &[u8], text: &str, wght: f32, target: f32) -> Result<JsValue, JsValue> {
    let f = Face::new(font, wght).map_err(err)?;
    let target = if target > 0.0 {
        target
    } else {
        let mut w: Vec<f32> = kern::kern(&f, text, 0.0, &[]).iter().map(|p| p.white).collect();
        if w.is_empty() {
            return Err(err("no measurable pairs"));
        }
        w.sort_by(|a, b| a.partial_cmp(b).unwrap());
        w[w.len() / 2]
    };
    let rows: Vec<String> = kern::kern(&f, text, target, &[])
        .iter()
        .map(|p| format!(
            r#"{{"pair":"{}{}","white":{:.1},"kern":{:.5}}}"#,
            p.left, p.right, p.white, p.kern))
        .collect();
    Ok(JsValue::from_str(&format!(
        r#"{{"target":{:.1},"em":{:.5},"pairs":[{}]}}"#,
        target, target / f.upem, rows.join(","))))
}

/// Search a candidate's parameter space for the closest match to a target face.
///
/// Both fonts come in as bytes. Nothing is uploaded — this runs in the tab.
#[wasm_bindgen]
pub fn fit_to(target: &[u8], candidate: Vec<u8>, text: &str) -> Result<JsValue, JsValue> {
    let tf = Face::new(target, 400.0).map_err(err)?;
    let grid = fit::render(&tf, text, 180, 1.0, 0.0).ok_or_else(|| err("target has no ink"))?;
    let bytes: &'static [u8] = Box::leak(candidate.into_boxed_slice());
    let best = fit::search(|w| Face::new(bytes, w).ok(), &grid, 180, text)
        .ok_or_else(|| err("nothing rendered"))?;
    Ok(JsValue::from_str(&format!(
        r#"{{"residual":{:.5},"wght":{:.0},"scaleX":{:.3},"track":{:.4}}}"#,
        best.residual, best.wght, best.sx, best.track)))
}

/// The presets, so a page renders the same five names the CSS does.
#[wasm_bindgen]
pub fn presets() -> JsValue {
    let rows: Vec<String> = PRESETS
        .iter()
        .map(|p| format!(
            r#"{{"name":"{}","wght":{:.0},"scaleX":{:.2},"track":{:.3},"residual":{}}}"#,
            p.name, p.wght, p.scale_x, p.track,
            p.residual.map_or("null".into(), |r| format!("{r:.3}"))))
        .collect();
    JsValue::from_str(&format!("[{}]", rows.join(",")))
}
