//! `bed_sheets CUT_DIR OUT_DIR STEM [--tries N] [--seed S]`: bundle a folder of
//! per-section cut files onto as few P2S bed sheets as it can. bed_sheets.py
//! until 2026-11-08; the same command writes the same bytes it did.
//!
//! Writes STEM-sheet1ofN.svg ... into OUT_DIR. Each section file is placed
//! WHOLE -- its parts stay together, so the number engraved on every part still
//! finds its neighbours -- at 0 or 90 degrees, GAP apart and GAP from the bed
//! edge. The paths are copied verbatim inside a translate (and rotate): what is
//! cut is what the section files say, and the section files are what
//! bore_split wrote and gated.
//!
//! Packing is maxrects, best short side fit, on each section's real content box
//! rather than its canvas, which carries a margin. Maxrects is greedy, so the
//! order sections arrive in decides how many sheets it needs. Largest first is
//! tried, then up to --tries seeded random orders, stopping as soon as one
//! reaches the area bound. The seed makes the search, and so the sheet,
//! reproducible: the generator is Python's own Mersenne Twister, seeded and
//! drawn from as random.Random was, so a seed means what it meant.
//!
//! Every sheet is re-read before the tool reports success, and it exits 1 if
//! any check fails: every section is placed exactly once; every path and
//! polyline is byte-identical to its section file's; every section lies inside
//! the bed; no two sections come closer than GAP. That last check is not
//! decoration: the first packer reserved the gap only to the right of and below
//! each placed section, and a one-sheet layout of the tight coil came out with
//! seven pairs touching.
use lasermade_tools::py::{self, mcos as cos, msin as sin};
use trumpet::bore::{fmax, fmin, pjoin, pts, sys_exit, BED_H, BED_W};
use trumpet::pyrandom::Mt;
use trumpet::xml::{parse, El};

const GAP: f64 = 4.0;

// --- geometry --------------------------------------------------------------------

type M = [f64; 6];

fn mul(m: &M, n: &M) -> M {
    [m[0] * n[0] + m[2] * n[1], m[1] * n[0] + m[3] * n[1], m[0] * n[2] + m[2] * n[3], m[1] * n[2] + m[3] * n[3], m[0] * n[4] + m[2] * n[5] + m[4], m[1] * n[4] + m[3] * n[5] + m[5]]
}

fn matrix(t: Option<&str>) -> M {
    let mut m = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let r = lasermade_tools::re::rx(r"(\w+)\(([^)]*)\)");
    let sp = lasermade_tools::re::rx(r"[ ,]+");
    for c in lasermade_tools::re::all(&r, t.unwrap_or("")) {
        let (op, args) = (lasermade_tools::re::g(&c, 1), lasermade_tools::re::g(&c, 2));
        let a: Vec<f64> = lasermade_tools::re::split(&sp, args.trim()).iter().map(|v| py::float(v).unwrap_or_else(|| sys_exit(&format!("ValueError: could not convert string to float: {}", py::str_repr(v))))).collect();
        let n = if op == "translate" {
            [1.0, 0.0, 0.0, 1.0, a[0], if a.len() > 1 { a[1] } else { 0.0 }]
        } else if op == "rotate" && a.len() == 1 {
            let (c, s) = (cos(py::radians(a[0])), sin(py::radians(a[0])));
            [c, s, -s, c, 0.0, 0.0]
        } else {
            sys_exit(&format!("unexpected transform {}", py::str_repr(t.unwrap_or(""))));
        };
        m = mul(&m, &n);
    }
    m
}

struct Acc {
    pts: Vec<(f64, f64)>,
    data: Vec<(String, String)>,
}

/// Every point drawn under el, in el's parent space, and every path's data.
fn geometry(el: &El, m: &M, acc: &mut Acc) {
    let m = mul(m, &matrix(el.get("transform")));
    let mut p: Vec<(f64, f64)> = vec![];
    if el.local() == "path" {
        let d = el.get("d").unwrap_or("");
        p = pts(d).unwrap_or_else(|e| sys_exit(&format!("ValueError: {e}")));
        acc.data.push((d.to_string(), el.get("stroke").unwrap_or("").to_string()));
    } else if el.local() == "polyline" {
        let s = el.get("points").unwrap_or("");
        let sp = lasermade_tools::re::rx(r"[\s,]+");
        let v: Vec<f64> = lasermade_tools::re::split(&sp, s.trim()).iter().map(|n| py::float(n).unwrap()).collect();
        p = v.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0], c[1])).collect();
        acc.data.push((s.to_string(), el.get("stroke").unwrap_or("").to_string()));
    }
    for (x, y) in p {
        acc.pts.push((m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]));
    }
    for c in &el.kids {
        geometry(c, &m, acc);
    }
}

