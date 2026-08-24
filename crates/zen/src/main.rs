//! zen — measure and fit type from the outlines.
//!
//!   zen kern  <font.ttf> <text> [wght]         per-pair optical corrections
//!   zen fit   <target.ttf> <candidate.ttf> <text>   best (wght, scaleX, track)
//!   zen show  <font.ttf>                        metrics and ratios
//!   zen lux   <font.ttf>                        fit the LUX wordmark, on features
//!   zen even  <font.ttf> <wght> <scaleX> <thicken> <diag-wght>  stroke spread, A-Z
//!   zen mark  <font.ttf> <text> <wght> <scaleX> <track> <flatten> <thicken> <diag-wght>

use std::env;
use std::fs;
use zen::{fit, kern, outline::Face};

/// Write to stdout, and treat a closed pipe as the end rather than a crash.
/// Rust ignores SIGPIPE, so `zen lux … | head -1` made `println!` panic with
/// "failed printing to stdout: Broken pipe" — a tool that exists to be piped
/// should not die of being piped.
fn out(s: &str) {
    use std::io::Write;
    if std::io::stdout().write_all(s.as_bytes()).is_err() {
        std::process::exit(0)
    }
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(2)
}

fn read(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| die(&format!("{path}: {e}")))
}

fn main() {
    let a: Vec<String> = env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("kern") => {
            if a.len() < 3 {
                die("zen kern <font> <text> [wght]")
            }
            let bytes = read(&a[1]);
            let wght: f32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(400.0);
            let face = Face::new(&bytes, wght).unwrap_or_else(|e| die(&e));

            // No tuned pairs handed in yet, so the target falls back to the face's
            // own mean rather than a number nobody chose. Reading GPOS is the next
            // increment; saying so beats inventing a constant and calling it learnt.
            let text = &a[2];
            let probe = kern::kern(&face, text, 0.0, &[]);
            if probe.is_empty() {
                die("no measurable pairs — are those glyphs in the font?")
            }
            let mut w: Vec<f32> = probe.iter().map(|p| p.white).collect();
            w.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let target = w[w.len() / 2];

            println!("target white {:.0} units ({:.4} em), from this string's own median",
                     target, target / face.upem);
            println!("{:<6}{:>8}{:>8}{:>10}", "pair", "white", "want", "kern");
            for p in kern::kern(&face, text, target, &[]) {
                println!("{}{}    {:>6.0}  {:>6.0}  {:>+9.4}",
                         p.left, p.right, p.white, p.want, p.kern);
            }
        }
        Some("fit") => {
            if a.len() < 4 {
                die("zen fit <target> <candidate> <text>")
            }
            let tb = read(&a[1]);
            let cb: &'static [u8] = Box::leak(read(&a[2]).into_boxed_slice());
            let text = a[3].clone();
            let tf = Face::new(&tb, 400.0).unwrap_or_else(|e| die(&e));
            let target = fit::render(&tf, &text, 180, 1.0, 0.0)
                .unwrap_or_else(|| die("target has no ink for that string"));
            match fit::search(|w| Face::new(cb, w).ok(), &target, 180, &text) {
                Some(f) => println!(
                    "residual {:.1}%   wght {:.0} · scaleX {:.2} · track {:+.3}em",
                    f.residual * 100.0, f.wght, f.sx, f.track),
                None => die("no candidate rendered"),
            }
        }
        Some("show") => {
            if a.len() < 2 {
                die("zen show <font>")
            }
            let bytes = read(&a[1]);
            for w in [400.0, 700.0, 900.0] {
                let f = Face::new(&bytes, w).unwrap_or_else(|e| die(&e));
                let l = f.glyph('L');
                let adv = l.as_ref().map(|g| g.advance).unwrap_or(0.0);
                let stem = l
                    .as_ref()
                    .map(|g| {
                        let xs = zen::cut(g, f.cap * 0.62);
                        if xs.len() >= 2 { xs[1] - xs[0] } else { 0.0 }
                    })
                    .unwrap_or(0.0);
                println!("wght {w:.0}: upem {} cap {} xh {}  L adv {adv:.0} stem {stem:.0} ({:.3} cap)",
                         f.upem, f.cap, f.xheight, stem / f.cap);
            }
        }
        Some("lux") => {
            if a.len() < 2 {
                die("zen lux <font> [drawn.svg] [track]")
            }
            let bytes = read(&a[1]);
            let art = read(a.get(2).map(String::as_str)
                .unwrap_or("/home/z/work/lux/logo/svg/lux-wordmark-white.svg"));
            let art = String::from_utf8_lossy(&art).to_string();
            let (c, want) = zen::lux::fit(&bytes, &art).unwrap_or_else(|| die("no candidate"));
            out(&format!("wght {:.0} · scaleX {:.3} · track {:+.3}em · flatten · thicken {:.0} · diagonals at wght {:.0}\n",
                         c.wght, c.scale_x, c.track, c.thicken, c.wght_diag));
            out(&format!("{:<10}{:>9}{:>9}{:>9}\n", "feature", "drawn", "zen", "off"));
            for ((n, got), (_, w)) in c.got.named().iter().zip(want.named()) {
                out(&format!("{n:<10}{w:>9.4}{got:>9.4}{:>8.1}%\n", (got / w - 1.0) * 100.0));
            }
            out(&format!("{:<10}{:>9.2}{:>9.2}   widest over narrowest\n",
                         "evenness", want.evenness(), c.got.evenness()));
        }
        Some("mark") => {
            if a.len() < 9 {
                die("zen mark <font> <text> <wght> <scaleX> <track> <flatten 0|1> <thicken> <diag-wght>")
            }
            let bytes = read(&a[1]);
            let n = |i: usize| -> f32 { a[i].parse().unwrap_or_else(|_| die(&format!("bad number: {}", a[i]))) };
            let face = Face::new(&bytes, n(3)).unwrap_or_else(|e| die(&e));
            // arg 8 is the weight diagonals are cut at; 0 means "same as the rest"
            let dw = n(8);
            let dface = (dw > 0.0).then(|| Face::new(&bytes, dw).unwrap_or_else(|e| die(&e)));
            // arg 10: a UX kern in em, negative to tuck the X under the U
            let ux: f32 = a.get(10).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let m = zen::shape::mark_kerned(&face, &a[2], n(5), a[6] == "1", n(7),
                                            dface.as_ref(), &[('U', 'X', ux)])
                .unwrap_or_else(|e| die(&e));
            // simplify tolerance, in font units. Optional so a suspected
            // simplify artefact can be ruled in or out without a rebuild.
            let tol: f32 = a.get(9).and_then(|s| s.parse().ok()).unwrap_or(1.0);
            out(&m.svg(n(4), tol));
        }
        Some("check") => {
            // Evaluate ONE cut against the drawn mark, using the same probes the
            // fit scores. `zen even` measures a letter in isolation and answers a
            // different question — comparing its numbers to these is how a cut
            // looks wrong when it is right.
            if a.len() < 7 {
                die("zen check <font> <wght> <scaleX> <track> <thicken> <diag-wght> [drawn.svg]")
            }
            let bytes = read(&a[1]);
            let n = |i: usize| -> f32 { a[i].parse().unwrap_or_else(|_| die(&format!("bad number: {}", a[i]))) };
            let art = String::from_utf8_lossy(&read(a.get(8).map(String::as_str)
                .unwrap_or("/home/z/work/lux/logo/svg/lux-wordmark-white.svg"))).to_string();
            let face = Face::new(&bytes, n(2)).unwrap_or_else(|e| die(&e));
            let want = zen::lux::drawn(&art, face.cap).unwrap_or_else(|| die("cannot read the drawn mark"));
            let dw = n(6);
            let dface = (dw > 0.0).then(|| Face::new(&bytes, dw).unwrap_or_else(|e| die(&e)));
            let ux: f32 = a.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let m = zen::shape::mark_kerned(&face, "LUX", n(4), true, n(5),
                                            dface.as_ref(), &[('U', 'X', ux)])
                .unwrap_or_else(|e| die(&e));
            let got = zen::lux::features(&m, n(3));
            out(&format!("{:<10}{:>9}{:>9}{:>9}\n", "feature", "drawn", "zen", "off"));
            for ((nm, g), (_, w)) in got.named().iter().zip(want.named()) {
                out(&format!("{nm:<10}{w:>9.4}{g:>9.4}{:>8.1}%\n", (g / w - 1.0) * 100.0));
            }
        }
        Some("even") => {
            if a.len() < 6 {
                die("zen even <font> <wght> <scaleX> <thicken> <diag-wght> [text]")
            }
            let bytes = read(&a[1]);
            let n = |i: usize| -> f32 { a[i].parse().unwrap_or_else(|_| die(&format!("bad number: {}", a[i]))) };
            let text = a.get(6).cloned()
                .unwrap_or_else(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string());
            let s = zen::lux::strokes(&bytes, &text, n(2), n(3), n(4), n(5));
            if s.is_empty() {
                die("no glyphs measured")
            }
            out(&format!("{:<5}{:>9}{:>9}{:>9}\n", "char", "stem", "bar", "bar/stem"));
            let mut all: Vec<f32> = Vec::new();
            for k in &s {
                out(&format!("{:<5}{:>9.4}{:>9.4}{:>9.2}\n", k.ch, k.stem, k.bar,
                             if k.stem > 0.0 { k.bar / k.stem } else { 0.0 }));
                if k.stem > 0.0 { all.push(k.stem) }
                if k.bar > 0.0 { all.push(k.bar) }
            }
            all.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let (lo, hi) = (all[0], all[all.len() - 1]);
            let mean = all.iter().sum::<f32>() / all.len() as f32;
            out(&format!("\n{} strokes  narrowest {lo:.4}  widest {hi:.4}  mean {mean:.4}\n",
                         all.len()));
            out(&format!("evenness {:.2} — widest over narrowest, 1.00 is monoline\n", hi / lo));
        }
        _ => {
            eprintln!("zen kern <font> <text> [wght]");
            eprintln!("zen fit  <target> <candidate> <text>");
            eprintln!("zen show <font>");
            eprintln!("zen even  <font> <wght> <scaleX> <thicken> <diag-wght> [text]");
            eprintln!("zen check <font> <wght> <scaleX> <track> <thicken> <diag-wght>");
            eprintln!("zen lux  <font> [track]");
            eprintln!("zen mark <font> <text> <wght> <scaleX> <track> <flatten> <thicken> <diag-wght>");
            std::process::exit(2)
        }
    }
}
