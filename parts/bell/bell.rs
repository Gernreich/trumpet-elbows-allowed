//! Generate a trumpet-profile bell as stacking square rings, at a chosen ring
//! budget. Rust since 2026-11-07 (bell.py until then; the move changed no
//! byte of any sheet). Usage and the reasoning are in DOC below, and in
//! NOTES.md [Two bell families] and [The rim ring is not always the steepest].
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, g, powf};

const RISE: f64 = 3.0;
const MINWALL: f64 = 2.0;
const RT: f64 = 5.0; // 10mm throat = the bore's channel
const PLATE: f64 = 16.0; // the bore's outside. Ring 0 has to cover this whole end face

/// (aperture, outer, wall, angle) per ring
type R = (f64, f64, f64, f64);

struct Bell {
    l: f64,
    lap: f64,
    overhang: f64,
    u0: f64,
    b: f64,
    gamma: f64,
}

impl Bell {
    fn rad(&self, z: f64) -> f64 {
        self.b * powf((self.l - z) + self.u0, -self.gamma)
    }

    fn rings(&self, plies: i64) -> (Vec<R>, f64) {
        let step = RISE * plies as f64;
        let (mut r, mut z, mut out) = (RT, 0.0f64, Vec::<R>::new());
        while z < self.l - 1e-9 {
            let gn = py::max(self.rad(py::min(z + step, self.l)) - self.rad(z), MINWALL - self.lap);
            out.push((2.0 * r, 2.0 * (r + gn + self.lap), gn + self.lap, py::degrees((gn / step).atan())));
            r += gn;
            z += step;
        }
        // Ring 0 is the flange onto the bore: it covers the whole end face and
        // stands proud of it.
        let (a0, o0, _, ang) = out[0];
        let o0 = py::max(o0, PLATE + 2.0 * self.overhang);
        out[0] = (a0, o0, (o0 - a0) / 2.0, ang);
        (out, step)
    }

    fn emit(&self, rs: &[R], plies: i64, step: f64, path: &str) -> (f64, f64) {
        let (gap, m) = (3.0, 3.0);
        let per = 1usize.max((rs.len() as f64).sqrt().ceil() as usize);
        let (mut lines, mut holes, mut y, mut w) = (vec![], vec![], m, 0.0f64);
        for chunk in rs.chunks(per) {
            let mut x = m;
            let rowh = fmax(chunk.iter().map(|q| q.1));
            for &(a, o, _, _) in chunk {
                let cx = x + o / 2.0;
                holes.push(format!("  <path d=\"{}\"/>", sq(cx, y + rowh / 2.0, a)));
                lines.push(format!("  <path d=\"{}\"/>", sq(cx, y + rowh / 2.0, o)));
                x += o + gap;
            }
            w = py::max(w, x - gap + m);
            y += rowh + gap;
        }
        let h = y - gap + m;
        let n = rs.len();
        let built = n as f64 * step;
        write(
            path,
            &format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.3}mm\" height=\"{h:.3}mm\"\n     viewBox=\"0 0 {w:.3} {h:.3}\">\n  <title>Trumpet-profile bell - {n} rings of {plies} ply, {}mm built on a {}mm profile, {:.0}mm square throat to {:.1}mm square rim</title>\n  <desc>1 user unit = 1mm. Bessel horn, gamma {}, throat ø{:.0} to rim ø{:.1}mm over {built:.0}mm. Each ring is {plies} lamination(s) of {}mm ply, {}mm of rise, lapping the ring below by {}mm.</desc>\n  <g fill=\"none\" stroke=\"#ff8000\" stroke-width=\"0.1\">\n{}\n  </g>\n  <g fill=\"none\" stroke=\"#000000\" stroke-width=\"0.1\">\n{}\n  </g>\n</svg>\n",
                g(built),
                g(self.l),
                rs[0].0,
                rs[n - 1].1,
                py::repr(self.gamma),
                2.0 * RT,
                rs[n - 1].1,
                g(RISE),
                g(step),
                py::repr(self.lap),
                holes.join("\n"),
                lines.join("\n")
            ),
        );
        (w, h)
    }
}

fn sq(cx: f64, cy: f64, s: f64) -> String {
    let h = s / 2.0;
    format!("M {:.4},{:.4} H {:.4} V {:.4} H {:.4} Z", cx - h, cy - h, cx + h, cy + h, cx - h)
}

