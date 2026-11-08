//! The first mouthpiece: round rings, a 6-ring cup down to a 5mm neck and a
//! 17-ring backbore. Rust since 2026-11-07 (mouthpiece.py until then; the
//! move changed no byte of any sheet).
//!
//!     target/release/mouthpiece [OUT.svg] [--numbers=no]
//!
//! It names `cup` and `backbore` backwards, and that is NOT to be fixed: it
//! would regenerate a cut part. NOTES.md [Known wrong, deliberately not fixed].
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, g};

const WALL: f64 = 3.0; // ring width, mm — also the ply thickness
const PLATE: f64 = 16.0; // station one, square, matching the bore's closing face
const BORE: f64 = 10.0; // the square bore this meets
const THROAT: f64 = 3.66; // a #27 drill, the standard trumpet mouthpiece throat
const GAP: f64 = 2.0; // spacing between parts on the sheet
const MARGIN: f64 = 3.0;
const NECK: f64 = 5.0; // the waist the cup runs down to before the backbore opens again
const NCUP: i64 = 6; // stations in the cup, station one included

fn square(cx: f64, cy: f64, s: f64) -> String {
    let h = s / 2.0;
    format!("M {:.4},{:.4} H {:.4} V {:.4} H {:.4} Z", cx - h, cy - h, cx + h, cy + h, cx - h)
}

fn main() {
    let a = argv();
    // Derived from BORE rather than typed, so the cup starts where the bore ends.
    let cup: Vec<f64> = (0..NCUP).map(|i| py::round_dp(BORE - (BORE - NECK) * i as f64 / (NCUP - 1) as f64, 3)).collect();
    let backbore: Vec<f64> = (0..17).map(|i| py::round_dp(THROAT + 0.40 * i as f64, 3)).collect();
    let profile: Vec<f64> = cup.iter().chain(&backbore).copied().collect();
    let outers: Vec<f64> = std::iter::once(PLATE).chain(profile[1..].iter().map(|a| a + 2.0 * WALL)).collect();

    // every ring must seat on the one below rather than falling through its aperture
    for i in 1..profile.len() {
        let lo = py::max(profile[i], profile[i - 1]) / 2.0;
        let hi = py::min(outers[i], outers[i - 1]) / 2.0;
        if outers[i] <= profile[i - 1] || hi - lo <= 0.0 {
            exit(format!("station {} falls through station {i}", i + 1));
        }
    }

    let w = MARGIN * 2.0 + outers.iter().fold(0.0, |s, o| s + o) + GAP * (outers.len() - 1) as f64;
    let h = MARGIN * 2.0 + fmax(outers.iter().copied());
    let cy = h / 2.0;
    let (mut parts, mut holes, mut x) = (vec![], vec![], MARGIN);
    for (i, (&o, &ap)) in outers.iter().zip(&profile).enumerate() {
        let cx = x + o / 2.0;
        let outer = if i == 0 { square(cx, cy, o) } else { circle(cx, cy, o) };
        // aperture and outline in separate stages -- holes before rims
        holes.push(format!("  <path d=\"{}\"/>", circle(cx, cy, ap)));
        parts.push(format!("  <path d=\"{outer}\"/>"));
        x += o + GAP;
    }
    let n = profile.len();
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Mouthpiece - {n} rings, aperture {} to {} to {}mm</title>\n  <desc>1 user unit = 1mm. {} cup rings in 4mm steps, then {} backbore rings in 0.40mm steps. Wall {}mm. Stack {}mm at {}mm a layer.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
        g(profile[0]),
        g(profile[cup.len() - 1]),
        g(profile[n - 1]),
        cup.len(),
        backbore.len(),
        g(WALL),
        g(n as f64 * WALL),
        g(WALL),
        holes.join("\n"),
        parts.join("\n")
    );

    let opts = Opts::of(&a);
    if let Some(b) = bare(&a) {
        exit(format!("{b} takes a value: write it as {b}=VALUE. Nothing has been drawn."));
    }
    for k in opts.keys() {
        if k != "numbers" {
            exit(format!("unknown option --{k}: numbers"));
        }
    }
    let numbers = opts.str("numbers", "yes").to_string();
    if numbers != "yes" && numbers != "no" {
        exit(format!("--numbers: yes or no, not {}", py::str_repr(&numbers)));
    }
    let dflt = in_cut_files("mouthpiece", "mouthpiece-parts-cut-files.svg");
    let out = positional(&a).into_iter().next().unwrap_or(dflt);
    write(&out, &svg);

    // --order=document: this profile narrows to the throat and opens again.
    if numbers == "yes" {
        if let Err(e) = number_rings(&out) {
            unnumbered(&out, &e);
        }
    }

    println!("  {out}: {n} rings, {w:.1} x {h:.1}mm, stack {}mm", g(n as f64 * WALL));
    println!("  apertures {} -> {} -> {}mm", g(profile[0]), g(THROAT), g(profile[n - 1]));
    println!(
        "  numbers   {}",
        if numbers == "yes" { format!("{n} rings engraved, 0 at the bore") } else { "NONE — --numbers=no; the rings carry nothing".to_string() }
    );
}
