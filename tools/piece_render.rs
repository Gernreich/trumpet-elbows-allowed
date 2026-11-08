// Copyright (C) 2026 Gernreich
//
//   GPL-3.0-or-later, NOT the CC0 the rest of this repository is under: it
//   reads the piece's outline out of snakebox.rs, which is Boxes.py code.
//   piece_render.py until 2026-11-08, under the same licence.
//
//   This program is free software: you can redistribute it and/or modify
//   it under the terms of the GNU General Public License as published by
//   the Free Software Foundation, either version 3 of the License, or
//   (at your option) any later version.
//
//   This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU General Public License for more details.
//
//   You should have received a copy of the GNU General Public License
//   along with this program.  If not, see <http://www.gnu.org/licenses/>.
//   A copy is in tools/LICENSE.GPL-3.0.txt.

//! `piece_render [--path UUR] [--open_faces N,E] [--label B1] [--out FILE]`:
//! draw one assembled piece from several angles, each face coloured by its
//! part. It reads the piece's outline from the generator that cuts it and its
//! part sizes from the cut parts, so the picture cannot drift from what is cut.
use lasermade_tools::py::{self, mcos as cos, msin as sin};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use trumpet::bore::{sys_exit, Design, Part};
use trumpet::snakebox::{Cell, Run, Snake};

const PLATE_COL: &str = "#b8b2a4";
const WALL_COLS: [&str; 8] = ["#3a5f8a", "#3f7d55", "#c8532f", "#d9962f", "#6b3a7a", "#2f7a7a", "#7a4a2f", "#4a6b2f"];
const IN_COL: &str = "#4a4a52";
const OUT_COL: &str = "#7a4a86";
type V3 = [i64; 3];
const FACES: [(V3, [V3; 4]); 6] = [
    ([0, 0, 1], [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]]),
    ([0, 0, -1], [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]]),
    ([1, 0, 0], [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]]),
    ([-1, 0, 0], [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]]),
    ([0, 1, 0], [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]]),
    ([0, -1, 0], [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]]),
];
const VIEWS: [(f64, f64, &str); 6] = [(0.0, 0.0, "front-right"), (90.0, 0.0, "back-right"), (180.0, 0.0, "back-left"), (270.0, 0.0, "front-left"), (20.0, -52.0, "from below, front"), (200.0, -52.0, "from below, back")];

/// Outline, tips and measured part sizes for one piece.
fn describe(d: &Design, path: &str, open_faces: &str) -> Result<(Vec<Cell>, Vec<Run>, Vec<usize>, BTreeMap<usize, Part>, Part), String> {
    let args: Vec<String> = if !open_faces.is_empty() { vec![format!("--open_faces={open_faces}"), "--path=".into()] } else { vec![format!("--path={path}")] };
    let mut all = args.clone();
    all.extend(d.common.iter().cloned());
    let b = Snake::from_args(&all)?;
    let cells = b.cells()?;
    let (runs, _) = b.outline(&cells)?;
    let tips = b.tip_runs(&cells, &runs)?;
    let parts = d.cut(&args)?;
    let walls: Vec<Part> = parts.iter().filter(|p| p.role == 'W').cloned().collect();
    let wall_runs: Vec<usize> = (0..runs.len()).filter(|k| !tips.contains(k)).collect();
    if walls.len() != wall_runs.len() {
        sys_exit(&format!("{} wall parts but {} wall runs", walls.len(), wall_runs.len()));
    }
    let plate = parts.iter().find(|p| p.role == 'P').cloned().ok_or("StopIteration")?;
    Ok((cells, runs, tips, wall_runs.into_iter().zip(walls).collect(), plate))
}

/// The boundary run carrying this cell face.
fn face_run(runs: &[Run], cell: (i64, i64), m: (i64, i64)) -> usize {
    let (cx, cy) = cell;
    let seg = match m {
        (1, 0) => (cx + 1, cy),
        (-1, 0) => (cx, cy + 1),
        (0, 1) => (cx + 1, cy + 1),
        _ => (cx, cy),
    };
    for (k, &(v, dd, n)) in runs.iter().enumerate() {
        if (dd.1, -dd.0) != m {
            continue;
        }
        for i in 0..n {
            if (v.0 + i * dd.0, v.1 + i * dd.1) == seg {
                return k;
            }
        }
    }
    panic!("AssertionError: no run for cell {cell:?} face {m:?}");
}

