//! Generate a bell whose section morphs from the bore's square to a round rim.
//! Rust since 2026-11-07 (bell-round.py until then; the move changed no byte
//! of any sheet). Usage and the reasoning are in DOC below, which --help
//! prints, and in NOTES.md [Two bell families].
#[path = "../common.rs"]
mod common;
#[path = "../rings_solidcraft.rs"]
mod rings_solidcraft;
use common::*;
use lasermade_tools::py::{self, g, powf};
use std::f64::consts::PI;

const RISE: f64 = 3.0;
const MINWALL: f64 = 2.0;
const DIAG: f64 = std::f64::consts::SQRT_2; // across the corners, against 1 across the flats

#[derive(Clone)]
struct Ring {
    ah: f64,
    ac: f64,
    oh: f64,
    oc: f64,
    gain: f64,
    rise: f64,
}

struct Bell {
    bore: f64,
    plate: f64,
    rt: f64,
    l: f64,
    rim: f64,
    gamma: f64,
    lap: f64,
    overhang: f64,
    u0: f64,
    b: f64,
}

impl Bell {
    fn rad(&self, z: f64) -> f64 {
        self.b * powf((self.l - z) + self.u0, -self.gamma)
    }

    /// 0 at the bore's square, 1 at the round rim, read off the ideal profile.
    fn roundness(&self, z: f64, morph: &str) -> f64 {
        let z = py::min(z, self.l);
        match morph {
            "flare" => (self.rad(z) - self.rt) / (self.rim - self.rt),
            "early" => py::min(1.0, 2.0 * z / self.l),
            _ => z / self.l,
        }
    }

    /// The n+1 apertures, from the bore's square at station 0 to the round rim
    /// at n, as rings; the 2mm wall held across the flats AND through the corners.
    fn stations(&self, plies: i64, morph: &str, law: &str) -> (Vec<Ring>, f64) {
        let step = RISE * plies as f64;
        let n = (self.l / step).ceil() as i64;
        let mut r = self.rt;
        let (mut hs, mut cs) = (vec![self.rt], vec![0.0f64]);
        for k in 1..=n {
            let (z0, z1) = (py::min((k - 1) as f64 * step, self.l), py::min(k as f64 * step, self.l));
            r += py::max(self.rad(z1) - self.rad(z0), MINWALL - self.lap);
            let t = self.roundness(k as f64 * step, morph);
            let mut h = if law == "width" { r } else { r * 2.0 / (4.0 - (4.0 - PI) * t * t).sqrt() };
            let (hl, cl) = (hs[hs.len() - 1], cs[cs.len() - 1]);
            h = py::max(py::max(h, hl + MINWALL - self.lap), (MINWALL - self.lap + support(hl, cl, DIAG)) / ((1.0 - t) * DIAG + t));
            hs.push(h);
            cs.push(t * h);
        }
        let mut rings: Vec<Ring> = (0..n as usize)
            .map(|k| Ring {
                ah: hs[k],
                ac: cs[k],
                oh: hs[k + 1] + self.lap,
                oc: cs[k + 1] + self.lap,
                gain: hs[k + 1] - hs[k],
                rise: py::min((k + 1) as f64 * step, self.l) - py::min(k as f64 * step, self.l),
            })
            .collect();
        // Ring 0 is the flange onto the bore: a SHARP square covering the whole
        // end face and standing proud of it.
        rings[0].oh = py::max(rings[0].oh, self.plate / 2.0 + self.overhang);
        rings[0].oc = 0.0;
        (rings, step)
    }

