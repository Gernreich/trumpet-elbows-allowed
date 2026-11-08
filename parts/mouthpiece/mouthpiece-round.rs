//! Generate a whole mouthpiece: backbore, throat, entrance and cup, in one
//! sheet, square at the bore and rounding as fast as the seat allows. Rust
//! since 2026-11-07 (mouthpiece-round.py until then; the move changed no byte
//! of any sheet). Usage and the reasoning are in DOC below, which --help
//! prints, and in NOTES.md [The mouthpiece's taper is 2.5 mm because it has
//! to be] and [The mouthpiece has two layouts].
#[path = "../common.rs"]
mod common;
#[path = "../rings_solidcraft.rs"]
mod rings_solidcraft;
use common::*;
use lasermade_tools::py::{self, g, powf};

const WALL: f64 = 3.0; // ring width, and the ply is 3mm so a ring is as thick as it is wide
const NECK: f64 = 5.0; // where the backbore run ends and the throat begins
const THROAT: f64 = 3.66; // a #27 drill, the standard trumpet mouthpiece throat
const GAP: f64 = 2.0;
const MARGIN: f64 = 3.0;
const DIAG: f64 = std::f64::consts::SQRT_2;

fn main() {
    let a = argv();
    help(&a, DOC);
    let opts = Opts::of(&a);
    if let Some(b) = bare(&a) {
        exit(format!("{b} takes a value: write it as {b}=VALUE. Nothing has been drawn."));
    }
    for k in opts.keys() {
        if !["taper", "seat", "rim", "bowl", "layout", "backbore", "power", "bore", "numbers", "solidcraft"].contains(&k) {
            exit(format!("unknown option --{k}: taper, seat, rim, bowl, layout, backbore, power, bore, numbers or solidcraft"));
        }
    }
    let taper = opts.float("taper", 2.5); // 4.0 cannot be made square
    let minseat = opts.float("seat", 1.0); // every joint, flats and corners alike
    // 17 because that is the sheet this repository SHIPS: the rim is not in the
    // filename, so a bare run has to rebuild what is shipped.
    let rim = opts.float("rim", 17.0);
    let bowl_n = opts.int("bowl", 4); // rings of cup: 4 x 3mm = a 12mm bowl
    let layout_ = opts.str("layout", "trumpet").to_string();
    let nbb = opts.int("backbore", 25); // trumpet layout: rings of backbore
    let back_p = opts.float("power", 2.0); // trumpet layout: 1 is a cone, higher opens later
    // --bore is the square air channel this closes onto; the plate is that plus
    // a 3mm wall each side. The throat does NOT follow it.
    let bore = opts.float("bore", 10.0);
    let plate = bore + 6.0;
    let numbers = opts.str("numbers", "yes").to_string();
    if numbers != "yes" && numbers != "no" {
        exit(format!("--numbers: yes or no, not {}", py::str_repr(&numbers)));
    }
    if bore <= THROAT {
        exit(format!("--bore must be wider than the ø{}mm throat", g(THROAT)));
    }
    if layout_ == "asbuilt" {
        exit("--layout=asbuilt is now --layout=legacy, and it is no longer the default.\n  It names a profile built before the trumpet layout existed; trumpet is the\n  one to cut new.");
    }
    if layout_ != "trumpet" && layout_ != "legacy" {
        exit(format!("--layout: trumpet or legacy, not {}", py::str_repr(&layout_)));
    }
    // Both the bore and the layout go in the name: they make one sheet a different part.
    let dflt = in_cut_files("mouthpiece", &format!("mouthpiece-bore{}-{layout_}-parts-cut-files.svg", g(bore)));
    let out_path = positional(&a).into_iter().next().unwrap_or(dflt);

    // The cup, as an ellipse arc through the (depth, radius) plane, its slope
    // zero at the rim.
    let ellipse_bowl = |from_dia: f64| -> Vec<f64> {
        if rim <= from_dia {
            exit(format!("--rim must open past the ø{}mm the cup starts from", g(from_dia)));
        }
        let (r0, rr, d) = (from_dia / 2.0, rim / 2.0, bowl_n as f64 * WALL);
        (0..bowl_n).map(|k| 2.0 * (rr * rr - (rr * rr - r0 * r0) * powf((d - WALL * (k + 1) as f64) / d, 2.0)).sqrt()).collect()
    };

    let (backbore, entrance, bowl): (Vec<f64>, Vec<f64>, Vec<f64>) = if layout_ == "trumpet" {
        // A real mouthpiece spends its length on the BACKBORE and keeps the cup short.
        let bb = (0..=nbb).map(|k| THROAT + (bore - THROAT) * powf((nbb - k) as f64 / nbb as f64, back_p)).collect();
        (bb, vec![], ellipse_bowl(THROAT))
    } else {
        // As built: a short backbore and a 51mm entrance, which is close to inverted.
        let steps = py::round((bore - NECK) / taper);
        let bb = (0..=steps).map(|i| bore - taper * i as f64).collect();
        let en: Vec<f64> = (0..17).map(|i| py::round_dp(THROAT + 0.40 * i as f64, 3)).collect();
        let last = en[en.len() - 1];
        (bb, en, ellipse_bowl(last))
    };
    let profile: Vec<f64> = backbore.iter().chain(&entrance).chain(&bowl).copied().collect();
    let n = profile.len();
    let hs: Vec<f64> = profile.iter().map(|ap| ap / 2.0).collect();

    // The wall each ring needs: the seat above a joint is the wall of the
    // NARROW ring there, less half the aperture step.
    let mut ws = vec![WALL; n];
    for i in 1..n {
        let step = profile[i] - profile[i - 1];
        if step > 0.0 {
            ws[i - 1] = py::max(ws[i - 1], minseat + step / 2.0);
        } else {
            ws[i] = py::max(ws[i], minseat - step / 2.0);
        }
    }

    // Roundness: the largest corner radius each station can afford.
    let mut cs = vec![0.0f64]; // station one is the bore
    for i in 1..n {
        let h = hs[i];
        let room = (if hs[i] > hs[i - 1] { ws[i - 1] } else { ws[i] }) - minseat;
        let prev = support(hs[i - 1], cs[i - 1], DIAG);
        let (lo, hi) = (py::max(h, prev - room), py::min(h * DIAG, prev + room));
        if lo > hi + 1e-9 {
            exit(format!(
                "station {} cannot hold {}mm of seat at any corner radius — try --taper below {} or --seat below {}",
                i + 1,
                py::repr(minseat),
                g(taper),
                g(minseat)
            ));
        }
        cs.push((h * DIAG - lo) / (DIAG - 1.0));
    }

    // Outers, and the checks, before anything is drawn. Station one's outer is
    // the sharp plate; everywhere else the aperture offset by its wall.
    let oh: Vec<f64> = std::iter::once(plate / 2.0).chain((1..n).map(|i| hs[i] + ws[i])).collect();
    let oc: Vec<f64> = std::iter::once(0.0).chain((1..n).map(|i| cs[i] + ws[i])).collect();

    let (mut fails, mut seats) = (vec![], vec![]);
    for i in 1..n {
        for s in [1.0, DIAG] {
            let room = py::min(support(oh[i], oc[i], s), support(oh[i - 1], oc[i - 1], s))
                - py::max(support(hs[i], cs[i], s), support(hs[i - 1], cs[i - 1], s));
            seats.push(room);
            if room <= 0.0 {
                fails.push(format!(
                    "station {} falls through station {i} {}",
                    i + 1,
                    if s == 1.0 { "across the flats" } else { "through the corners" }
                ));
            } else if room < minseat - 1e-6 {
                fails.push(format!("station {} seats on only {room:.2}mm at station {i}", i + 1));
            }
        }
    }

    let layout = |per: usize| -> (Vec<(usize, f64, f64)>, f64, f64) {
        let (mut placed, mut y, mut w) = (vec![], MARGIN, 0.0f64);
        let mut k = 0;
        while k < n {
            let idx = k..(k + per).min(n);
            let (mut x, rowh) = (MARGIN, fmax(idx.clone().map(|i| 2.0 * oh[i])));
            for i in idx {
                placed.push((i, x + oh[i], y + rowh / 2.0));
                x += 2.0 * oh[i] + GAP;
            }
            w = py::max(w, x - GAP + MARGIN);
            y += rowh + GAP;
            k += per;
        }
        (placed, w, y - GAP + MARGIN)
    };

    if !fails.is_empty() {
        for f in &fails {
            println!("  x {f}");
        }
        exit(format!("  {out_path} not written: {} problem(s)", fails.len()));
    }

    // the mouthpiece as a parametric SolidCraft design (2026-11-08)
    if let Some(f) = opts.get("solidcraft") {
        if layout_ != "trumpet" {
            exit("--solidcraft draws the trumpet layout; the legacy one is a record of what was built before it");
        }
        use lasermade_tools::solidcraft::{n as nn, Script};
        let mut s = Script::new();
        s.len("bore", format!("{} mm", nn(bore)), "the bore the mouthpiece meets, square");
        s.len("throat", format!("{} mm", nn(THROAT)), "the throat, round: a #27 drill");
        s.len("rim", format!("{} mm", nn(rim)), "the cup's rim, inside");
        s.num("power", nn(back_p), "how the backbore opens: 1 is a cone, higher opens later");
        s.len("seat", format!("{} mm", nn(minseat)), "the least every joint seats on, flats and corners alike");
        s.len("ply", format!("{} mm", nn(WALL)), "the ply: each ring's rise, and its least wall");
        s.num("nbb", nn(nbb as f64), "read only: backbore rings, the default build's");
        s.num("nbowl", nn(bowl_n as f64), "read only: bowl rings, the default build's");
        let ro = |s: &mut Script, k: &str, e: String| s.num(k, e, "read only");
        ro(&mut s, "B_", "bore/(1 mm)".into());
        ro(&mut s, "T_", "throat/(1 mm)".into());
        ro(&mut s, "W_", "ply/(1 mm)".into());
        ro(&mut s, "S_", "seat/(1 mm)".into());
        ro(&mut s, "RR_", "rim/(1 mm)/2".into());
        // the profile: the backbore's power law, then the elliptic bowl
        let mut pe: Vec<String> = (0..=nbb).map(|k| format!("T_ + (B_ - T_)*({}/nbb)^power", nbb - k)).collect();
        pe.extend((0..bowl_n).map(|k| format!("2*sqrt(RR_^2 - (RR_^2 - (T_/2)^2)*((nbowl*W_ - W_*{})/(nbowl*W_))^2)", k + 1)));
        for (i, e) in pe.iter().enumerate() {
            ro(&mut s, &format!("p_{i}"), e.clone());
            ro(&mut s, &format!("h_{i}"), format!("p_{i}/2"));
        }
        // each ring's wall: the ply, or wider where the next station steps
        let mut we: Vec<String> = vec!["W_".into(); n];
        for i in 1..n {
            if profile[i] - profile[i - 1] > 0.0 {
                we[i - 1] = format!("max({}, S_ + (p_{i} - p_{})/2)", we[i - 1], i - 1);
            } else {
                we[i] = format!("max({}, S_ - (p_{i} - p_{})/2)", we[i], i - 1);
            }
        }
        for (i, e) in we.iter().enumerate() {
            ro(&mut s, &format!("w_{i}"), e.clone());
        }
        // each corner: as round as the seat on the station below allows
        ro(&mut s, "c_0", "0".into());
        for i in 1..n {
            let room = if hs[i] > hs[i - 1] { format!("w_{}", i - 1) } else { format!("w_{i}") };
            let prev = format!("((h_{0} - c_{0})*sqrt(2) + c_{0})", i - 1);
            ro(&mut s, &format!("c_{i}"), format!("(h_{i}*sqrt(2) - max(h_{i}, {prev} - ({room} - S_)))/(sqrt(2) - 1)"));
        }
        let rs: Vec<rings_solidcraft::Ring> = (0..n)
            .map(|i| {
                let (o, c) = if i == 0 { ("(B_ + 2*W_)/2".to_string(), "0".to_string()) } else { (format!("(h_{i} + w_{i})"), format!("(c_{i} + w_{i})")) };
                rings_solidcraft::Ring { e: [format!("h_{i}"), format!("c_{i}"), o, c], v: [hs[i], cs[i], oh[i], oc[i]] }
            })
            .collect();
        rings_solidcraft::stack(&mut s, &rs, 1, "ply");
        write(f, &s.text());
        println!("  wrote {f}: {n} rings");
        if std::env::var("SC_VOLUMES").is_ok() {
            for v in rings_solidcraft::volumes(&rs, 1, WALL) {
                println!("{v:.4}");
            }
        }
        std::process::exit(0);
    }
    let key = |p: usize| {
        let (_, w, h) = layout(p);
        (py::round_dp(py::max(w, h), 6), py::round_dp(w * h, 6))
    };
    let per = (2..=n).fold(1, |b, p| if key(p).partial_cmp(&key(b)) == Some(std::cmp::Ordering::Less) { p } else { b });
    let (placed, w, h) = layout(per);
    // aperture and outline as separate paths so each can take its own cut stage
    let holes: Vec<String> = placed.iter().map(|&(i, cx, cy)| format!("  <path d=\"{}\"/>", rounded(cx, cy, hs[i], cs[i]))).collect();
    let parts: Vec<String> = placed.iter().map(|&(i, cx, cy)| format!("  <path d=\"{}\"/>", rounded(cx, cy, oh[i], oc[i]))).collect();

    let round_at = (0..n).find(|&i| (cs[i] - hs[i]).abs() < 1e-6).unwrap_or_else(|| exit("TypeError: list indices must be integers or slices, not NoneType"));
    let (wmin, wmax) = (fmin(ws.iter().copied()), fmax(ws.iter().copied()));
    let (smin, smax) = (fmin(seats.iter().copied()), fmax(seats.iter().copied()));
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Square-to-round mouthpiece, {layout_} layout - {n} rings, {}mm square bore to {}mm throat to ø{}mm lip</title>\n  <desc>1 user unit = 1mm. Station one is a sharp {}mm square aperture in a sharp {}mm square plate, matching the bore corner for corner. The corners round away going up and the section is a true circle from the {}mm station on, so the {}mm throat and the whole cup are round. The {layout_} layout: {} backbore rings, {} entrance rings, then {} bowl rings to a ø{}mm rim. Wall {} to {wmax:.2}mm, 3mm of rise a ring, stack {}mm. Every joint seats on at least {smin:.2}mm per side, flats and corners alike. Rings stack; they do not telescope.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
        g(bore),
        g(THROAT),
        g(profile[n - 1]),
        g(bore),
        g(plate),
        g(profile[round_at]),
        g(THROAT),
        backbore.len(),
        entrance.len(),
        bowl.len(),
        g(rim),
        g(wmin),
        g(n as f64 * WALL),
        holes.join("\n"),
        parts.join("\n")
    );
    write(&out_path, &svg);

    // --order=document, always: a mouthpiece narrows to the throat and opens again.
    if numbers == "yes" {
        if let Err(e) = number_rings(&out_path) {
            unnumbered(&out_path, &e);
        }
    }

    println!("  {out_path}: {n} rings, {w:.1} x {h:.1}mm, stack {}mm", g(n as f64 * WALL));
    println!("    airway     {}mm square -> {} throat -> ø{} lip", g(bore), g(THROAT), g(profile[n - 1]));
    println!("    layout     {layout_}");
    if layout_ == "trumpet" {
        println!(
            "    backbore   {} rings, {}mm, ø{} down to the ø{} throat, power {}",
            backbore.len(),
            g((backbore.len() - 1) as f64 * WALL),
            g(bore),
            g(THROAT),
            g(back_p)
        );
    } else {
        println!("    backbore   {} rings in {}mm steps to the ø{} neck", backbore.len(), g(taper), g(NECK));
        println!(
            "    entrance   {} rings, ø{} throat opening to ø{} over {}mm",
            entrance.len(),
            g(THROAT),
            g(entrance[entrance.len() - 1]),
            g(entrance.len() as f64 * WALL)
        );
    }
    println!(
        "    bowl       {} rings, ø{} to a ø{}mm rim over {}mm",
        bowl.len(),
        g(profile[backbore.len() + entrance.len() - 1]),
        g(rim),
        g(bowl_n as f64 * WALL)
    );
    println!("    wall       {wmin:.2}-{wmax:.2}mm, thickest where the bowl turns");
    println!("    round from station {} (ø{}mm), {}mm up from the bore", round_at + 1, g(profile[round_at]), g(round_at as f64 * WALL));
    println!("    seat       {smin:.2}-{smax:.2}mm per side, flats and corners alike");
    println!(
        "    numbers    {}",
        if numbers == "yes" { "each ring engraved with its hex index, 0 at the bore" } else { "NONE — --numbers=no; the rings carry nothing" }
    );
    println!("    ok, written");
}

