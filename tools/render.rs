//! Render a bore written in the agreed notation as a 3D view: bore_render.py,
//! Rust since 2026-11-08. Coloured by piece, which is what you cut, or by
//! direction of travel, which is what you follow when reading the shape.
use crate::bore::{dirs, neg, Design};
use lasermade_tools::py::{self, mcos as cos, msin as sin};
use std::collections::{HashMap, HashSet};

pub const PAL: [&str; 8] = ["#2f6f4e", "#b5563a", "#3a5f8a", "#8a6b2f", "#6b3a7a", "#2f7a7a", "#7a4a2f", "#4a6b2f"];
pub const OPEN_IN: &str = "#1c1c20";
pub const OPEN_OUT: &str = "#e0457b";
/// one hue per direction of travel; down is grey, a block you can get
pub fn dircol(d: char) -> &'static str {
    match d {
        'N' => "#2f6ea8",
        'S' => "#e08a2e",
        'E' => "#3f8f5e",
        'W' => "#9b4a9c",
        'U' => "#cf4433",
        _ => "#8b9096",
    }
}
pub fn dirname(d: char) -> &'static str {
    match d {
        'N' => "north",
        'S' => "south",
        'E' => "east",
        'W' => "west",
        'U' => "up",
        _ => "down",
    }
}
const LIGHT: [f64; 3] = [0.35, 1.0, 0.55]; // over your left shoulder, from above

/// n colours that stay apart, for when there are more pieces than palette.
pub fn wheel(n: usize) -> Vec<String> {
    if n <= PAL.len() {
        return PAL[..n].iter().map(|s| s.to_string()).collect();
    }
    let mut out = vec![];
    for i in 0..n {
        let h = crate::snakebox::pymod(i as f64 * 0.618033988749895, 1.0);
        let li = 0.42 + 0.13 * (i % 3) as f64 / 2.0;
        let c = (1.0 - (2.0 * li - 1.0).abs()) * 0.62;
        let x = c * (1.0 - (crate::snakebox::pymod(h * 6.0, 2.0) - 1.0).abs());
        let m = li - c / 2.0;
        let (r, g, b) = [(c, x, 0.0), (x, c, 0.0), (0.0, c, x), (0.0, x, c), (x, 0.0, c), (c, 0.0, x)][(h * 6.0) as i64 as usize % 6];
        out.push(format!("#{:02x}{:02x}{:02x}", ((r + m) * 255.0) as i64, ((g + m) * 255.0) as i64, ((b + m) * 255.0) as i64));
    }
    out
}

/// The face's colour under a fixed light, so the solid reads as a solid.
fn shade(col: &str, nrm: [i64; 3]) -> String {
    let n = py::powf(LIGHT.iter().fold(0.0, |s, a| s + a * a), 0.5);
    let d = LIGHT.iter().zip(nrm).fold(0.0, |s, (a, b)| s + a * b as f64) / n;
    let f = 0.68 + 0.32 * (d + 1.0) / 2.0 + if nrm[1] > 0 { 0.08 } else { 0.0 };
    let v: Vec<i64> = [1, 3, 5].iter().map(|&i| i64::from_str_radix(&col[i..i + 2], 16).unwrap()).collect();
    format!("#{:02x}{:02x}{:02x}", ((v[0] as f64 * f) as i64).min(255), ((v[1] as f64 * f) as i64).min(255), ((v[2] as f64 * f) as i64).min(255))
}

type Cell = [i64; 3];
/// (kind, blocks, in, out, code) per piece
pub type Piece = (String, usize, char, char, String);

impl Design {
    /// Cells in walk order, tagged with the piece they belong to.
    pub fn layout(&self, text: &str) -> Result<(Vec<(Cell, usize)>, Vec<Piece>), String> {
        if !self.cubic() {
            return Err(format!(
                "bore_render draws lattice cubes, and this lattice is not cubic (section {}, straight {}). Use viewer.py, which measures the real boxes.",
                py::g(self.block),
                py::g(self.straight)
            ));
        }
        let sp = self.specs_for(text)?;
        let (mut cells, mut pieces) = (vec![], vec![]);
        for (i, (gr, p)) in sp.groups.iter().zip(&sp.plan).enumerate() {
            pieces.push((p.kind.clone(), gr.len(), sp.rec[gr[0]].inn, sp.rec[gr[gr.len() - 1]].out, p.code.clone()));
            for &j in gr {
                cells.push((sp.rec[j].pos, i));
            }
        }
        Ok((cells, pieces))
    }
}