fn bbox(p: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    (fmin(p.iter().map(|q| q.0)), fmin(p.iter().map(|q| q.1)), fmax(p.iter().map(|q| q.0)), fmax(p.iter().map(|q| q.1)))
}

// --- packing ---------------------------------------------------------------------

type Item = (String, f64, f64);
/// (sheet, key, rot, x, y)
type Placed = (usize, String, i64, f64, f64);
type Free = (f64, f64, f64, f64);

/// items placed in the order given; Err is the item that fits no bed
fn maxrects(items: &[Item], bw: f64, bh: f64) -> Result<Vec<Placed>, String> {
    let mut sheets: Vec<Vec<Free>> = vec![];
    let mut out = vec![];
    for (key, w, h) in items {
        let (w, h) = (*w, *h);
        let mut best: Option<(usize, Option<(f64, f64)>, i64, f64, f64, f64, f64)> = None;
        for (si, free) in sheets.iter().enumerate() {
            for &(fx, fy, fw, fh) in free {
                for (rot, a, b) in [(0, w, h), (90, h, w)] {
                    if a <= fw && b <= fh {
                        let s = (py::min(fw - a, fh - b), py::max(fw - a, fh - b));
                        let better = match &best {
                            None => true,
                            Some(bb) => {
                                let bs = bb.1.unwrap();
                                (si, s.0, s.1) < (bb.0, bs.0, bs.1) && !(si == bb.0 && s.0 == bs.0 && s.1 == bs.1) && {
                                    si < bb.0 || (si == bb.0 && (s.0 < bs.0 || (s.0 == bs.0 && s.1 < bs.1)))
                                }
                            }
                        };
                        if better {
                            best = Some((si, Some(s), rot, fx, fy, a, b));
                        }
                    }
                }
            }
        }
        if best.is_none() {
            sheets.push(vec![(GAP, GAP, bw - 2.0 * GAP, bh - 2.0 * GAP)]);
            let si = sheets.len() - 1;
            let (fx, fy, fw, fh) = sheets[si][0];
            for (rot, a, b) in [(0, w, h), (90, h, w)] {
                if a <= fw && b <= fh {
                    best = Some((si, None, rot, fx, fy, a, b));
                    break;
                }
            }
            if best.is_none() {
                return Err(format!("{key} ({w:.1} x {h:.1}mm) does not fit the bed"));
            }
        }
        let (si, _, rot, x, y, a, b) = best.unwrap();
        out.push((si, key.clone(), rot, x, y));
        let (px0, py0, px1, py1) = (x - GAP, y - GAP, x + a + GAP, y + b + GAP);
        let mut new: Vec<Free> = vec![];
        for &(fx, fy, fw, fh) in &sheets[si] {
            let (fx1, fy1) = (fx + fw, fy + fh);
            if px0 >= fx1 || px1 <= fx || py0 >= fy1 || py1 <= fy {
                new.push((fx, fy, fw, fh));
                continue;
            }
            if px0 > fx {
                new.push((fx, fy, px0 - fx, fh));
            }
            if px1 < fx1 {
                new.push((px1, fy, fx1 - px1, fh));
            }
            if py0 > fy {
                new.push((fx, fy, fw, py0 - fy));
            }
            if py1 < fy1 {
                new.push((fx, py1, fw, fy1 - py1));
            }
        }
        let new: Vec<Free> = new.into_iter().filter(|r| r.2 > 0.0 && r.3 > 0.0).collect();
        sheets[si] = new
            .iter()
            .enumerate()
            .filter(|(i, r)| !new.iter().enumerate().any(|(j, o)| j != *i && o.0 <= r.0 && o.1 <= r.1 && o.0 + o.2 >= r.0 + r.2 && o.1 + o.3 >= r.1 + r.3 && (o != *r || j < *i)))
            .map(|(_, r)| *r)
            .collect();
    }
    Ok(out)
}

fn sheets_needed(p: &[Placed]) -> usize {
    p.iter().map(|q| q.0).max().unwrap() + 1
}

