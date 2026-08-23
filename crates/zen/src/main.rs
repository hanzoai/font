//! zen — measure and fit type from the outlines.
//!
//!   zen kern  <font.ttf> <text> [wght]         per-pair optical corrections
//!   zen fit   <target.ttf> <candidate.ttf> <text>   best (wght, scaleX, track)
//!   zen show  <font.ttf>                        metrics and ratios

use std::env;
use std::fs;
use zen::{fit, kern, outline::Face};

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
        _ => {
            eprintln!("zen kern <font> <text> [wght]");
            eprintln!("zen fit  <target> <candidate> <text>");
            eprintln!("zen show <font>");
            std::process::exit(2)
        }
    }
}