/// The docstring, which --help prints
const DOC: &str = r##"Generate a whole mouthpiece: backbore, throat, entrance and cup, in one sheet.

    target/release/mouthpiece-round [OUT.svg]
    target/release/mouthpiece-round --taper=2.0     # finer steps down the bore-side run
    target/release/mouthpiece-round --seat=1.25     # demand more seat, round more slowly
    target/release/mouthpiece-round --rim=16.5      # a narrower rim
    target/release/mouthpiece-round --bowl=5        # a 15mm cup instead of 12mm
    target/release/mouthpiece-round --layout=trumpet   # real proportions, for a new one
    target/release/mouthpiece-round --numbers=no      # no engraved ring numbers

`mouthpiece.rs` puts a 10mm ROUND aperture in its 16mm square plate, so the joint that
meets the bore throws away the bore's corners: a 10mm square channel opens into a 10mm
circle and 21% of the area steps out of the airway at station one. This one starts as the
bore actually is -- a sharp 10mm square inside a sharp 16mm plate -- and rounds the corners
away going up, exactly as bell-round.rs does going out. Same rounded squares, same exact
offsets, opposite direction.

THE 4mm TAPER HAD TO GO. A ring's outer is its aperture offset by WALL, so the seat the
next ring lands on is WALL minus how much the two apertures differ in that direction. On
the flats that is half the aperture step: the old 4mm steps left 3 - 2 = 1.00mm. Through
the corners of a SHARP square the same step costs step x root2 = 2.83mm, leaving 0.17mm --
and rounding a corner pulls the diagonal in further, which takes it negative. So a square
mouthpiece cannot be built on a 4mm taper at all. At 2.5mm the corner seat is 1.23mm with
room to round. Those two figures are the STEP's, not the bore's, so they hold whatever
`--bore` says. The run is the same cone either way, sampled finer -- at the 10mm bore one
more ring and 3mm more mouthpiece, and no change to the profile the air sees.