    /// Everything that has to be true before this is worth cutting.
    fn check(&self, rings: &[Ring]) -> (Vec<String>, Vec<String>) {
        let (mut notes, mut fails) = (vec![], vec![]);
        let lap = self.lap;
        let seats: Vec<f64> = rings
            .windows(2)
            .flat_map(|w| [1.0, DIAG].map(|s| support(w[0].oh, w[0].oc, s) - support(w[1].ah, w[1].ac, s)))
            .collect();
        let (smin, smax) = (fmin(seats.iter().copied()), fmax(seats.iter().copied()));
        if smin < lap - 1e-6 {
            fails.push(format!("a joint seats on {smin:.2}mm, under the {}mm lap", g(lap)));
        }
        notes.push(format!("seat        {smin:.2}-{smax:.2}mm per side, flats and corners alike"));

        let proud = rings[0].oh - self.plate / 2.0;
        if rings[0].ah > self.bore / 2.0 + 1e-9 {
            fails.push(format!(
                "the flange aperture is ø{:.1}, wider than the ø{} bore — it would not cover the end face",
                2.0 * rings[0].ah,
                g(self.bore)
            ));
        }
        if proud < 0.0 {
            fails.push(format!(
                "the flange is ø{:.1}, inside the ø{} bore outside — it would not cover the end face",
                2.0 * rings[0].oh,
                g(self.plate)
            ));
        }
        notes.push(format!(
            "flange      ø{:.0} aperture in a ø{:.0} square: covers the bore's {}-{} face, {proud:.1}mm proud",
            2.0 * rings[0].ah,
            2.0 * rings[0].oh,
            g(self.bore),
            g(self.plate)
        ));

        let walls: Vec<(f64, f64)> =
            rings.iter().flat_map(|r| [1.0, DIAG].map(|s| (support(r.oh, r.oc, s) - support(r.ah, r.ac, s), s))).collect();
        let thin = walls[1..].iter().fold(walls[0], |a, &b| if b.partial_cmp(&a) == Some(std::cmp::Ordering::Less) { b } else { a });
        let thick = walls[1..].iter().fold(walls[0], |a, &b| if b.partial_cmp(&a) == Some(std::cmp::Ordering::Greater) { b } else { a });
        if thin.0 < MINWALL - 1e-9 {
            fails.push(format!("wall falls to {:.2}mm, under the {}mm minimum", thin.0, py::repr(MINWALL)));
        }
        notes.push(format!(
            "wall        {:.2}-{:.2}mm (thinnest {})",
            thin.0,
            thick.0,
            if thin.1 == 1.0 { "across a flat" } else { "through a corner" }
        ));

        // The bore must never step inward, corners included.
        let mut lips = 0;
        for w in rings.windows(2) {
            for s in [1.0, DIAG] {
                if support(w[1].ah, w[1].ac, s) < support(w[0].ah, w[0].ac, s) - 1e-9 {
                    lips += 1;
                }
            }
        }
        if lips > 0 {
            fails.push(format!("the bore steps inward at {lips} place(s) — the corners round faster than the horn opens"));
        }
        notes.push(if lips == 0 { "bore        opens at every station, corners included".to_string() } else { format!("bore        {lips} inward step(s)") });
        (notes, fails)
    }
}

/// Rows of `per`, smallest first; nothing nested.
fn layout(rings: &[Ring], per: usize) -> (Vec<(Ring, f64, f64)>, f64, f64) {
    let (gap, m) = (3.0, 3.0);
    let (mut placed, mut y, mut w) = (vec![], m, 0.0f64);
    for chunk in rings.chunks(per) {
        let mut x = m;
        let rowh = fmax(chunk.iter().map(|r| 2.0 * r.oh));
        for r in chunk {
            let o = 2.0 * r.oh;
            placed.push((r.clone(), x + o / 2.0, y + rowh / 2.0));
            x += o + gap;
        }
        w = py::max(w, x - gap + m);
        y += rowh + gap;
    }
    (placed, w, y - gap + m)
}

