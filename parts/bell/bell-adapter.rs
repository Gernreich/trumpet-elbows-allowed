//! Generate the adapter that carries the 7 x 14 port up to the bell's throat.
//! Rust since 2026-11-07 (bell-adapter.py until then; the move changed no
//! byte of the sheet). Usage and the reasoning are in DOC below, and in
//! NOTES.md [The adapter is what makes a ported bore take a bell].
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, g};
use lasermade_tools::text;

const RISE: f64 = 3.0;
const LAP: f64 = 3.0;
const MINWALL: f64 = 2.0; // ply, the seat per side, the thinnest wall
const BORE: f64 = 10.0; // the bore's air channel, and the bell's throat
const PLATE: f64 = 16.0; // the bore's outside, and the face the bell's ring 0 covers

struct Ring {
    aa: f64,
    ab: f64,
    oa: f64,
    ob: f64,
    kind: &'static str,
}

/// 0 to 1 with zero slope at both ends. The whole of "smooth" is in here.
fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// The m+1 apertures, port at 0 and the bell's throat at m, as (a, b)
/// half-widths: a ACROSS the cheek's band, b ALONG the duct.
fn stations(m: i64, pa: f64, pl: f64) -> Vec<(f64, f64)> {
    let (a0, b0) = (pa / 2.0, pl / 2.0);
    let (a1, b1) = (BORE / 2.0, BORE / 2.0);
    let (aa0, aa1) = (4.0 * a0 * b0, 4.0 * a1 * b1);
    let (r0, r1) = (b0 / a0, b1 / a1);
    let mut out: Vec<(f64, f64)> = (0..=m)
        .map(|k| {
            let s = smoothstep(k as f64 / m as f64);
            let area = aa0 + (aa1 - aa0) * s;
            let ratio = r0 + (r1 - r0) * s;
            let a = (area / (4.0 * ratio)).sqrt();
            (a, ratio * a)
        })
        .collect();
    // The ends are solved, not interpolated towards.
    let n = out.len();
    out[0] = (a0, b0);
    out[n - 1] = (a1, b1);
    out
}

/// One ring per gap, plus the collar that presents the bore's own end face.
fn rings_from(sts: &[(f64, f64)]) -> Vec<Ring> {
    let mut rings: Vec<Ring> = sts.windows(2).map(|w| Ring { aa: w[0].0, ab: w[0].1, oa: w[1].0 + LAP, ob: w[1].1 + LAP, kind: "morph" }).collect();
    let (aa, ab) = sts[sts.len() - 1];
    rings.push(Ring { aa, ab, oa: PLATE / 2.0, ob: PLATE / 2.0, kind: "collar" });
    rings
}