/// Largest first, then seeded random orders; the first order that reaches the
/// area bound wins, otherwise the one with fewest sheets.
fn search(items: &[Item], tries: i64, seed: i64) -> (Vec<Placed>, i64) {
    let mut first = items.to_vec();
    first.sort_by(|a, b| (-a.1 * a.2).partial_cmp(&(-b.1 * b.2)).unwrap());
    let mut best = maxrects(&first, BED_W, BED_H).unwrap_or_else(|e| sys_exit(&e));
    let floor = (items.iter().fold(0.0, |s, t| s + t.1 * t.2) / ((BED_W - 2.0 * GAP) * (BED_H - 2.0 * GAP))).ceil() as i64;
    let mut rng = Mt::new(seed);
    for i in 0..tries.max(0) {
        if sheets_needed(&best) as i64 <= floor {
            break;
        }
        let mut order = items.to_vec();
        rng.shuffle(&mut order);
        if i % 2 == 1 {
            let keys: Vec<f64> = order.iter().map(|t| -py::max(t.1, t.2) + rng.uniform(0.0, 40.0)).collect();
            let mut idx: Vec<usize> = (0..order.len()).collect();
            idx.sort_by(|&a, &b| keys[a].partial_cmp(&keys[b]).unwrap());
            order = idx.into_iter().map(|k| order[k].clone()).collect();
        }
        let Ok(placed) = maxrects(&order, BED_W, BED_H) else { continue };
        if sheets_needed(&placed) < sheets_needed(&best) {
            best = placed;
        }
    }
    (best, floor)
}

fn section_of(name: &str) -> String {
    let r = lasermade_tools::re::rx(r"(\d+of\d+)");
    lasermade_tools::re::g(&lasermade_tools::re::first(&r, name).unwrap_or_else(|| sys_exit(&format!("TypeError: 'NoneType' object is not subscriptable ({name})"))), 1).to_string()
}

struct Section {
    x0: f64,
    y0: f64,
    h: f64,
    body: String,
}

fn write(placed: &[Placed], sections: &std::collections::HashMap<String, Section>, out_dir: &str, stem: &str) -> Vec<(String, usize, String)> {
    let n = sheets_needed(placed);
    std::fs::create_dir_all(out_dir).unwrap_or_else(|e| sys_exit(&format!("{out_dir}: {e}")));
    let mut written = vec![];
    for si in 0..n {
        let mut here: Vec<&Placed> = placed.iter().filter(|p| p.0 == si).collect();
        here.sort_by(|a, b| a.1.cmp(&b.1));
        let mut groups = vec![];
        for &(_, ref f, rot, x, y) in &here {
            let s = &sections[f];
            let t = if *rot == 0 { format!("translate({:.3},{:.3})", x - s.x0, y - s.y0) } else { format!("translate({:.3},{:.3}) rotate(90)", x + s.h + s.y0, y - s.x0) };
            groups.push(format!("<g id=\"section-{}\" transform=\"{t}\">{}</g>", section_of(f), s.body));
        }
        let nums: Vec<String> = here.iter().map(|p| section_of(&p.1).split("of").next().unwrap().trim_start_matches('0').to_string()).collect();
        let nums = nums.join(", ");
        let path = pjoin(out_dir, &format!("{stem}-sheet{}of{n}.svg", si + 1));
        let body = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{BED_W:.2}mm\" height=\"{BED_H:.2}mm\" viewBox=\"0 0 {BED_W:.2} {BED_H:.2}\">\n<title>{stem} - sheet {} of {n}, sections {nums}</title>\n<desc>1 user unit = 1mm, the whole xTool P2S bed ({BED_W:.0} x {BED_H:.0}mm). Each section file is copied whole and unchanged, {GAP:.0}mm apart; written by tools/bed_sheets.rs. black #000000 cuts, blue #0000ff engraves.</desc>\n{}\n</svg>\n",
            si + 1,
            groups.join("\n")
        );
        std::fs::write(&path, body).unwrap_or_else(|e| sys_exit(&format!("{path}: {e}")));
        written.push((path, here.len(), nums));
    }
    written
}

/// Re-read what was written. Returns a list of failures.
fn check(written: &[(String, usize, String)], originals: &std::collections::BTreeMap<String, Vec<(String, String)>>) -> Vec<String> {
    let mut bad = vec![];
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (path, _, _) in written {
        let root = parse(&std::fs::read_to_string(path).unwrap()).unwrap_or_else(|e| sys_exit(&format!("{path}: {e}")));
        let mut boxes: Vec<(String, (f64, f64, f64, f64))> = vec![];
        for g in root.kids.iter().filter(|k| k.local() == "g") {
            let sec = g.get("id").unwrap_or("").split_once('-').map(|x| x.1).unwrap_or("").to_string();
            if seen.contains_key(&sec) {
                bad.push(format!("section {sec} placed twice"));
            }
            seen.insert(sec.clone(), path.clone());
            let mut acc = Acc { pts: vec![], data: vec![] };
            geometry(g, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0], &mut acc);
            let mut data = acc.data.clone();
            data.sort();
            if Some(&data) != originals.get(&sec) {
                bad.push(format!("section {sec}: paths differ from its section file"));
            }
            let b = bbox(&acc.pts);
            if b.0 < 0.0 || b.1 < 0.0 || b.2 > BED_W || b.3 > BED_H {
                bad.push(format!("section {sec} leaves the bed"));
            }
            boxes.push((sec, b));
        }
        for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                let (a, b) = (boxes[i].1, boxes[j].1);
                let g = GAP - 0.01;
                if a.0 < b.2 + g && b.0 < a.2 + g && a.1 < b.3 + g && b.1 < a.3 + g {
                    bad.push(format!("sections {} and {} closer than {}mm", boxes[i].0, boxes[j].0, py::g(GAP)));
                }
            }
        }
    }
    for sec in originals.keys() {
        if !seen.contains_key(sec) {
            bad.push(format!("section {sec} not placed"));
        }
    }
    bad
}

