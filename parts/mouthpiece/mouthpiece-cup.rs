//! Extra rings that turn the end of the mouthpiece into a cup. Rust since
//! 2026-11-07 (mouthpiece-cup.py until then; the move changed no byte of any
//! sheet). Usage and the reasoning are in DOC below and in NOTES.md
//! [Two routes to the same mouthpiece].
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, g, powf};

const WALL: f64 = 3.0; // ring width, and the ply thickness
const RISE: f64 = 3.0; // one ring, one lamination
const MINSEAT: f64 = 1.0;
const GAP: f64 = 2.0;
const MARGIN: f64 = 3.0;

fn main() {
    let a = argv();
    let opts = Opts::of(&a);
    if let Some(b) = bare(&a) {
        exit(format!("{b} takes a value: write it as {b}=VALUE. Nothing has been drawn."));
    }
    for k in opts.keys() {
        if !["rim", "rings", "onto", "start", "numbers"].contains(&k) {
            exit(format!("unknown option --{k}: rim, rings, onto, start or numbers"));
        }
    }
    let numbers = opts.str("numbers", "yes").to_string();
    if numbers != "yes" && numbers != "no" {
        exit(format!("--numbers: yes or no, not {}", py::str_repr(&numbers)));
    }
    let rim = opts.float("rim", 16.5); // rim aperture; a trumpet is 16 to 17mm inside
    let onto = opts.float("onto", 10.06); // the aperture this stacks onto
    let rings = opts.int("rings", 4); // 4 x 3mm = a 12mm bowl
    // Refuse to guess: the default --start holds only for the default --onto.
    let start = if let Some(s) = opts.get("start") {
        int(s)
    } else if opts.has("onto") {
        exit("--onto given without --start: the ring below is not the one this assumes.\n  --start is the number the first cup ring carries -- the ring count of the\n  stack it goes onto. Pass it, or --numbers=no for a bare sheet.")
    } else {
        23 // mouthpiece.rs writes rings 0..22 and ends at the default --onto
    };
    if start < 0 {
        exit("--start cannot be negative");
    }
    let dflt = in_cut_files("mouthpiece", "mouthpiece-cup-parts-cut-files.svg");
    let out_path = positional(&a).into_iter().next().unwrap_or(dflt);
    if rim <= onto {
        exit(format!("--rim must open past the ø{}mm it stacks onto", g(onto)));
    }
    if rings < 1 {
        exit("--rings must be at least 1");
    }

    let (r0, rr, d) = (onto / 2.0, rim / 2.0, rings as f64 * RISE);
    let depth = |k: i64| d - RISE * (k + 1) as f64; // k = 0 is the first ring above
    let apertures: Vec<f64> = (0..rings).map(|k| 2.0 * (rr * rr - (rr * rr - r0 * r0) * powf(depth(k) / d, 2.0)).sqrt()).collect();
    let outers: Vec<f64> = apertures.iter().map(|ap| ap + 2.0 * WALL).collect();

    // every joint, the one onto the glued stack first
    let (mut below, mut seats, mut fails) = (onto, vec![], vec![]);
    for (i, &ap) in apertures.iter().enumerate() {
        let seat = (below + 2.0 * WALL - ap) / 2.0;
        seats.push(seat);
        let wh = if i == 0 { "the glued stack".to_string() } else { format!("ring {i}") };
        if seat <= 0.0 {
            fails.push(format!("ring {} falls through {wh}", i + 1));
        } else if seat < MINSEAT - 1e-9 {
            fails.push(format!("ring {} seats on only {seat:.2}mm at {wh}", i + 1));
        }
        below = ap;
    }
    if !fails.is_empty() {
        for f in &fails {
            println!("  x {f}");
        }
        exit(format!("  {out_path} not written: {} problem(s) — try --rings above {rings}", fails.len()));
    }

    let w = MARGIN * 2.0 + outers.iter().fold(0.0, |s, o| s + o) + GAP * (outers.len() - 1) as f64;
    let h = MARGIN * 2.0 + fmax(outers.iter().copied());
    let (cy, mut x, mut parts, mut holes) = (h / 2.0, MARGIN, vec![], vec![]);
    for (&ap, &o) in apertures.iter().zip(&outers) {
        // aperture and outline in separate stages -- holes before rims
        holes.push(format!("  <path d=\"{}\"/>", circle(x + o / 2.0, cy, ap)));
        parts.push(format!("  <path d=\"{}\"/>", circle(x + o / 2.0, cy, o)));
        x += o + GAP;
    }
    let smin = fmin(seats.iter().copied());
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Mouthpiece cup extension - {rings} rings, ø{} to ø{}mm rim</title>\n  <desc>1 user unit = 1mm. {rings} rings that stack on an existing mouthpiece ending at ø{}mm and open it into a {}mm cup with a ø{}mm rim. The bowl is an ellipse arc, so the wall stands parallel to the axis at the rim and turns toward the throat going down. Wall {}mm, {}mm of rise a ring. Every joint seats on at least {smin:.2}mm per side, the joint onto the glued stack included. These are ADDITIONAL rings; they replace nothing.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
        g(onto),
        g(rim),
        g(onto),
        g(d),
        g(rim),
        g(WALL),
        g(RISE),
        holes.join("\n"),
        parts.join("\n")
    );
    write(&out_path, &svg);

    // --order=document, and --start, because this sheet continues someone else's stack.
    if numbers == "yes" {
        if let Err(e) = number_rings_with(&out_path, &[&format!("--start={start}")]) {
            unnumbered(&out_path, &e);
        }
    }

    println!("  {out_path}: {rings} rings, {w:.1} x {h:.1}mm, adds {}mm of cup", g(d));
    println!("    stacks onto ø{} and opens to a ø{}mm rim", g(onto), g(rim));
    println!(
        "    numbers     {}",
        if numbers == "yes" {
            format!("{start} to {}, continuing the stack below", start + rings - 1)
        } else {
            "NONE — --numbers=no; the rings carry nothing".to_string()
        }
    );
    let half = py::degrees((((apertures[0] - onto) / 2.0) / RISE).atan());
    let n = apertures.len();
    let last = if rings > 1 { apertures[n - 1] - apertures[n - 2] } else { 0.0 };
    println!("    first ring opens {:.2}mm ({half:.0}° half-angle), the rim ring {last:.2}mm — a bowl, not a cone", apertures[0] - onto);
    for (i, (ap, o)) in apertures.iter().zip(&outers).enumerate() {
        println!("    ring {}  aperture ø{ap:6.2}  outer ø{o:6.2}  seats {:.2}mm", i + 1, seats[i]);
    }
}