fn check(sts: &[(f64, f64)], rings: &[Ring], pa: f64, pl: f64) -> (Vec<String>, Vec<String>) {
    let (mut notes, mut fails) = (vec![], vec![]);
    let walls: Vec<(f64, &str, &str)> =
        rings.iter().flat_map(|r| [(r.oa - r.aa, r.kind, "across"), (r.ob - r.ab, r.kind, "along")]).collect();
    // min() and max() of the tuples: the number, then the names, first of equals
    let lt = |a: &(f64, &str, &str), b: &(f64, &str, &str)| a.0 < b.0 || (a.0 == b.0 && (a.1, a.2) < (b.1, b.2));
    let thin = walls[1..].iter().fold(walls[0], |a, b| if lt(b, &a) { *b } else { a });
    let thick = walls[1..].iter().fold(walls[0], |a, b| if lt(&a, b) { *b } else { a });
    if thin.0 < MINWALL - 1e-9 {
        fails.push(format!("wall falls to {:.2}mm {}, under the {}mm minimum", thin.0, thin.2, g(MINWALL)));
    }
    notes.push(format!("wall        {:.2}-{:.2}mm, thinnest {} on the {} ring", thin.0, thick.0, thin.2, thin.1));

    let seats: Vec<f64> = rings.windows(2).map(|w| py::min(w[0].oa - w[1].aa, w[0].ob - w[1].ab)).collect();
    if !seats.is_empty() && fmin(seats.iter().copied()) < LAP - 1e-9 {
        fails.push(format!("a joint seats on {:.2}mm, under the {}mm lap", fmin(seats.iter().copied()), g(LAP)));
    }
    notes.push(format!(
        "seat        {:.2}-{:.2}mm per side over {} joint(s)",
        fmin(seats.iter().copied()),
        fmax(seats.iter().copied()),
        seats.len()
    ));

    let areas: Vec<f64> = sts.iter().map(|&(a, b)| 4.0 * a * b).collect();
    let amin = fmin(areas.iter().copied());
    notes.push(format!("area        {:.1} to {:.1}mm2, {amin:.1} at its narrowest", areas[0], areas[areas.len() - 1]));
    if amin < py::min(areas[0], areas[areas.len() - 1]) - 1e-9 {
        fails.push(format!("the airway pinches to {amin:.1}mm2, under both ends"));
    }

    // A step inward is a lip for the air to catch on.
    let lips = sts.windows(2).filter(|w| w[1].0 < w[0].0 - 1e-9).count();
    let n = sts.len();
    notes.push(if lips == 0 {
        format!("across      {:.2} to {:.2}mm, opening at every station", 2.0 * sts[0].0, 2.0 * sts[n - 1].0)
    } else {
        format!("across      steps inward at {lips} station(s)")
    });
    if lips > 0 {
        fails.push(format!("the section steps inward across the band at {lips} station(s)"));
    }

    // The long axis MUST close, and does so by design; it is reported, not checked.
    notes.push(format!(
        "along       {:.2} to {:.2}mm, closing {:.2}mm over {} ring(s)",
        2.0 * sts[0].1,
        2.0 * sts[n - 1].1,
        2.0 * (sts[0].1 - sts[n - 1].1),
        rings.len() - 1
    ));

    let top = &rings[rings.len() - 1];
    if (2.0 * top.aa - BORE).abs() > 1e-9 || (2.0 * top.oa - PLATE).abs() > 1e-9 {
        fails.push(format!(
            "the collar is {:.2} in {:.2}, not the {} in {} the bell's ring 0 seats on",
            2.0 * top.aa,
            2.0 * top.oa,
            g(BORE),
            g(PLATE)
        ));
    }
    notes.push(format!(
        "collar      ø{:.0} aperture in a ø{:.0} face - the bore's own end, so the bell stacks unchanged",
        2.0 * top.aa,
        2.0 * top.oa
    ));

    let foot = (2.0 * rings[0].oa, 2.0 * rings[0].ob);
    notes.push(format!("footprint   {:.2} x {:.2}mm on the cheek, around a {} x {} port", foot.0, foot.1, g(pa), g(pl)));
    (notes, fails)
}

fn rect(cx: f64, cy: f64, a: f64, b: f64) -> String {
    format!("M {:.4},{:.4} H {:.4} V {:.4} H {:.4} Z", cx - a, cy - b, cx + a, cy + b, cx - a)
}

/// In a row, port first, so the sheet reads in the order the rings stack.
fn layout(rings: &[Ring]) -> (Vec<(usize, f64, f64)>, f64, f64) {
    let (gap, m) = (3.0, 3.0);
    let (mut placed, mut x) = (vec![], m);
    let rowh = fmax(rings.iter().map(|r| 2.0 * r.ob));
    for (i, r) in rings.iter().enumerate() {
        let w = 2.0 * r.oa;
        placed.push((i, x + w / 2.0, m + rowh / 2.0));
        x += w + gap;
    }
    (placed, x - gap + m, rowh + 2.0 * m)
}