#[allow(clippy::too_many_arguments)]
fn emit(bl: &Bell, rings: &[Ring], plies: i64, step: f64, path: &str, morph: &str, law: &str) -> (f64, f64) {
    // The row split makes the longest side as short as it will go, area breaking ties.
    let key = |p: usize| {
        let (_, w, h) = layout(rings, p);
        (py::round_dp(py::max(w, h), 6), py::round_dp(w * h, 6))
    };
    let per = (2..=rings.len()).fold(1, |a, p| if key(p).partial_cmp(&key(a)) == Some(std::cmp::Ordering::Less) { p } else { a });
    let (placed, w, h) = layout(rings, per);
    let holes: Vec<String> = placed.iter().map(|(r, cx, cy)| format!("  <path d=\"{}\"/>", rounded(*cx, *cy, r.ah, r.ac))).collect();
    let lines: Vec<String> = placed.iter().map(|(r, cx, cy)| format!("  <path d=\"{}\"/>", rounded(*cx, *cy, r.oh, r.oc))).collect();
    let (ap, rim) = (2.0 * rings[0].ah, 2.0 * rings[rings.len() - 1].oh);
    let n = rings.len();
    let built = n as f64 * step;
    write(
        path,
        &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Square-to-round trumpet bell - {n} rings of {plies} ply, {}mm built on a {}mm profile, {ap:.0}mm square throat to ø{rim:.1}mm round rim</title>\n  <desc>1 user unit = 1mm. Bessel horn, gamma {}, {ap:.0}mm square throat to ø{rim:.1}mm round rim over {built:.0}mm. The section morphs square to round on the {morph} schedule, holding {law}. Each ring is {plies} lamination(s) of {}mm ply, {}mm of rise, seating on the ring below over {}mm per side. Rings stack; they do not telescope.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
            g(built),
            g(bl.l),
            py::repr(bl.gamma),
            g(RISE),
            g(step),
            py::repr(bl.lap),
            holes.join("\n"),
            lines.join("\n")
        ),
    );
    (w, h)
}

/// The bell as a parametric SolidCraft design (2026-11-08), `--solidcraft=FILE`:
/// the bore, the length, the mouth (or rim), gamma, the lap, the overhang,
/// the ply and the wall are parameters, and every ring's size is worked out
/// from them as stations() works it out, the max() at each step included. How
/// many rings there are and how many plies each are the default build's.
fn solidcraft(bl: &Bell, rings: &[Ring], plies: i64, by_mouth: bool, morph: &str, law: &str) -> (String, Vec<f64>) {
    use lasermade_tools::solidcraft::{n as nn, Script};
    let mut s = Script::new();
    s.len("bore", format!("{} mm", nn(bl.bore)), "the bore the bell meets, square");
    s.len("length", format!("{} mm", nn(bl.l)), "the horn's length, throat to rim, on the profile");
    let rim = if by_mouth {
        let m = 2.0 * bl.rim / if law == "area" { PI.sqrt() / 2.0 } else { 1.0 };
        s.len("mouth", format!("{} mm", nn(m)), "the mouth: the rim's diameter if it were round of the same area");
        if law == "area" { "mouth/(1 mm)/2*sqrt(pi)/2" } else { "mouth/(1 mm)/2" }
    } else {
        s.len("rim", format!("{} mm", nn(2.0 * bl.rim)), "the round rim's diameter");
        "rim/(1 mm)/2"
    };
    s.num("gamma", nn(bl.gamma), "the Bessel horn's flare");
    s.len("lap", format!("{} mm", nn(bl.lap)), "how far each ring seats on the one below, per side");
    s.len("overhang", format!("{} mm", nn(bl.overhang)), "the flange past the bore's outside");
    s.len("ply", format!("{} mm", nn(RISE)), "the ply: each lamination's rise, and the bore's walls");
    s.len("min_wall", format!("{} mm", nn(MINWALL)), "the thinnest a ring's wall may be, flats and corners alike");
    s.num("plies", nn(plies as f64), "laminations a ring; the rings are numbered and placed as the default build's");
    let ro = |s: &mut Script, k: &str, e: String| s.num(k, e, "read only");
    ro(&mut s, "B_", "bore/(1 mm)".into());
    ro(&mut s, "L_", "length/(1 mm)".into());
    ro(&mut s, "RT_", "B_/2".into());
    ro(&mut s, "RIM_", rim.into());
    ro(&mut s, "U0_", "L_/((RIM_/RT_)^(1/gamma) - 1)".into());
    ro(&mut s, "BB_", "RT_*(L_ + U0_)^gamma".into());
    ro(&mut s, "STEP_", "plies*ply/(1 mm)".into());
    ro(&mut s, "LAP_", "lap/(1 mm)".into());
    ro(&mut s, "MW_", "min_wall/(1 mm)".into());
    let rad = |z: &str| format!("BB_*((L_ - ({z})) + U0_)^(-gamma)");
    let n = rings.len();
    ro(&mut s, "r_0", "RT_".into());
    ro(&mut s, "h_0", "RT_".into());
    ro(&mut s, "c_0", "0".into());
    for k in 1..=n {
        let (z0, z1) = (format!("min({}*STEP_, L_)", k - 1), format!("min({k}*STEP_, L_)"));
        ro(&mut s, &format!("r_{k}"), format!("r_{} + max({} - {}, MW_ - LAP_)", k - 1, rad(&z1), rad(&z0)));
        let t = match morph {
            "flare" => format!("({} - RT_)/(RIM_ - RT_)", rad(&z1)),
            "early" => format!("min(1, 2*{z1}/L_)"),
            _ => format!("{z1}/L_"),
        };
        ro(&mut s, &format!("t_{k}"), t);
        let raw = if law == "width" { format!("r_{k}") } else { format!("r_{k}*2/sqrt(4 - (4 - pi)*t_{k}^2)") };
        let (hl, cl) = (format!("h_{}", k - 1), format!("c_{}", k - 1));
        ro(
            &mut s,
            &format!("h_{k}"),
            format!("max(max({raw}, {hl} + MW_ - LAP_), (MW_ - LAP_ + (({hl} - {cl})*sqrt(2) + {cl}))/((1 - t_{k})*sqrt(2) + t_{k}))"),
        );
        ro(&mut s, &format!("c_{k}"), format!("t_{k}*h_{k}"));
    }
    let mut rs = vec![];
    for (k, r) in rings.iter().enumerate() {
        let (oh, oc) = if k == 0 {
            ("max(h_1 + LAP_, (B_ + 2*ply/(1 mm))/2 + overhang/(1 mm))".to_string(), "0".to_string())
        } else {
            (format!("(h_{} + LAP_)", k + 1), format!("(c_{} + LAP_)", k + 1))
        };
        rs.push(rings_solidcraft::Ring { e: [format!("h_{k}"), format!("c_{k}"), oh, oc], v: [r.ah, r.ac, r.oh, r.oc] });
    }
    rings_solidcraft::stack(&mut s, &rs, plies as usize, "ply");
    (s.text(), rings_solidcraft::volumes(&rs, plies as usize, RISE))
}

