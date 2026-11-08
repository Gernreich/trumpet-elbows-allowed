//! How the display drawings and the turn page read a ring sheet: each
//! subpath's width and corner radius, the rounded-square outline they make,
//! the isometric they are drawn in, and part-view's solid. bell-view.py held
//! sections() and outline() and part-view.py executed them out of it; here
//! they are one module that both use, so the two still cannot disagree.
#![allow(dead_code)]
use crate::common::{exit, fmax, fmin};
use lasermade_tools::py::{self, hypot, mcos as cos, msin as sin};
use lasermade_tools::re;
use lasermade_tools::text;
use lasermade_tools::turn3d::{Num::{F, I}, J};

pub type P = (f64, f64);

/// Where a display drawing goes, given the cut file it was read from: beside
/// the PART, climbing out of cut-files/, with the source's "-cut-files" dropped.
pub fn display_out(src: &str, suffix: &str) -> String {
    let name = text::pathstr(src).rsplit('/').next().unwrap_or("").to_string();
    // Path.stem: the name without its last suffix
    let mut stem = match name.rfind('.') {
        Some(k) if k > 0 => name[..k].to_string(),
        _ => name.clone(),
    };
    if let Some(s) = stem.strip_suffix("-cut-files") {
        stem = s.to_string();
    }
    let abs = std::fs::canonicalize(src).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| text::abspath(src));
    let mut d = abs.rsplit_once('/').map(|(a, _)| a.to_string()).unwrap_or_default();
    if d.rsplit('/').next() == Some("cut-files") {
        d = d.rsplit_once('/').map(|(a, _)| a.to_string()).unwrap_or_default();
    }
    let cwd = std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    text::relpath(&format!("{d}/{stem}{suffix}"), &cwd)
}

/// The isometric: x and y in plan, z up, seen from +X +Y +Z.
pub fn iso(x: f64, y: f64, z: f64) -> P {
    let (c30, s30) = (cos(py::radians(30.0)), sin(py::radians(30.0)));
    ((x - y) * c30, (x + y) * s30 - z)
}

/// (width, corner radius) for each subpath, whatever commands drew it. The
/// corner radius is the arc radius: 0 where the corners are H and V, half the
/// width where the outline is all arcs.
pub fn sections(d: &str) -> Vec<(f64, f64)> {
    let rx = re::rx(r"[MmLlHhVvZzCcSsQqAa]|-?\d*\.?\d+(?:[eE]-?\d+)?");
    let toks: Vec<String> = re::findall1(&rx, d, 0);
    let mut i = 0usize;
    let (mut x, mut y) = (0.0f64, 0.0f64);
    let mut cmd: Option<char> = None;
    let mut subs: Vec<Vec<f64>> = Vec::new();
    let mut arcs: Vec<Vec<f64>> = Vec::new();
    let isl = |t: &str| t.len() == 1 && t.as_bytes()[0].is_ascii_alphabetic();
    while i < toks.len() {
        if isl(&toks[i]) {
            cmd = toks[i].chars().next();
            i += 1;
            continue;
        }
        let Some(cm) = cmd else {
            i += 1;
            continue;
        };
        let rel = cm.is_ascii_lowercase();
        let mut n = || {
            let t = toks.get(i).unwrap_or_else(|| exit("IndexError: list index out of range"));
            let v = py::float(t).unwrap_or_else(|| exit(format!("ValueError: could not convert string to float: {}", py::str_repr(t))));
            i += 1;
            v
        };
        match cm.to_ascii_uppercase() {
            'M' => {
                let (dx, dy) = (n(), n());
                (x, y) = if rel { (x + dx, y + dy) } else { (dx, dy) };
                subs.push(vec![x]);
                arcs.push(vec![0.0]);
                cmd = Some(if rel { 'l' } else { 'L' });
            }
            'L' => {
                let (dx, dy) = (n(), n());
                (x, y) = if rel { (x + dx, y + dy) } else { (dx, dy) };
                need(subs.last_mut()).push(x);
            }
            'H' => {
                let v = n();
                x = if rel { x + v } else { v };
                need(subs.last_mut()).push(x);
            }
            'V' => {
                let v = n();
                y = if rel { y + v } else { v };
                need(subs.last_mut()).push(x);
            }
            'C' => {
                n();
                n();
                n();
                n();
                let (dx, dy) = (n(), n());
                (x, y) = if rel { (x + dx, y + dy) } else { (dx, dy) };
                need(subs.last_mut()).push(x);
            }
            'S' | 'Q' => {
                n();
                n();
                let (dx, dy) = (n(), n());
                (x, y) = if rel { (x + dx, y + dy) } else { (dx, dy) };
                need(subs.last_mut()).push(x);
            }
            'A' => {
                let rxv = n();
                n();
                n();
                n();
                n();
                let (dx, dy) = (n(), n());
                (x, y) = if rel { (x + dx, y + dy) } else { (dx, dy) };
                need(subs.last_mut()).push(x);
                need(arcs.last_mut()).push(rxv.abs());
            }
            _ => i += 1,
        }
    }
    let _ = y;
    subs.iter()
        .zip(&arcs)
        .filter(|(v, _)| v.len() > 1)
        .map(|(v, a)| (fmax(v.iter().copied()) - fmin(v.iter().copied()), fmax(a.iter().copied())))
        .collect()
}