fn emit(rings: &[Ring], path: &str, plies: i64, step: f64, pa: f64, pl: f64) -> (f64, f64) {
    let (placed, w, h) = layout(rings);
    let holes: Vec<String> = placed.iter().map(|&(i, cx, cy)| format!("  <path d=\"{}\"/>", rect(cx, cy, rings[i].aa, rings[i].ab))).collect();
    let lines: Vec<String> = placed.iter().map(|&(i, cx, cy)| format!("  <path d=\"{}\"/>", rect(cx, cy, rings[i].oa, rings[i].ob))).collect();
    let tall = rings.len() as f64 * step;
    let (pa, pl, bore, plate) = (g(pa), g(pl), g(BORE), g(PLATE));
    write(
        path,
        &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Port-to-bell adapter - {} rings of {plies} ply, {}mm, {pa} x {pl} port to a {bore}mm square throat</title>\n  <desc>1 user unit = 1mm. The section runs from the cheek's {pa} x {pl} port to the bore's {bore}mm square, holding area on a smoothstep so both ends are tangent. The last ring is a collar, {bore}mm square in a {plate}mm square face: that is the bore's own end face, so the bell stacks on it unchanged. Each ring is {plies} lamination(s) of {}mm ply, {}mm of rise, seating on the ring below over {}mm per side. Rings stack; they do not telescope. No kerf compensation: these are glued, as bell.py's are. The wide side of each ring lies ALONG the duct; the narrow side goes across the cheek's band. orange #ff8000 cuts the apertures first, black #000000 frees the rings.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
            rings.len(),
            g(tall),
            g(RISE),
            g(step),
            g(LAP),
            holes.join("\n"),
            lines.join("\n")
        ),
    );
    (w, h)
}

fn main() {
    let a = argv();
    let opts = Opts::of(&a);
    for k in opts.keys() {
        if !["rings", "plies", "port", "out"].contains(&k) {
            exit(format!("error: --{k} is not an option here. Try --rings, --plies, --port or --out."));
        }
    }
    let (mut pa, mut pl) = (7.0, 14.0); // ribbon_bore.rs's port, as CUT
    if let Some(p) = opts.get("port") {
        let v: Vec<Option<f64>> = p.to_lowercase().split('x').map(py::float).collect();
        match v.as_slice() {
            [Some(x), Some(y)] => (pa, pl) = (*x, *y),
            _ => exit(format!("error: --port={p} should read like --port=7x14.")),
        }
    }
    let plies = opts.int("plies", 1);
    let step = RISE * plies as f64;

    // The long axis has (pl - BORE)/2 to lose and may lose LAP - MINWALL a ring.
    let floor = 3i64.max((((pl - BORE) / 2.0) / (LAP - MINWALL)).ceil() as i64);
    let m = opts.int("rings", 6);
    if m < floor {
        exit(format!(
            "error: --rings={m} cannot hold the wall. The long axis closes {}mm and may close {}mm a ring, so {floor} is the floor.",
            g((pl - BORE) / 2.0),
            g(LAP - MINWALL)
        ));
    }

    let sts = stations(m, pa, pl);
    let rings = rings_from(&sts);
    let (notes, fails) = check(&sts, &rings, pa, pl);

    let name = format!("bell-adapter-port{}x{}-to-bore{}-{}rings-x{plies}-cut-files.svg", g(pa), g(pl), g(BORE), rings.len());
    let path = match opts.get("out") {
        Some(o) => o.to_string(),
        None => {
            let d = format!("{}/parts/bell/cut-files", repo());
            let _ = std::fs::create_dir(&d);
            let cwd = std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
            text::relpath(&format!("{d}/{name}"), &cwd)
        }
    };

    println!("\n  port-to-bell adapter, {} x {} to {} x {}", g(pa), g(pl), g(BORE), g(BORE));
    println!("  {} rings of {plies} ply, {}mm tall\n", rings.len(), g(rings.len() as f64 * step));
    println!("  {:>4}  {:>14}  {:>14}  {:>8}", "ring", "across", "along", "area");
    for (k, r) in rings.iter().enumerate() {
        println!(
            "  {k:>4}  {:6.2} in {:5.2}  {:6.2} in {:5.2}  {:7.1}mm2{}",
            2.0 * r.aa,
            2.0 * r.oa,
            2.0 * r.ab,
            2.0 * r.ob,
            4.0 * r.aa * r.ab,
            if r.kind == "collar" { "   the collar the bell sits on" } else { "" }
        );
    }
    println!();
    for x in &notes {
        println!("  {x}");
    }
    println!();
    if !fails.is_empty() {
        for x in &fails {
            println!("  FAIL  {x}");
        }
        println!("\n  {} check(s) failed. Nothing written.", fails.len());
        std::process::exit(1);
    }
    let (w, h) = emit(&rings, &path, plies, step, pa, pl);
    println!("  wrote {path}  ({w:.0} x {h:.0}mm)");
}