ROUNDNESS IS NOT SCHEDULED, IT IS AS FAST AS THE SEAT ALLOWS. Each station takes the
largest corner radius that still leaves MINSEAT in both directions, in closed form. The
part therefore rounds as quickly as its own geometry permits and no quicker -- on the
default profile it reaches a true circle at station 3 (ø9.026), 6mm up from the bore, and
the throat and the whole cup are round. A run prints that station rather than assuming it.

THE TWO LAYOUTS differ in where the length goes, and at the 10mm bore in how much of it
there is -- the backbore is what scales with the bore.

    --layout=trumpet   75mm backbore, 12mm cup            30 rings, 90mm   (default)
    --layout=legacy    9mm backbore, 51mm entrance,
                       12mm cup                           24 rings, 72mm

A real mouthpiece spends its length on the backbore and keeps the cup short: the throat
opens into the cup almost at once, and the long gradual run is on the other side of it.
`legacy` has that close to inverted -- a short steep backbore and 51mm of near-cylindrical
entrance on the LIP side of the throat, which is a 51mm-deep cup by any honest reading. It
exists because a mouthpiece was built to that profile before the trumpet layout did, and
its sheet is kept as a record of that part. It was the default until 2026-09-03 and called
`asbuilt`, which said when it was made rather than whether to cut it. Do not cut it new.