const FACES: [([i64; 3], [[i64; 3]; 4]); 6] = [
    ([0, 1, 0], [[0, 1, 0], [1, 1, 0], [1, 1, 1], [0, 1, 1]]),
    ([0, -1, 0], [[0, 0, 0], [0, 0, 1], [1, 0, 1], [1, 0, 0]]),
    ([1, 0, 0], [[1, 0, 0], [1, 0, 1], [1, 1, 1], [1, 1, 0]]),
    ([-1, 0, 0], [[0, 0, 0], [0, 1, 0], [0, 1, 1], [0, 0, 1]]),
    ([0, 0, 1], [[0, 0, 1], [0, 1, 1], [1, 1, 1], [1, 0, 1]]),
    ([0, 0, -1], [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]),
];

/// One view: the SVG body for it
#[allow(clippy::too_many_arguments)]
pub fn render(cells: &[(Cell, usize)], pieces: &[Piece], az: f64, w: f64, h: f64, title: &str, el: f64, openings: &HashMap<(Cell, Cell), &str>, colour: &dyn Fn(Cell, usize) -> String) -> String {
    let (ca, sa) = (cos(py::radians(az)), sin(py::radians(az)));
    let (ce, se) = (cos(py::radians(el)), sin(py::radians(el)));
    let rot = |p: [f64; 3]| -> [f64; 3] {
        let (x, y, z) = (p[0], p[1], p[2]);
        let (x, z) = (x * ca + z * sa, -x * sa + z * ca);
        let (y, z) = (y * ce - z * se, y * se + z * ce);
        [x, y, z]
    };
    let proj = |p: [f64; 3]| -> (f64, f64) {
        let [x, y, z] = rot(p);
        ((x - z) * 0.866, (x + z) * 0.5 - y)
    };
    let fl = |c: [i64; 3]| [c[0] as f64, c[1] as f64, c[2] as f64];
    let occ: HashSet<Cell> = cells.iter().map(|c| c.0).collect();
    let mut faces = vec![];
    for &(c, pid) in cells {
        for (nrm, cor) in FACES {
            let nb = [c[0] + nrm[0], c[1] + nrm[1], c[2] + nrm[2]];
            if occ.contains(&nb) {
                continue;
            }
            let rn = rot(fl(nrm));
            if rn[0] + rn[1] + rn[2] <= 0.01 {
                continue;
            }
            let pts: Vec<Cell> = cor.iter().map(|v| [c[0] + v[0], c[1] + v[1], c[2] + v[2]]).collect();
            let cen = [0, 1, 2].map(|i| pts.iter().map(|p| p[i]).sum::<i64>() as f64 / 4.0);
            let rc = rot(cen);
            faces.push((rc[0] + rc[1] + rc[2], pts, pid, c, nrm));
        }
    }
    faces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let pid_of: HashMap<Cell, usize> = cells.iter().map(|&(c, i)| (c, i)).collect();
    let pp: Vec<(f64, f64)> = faces.iter().flat_map(|f| f.1.iter().map(|&p| proj(fl(p)))).collect();
    if pp.is_empty() {
        return String::new();
    }
    let minx = crate::bore::fmin(pp.iter().map(|q| q.0));
    let maxx = crate::bore::fmax(pp.iter().map(|q| q.0));
    let miny = crate::bore::fmin(pp.iter().map(|q| q.1));
    let maxy = crate::bore::fmax(pp.iter().map(|q| q.1));
    let or1 = |v: f64| if v == 0.0 { 1.0 } else { v };
    let s = py::min((w - 40.0) / or1(maxx - minx), (h - 60.0) / or1(maxy - miny));
    let ox = 20.0 - minx * s + ((w - 40.0) - (maxx - minx) * s) / 2.0;
    let oy = 40.0 - miny * s;
    let pt = |p: [f64; 3]| {
        let (a, b) = proj(p);
        (ox + a * s, oy + b * s)
    };
    let mut out = vec![format!("<text x=\"{:.0}\" y=\"26\" class=\"c\">{title}</text>", w / 2.0)];
    let cross = |a: [i64; 3], b: [i64; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    for (_, pts, pid, cell, nrm) in &faces {
        let tag = openings.get(&(*cell, *nrm)).copied();
        let base = colour(*cell, *pid);
        let col = match tag {
            Some("IN") => OPEN_IN.to_string(),
            Some(_) => OPEN_OUT.to_string(),
            None => shade(&base, *nrm),
        };
        let pq: Vec<(f64, f64)> = pts.iter().map(|&p| pt(fl(p))).collect();
        let s = pq.iter().map(|(a, b)| format!("{a:.1},{b:.1}")).collect::<Vec<_>>().join(" ");
        out.push(format!(
            "<polygon points=\"{s}\" fill=\"{col}\" fill-opacity=\"{}\" stroke=\"#333\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
            if tag.is_some() { "0.98" } else { "1.0" },
            if tag.is_some() { "1.8" } else { "0.8" }
        ));
        if let Some(t) = tag {
            let ax = pq.iter().fold(0.0, |s, q| s + q.0) / 4.0;
            let ay = pq.iter().fold(0.0, |s, q| s + q.1) / 4.0;
            out.push(format!("<text x=\"{ax:.0}\" y=\"{:.0}\" class=\"op\">{t}</text>", ay + 5.0));
        }
        for k in 0..4 {
            let (p0, p1) = (pts[k], pts[(k + 1) % 4]);
            let e = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let d = cross(*nrm, e);
            let nb = [cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]];
            if pid_of.get(&nb).is_some_and(|q| q != pid) {
                let ((ax, ay), (bx, by)) = (pt(fl(p0)), pt(fl(p1)));
                out.push(format!("<line x1=\"{ax:.1}\" y1=\"{ay:.1}\" x2=\"{bx:.1}\" y2=\"{by:.1}\" stroke=\"#111\" stroke-width=\"3.2\" stroke-linecap=\"round\"/>"));
            }
        }
    }
    for i in 0..pieces.len() {
        let cs: Vec<Cell> = cells.iter().filter(|c| c.1 == i).map(|c| c.0).collect();
        let cen = [0, 1, 2].map(|k| cs.iter().map(|c| c[k]).sum::<i64>() as f64 / cs.len() as f64 + 0.5);
        let (x, y) = pt(cen);
        out.push(format!("<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"11\" fill=\"#fff\" fill-opacity=\"0.9\" stroke=\"#111\" stroke-width=\"1.2\"/>"));
        out.push(format!("<text x=\"{x:.1}\" y=\"{:.1}\" class=\"pn\">{}</text>", y + 4.5, i + 1));
    }
    out.join("\n")
}

