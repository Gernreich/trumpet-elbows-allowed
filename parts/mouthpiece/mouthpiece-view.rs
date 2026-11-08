//! Draw the mouthpiece as built: rings stacked, seen from above the cup.
//! Rust since 2026-11-07 (mouthpiece-view.py until then). These rings are
//! round -- read from the arc radii in the cut file -- except station one, the
//! square plate that meets the bore. Circles are sampled and projected, so a
//! circle in plan becomes a true ellipse. DISPLAY ONLY.
//!
//!     target/release/mouthpiece-view [OUT.svg] [--src=SHEET.svg]
//!
//! --src names the cut file, the shipped trumpet layout by default; a bare
//! argument is the OUTPUT. Rings are paired by index, in file order, which is
//! assembly order: by size interleaves the cup and the backbore.
#[path = "../common.rs"]
mod common;
#[path = "../views.rs"]
mod views;
use common::*;
use lasermade_tools::py::{self, mcos as cos, msin as sin};
use lasermade_tools::re;
use std::f64::consts::PI;
use views::{display_out, iso, read, P};

const RISE: f64 = 3.0;
const N: usize = 96;

/// A path's command ENDPOINTS, not every number in it: an arc's
/// `large,sweep` looks exactly like a coordinate pair.
fn ends(d: &str) -> Vec<P> {
    let (mut out, mut x, mut y) = (vec![], 0.0f64, 0.0f64);
    let num = re::rx(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?");
    for m in re::all(&re::rx(r"([MLAZmlaz])([^MLAZmlaz]*)"), d) {
        let cmd = re::g(&m, 1).chars().next().unwrap();
        let n: Vec<f64> = re::findall1(&num, re::g(&m, 2), 0).iter().map(|v| float(v)).collect();
        let u = cmd.is_ascii_uppercase();
        match cmd.to_ascii_uppercase() {
            'M' | 'L' => {
                let mut i = 0;
                while i + 1 < n.len() {
                    (x, y) = if u { (n[i], n[i + 1]) } else { (x + n[i], y + n[i + 1]) };
                    out.push((x, y));
                    i += 2;
                }
            }
            'A' => {
                let mut i = 0;
                while (i as i64) < n.len() as i64 - 6 {
                    let (ex, ey) = (n[i + 5], n[i + 6]);
                    (x, y) = if u { (ex, ey) } else { (x + ex, y + ey) };
                    out.push((x, y));
                    i += 7;
                }
            }
            _ => {}
        }
    }
    out
}

/// Each path as (centre x, centre y, half-width, diameters it draws, largest first).
fn circles(chunk: &str) -> Vec<(f64, f64, f64, Vec<f64>)> {
    let mut out = vec![];
    let arc = re::rx(r"[Aa]\s*([\d.]+),");
    for d in views::path_ds(chunk) {
        let pts = ends(&d);
        if pts.is_empty() {
            continue;
        }
        let xs = || pts.iter().map(|q| q.0);
        let ys = || pts.iter().map(|q| q.1);
        let (cx, cy) = ((fmin(xs()) + fmax(xs())) / 2.0, (fmin(ys()) + fmax(ys())) / 2.0);
        let half = py::max(fmax(xs()) - fmin(xs()), fmax(ys()) - fmin(ys())) / 2.0;
        let mut r: Vec<f64> = re::findall1(&arc, &d, 1).iter().map(|v| py::round_dp(float(v) * 2.0, 3)).collect();
        r.sort_by(|a, b| b.partial_cmp(a).unwrap());
        r.dedup();
        out.push((cx, cy, half, r));
    }
    out
}

/// outer boundary then aperture, each as projected points
fn ring_pts(a: f64, o: f64, round_: bool, z: f64) -> Vec<Vec<P>> {
    let mut out = vec![];
    for dia in [o, a] {
        let r = dia / 2.0;
        if round_ || dia == a {
            out.push((0..N).map(|i| iso(r * cos(2.0 * PI * i as f64 / N as f64), r * sin(2.0 * PI * i as f64 / N as f64), z)).collect());
        } else {
            out.push([(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().map(|&(sx, sy)| iso(sx * r, sy * r, z)).collect());
        }
    }
    out
}

/// The visible half of one wall, as one path: back faces culled, every quad
/// wound one way so a nonzero fill cannot cancel.
fn wall(edge: &[P], z: f64, outward: bool) -> String {
    let mut subs = vec![];
    for i in 0..edge.len() {
        let ((ax, ay), (bx, by)) = (edge[i], edge[(i + 1) % edge.len()]);
        // outward normal of a CCW edge; the viewer is at +X +Y +Z
        let (nx, ny) = (by - ay, -(bx - ax));
        if (if outward { nx + ny } else { -(nx + ny) }) <= 0.0 {
            continue;
        }
        let mut q = [iso(ax, ay, z), iso(bx, by, z), iso(bx, by, z + RISE), iso(ax, ay, z + RISE)];
        let area = (0..4).fold(0.0, |s, j| s + (q[j].0 * q[(j + 1) % 4].1 - q[(j + 1) % 4].0 * q[j].1));
        if area < 0.0 {
            q.reverse();
        }
        let p: Vec<String> = q.iter().map(|p| format!("{:.2},{:.2}", p.0, p.1)).collect();
        subs.push(format!("M{}Z", p.join("L")));
    }
    subs.join(" ")
}

fn main() {
    let mut a: Vec<String> = std::env::args().collect();
    let s: Vec<&String> = a[1..].iter().filter(|x| x.starts_with("--src=")).collect();
    let src_path = match s.first() {
        Some(x) => x.split_once('=').unwrap().1.to_string(),
        None => format!("{}/parts/mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg", repo()),
    };
    a = std::iter::once(a[0].clone()).chain(a[1..].iter().filter(|x| !x.starts_with("--src=")).cloned()).collect();
    if !std::path::Path::new(&src_path).exists() {
        exit(format!("  no such cut file: {src_path}\n  usage: mouthpiece-view [OUT.svg] [--src=SHEET.svg]"));
    }
    let src = read(&src_path);

    let group = |colour: &str| views::group(&src, colour).map(circles).unwrap_or_default();
    let (aps, outs) = (group("#ff8000"), group("#000000"));
    let mut rings: Vec<(f64, f64, bool)> = vec![];
    if !aps.is_empty() {
        if aps.len() != outs.len() {
            exit(format!("AssertionError: {} apertures, {} outlines in {src_path}", aps.len(), outs.len()));
        }
        for ((ax, ay, ah, ar), (ox, oy, oh, orr)) in aps.iter().zip(&outs) {
            if !((ax - ox).abs() < 0.5 && (ay - oy).abs() < 0.5) {
                exit(format!("AssertionError: aperture at ({ax:.1},{ay:.1}) is not concentric with its outline"));
            }
            // a radius only exists where the path has arcs; station one is square
            rings.push((ar.first().copied().unwrap_or(2.0 * ah), orr.first().copied().unwrap_or(2.0 * oh), !ar.is_empty()));
        }
    } else {
        exit(format!(
            "  {src_path}: no orange aperture group. That is a sheet from before the\n  apertures were split out; regenerate it with mouthpiece-round."
        ));
    }
    if rings.is_empty() {
        exit(format!("AssertionError: no rings in {src_path}"));
    }

    let (top, side, edge, bg) = ("#d8b445", "#9c7c1c", "#5e4a10", "#eef2f8");
    let mut faces: Vec<(f64, i32, String, &str)> = vec![];
    let mut z = 0.0;
    for &(a, o, rnd) in &rings {
        let r = o / 2.0;
        let edge_pts: Vec<P> = if rnd {
            (0..N).map(|i| (r * cos(2.0 * PI * i as f64 / N as f64), r * sin(2.0 * PI * i as f64 / N as f64))).collect()
        } else {
            // station one is a square plate with four flat sides
            vec![(-r, -r), (r, -r), (r, r), (-r, r)]
        };
        let ri = a / 2.0;
        let bore: Vec<P> = (0..N).map(|i| (ri * cos(2.0 * PI * i as f64 / N as f64), ri * sin(2.0 * PI * i as f64 / N as f64))).collect();
        // the bore first: the near half of the outer wall is drawn over it
        faces.push((z, 0, wall(&bore, z, false), side));
        faces.push((z, 1, wall(&edge_pts, z, true), side));
        let rp = ring_pts(a, o, rnd, z + RISE);
        let j = |v: &[P]| v.iter().map(|q| format!("{:.2},{:.2}", q.0, q.1)).collect::<Vec<_>>().join("L");
        faces.push((z, 2, format!("M{}ZM{}Z", j(&rp[0]), j(&rp[1])), top));
        z += RISE;
    }
    faces.sort_by(|x, y| (x.0, x.1).partial_cmp(&(y.0, y.1)).unwrap());

    let all: Vec<&str> = faces.iter().map(|f| f.2.as_str()).collect();
    let all = all.join(" ");
    let pts: Vec<P> = re::all(&re::rx(r"(-?[\d.]+),(-?[\d.]+)"), &all).iter().map(|m| (float(re::g(m, 1)), float(re::g(m, 2)))).collect();
    let (x0, x1) = (fmin(pts.iter().map(|p| p.0)), fmax(pts.iter().map(|p| p.0)));
    let (y0, y1) = (fmin(pts.iter().map(|p| p.1)), fmax(pts.iter().map(|p| p.1)));
    let m = 8.0;
    let (w, h) = (x1 - x0 + 2.0 * m, y1 - y0 + 2.0 * m);
    let body: Vec<String> = faces
        .iter()
        .map(|(_, _, d, c)| {
            format!(
                "    <path d=\"{d}\" fill=\"{c}\" {}",
                if *c == side { "fill-rule=\"nonzero\"/>".to_string() } else { format!("fill-rule=\"evenodd\" stroke=\"{edge}\" stroke-width=\"0.2\"/>") }
            )
        })
        .collect();
    // Named after the SOURCE, so the drawing and its sheet carry one name.
    let out = if a.len() > 1 { a[1].clone() } else { display_out(&src_path, "-view.svg") };
    let n = rings.len();
    write(
        &out,
        &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.1}mm\" height=\"{h:.1}mm\"\n     viewBox=\"{:.1} {:.1} {w:.1} {h:.1}\" role=\"img\"\n     aria-label=\"The mouthpiece assembled from {n} rings, seen from above the cup: the cup narrowing to a throat, then the backbore opening slowly below it\">\n  <!-- DISPLAY ONLY - not a cut file. Generated by mouthpiece-view.rs. -->\n  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{bg}\"/>\n{}\n</svg>\n",
            x0 - m,
            y0 - m,
            x0 - m,
            y0 - m,
            body.join("\n")
        ),
    );
    let throat = fmin(rings.iter().map(|r| r.0));
    println!(
        "  {out}: {n} rings, {:.0}mm plate at the instrument to ø{:.2} at the lip, ø{throat:.2} throat, {w:.0} x {h:.0}mm",
        rings[0].1,
        rings[n - 1].0
    );
}