fn main() {
    let a = argv();
    help(&a, DOC);
    let args = positional(&a);
    let opts = Opts::of(&a);
    if let Some(b) = bare(&a) {
        exit(format!("{b} takes a value: write it as {b}=VALUE. Nothing has been drawn."));
    }
    for k in opts.keys() {
        if !["morph", "law", "length", "rim", "mouth", "gamma", "lap", "overhang", "bore", "numbers", "out", "solidcraft"].contains(&k) {
            exit(format!("unknown option --{k}: morph, law, length, rim, mouth, gamma, lap, overhang, bore, numbers, out or solidcraft"));
        }
    }
    let numbers = opts.str("numbers", "yes").to_string();
    if numbers != "yes" && numbers != "no" {
        exit(format!("--numbers: yes or no, not {}", py::str_repr(&numbers)));
    }
    if opts.has("rim") && opts.has("mouth") {
        exit("--rim and --mouth both set the same thing from different ends; pick one");
    }
    let morph = opts.str("morph", "linear").to_string();
    let law = opts.str("law", "area").to_string();
    if !["linear", "flare", "early"].contains(&morph.as_str()) {
        exit(format!("--morph: linear, flare or early, not {}", py::str_repr(&morph)));
    }
    if !["area", "width"].contains(&law.as_str()) {
        exit(format!("--law: area or width, not {}", py::str_repr(&law)));
    }

    // --bore is the air channel this closes onto; the plate is that plus a
    // 3mm wall each side. --rim is the bore's width AT the rim.
    let bore = opts.float("bore", 10.0);
    let plate = bore + 6.0;
    let rt = bore / 2.0;
    let l = opts.float("length", 201.0);
    let mut rim = opts.float("rim", 61.5 * 2.0) / 2.0;
    let gamma = opts.float("gamma", 0.7);
    let lap = opts.float("lap", 3.0);
    let overhang = opts.float("overhang", 3.0);
    // --mouth is the airway at the rim; --rim is the SQUARE bell's width there.
    if let Some(m) = opts.get("mouth") {
        rim = float(m) / 2.0 * if law == "area" { PI.sqrt() / 2.0 } else { 1.0 };
    }
    if l < 2.0 * RISE {
        exit(format!("--length must be at least two rings, {}mm", g(2.0 * RISE)));
    }
    if rim <= rt {
        exit(format!("--rim must open past the {:.0}mm throat", 2.0 * rt));
    }
    let u0 = l / (powf(rim / rt, 1.0 / gamma) - 1.0);
    let b = rt * powf(l + u0, gamma);
    let bl = Bell { bore, plate, rt, l, rim, gamma, lap, overhang, u0, b };

    let budgets: Vec<i64> = if args.is_empty() { vec![67, 20, 15, 10] } else { vec![int(&args[0])] };
    for &want in &budgets {
        let mut plies = 1;
        let (rings, step) = loop {
            let (rings, step) = bl.stations(plies, &morph, &law);
            if rings.len() as i64 <= want {
                break (rings, step);
            }
            plies += 1;
        };
        let (notes, fails) = bl.check(&rings);
        let n = rings.len();
        let built = n as f64 * step;
        let rimd = 2.0 * rings[n - 1].oh;
        let name = match opts.get("out") {
            Some(o) => {
                if budgets.len() > 1 {
                    exit("  --out names one sheet; give a ring budget too, or drop --out and take the generated names");
                }
                o.to_string()
            }
            None => in_cut_files("bell", &format!("bell-round{}-{built:.0}mm-{n}rings-x{plies}-rim{rimd:.0}-cut-files.svg", g(bore))),
        };
        let (ap, rim) = (2.0 * rings[0].ah, 2.0 * rings[n - 1].oh);
        let mouth = rim - 2.0 * lap; // the hole, not the outside of the rim ring
        let a0 = py::degrees((rings[0].gain / rings[0].rise).atan());
        let a1 = py::degrees((rings[n - 1].gain / rings[n - 1].rise).atan());
        println!("\n  {name}  {n} rings x {plies} ply ({}mm rise), {morph} morph, holding {law}", g(step));
        println!("    section     {ap:.0}mm square -> ø{rim:.1}mm round over {built:.0}mm   (ø{mouth:.1} of air at the mouth)");
        println!("    flat angle  {a0:.1}° -> {a1:.1}°   (against the ring's own rise, {}mm at the rim)", g(rings[n - 1].rise));
        for x in &notes {
            println!("    {x}");
        }
        for x in &fails {
            println!("    x {x}");
        }
        if !fails.is_empty() {
            exit(format!("  {name} not written: {} problem(s)", fails.len()));
        }
        if let Some(f) = opts.get("solidcraft") {
            let (js, vols) = solidcraft(&bl, &rings, plies, opts.has("mouth"), &morph, &law);
            write(f, &js);
            println!("    wrote {f}: {} bodies", vols.len());
            if opts.has("volumes") || std::env::var("SC_VOLUMES").is_ok() {
                for v in vols {
                    println!("{v:.4}");
                }
            }
            std::process::exit(0);
        }
        let (w, h) = emit(&bl, &rings, plies, step, &name, &morph, &law);
        if numbers == "yes" {
            if let Err(out) = number_rings(&name) {
                unnumbered(&name, &out);
            }
        }
        println!(
            "    sheet       {w:.0} x {h:.0}mm per pass, {plies} pass(es), {} pieces, {:.2} m2",
            n as i64 * plies,
            w * h * plies as f64 / 1e6
        );
        // The leftover between a whole number of rings and L is a flat collar at the rim.
        let over = built - l;
        if over > 0.01 {
            let near = [n as i64 - 1, n as i64]
                .iter()
                .filter(|&&m| m != 0)
                .map(|&m| ((l - m as f64 * step).abs(), m as f64 * step))
                .fold(None::<(f64, f64)>, |acc, t| match acc {
                    Some(a) if t.partial_cmp(&a) != Some(std::cmp::Ordering::Less) => Some(a),
                    _ => Some(t),
                })
                .unwrap()
                .1;
            let hint = if over <= step / 2.0 { String::new() } else { format!(", --length={} divides evenly", g(near)) };
            println!(
                "    overshoot   {}mm of rings on a {}mm profile; the rim ring is {}mm tall and draws {}mm of curve{hint}",
                g(built),
                g(l),
                g(step),
                g(step - over)
            );
        }
        println!(
            "    numbers     {}",
            if numbers == "yes" { "each ring engraved with its hex index, 0 at the bore" } else { "NONE — --numbers=no; the rings carry nothing" }
        );
        println!("    ok, written");
    }
}