fn main() {
    let a = argv();
    let args = positional(&a);
    let opts = Opts::of(&a);
    if let Some(b) = bare(&a) {
        exit(format!("{b} takes a value: write it as {b}=VALUE. Nothing has been drawn."));
    }
    for k in opts.keys() {
        if !["length", "rim", "gamma", "lap", "overhang", "numbers", "out"].contains(&k) {
            exit(format!("unknown option --{k}: length, rim, gamma, lap, overhang, numbers or out"));
        }
    }
    // Every sheet this run writes gets its own rings numbered from 0.
    let numbers = opts.str("numbers", "yes").to_string();
    if numbers != "yes" && numbers != "no" {
        exit(format!("--numbers: yes or no, not {}", py::str_repr(&numbers)));
    }
    let l = opts.float("length", 201.0);
    let rim = opts.float("rim", 61.5 * 2.0) / 2.0;
    let gamma = opts.float("gamma", 0.7);
    let lap = opts.float("lap", 3.0);
    let overhang = opts.float("overhang", 3.0);
    if l < 2.0 * RISE {
        exit(format!("--length must be at least two rings, {}mm", g(2.0 * RISE)));
    }
    if rim <= RT {
        exit(format!("--rim must open past the ø{:.0}mm throat", 2.0 * RT));
    }
    let u0 = l / (powf(rim / RT, 1.0 / gamma) - 1.0);
    let b = RT * powf(l + u0, gamma);
    let bl = Bell { l, lap, overhang, u0, b, gamma };

    let budgets: Vec<i64> = if args.is_empty() { vec![67, 20, 15, 10] } else { vec![int(&args[0])] };
    for &want in &budgets {
        let mut plies = 1;
        let (rs, step) = loop {
            let (rs, step) = bl.rings(plies);
            if rs.len() as i64 <= want {
                break (rs, step);
            }
            plies += 1;
        };
        let n = rs.len();
        let built = n as f64 * step;
        // square: every ring on this sheet is a square, throat to rim.
        let name = match opts.get("out") {
            Some(o) => {
                if budgets.len() > 1 {
                    exit("  --out names one sheet; give a ring budget too, or drop --out and take the generated names");
                }
                o.to_string()
            }
            None => in_cut_files("bell", &format!("bell-square{:.0}-{built:.0}mm-{n}rings-x{plies}-rim{:.0}-cut-files.svg", rs[0].0, rs[n - 1].1)),
        };
        let (w, h) = bl.emit(&rs, plies, step, &name);
        // --order=document: a bell telescopes, so the order it is written in is
        // the order it is glued in. A sheet that cannot be numbered is removed.
        let numbered = if numbers != "yes" {
            "NONE — --numbers=no; the rings carry nothing".to_string()
        } else {
            if let Err(out) = number_rings(&name) {
                unnumbered(&name, &out);
            }
            format!("{n} rings engraved, 0 at the bore")
        };
        let angles: Vec<f64> = rs.iter().map(|r| r.3).collect();
        let steep = fmax(angles.iter().copied());
        let last = angles[n - 1];
        let rimnote = if (steep - last).abs() < 0.05 { String::new() } else { format!(" (rim ring {last:.1}°)") };
        println!(
            "  {name}  {n:>2} rings x {plies} ply ({}mm rise)  ø{:.0}->ø{:.1}mm  {:.1}°->{steep:.1}°{rimnote}  wall {:.1}-{:.1}mm  sheet {w:.0}x{h:.0}mm",
            g(step),
            rs[0].0,
            rs[n - 1].1,
            fmin(angles.iter().copied()),
            fmin(rs.iter().map(|r| r.2)),
            fmax(rs.iter().map(|r| r.2))
        );
        println!("      numbers: {numbered}");

        // A whole number of rings rarely lands on L; the leftover is a flat
        // collar at the rim. Say so, and which length would divide evenly.
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
            let hint = if over <= step / 2.0 { String::new() } else { format!("   --length={} divides evenly", g(near)) };
            println!(
                "      {}mm of rings on a {}mm profile: the rim ring is {}mm tall and draws {}mm of curve{hint}",
                g(built),
                g(l),
                g(step),
                g(step - over)
            );
        }
    }
}