impl Design {
    /// bore_render.main: the report on `o`, the drawing to `out`
    pub fn render_main(&self, text: &str, by_direction: bool, out: Option<&str>, o: &mut String) -> Result<(), String> {
        let (cells, pieces) = self.layout(text)?;
        let (rec, _, _) = self.coplanar_pieces(text)?;
        let mut seen = HashSet::new();
        let mut clash = 0;
        for (c, _) in &cells {
            if !seen.insert(*c) {
                clash += 1;
            }
        }
        o.push_str(&format!("bore: {text}\n"));
        o.push_str(&format!("  {} cells, {} pieces\n", cells.len(), pieces.len()));
        for (i, (k, n, a, b, code)) in pieces.iter().enumerate() {
            let label = if k == "elbow" { k.clone() } else { format!("{k} {n}") };
            o.push_str(&format!("   {:<3} {label:<13} {code:<8} {a} -> {b}\n", i + 1));
        }
        o.push_str(&format!("\n  {}\n", if clash > 0 { format!("!! the bore runs into itself at {clash} cell(s)") } else { "no self-intersection: every cell is used once".into() }));
        let (first, last) = (&rec[0], &rec[rec.len() - 1]);
        let openings: HashMap<(Cell, Cell), &str> = HashMap::from([((first.pos, neg(dirs(first.inn))), "IN"), ((last.pos, dirs(last.out)), "OUT")]);
        let pal = wheel(pieces.len());
        let dirof: HashMap<Cell, char> = rec.iter().map(|r| (r.pos, r.out)).collect();
        let turns: HashSet<Cell> = rec.iter().filter(|r| r.inn != r.out).map(|r| r.pos).collect();
        let colour = |c: Cell, pid: usize| -> String { if by_direction { dircol(dirof[&c]).to_string() } else { pal[pid % pal.len()].clone() } };
        let views = [(0.0, 0.0, "front-right"), (90.0, 0.0, "back-right"), (180.0, 0.0, "back-left"), (270.0, 0.0, "front-left"), (0.0, 62.0, "from above"), (0.0, -34.0, "from below")];
        let (w, h) = (400i64, 360i64);
        let mut body = vec![];
        for (i, (az, el, nm)) in views.iter().enumerate() {
            let gg = render(&cells, &pieces, *az, w as f64, h as f64, nm, *el, &openings, &colour);
            body.push(format!("<g transform=\"translate({},{})\">{gg}</g>", (i as i64 % 3) * w, (i as i64 / 3) * h));
        }
        let ly = 2 * h + 16;
        let mut leg = vec![format!(
            "<text x=\"24\" y=\"{ly}\" class=\"hd\">{} — {} blocks, {} pieces, {}</text>",
            text.trim(),
            rec.len(),
            pieces.len(),
            if by_direction { "coloured by direction of travel" } else { "assembled" }
        )];
        let mut rows: Vec<(String, String)> = if by_direction {
            "NSEWUD"
                .chars()
                .filter(|&d| rec.iter().any(|r| r.out == d))
                .map(|d| {
                    let n = rec.iter().filter(|r| r.out == d).count();
                    let t = rec.iter().filter(|r| r.out == d && turns.contains(&r.pos)).count();
                    (dircol(d).to_string(), format!("{d}  {:<7}{n:>3} blocks{}", dirname(d), if t > 0 { format!(", {t} turning") } else { String::new() }))
                })
                .collect()
        } else {
            pieces
                .iter()
                .enumerate()
                .map(|(i, (k, cnt, a, b, code))| (pal[i % pal.len()].clone(), format!("{}. {code:<6} {:<12}{a} to {b}", i + 1, if k == "elbow" { k.clone() } else { format!("{k} {cnt}") })))
                .collect()
        };
        rows.push((OPEN_IN.into(), "IN   the bore mouth, tabs".into()));
        rows.push((OPEN_OUT.into(), "OUT  the far end, notches".into()));
        for (i, (col, name)) in rows.iter().enumerate() {
            let (cx, cy) = (24 + (i as i64 % 2) * 560, ly + 28 + (i as i64 / 2) * 26);
            leg.push(format!("<rect x=\"{cx}\" y=\"{}\" width=\"20\" height=\"14\" rx=\"2\" fill=\"{col}\" stroke=\"#222\" stroke-width=\"0.9\"/>", cy - 11));
            leg.push(format!("<text x=\"{}\" y=\"{cy}\" class=\"lg\">{name}</text>", cx + 30));
        }
        let hh = ly + 28 + ((rows.len() as i64 + 1) / 2) * 26 + 14;
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w3}\" height=\"{hh}\" viewBox=\"0 0 {w3} {hh}\"><style>.c{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222;text-anchor:middle}}.hd{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222}}.lg{{font:13px ui-monospace,Menlo,monospace;fill:#333}}.op{{font:700 15px ui-sans-serif,sans-serif;fill:#fff;text-anchor:middle}}.pn{{font:700 12px ui-sans-serif,sans-serif;fill:#111;text-anchor:middle}}</style><rect width=\"{w3}\" height=\"{hh}\" fill=\"#fff\"/>{}{}</svg>",
            body.concat(),
            leg.concat(),
            w3 = w * 3
        );
        let name = out.map(|s| s.to_string()).unwrap_or_else(|| if by_direction { "bore3d_directions.svg".into() } else { "bore3d.svg".into() });
        std::fs::write(&name, svg).map_err(|e| format!("{name}: {e}"))?;
        o.push_str(&format!("\n  wrote {name}  ({} x {hh})\n", w * 3));
        Ok(())
    }
}
