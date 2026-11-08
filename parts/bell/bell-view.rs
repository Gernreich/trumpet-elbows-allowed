//! Draw a bell as it looks built: rings stacked, seen from above the rim.
//! Rust since 2026-11-07 (bell-view.py until then). Isometric, throat at the
//! back, rim at the front, drawn back to front; each ring's corner radius is
//! read from the arcs its corners are drawn with, so a square-to-round bell is
//! drawn as one. Ring sizes come out of the cut file. DISPLAY ONLY.
//!
//!     target/release/bell-view cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg [OUT.svg]
#[path = "../common.rs"]
mod common;
#[path = "../views.rs"]
mod views;
use common::*;
use lasermade_tools::re;
use views::*;

/// The run of the outline that faces the viewer, as an ordered list of points.
fn facing(pts: &[P]) -> Vec<P> {
    let n = pts.len();
    let vis: Vec<bool> = (0..n)
        .map(|i| {
            let ((ax, ay), (bx, by)) = (pts[i], pts[(i + 1) % n]);
            (by - ay) - (bx - ax) > 0.0
        })
        .collect();
    if vis.iter().all(|&v| v) {
        let mut v = pts.to_vec();
        v.push(pts[0]);
        return v;
    }
    let start = (0..n).find(|&i| vis[i] && !vis[(i + n - 1) % n]).unwrap_or_else(|| exit("StopIteration"));
    let mut run = vec![pts[start]];
    let mut i = start;
    while vis[i % n] {
        i += 1;
        run.push(pts[i % n]);
    }
    run
}

fn poly(points: &[P]) -> String {
    let mut s: String = points.iter().enumerate().map(|(i, q)| format!("{}{:.2},{:.2}", if i == 0 { 'M' } else { 'L' }, q.0, q.1)).collect();
    s.push('Z');
    s
}