`--backbore` sets the backbore rings and `--power` how it opens: 1 is a straight cone,
higher keeps it near-cylindrical off the throat and opens it later, as a real one does.

THE WALL IS NOT A CONSTANT, and it cannot be. A ring's outer is its aperture plus two
walls, so the seat above it is the wall of whichever ring is narrow there, less half the
aperture step. A real cup turns far too fast for 3mm: straight off the throat it goes ø3.66
to ø11.25 in one ring, which wants 4.80mm of wall to have anything to sit on. So each ring
takes the wall its own step demands -- 3mm nearly everywhere, 4.80mm on the throat ring of
a trumpet cup, and nowhere else. Holding the wall at 3mm would have forced a 36mm cup, which
is not a cup.

THE CUP IS A BOWL, NOT A CONE. This used to stop at ø10.06 opening 0.40mm a ring -- a 3.8
degree half-angle, a tube, against the 16 to 17mm a trumpet rim is across inside. The bowl
is an ellipse arc whose slope is zero at the rim, so the wall stands parallel to the axis
where the lip sits and turns toward the throat going down; its first ring opens 3.21mm and
its rim ring 0.33mm. A cone would meet the rim at an angle and feel like a funnel.

The three runs are named for the anatomy, which `mouthpiece.rs` gets backwards:

    BACKBORE   the bore down to 5mm, the end that closes onto it, square becoming round
    ENTRANCE   the 3.66mm throat, then 0.40mm a ring out to 10.06 -- 51mm of it, which is
               long for a mouthpiece and is where to look first if it plays stuffy
    BOWL       10.06 -> 16.5mm over 12mm, the cup proper, ending at the rim

A sheet from here is the same part as a `--layout=legacy` sheet plus mouthpiece-cup.rs's
four rings, ring for ring, so both routes number identically. `mouthpiece-cup.rs` stays for
retrofitting a mouthpiece already glued without a bowl.

The bowl is a staircase of 3mm ply. Sand or fill the steps and round the rim over before
playing it; as cut the rim edge is a square corner and your lip will say so.
"##;