/// The docstring. This program takes no --help (mouthpiece-cup.py did not
/// either), so this is documentation only.
#[allow(dead_code)]
const DOC: &str = r##"Extra rings that turn the end of the mouthpiece into a cup.

    target/release/mouthpiece-cup [OUT.svg]
    target/release/mouthpiece-cup --rim=17.0        # a wider rim
    target/release/mouthpiece-cup --rings=5         # a deeper bowl, 15mm instead of 12
    target/release/mouthpiece-cup --onto=10.06      # stack onto a different aperture
    target/release/mouthpiece-cup --start=23        # the number its first ring carries
    target/release/mouthpiece-cup --numbers=no      # no engraved ring numbers

THE NUMBERS CONTINUE A STACK, THEY DO NOT START ONE. These rings are glued on top of a
mouthpiece whose rings are already numbered 0 upward, so numbering these from 0 would put
a second ring 0 in one mouthpiece -- the exact confusion the numbers exist to prevent.
`--start` is where they carry on from: 23 by default, because `mouthpiece.rs` writes 23
rings (0 to 22) and its last aperture is the ø10.06 this stacks onto. Change `--onto` and
that no longer holds, so `--start` becomes required rather than assumed.

`mouthpiece-round.rs` ends its cup run at ø10.06, opening 0.40mm a ring: a 3.8 degree
half-angle, which is a tube with ambitions. A trumpet rim is 16 to 17mm across inside, so
the bowl is not shallow, it is absent. These rings are what is missing, and they STACK ON
TOP of a mouthpiece that is already glued -- nothing here replaces a part you have cut.

A CUP IS A BOWL, NOT A CONE, and the difference is where the wall stands up. The profile
here is an ellipse arc through the (depth, radius) plane:

    r(d) = sqrt(R^2 - (R^2 - r0^2) (d/D)^2)

with d measured DOWN from the rim, R the rim radius, r0 the aperture it lands on and D the
depth. Its slope dr/dd is -k d / r, which is zero at d = 0: the wall is parallel to the axis
at the rim and turns hard toward the throat as it goes down. That is what your lip sits on.
A straight cone would meet the rim at an angle and feel like a funnel.

THE FIRST RING HAS TO SEAT ON GLUED WORK. Its outer is the aperture below it plus two
walls, so the seat is WALL minus half the aperture step -- the same rule as the rest of the
mouthpiece, but the ring below is one you cannot re-cut. Every joint is checked, the joint
onto the existing stack included, and nothing is written if one is short.

Rings are 3mm ply and the bowl is a staircase of them. Sand or fill the steps and round the
rim over before playing it; the rim edge as cut is a square corner and your lip will say so.
"##;