#[derive(Clone)]
enum Key {
    Plate,
    Run(usize),
}

fn view(occ: &BTreeSet<V3>, runs: &[Run], colour: &HashMap<usize, (&str, Option<&str>)>, az: f64, el: f64, w: f64, h: f64, title: &str) -> String {
    let (ca, sa) = (cos(py::radians(az)), sin(py::radians(az)));
    let (ce, se) = (cos(py::radians(el)), sin(py::radians(el)));
    let rot = |p: [f64; 3]| -> [f64; 3] {
        let [x, y, z] = p;
        let (x, z) = (x * ca + z * sa, -x * sa + z * ca);
        let (y, z) = (y * ce - z * se, y * se + z * ce);
        [x, y, z]
    };
    let proj = |p: [f64; 3]| -> (f64, f64) {
        let [x, y, z] = rot(p);
        ((x - z) * 0.866, (x + z) * 0.5 - y)
    };
    let f3 = |v: V3| [v[0] as f64, v[1] as f64, v[2] as f64];
    let mut quads: Vec<(f64, Vec<[f64; 3]>, &str, Option<&str>)> = vec![];
    for c in occ {
        for (nrm, cor) in FACES {
            if occ.contains(&[c[0] + nrm[0], c[1] + nrm[1], c[2] + nrm[2]]) {
                continue;
            }
            let key = if nrm[2] != 0 { Key::Plate } else { Key::Run(face_run(runs, (c[0], c[1]), (nrm[0], nrm[1]))) };
            let (col, tag) = match key {
                Key::Plate => (PLATE_COL, None),
                Key::Run(k) => colour[&k],
            };
            let rn = rot(f3(nrm));
            if rn[0] + rn[1] + rn[2] <= 0.01 {
                continue;
            }
            let pts: Vec<[f64; 3]> = cor.iter().map(|v| f3([c[0] + v[0], c[1] + v[1], c[2] + v[2]])).collect();
            let mid = |i: usize| pts.iter().fold(0.0, |s, p| s + p[i]) / 4.0;
            let rc = rot([mid(0), mid(1), mid(2)]);
            quads.push((rc[0] + rc[1] + rc[2], pts, col, tag));
        }
    }
    quads.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let pp: Vec<(f64, f64)> = quads.iter().flat_map(|q| q.1.iter().map(|&p| proj(p))).collect();
    let (minx, maxx) = (trumpet::bore::fmin(pp.iter().map(|p| p.0)), trumpet::bore::fmax(pp.iter().map(|p| p.0)));
    let (miny, maxy) = (trumpet::bore::fmin(pp.iter().map(|p| p.1)), trumpet::bore::fmax(pp.iter().map(|p| p.1)));
    let s = py::min((w - 46.0) / (maxx - minx), (h - 74.0) / (maxy - miny));
    let ox = 23.0 - minx * s + ((w - 46.0) - (maxx - minx) * s) / 2.0;
    let oy = 52.0 - miny * s;
    let pt = |p: [f64; 3]| {
        let (a, b) = proj(p);
        (ox + a * s, oy + b * s)
    };
    let mut out = vec![format!("<text x=\"{:.0}\" y=\"30\" class=\"t\">{title}</text>", w / 2.0)];
    for (_, pts, col, tag) in &quads {
        let pp: Vec<(f64, f64)> = pts.iter().map(|&p| pt(p)).collect();
        let st = pp.iter().map(|(a, b)| format!("{a:.1},{b:.1}")).collect::<Vec<_>>().join(" ");
        out.push(format!(
            "<polygon points=\"{st}\" fill=\"{col}\" fill-opacity=\"{}\" stroke=\"#222\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
            if tag.is_some() { "0.98" } else { "0.9" },
            if tag.is_some() { "1.6" } else { "0.9" }
        ));
        if let Some(t) = tag {
            let cx = pp.iter().fold(0.0, |s, p| s + p.0) / 4.0;
            let cy = pp.iter().fold(0.0, |s, p| s + p.1) / 4.0;
            out.push(format!("<text x=\"{cx:.0}\" y=\"{:.0}\" class=\"o\">{t}</text>", cy + 5.0));
        }
    }
    out.join("\n")
}

