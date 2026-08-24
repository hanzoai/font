//! Read a drawn mark back as polylines, so a fit target is a FILE and not a
//! constant somebody typed.
//!
//! `lux.rs` used to carry the drawn LUX's measurements as five hardcoded floats
//! with a comment explaining that this was deliberate. It was not deliberate
//! enough: adding a sixth measurement then meant deriving it by hand from 633
//! bytes of path data, which is exactly the step that invites a wrong number
//! nothing can catch. Reading the artwork costs seventy lines and makes every
//! measurement re-derivable from the thing being matched.
//!
//! Only the commands this artwork uses — M/L/H/V/C/Q/Z, absolute and relative,
//! plus `<polygon points>`. Anything else is skipped rather than guessed at: a
//! silently mis-parsed arc would move a target and look like a bad fit.

use crate::outline::Glyph;

const STEPS: usize = 20;

fn nums(s: &str) -> Vec<f32> {
    let mut out = Vec::new();
    let (b, mut i) = (s.as_bytes(), 0);
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' {
            let start = i;
            i += 1;
            while i < b.len() {
                let d = b[i] as char;
                if d.is_ascii_digit() || d == '.' {
                    i += 1;
                } else if (d == '-' || d == '+') && matches!(b[i - 1] as char, 'e' | 'E') {
                    i += 1;
                } else {
                    break;
                }
            }
            if let Ok(v) = s[start..i].parse() {
                out.push(v);
            }
        } else {
            i += 1;
        }
    }
    out
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!("{name}=\""))? + name.len() + 2;
    let end = tag[at..].find('"')? + at;
    Some(&tag[at..end])
}

fn bez(from: (f32, f32), pts: &[(f32, f32)], out: &mut Vec<(f32, f32)>) {
    for k in 1..=STEPS {
        let t = k as f32 / STEPS as f32;
        let mut q: Vec<(f32, f32)> = core::iter::once(from).chain(pts.iter().copied()).collect();
        while q.len() > 1 {
            q = q.windows(2)
                .map(|w| (w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t))
                .collect();
        }
        out.push(q[0]);
    }
}

fn path(d: &str) -> Vec<Vec<(f32, f32)>> {
    let (mut out, mut cur) = (Vec::new(), Vec::new());
    let (mut pen, mut start) = ((0.0f32, 0.0f32), (0.0f32, 0.0f32));
    let mut cmd = ' ';
    let (b, mut i) = (d.as_bytes(), 0);
    let flush = |cur: &mut Vec<(f32, f32)>, out: &mut Vec<Vec<(f32, f32)>>| {
        if cur.len() > 2 {
            out.push(core::mem::take(cur));
        } else {
            cur.clear();
        }
    };
    while i < d.len() {
        let c = b[i] as char;
        if c.is_ascii_alphabetic() {
            cmd = c;
            i += 1;
            if cmd == 'Z' || cmd == 'z' {
                flush(&mut cur, &mut out);
                pen = start;
            }
            continue;
        }
        if !(c.is_ascii_digit() || c == '-' || c == '+' || c == '.') {
            i += 1;
            continue;
        }
        // one operand run for this command
        let start_i = i;
        while i < d.len() && !(b[i] as char).is_ascii_alphabetic() {
            i += 1;
        }
        let v = nums(&d[start_i..i]);
        let rel = cmd.is_ascii_lowercase();
        let need = match cmd.to_ascii_uppercase() {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'S' | 'Q' => 4,
            _ => 0,
        };
        if need == 0 {
            continue;
        }
        for chunk in v.chunks(need) {
            if chunk.len() < need {
                break;
            }
            let p = |k: usize| {
                if rel { (pen.0 + chunk[k], pen.1 + chunk[k + 1]) } else { (chunk[k], chunk[k + 1]) }
            };
            match cmd.to_ascii_uppercase() {
                'M' => {
                    flush(&mut cur, &mut out);
                    pen = p(0);
                    start = pen;
                    cur.push(pen);
                    // a second pair after M is an implicit lineto
                    cmd = if rel { 'l' } else { 'L' };
                }
                'L' | 'T' => {
                    pen = p(0);
                    cur.push(pen);
                }
                'H' => {
                    pen = (if rel { pen.0 + chunk[0] } else { chunk[0] }, pen.1);
                    cur.push(pen);
                }
                'V' => {
                    pen = (pen.0, if rel { pen.1 + chunk[0] } else { chunk[0] });
                    cur.push(pen);
                }
                'C' => {
                    let (a, b2, e) = (p(0), p(2), p(4));
                    bez(pen, &[a, b2, e], &mut cur);
                    pen = e;
                }
                'S' | 'Q' => {
                    let (a, e) = (p(0), p(2));
                    bez(pen, &[a, e], &mut cur);
                    pen = e;
                }
                _ => {}
            }
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// Every filled contour in an SVG, as one Glyph in SVG coordinates (y DOWN).
pub fn read(src: &str) -> Glyph {
    let mut contours = Vec::new();
    for (i, part) in src.split("<path").enumerate() {
        if i == 0 {
            continue;
        }
        if let Some(d) = attr(part, "d") {
            contours.extend(path(d));
        }
    }
    for (i, part) in src.split("<polygon").enumerate() {
        if i == 0 {
            continue;
        }
        if let Some(p) = attr(part, "points") {
            let v = nums(p);
            if v.len() >= 6 {
                contours.push(v.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0], c[1])).collect());
            }
        }
    }
    Glyph { contours, advance: 0.0 }
}

/// The viewBox, if the file declares one.
pub fn viewbox(src: &str) -> Option<(f32, f32, f32, f32)> {
    let v = nums(attr(src, "viewBox")?);
    (v.len() == 4).then(|| (v[0], v[1], v[2], v[3]))
}

/// Read a drawn mark and hand it back in FONT orientation — y up, baseline at 0,
/// scaled so the viewBox height is `cap`. Both marks then measure identically.
pub fn as_mark(src: &str, cap: f32) -> Option<crate::shape::Mark> {
    let g = read(src);
    let (_, _, _, vh) = viewbox(src)?;
    if vh <= 0.0 {
        return None;
    }
    let k = cap / vh;
    let flipped = Glyph {
        contours: g
            .contours
            .iter()
            .map(|c| c.iter().map(|&(x, y)| (x * k, cap - y * k)).collect())
            .collect(),
        advance: 0.0,
    };
    Some(crate::shape::Mark { glyphs: vec![(flipped, 0.0)], cap })
}