/// The docstring. This program takes no --help (bell-adapter.py did not
/// either: a bare --switch is ignored here), so this is documentation only.
#[allow(dead_code)]
const DOC: &str = r##"Generate the adapter that carries the 7 x 14 port up to the bell's throat.

    target/release/bell-adapter                  # the default 6-ring adapter
    target/release/bell-adapter --rings=8        # a longer, gentler transition
    target/release/bell-adapter --plies=2        # 6mm of rise a ring instead of 3
    target/release/bell-adapter --port=7x14      # a different port
    target/release/bell-adapter --out=FILE.svg   # somewhere other than cut-files/

A ported bore does not open at its mouth; it opens through a 7 x 14mm slot in the
cheek, and the bell's throat is the bore's own 10mm square. Those two are the same
area to within 2mm2 and nothing like the same shape, so something has to turn one
into the other. This is that part, and it is the reason the bell itself needs no
change: the top ring here presents exactly the face the bore's end presents, a
10mm square hole in a 16mm square, so bell-round.rs's ring 0 lands on it as it was
always going to land on the bore.

WHY THE PORT IS NOT SQUARE is set out in ribbon_bore.rs beside PORT_ACROSS, and it
is worth repeating because it is what this part exists to undo. The tab slots run
up both cheek walls the whole length of the bore, so what a port has to clear is
measured ACROSS the run and nothing else. Length ALONG the run is free. A 7 x 14
slot therefore clears the slots by 1.66mm where a bore-square 10 x 10 clears them
by 0.16mm, and buys back the area it gave away by growing in the direction that
costs nothing. The price is that the opening is the wrong shape for a horn, and
the price is paid here.

AREA IS THE SCHEDULE, and the aspect ratio is what moves. Each station encloses
the area the schedule asks for, 98mm2 at the port to 100mm2 at the throat, while
the ratio of the two half-widths runs 2:1 down to 1:1. Holding area is the whole
point: a horn that pinches anywhere is a horn with a step in it, and interpolating
the two half-widths straight -- the obvious thing -- dips the area 4% in the
middle, because the long axis loses faster than the short axis gains.

BOTH ENDS ARE TANGENT, which is what "smooth" has to mean for a part whose ends
are bolted to other parts. The schedule is a smoothstep, 3t^2 - 2t^3, on the area
and on the aspect alike, so the section leaves the port and arrives at the throat
with zero rate of change. A linear schedule meets the port at full tilt and puts a
visible crease in the first joint.

THE SECTION IS A SHARP RECTANGLE ALL THE WAY UP and never a rounded one. Both ends
are sharp -- the port is a slot with square corners, the throat is the bore's
square -- so there is nothing to round and nothing to round back. bell-round.rs
rounds because it has to reach a circle; this does not.

RINGS STACK; THEY DO NOT TELESCOPE, the same as every other ring in this project.
Each ring's outer contour is the NEXT station's aperture grown by LAP, so each
seats on the one below by LAP per side, and a ring's wall is its own gain plus the
lap. That is also what sets the floor on how fast the section may move: a
half-width may SHRINK by at most LAP - MINWALL in one ring, or the wall on that
side thins past the minimum. The long axis has 2mm to lose and may lose 1mm a
ring, so three rings is the hard floor and the default is six.

NO KERF COMPENSATION, matching bell.rs and bell-round.rs. These rings are glued
laminations, not a press fit: a tenth of a millimetre goes into the glue line.
The bore's sheets compensate because their tabs have to enter their mortices.
"##;