/// A rounded square of half-width h and corner radius c, counter-clockwise:
/// c = 0 gives the four corners and nothing else, c = h a circle.
pub fn outline(h: f64, c: f64, per: usize) -> Vec<P> {
    let c = py::min(py::max(c, 0.0), h);
    let s = h - c;
    let mut pts = Vec::new();
    for ((cx, cy), a0) in [((s, -s), -90.0), ((s, s), 0.0), ((-s, s), 90.0), ((-s, -s), 180.0)] {
        if c <= 1e-9 {
            pts.push((cx, cy));
        } else {
            for k in 0..=per {
                let t = py::radians(a0 + 90.0 * k as f64 / per as f64);
                pts.push((cx + c * cos(t), cy + c * sin(t)));
            }
        }
    }
    pts
}

/// The group of one stroke colour, its inside, if the sheet has one.
pub fn group<'a>(s: &'a str, colour: &str) -> Option<&'a str> {
    let g = re::rx(&format!(r#"(?s)<g[^>]*stroke="{colour}"[^>]*>(.*?)</g>"#));
    re::first(&g, s).map(|m| m.get(1).unwrap().as_str())
}

/// Every <path>'s d, in file order
pub fn path_ds(s: &str) -> Vec<String> {
    re::findall1(&re::rx(r#"<path\b[^>]*\bd="([^"]+)""#), s, 1)
}

/// "N.Nmm of rise" from the sheet's own description
pub fn rise_of(s: &str) -> Option<f64> {
    re::first(&re::rx(r"([\d.]+)mm of rise"), s).map(|m| {
        let t = re::g(&m, 1);
        py::float(t).unwrap_or_else(|| exit(format!("ValueError: could not convert string to float: {}", py::str_repr(t))))
    })
}

/// Path.read_text(), or the error Python would have died of
pub fn read(path: &str) -> String {
    text::read_strict(path).unwrap_or_else(|e| exit(format!("FileNotFoundError: [Errno 2] {e}: {}", py::str_repr(path))))
}

// --- part-view's solid -------------------------------------------------------

/// A rounded square as exactly n points, evenly spaced along its own perimeter.
pub fn prof(h: f64, c: f64, n: usize) -> Vec<P> {
    let mut p = outline(h, c, 24);
    p.push(p[0]);
    let mut run = vec![0.0f64];
    for w in p.windows(2) {
        let last = run[run.len() - 1];
        run.push(last + hypot(w[1].0 - w[0].0, w[1].1 - w[0].1));
    }
    let total = run[run.len() - 1];
    let mut out = Vec::new();
    let mut j = 0usize;
    for k in 0..n {
        let t = total * k as f64 / n as f64;
        while j + 1 < run.len() - 1 && run[j + 1] < t {
            j += 1;
        }
        let span = run[j + 1] - run[j];
        let u = if span < 1e-12 { 0.0 } else { (t - run[j]) / span };
        out.push((p[j].0 + (p[j + 1].0 - p[j].0) * u, p[j].1 + (p[j + 1].1 - p[j].1) * u));
    }
    out
}

/// (aperture w, aperture c, outer w, outer c)
pub type Ring = (f64, f64, f64, f64);

/// The rings, bottom of the stack first, plus the rise per ring.
pub fn rings_of(path: &str) -> (Vec<Ring>, f64) {
    let s = read(path);
    let grp = |colour: &str| -> Vec<Option<(f64, f64)>> {
        match group(&s, colour) {
            None => vec![],
            Some(inner) => path_ds(inner).iter().map(|d| sections(d).first().copied()).collect(),
        }
    };
    let (aps, outs) = (grp("#ff8000"), grp("#000000"));
    let mut pairs = Vec::new();
    if !aps.is_empty() {
        // by index, in file order: ring i is aps[i] with outs[i]
        if aps.len() != outs.len() {
            exit(format!("AssertionError: {} apertures, {} outlines", aps.len(), outs.len()));
        }
        for (a, o) in aps.iter().zip(&outs) {
            if let (Some(a), Some(o)) = (a, o) {
                pairs.push((a.0, a.1, o.0, o.1));
            }
        }
    } else {
        // a file from before the colour split
        for d in path_ds(&s) {
            let mut w = sections(&d);
            if w.len() == 2 {
                w.sort_by(|p, q| p.partial_cmp(q).unwrap());
                pairs.push((w[0].0, w[0].1, w[1].0, w[1].1));
            }
        }
    }
    (pairs, rise_of(&s).unwrap_or(3.0))
}

/// Vertices and quads (indices, face, ring) for the stack: per ring the
/// outside band, the bore band and the top annulus; the first ring's underside.
pub fn solid(pairs: &[Ring], rise: f64, n: usize) -> (Vec<[f64; 3]>, Vec<([usize; 4], i64, i64)>) {
    let (mut v, mut q) = (Vec::new(), Vec::new());
    let mut base = 0usize;
    for (i, &(aw, ac, ow, oc)) in pairs.iter().enumerate() {
        let (z0, z1) = (i as f64 * rise, (i + 1) as f64 * rise);
        let a = prof(aw / 2.0, ac, n);
        let o = prof(ow / 2.0, oc, n);
        for z in [z0, z1] {
            v.extend(a.iter().map(|p| [p.0, p.1, z]));
            v.extend(o.iter().map(|p| [p.0, p.1, z]));
        }
        let (a0, o0) = (base, base + n);
        let (a1, o1) = (base + 2 * n, base + 3 * n);
        let s = i as i64;
        for k in 0..n {
            let j = (k + 1) % n;
            q.push(([o0 + k, o0 + j, o1 + j, o1 + k], 1, s));
            q.push(([a1 + k, a1 + j, a0 + j, a0 + k], 2, s));
            q.push(([a1 + k, o1 + k, o1 + j, a1 + j], 0, s));
            if i == 0 {
                q.push(([a0 + j, o0 + j, o0 + k, a0 + k], 3, s));
            }
        }
        base += 4 * n;
    }
    (v, q)
}

/// n colours round the hue circle, for colouring by ring
pub fn wheel(n: usize) -> Vec<String> {
    let m = n.max(1);
    (0..m)
        .map(|i| {
            let h = pymod(i as f64 * 320.0 / m as f64 + 20.0, 360.0);
            let (c, x) = (0.55, 0.55 * (1.0 - (pymod(h / 60.0, 2.0) - 1.0).abs()));
            let (r, g, b) = [(c, x, 0.0), (x, c, 0.0), (0.0, c, x), (0.0, x, c), (x, 0.0, c), (c, 0.0, x)][((h / 60.0).floor() as i64).rem_euclid(6) as usize];
            let mm = 0.34;
            format!("#{:02x}{:02x}{:02x}", ((r + mm) * 255.0) as i64, ((g + mm) * 255.0) as i64, ((b + mm) * 255.0) as i64)
        })
        .collect()
}

/// Python's % on floats: the result takes the divisor's sign
fn pymod(a: f64, b: f64) -> f64 {
    let m = a % b;
    if m != 0.0 && (m < 0.0) != (b < 0.0) {
        m + b
    } else {
        m
    }
}

fn n(x: f64) -> J {
    J::N(F(x))
}
fn i(x: i64) -> J {
    J::N(I(x))
}
fn strs(v: &[&str]) -> J {
    J::A(v.iter().map(|s| J::S(s.to_string())).collect())
}

/// part-view's page data: the solid, and what the panel says about it
pub fn data_for(path: &str, facets: usize) -> J {
    let (pairs, rise) = rings_of(path);
    if pairs.is_empty() {
        exit(format!("AssertionError: no rings in {path}"));
    }
    let (v, q) = solid(&pairs, rise, facets);
    let (ap0, rim) = (pairs[0].0, pairs[pairs.len() - 1].2);
    let rnd = pairs.iter().filter(|p| p.3 > 0.4 * p.2).count();
    let shape = if rnd > 0 && pairs[0].1 < 0.1 {
        "square throat to round rim"
    } else if pairs[0].1 < 0.1 {
        "square throat to square rim"
    } else {
        "round"
    };
    J::obj(vec![
        ("V", J::A(v.iter().map(|p| J::A(p.iter().map(|&x| n(py::round_dp(x, 3))).collect())).collect())),
        (
            "Q",
            J::A(q.iter().map(|(vs, f, s)| J::obj(vec![("v", J::A(vs.iter().map(|&k| i(k as i64)).collect())), ("f", i(*f)), ("s", i(*s))])).collect()),
        ),
        ("rings", i(pairs.len() as i64)),
        ("rise", n(rise)),
        ("height", n(py::round_dp(pairs.len() as f64 * rise, 1))),
        ("throat", n(py::round_dp(ap0, 2))),
        ("rim", n(py::round_dp(rim, 2))),
        ("shape", J::S(shape.into())),
        ("pal", J::A(wheel(pairs.len()).into_iter().map(J::S).collect())),
        ("faces", strs(&["ring face", "outside", "the bore", "underside"])),
        ("facecol", strs(&["#d8b445", "#9c7c1c", "#4a3a0c", "#7a6216"])),
        ("keycol", strs(&["#e8c95f", "#a8871f", "#3a2d08", "#7a6216"])),
    ])
}

/// x, where Python would have died appending to None
fn need<T>(o: Option<T>) -> T {
    o.unwrap_or_else(|| crate::common::exit("AttributeError: 'NoneType' object has no attribute 'append'"))
}