type Sec = (f64, f64);

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 2 {
        exit("  usage: bell-view SHEET.svg [OUT.svg]");
    }
    let src = &a[1];
    let s = read(src);

    // A RING IS TWO PATHS NOW, NOT ONE: read the orange and black groups and
    // pair them by index, in file order.
    let grp = |colour: &str| -> Vec<Sec> {
        match group(&s, colour) {
            None => vec![],
            Some(inner) => path_ds(inner).iter().flat_map(|d| sections(d)).collect(),
        }
    };
    let (aps, outs) = (grp("#ff8000"), grp("#000000"));
    let lt = |x: &(Sec, Sec), y: &(Sec, Sec)| x.partial_cmp(y) == Some(std::cmp::Ordering::Less);
    let mut rings: Vec<(Sec, Sec)> = Vec::new();
    if aps.is_empty() {
        // a file from before the split: one path, two sections
        for d in path_ds(&s) {
            let w = sections(&d);
            if w.len() == 2 {
                let (p, q) = (w[0], w[1]);
                rings.push(if q.partial_cmp(&p) == Some(std::cmp::Ordering::Less) { (q, p) } else { (p, q) });
            }
        }
    } else {
        if aps.len() != outs.len() {
            exit(format!("AssertionError: {} apertures against {} outlines in {src}", aps.len(), outs.len()));
        }
        for (&ap, &o) in aps.iter().zip(&outs) {
            rings.push(if o.partial_cmp(&ap) == Some(std::cmp::Ordering::Less) { (o, ap) } else { (ap, o) });
        }
    }
    rings.sort_by(|x, y| if lt(x, y) { std::cmp::Ordering::Less } else if lt(y, x) { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Equal });
    if rings.is_empty() {
        exit(format!("AssertionError: no rings in {src}"));
    }
    let rise = rise_of(&s).unwrap_or(if rings.len() == 5 { 1.5 } else { 3.0 });

    let (top, side, edge, bg) = ("#d8b445", "#9c7c1c", "#5e4a10", "#eef2f8");
    let mut faces: Vec<(f64, String, Option<&str>)> = Vec::new();
    let mut z = 0.0;
    for &((aw, ac), (ow, oc)) in &rings {
        let (ho, hoc) = (ow / 2.0, oc);
        let (ha, hac) = (aw / 2.0, ac);
        let (out_pts, ap_pts) = (outline(ho, hoc, 14), outline(ha, hac, 14));
        // the outside of the ring, as one band
        let band = facing(&out_pts);
        let mut b: Vec<P> = band.iter().map(|&(px, py)| iso(px, py, z)).collect();
        b.extend(band.iter().rev().map(|&(px, py)| iso(px, py, z + rise)));
        faces.push((z, poly(&b), Some(side)));
        // a sharp corner is a real edge and gets a line; a rounded one does not
        if hoc <= 1e-9 {
            let (p, q) = (iso(ho, ho, z), iso(ho, ho, z + rise));
            faces.push((z + 0.0005, format!("M{:.2},{:.2}L{:.2},{:.2}", p.0, p.1, q.0, q.1), None));
        }
        // top annulus: the outer outline with the aperture as a hole
        let o: Vec<P> = out_pts.iter().map(|&(px, py)| iso(px, py, z + rise)).collect();
        let ap: Vec<P> = ap_pts.iter().map(|&(px, py)| iso(px, py, z + rise)).collect();
        faces.push((z + 0.001, poly(&o) + &poly(&ap), Some(top)));
        z += rise;
    }
    faces.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

    let all: String = faces.iter().map(|f| f.1.as_str()).collect();
    let pts: Vec<P> = re::all(&re::rx(r"([-\d.]+),([-\d.]+)"), &all).iter().map(|m| (float(re::g(m, 1)), float(re::g(m, 2)))).collect();
    let x0 = fmin(pts.iter().map(|p| p.0));
    let x1 = fmax(pts.iter().map(|p| p.0));
    let y0 = fmin(pts.iter().map(|p| p.1));
    let y1 = fmax(pts.iter().map(|p| p.1));
    let m = 10.0;
    let (w, h) = (x1 - x0 + 2.0 * m, y1 - y0 + 2.0 * m);
    let body: Vec<String> = faces
        .iter()
        .map(|(_, d, c)| match c {
            None => format!("    <path d=\"{d}\" fill=\"none\" stroke=\"{edge}\" stroke-width=\"0.4\"/>"),
            Some(c) => format!("    <path d=\"{d}\" fill=\"{c}\" fill-rule=\"evenodd\" stroke=\"{edge}\" stroke-width=\"0.25\"/>"),
        })
        .collect();

    // The label describes the object in the file.
    let shape = |w: f64, c: f64| if c <= 1e-9 { "square" } else if (c - w / 2.0).abs() < 0.01 { "round" } else { "rounded-square" };
    let n = rings.len();
    let (first, last) = (shape(rings[0].0 .0, rings[0].0 .1), shape(rings[n - 1].1 .0, rings[n - 1].1 .1));
    let what = if first == last { format!("{first} rings stacked") } else { format!("rings stacked and morphing from {first} at the throat to {last} at the rim") };

    let dst = if a.len() > 2 { a[2].clone() } else { display_out(src, "-view.svg") };
    let name = lasermade_tools::text::pathstr(src).rsplit('/').next().unwrap_or("").to_string();
    write(
        &dst,
        &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.1}mm\" height=\"{h:.1}mm\"\n     viewBox=\"{:.1} {:.1} {w:.1} {h:.1}\" role=\"img\"\n     aria-label=\"A {n}-ring bell seen from above the rim: {what}, widening toward the viewer, the bore visible down the middle\">\n  <!-- DISPLAY ONLY - not a cut file. Generated by bell-view.rs from {name}. -->\n  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"{bg}\"/>\n{}\n</svg>\n",
            x0 - m,
            y0 - m,
            x0 - m,
            y0 - m,
            body.join("\n")
        ),
    );
    println!("  {dst}: {n} rings, {first} throat to {last} rim, {w:.0} x {h:.0}mm");
}