/// The docstring, which --help prints
const DOC: &str = r##"Generate a bell whose section morphs from the bore's square to a round rim.

    target/release/bell-round                     # all four ring budgets
    target/release/bell-round 20                  # one, at most 20 rings
    target/release/bell-round 67 --morph=flare    # a different square-to-round schedule
    target/release/bell-round 67 --law=width      # follow the half-width, not the area
    target/release/bell-round --length=100        # a 100mm bell, same throat and rim
    target/release/bell-round --length=100 --rim=80   # ... and a narrower rim to suit
    target/release/bell-round --length=152 --mouth=80 --bore=10   # the 10mm trumpet's bell
    target/release/bell-round 20 --out=FILE.svg   # one sheet, named, not into cut-files/
    target/release/bell-round --numbers=no       # no engraved ring numbers

Also `--gamma` (the profile exponent), `--lap` (the seat, per side) and
`--overhang` (how far ring 0 stands proud of the bore's face). Every one of
these was accepted and named nowhere in this docstring; the error message you
get for a wrong one has always listed them, which is not documentation.

The four bells bell.rs makes are square end to end: a square bore opening into a square
rim. This one keeps the square only where it has to. Station one is the same 16mm square
that closes onto the bore, and the corners are rounded away up the horn until the rim is
a true circle. Same Bessel profile, gamma 0.7, same 3mm ply, same 1.5mm lap.

THE RINGS STACK; THEY DO NOT TELESCOPE. Each ring's outer contour is the NEXT ring's
aperture offset outward by LAP, and that is what fixes the seat. Offsetting a rounded
square outward by d gives another rounded square -- half-width h+d, corner radius c+d --
and the gap between the two is exactly d in every direction, corners included. So every
joint seats on 1.5mm per side however square or however round the two rings are. A ring's
own wall then works out to its radius gain plus the lap, exactly as in bell.rs.

A ROUNDED SQUARE IS THE RIGHT SHAPE and a superellipse is not. Rounded squares offset
exactly, they draw in H, V and A alone, corner radius 0 is the bore's square exactly, and
corner radius = half-width is a circle exactly. Nothing is approximated at either end of
the morph, so station one really does close onto the bore and the rim really is round.

AREA, NOT WIDTH, is held to the profile. A circle inscribed in a square has 21% less
area, so rounding the corners at constant width would quietly choke the horn at the very
place it should be opening. Each station is widened instead to enclose the area the square
bell would have had -- by nothing at the throat, by 12.8% at the round rim. `--law=width`
turns that off and follows the half-width, at the cost of the area schedule.

The profile constants are restated here rather than imported: bell.rs writes its four
sheets at import time, so importing it would cut a new set of files as a side effect.

Ring sizes and the rise are stated in the file, so bell-section.rs and bell-view.rs read
these sheets like any other. verify_bell.rs does not -- it takes an arc as proof that it
is looking at the mouthpiece -- so the checks it would have run are run here instead,
before anything is written.
"##;