const USAGE: &str = "usage: bed_sheets [-h] [--tries TRIES] [--seed SEED] cut_dir out_dir stem";

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (mut pos, mut tries, mut seed) = (vec![], 20000i64, 1i64);
    let mut k = 0;
    let int = |name: &str, v: Option<&String>| -> i64 {
        let v = v.unwrap_or_else(|| {
            eprintln!("{USAGE}\nbed_sheets: error: argument --{name}: expected one argument");
            std::process::exit(2)
        });
        trumpet::bore::pyint(v).unwrap_or_else(|| {
            eprintln!("{USAGE}\nbed_sheets: error: argument --{name}: invalid int value: {}", py::str_repr(v));
            std::process::exit(2)
        })
    };
    while k < argv.len() {
        let a = &argv[k];
        if a == "-h" || a == "--help" {
            println!("{USAGE}\n\nBundle a folder of per-section cut files onto as few P2S bed sheets as it can.");
            return;
        } else if a == "--tries" || a == "--seed" {
            let v = int(&a[2..], argv.get(k + 1));
            if a == "--tries" { tries = v } else { seed = v }
            k += 2;
            continue;
        } else if let Some(v) = a.strip_prefix("--tries=") {
            tries = int("tries", Some(&v.to_string()));
        } else if let Some(v) = a.strip_prefix("--seed=") {
            seed = int("seed", Some(&v.to_string()));
        } else if a.starts_with('-') && a.len() > 1 {
            eprintln!("{USAGE}\nbed_sheets: error: unrecognized arguments: {a}");
            std::process::exit(2);
        } else {
            pos.push(a.clone());
        }
        k += 1;
    }
    if pos.len() != 3 {
        let missing = ["cut_dir", "out_dir", "stem"][pos.len().min(3)..].join(", ");
        if pos.len() > 3 {
            eprintln!("{USAGE}\nbed_sheets: error: unrecognized arguments: {}", pos[3..].join(" "));
        } else {
            eprintln!("{USAGE}\nbed_sheets: error: the following arguments are required: {missing}");
        }
        std::process::exit(2);
    }
    let (cut_dir, out_dir, stem) = (&pos[0], &pos[1], &pos[2]);
    let mut names: Vec<String> = std::fs::read_dir(cut_dir)
        .unwrap_or_else(|e| sys_exit(&format!("FileNotFoundError: {e}: {}", py::str_repr(cut_dir))))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut sections = std::collections::HashMap::new();
    let mut originals = std::collections::BTreeMap::new();
    let mut items: Vec<Item> = vec![];
    for f in names.iter().filter(|f| f.ends_with(".svg")) {
        let p = pjoin(cut_dir, f);
        let root = parse(&std::fs::read_to_string(&p).unwrap()).unwrap_or_else(|e| sys_exit(&format!("{p}: {e}")));
        let mut acc = Acc { pts: vec![], data: vec![] };
        geometry(&root, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0], &mut acc);
        let (x0, y0, x1, y1) = bbox(&acc.pts);
        let body: String = root.kids.iter().filter(|c| c.local() != "title" && c.local() != "desc").map(|c| c.tostring()).collect();
        sections.insert(f.clone(), Section { x0, y0, h: y1 - y0, body });
        let mut data = acc.data;
        data.sort();
        originals.insert(section_of(f), data);
        items.push((f.clone(), x1 - x0, y1 - y0));
    }
    if items.is_empty() {
        sys_exit(&format!("no section files in {cut_dir}"));
    }
    let (placed, floor) = search(&items, tries, seed);
    let written = write(&placed, &sections, out_dir, stem);
    let mut o = String::new();
    for (path, k, nums) in &written {
        o.push_str(&format!("  {}: {k} sections ({nums})\n", path.rsplit('/').next().unwrap()));
    }
    o.push_str(&format!("  {} sheet(s); the area bound is {floor}\n", written.len()));
    let bad = check(&written, &originals);
    for b in &bad {
        o.push_str(&format!("  FAIL {b}\n"));
    }
    o.push_str(&format!("  {} sections, {} failed\n", items.len(), bad.len()));
    print!("{o}");
    std::process::exit(if bad.is_empty() { 0 } else { 1 });
}