fn main() {
    let (mut path, mut open_faces, mut label, mut out) = ("UUR".to_string(), String::new(), String::new(), "piece_3d.svg".to_string());
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: piece_render [-h] [--path PATH] [--open_faces OPEN_FACES] [--label LABEL] [--out OUT]";
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        let (k, inline) = match a.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        if k == "-h" || k == "--help" {
            println!("{usage}\n\nDraw one assembled piece from several angles, each face coloured by its part.");
            return;
        }
        let slot = match k.as_str() {
            "--path" => &mut path,
            "--open_faces" => &mut open_faces,
            "--label" => &mut label,
            "--out" => &mut out,
            _ => {
                eprintln!("{usage}\npiece_render: error: unrecognized arguments: {a}");
                std::process::exit(2);
            }
        };
        *slot = match inline {
            Some(v) => v,
            None => {
                i += 1;
                argv.get(i).cloned().unwrap_or_else(|| {
                    eprintln!("{usage}\npiece_render: error: argument {k}: expected one argument");
                    std::process::exit(2)
                })
            }
        };
        i += 1;
    }
    if label.is_empty() {
        label = if path.is_empty() { "piece".into() } else { path.clone() };
    }
    let d = Design::default();
    let (cells, runs, tips, wallpart, plate) = describe(&d, &path, &open_faces).unwrap_or_else(|e| sys_exit(&format!("ValueError: {e}")));
    let occ: BTreeSet<V3> = cells.iter().map(|c| [c.0, c.1, 0]).collect();
    let mut colour: HashMap<usize, (&str, Option<&str>)> = HashMap::new();
    let mut legend: Vec<(&str, String)> = vec![(PLATE_COL, format!("{label}P  x2  {:.2} x {:.2}", plate.w, plate.h))];
    for (i, (k, p)) in wallpart.iter().enumerate() {
        let col = WALL_COLS[i % WALL_COLS.len()];
        colour.insert(*k, (col, None));
        let n = runs[*k].2;
        legend.push((col, format!("{label}W  {n} cell{}  {:.2}", if n > 1 { "s" } else { " " }, py::max(p.w, p.h))));
    }
    colour.insert(tips[0], (IN_COL, Some("IN")));
    colour.insert(tips[1], (OUT_COL, Some("OUT")));
    legend.push((IN_COL, "entry - open, carries the TABS".into()));
    legend.push((OUT_COL, "exit  - open, carries the NOTCHES".into()));
    let (w, h) = (400i64, 360i64);
    let body: Vec<String> = VIEWS
        .iter()
        .enumerate()
        .map(|(i, &(az, el, nm))| format!("<g transform=\"translate({},{})\">{}</g>", (i as i64 % 3) * w, (i as i64 / 3) * h, view(&occ, &runs, &colour, az, el, w as f64, h as f64, nm)))
        .collect();
    let ly = 2 * h + 16;
    let what = format!("--path={path}{}", if open_faces.is_empty() { String::new() } else { format!(" --open_faces={open_faces}") });
    let mut leg = vec![format!("<text x=\"24\" y=\"{ly}\" class=\"h\">{label} = {what} — 2 plates + {} walls</text>", wallpart.len())];
    for (i, (col, name)) in legend.iter().enumerate() {
        let (cx, cy) = (24 + (i as i64 % 2) * 560, ly + 28 + (i as i64 / 2) * 26);
        leg.push(format!("<rect x=\"{cx}\" y=\"{}\" width=\"20\" height=\"14\" rx=\"2\" fill=\"{col}\" stroke=\"#222\" stroke-width=\"0.9\"/>", cy - 11));
        leg.push(format!("<text x=\"{}\" y=\"{cy}\" class=\"l\">{name}</text>", cx + 30));
    }
    let hh = ly + 28 + ((legend.len() as i64 + 1) / 2) * 26 + 14;
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{hh}\" viewBox=\"0 0 {} {hh}\"><style>.t{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222;text-anchor:middle}}.h{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222}}.l{{font:13px ui-monospace,Menlo,monospace;fill:#333}}.o{{font:700 15px ui-sans-serif,sans-serif;fill:#fff;text-anchor:middle}}</style><rect width=\"{}\" height=\"{hh}\" fill=\"#fff\"/>{}{}</svg>",
        w * 3,
        w * 3,
        w * 3,
        body.concat(),
        leg.concat()
    );
    std::fs::write(&out, svg).unwrap_or_else(|e| sys_exit(&format!("{out}: {e}")));
    println!("{out}  {} x {hh}   {} cells, 2 plates + {} walls", w * 3, cells.len(), wallpart.len());
}
