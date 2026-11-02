// Piece counts, distinct piece shapes and plate sizes, read off bore_split.py and
// cached in parts.json so the tables do not have to shell out on every run.
//
// The plate is the bounding box the piece is cut from, which is what goes on the
// sheet and what the operator handles. Its average is the laser-cutting number:
// fewer, larger parts is less weeding, less sorting and fewer fingers to align.
// Note it is a bounding box, not the cut outline -- an L-shaped piece leaves the
// corner behind, so this overstates material for those and is a size measure
// rather than an area-used one.
//
//   target/release/parts            (rebuild the cache)

use coilsearch::bore::{cap_num, kinds, run_split};
use coilsearch::corpus::{pad_end, read, root, walks};
use coilsearch::jsfmt::{fixed, num, round};
use coilsearch::json::J;
use regex::Regex;
use std::path::Path;

struct Origin {
    name: String,
    walk: String,
    promoted_to: String,
}

// The walk this whole search started from is no longer in walks/: it won a category
// and was promoted out to a sibling directory of its own. Every "against the walk
// this started from" figure on the page still compares to it, so it is measured here
// too, from wherever it now lives. Found rather than written down: derived.txt says
// which coil came from staircase_coil, and the promotion convention is the same
// identifier with coil_ dropped and the underscores hyphenated.
fn origin(root: &Path) -> Option<Origin> {
    let text = read(&root.join("derived.txt"));
    let rows: Vec<Vec<&str>> = text
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().collect())
        .collect();
    let row = rows.iter().find(|r| r[1..].contains(&"staircase_coil"))?;
    let name = row[0].to_string();
    if root.join("walks").join(format!("{name}.txt")).exists() {
        return None; // still in the set
    }
    let dir = name.strip_prefix("coil_").unwrap_or(&name).replace('_', "-");
    // Promoted designs sit DIRECTLY under the family (since 2026-09-15).
    let family = root.join("..");
    let f = family.join(&dir).join(format!("{dir}.txt"));
    if f.exists() {
        return Some(Origin { name, walk: read(&f).trim().to_string(), promoted_to: format!("../{dir}/{dir}.html") });
    }
    let shown = family.canonicalize().unwrap_or(family);
    eprintln!(
        "derived.txt says the search started from {name}, which is neither in\nwalks/ nor anywhere under {}. Nothing can be measured against it,\nand guessing a substitute is what this replaced.",
        shown.display()
    );
    std::process::exit(1);
}

/// How many pieces you lay before the sequence starts over.
fn period(a: &[String]) -> usize {
    for p in 1..=a.len() {
        if (0..a.len().saturating_sub(p)).all(|i| a[i] == a[i + p]) {
            return p;
        }
    }
    a.len()
}

fn main() {
    let root = root();
    let mut sources: Vec<(String, String, Option<String>)> = walks().into_iter().map(|(n, w)| (n, w, None)).collect();
    if let Some(o) = origin(&root) {
        sources.push((o.name, o.walk, Some(o.promoted_to)));
    }

    let pieces_re = Regex::new(r"(\d+) pieces to assemble").unwrap();
    let plate_re = Regex::new(r"(?m)^\s+\d+\s+(\d+)-(\d+)\s+\w+\s+\w+\s+\w+\s+\d+x\d+ bl (\d+)x(\d+)").unwrap();
    // A cut-list entry is <bore><n>-<design>-NNofMM-<kind>-<shape>-cut-files.svg since
    // 2026-09-03; hyphens fold back to underscores so the strings match what
    // parts.json has held.
    let seq_re = Regex::new(r"(?m)^\s+\d+\s+\S*?-\d+of\d+-(\S+?)-cut-files\.svg").unwrap();

    let mut out: Vec<(String, J)> = Vec::new();
    for (name, walk, promoted) in &sources {
        let res = run_split(walk, true).unwrap_or_else(|(_, so)| {
            print!("{so}");
            eprintln!("bore_split.py refused {name}");
            std::process::exit(1)
        });
        let pieces = cap_num(&pieces_re, &res);
        let plates: Vec<(f64, f64, f64)> = plate_re
            .captures_iter(&res)
            .map(|m| {
                let n = |i: usize| m[i].parse::<f64>().unwrap();
                (n(1), n(2), n(3) * n(4))
            })
            .collect();
        // The bore's mouth and exit are forced -- every design has them and no design
        // chooses them -- so nothing here is judged on them. This is the block range
        // belonging to the pieces in between.
        let interior = if plates.len() > 2 {
            [plates[1].0, plates[plates.len() - 2].1]
        } else {
            [1.0, plates.last().map(|p| p.1).unwrap_or(0.0)]
        };
        let k = kinds(&res);
        let elbows = k.iter().find(|(n, _)| n == "elbow").map(|e| e.1).unwrap_or(0);
        let seq: Vec<String> = seq_re.captures_iter(&res).map(|m| m[1].replace('-', "_")).collect();
        let mut shapes: Vec<String> = Vec::new();
        for s in &seq {
            if !shapes.contains(s) {
                shapes.push(s.clone());
            }
        }
        // The rhythm: how many pieces you lay before the sequence starts over, over
        // the interior. Descriptive, not scored.
        let inner_seq: Vec<String> = if seq.len() >= 2 { seq[1..seq.len() - 1].to_vec() } else { vec![] };
        let rhythm = if inner_seq.is_empty() { 0 } else { period(&inner_seq) };
        let mut interior_distinct: Vec<&String> = Vec::new();
        for s in &inner_seq {
            if !interior_distinct.contains(&s) {
                interior_distinct.push(s);
            }
        }
        let inner: Vec<f64> = if plates.len() > 2 { plates[1..plates.len() - 1].iter().map(|p| p.2).collect() } else { vec![] };
        let areas: Vec<f64> = if !inner.is_empty() { inner } else { plates.iter().map(|p| p.2).collect() };
        let total = areas.iter().fold(0.0, |a, x| a + x);
        let mean = total / areas.len() as f64;
        let repeats = if rhythm > 0 { inner_seq.len() as f64 / rhythm as f64 } else { 0.0 };
        let mut sorted = shapes.clone();
        sorted.sort();

        let mut entry: Vec<(&str, J)> = vec![
            ("pieces", J::Num(pieces)),
            ("kinds", J::Obj(k.iter().map(|(n, c)| (n.clone(), J::Num(*c as f64))).collect())),
            ("elbows", J::Num(elbows as f64)),
        ];
        if let Some(p) = promoted {
            entry.push(("promotedTo", J::s(p)));
            entry.push(("walk", J::s(walk)));
        }
        entry.extend([
            ("interiorBlocks", J::nums(interior)),
            ("innerPieces", J::Num(coilsearch::jsfmt::max(0.0, pieces - 2.0))),
            ("distinct", J::Num(shapes.len() as f64)),
            ("shapes", J::Arr(sorted.iter().map(|s| J::s(s)).collect())),
            ("rhythm", J::Num(rhythm as f64)),
            ("repeats", J::Num(repeats)),
            ("interiorDistinct", J::Num(interior_distinct.len() as f64)),
            ("meanPlate", J::Num(mean)),
            ("maxPlate", J::Num(areas.iter().copied().fold(f64::NEG_INFINITY, coilsearch::jsfmt::max))),
            ("minPlate", J::Num(areas.iter().copied().fold(f64::INFINITY, coilsearch::jsfmt::min))),
            ("totalPlate", J::Num(total)),
        ]);
        println!(
            "{} {} pieces {} distinct rhythm {} x{} mean plate {} mm2 elbows {}",
            pad_end(name, 18),
            num(pieces),
            shapes.len(),
            rhythm,
            fixed(repeats, 1),
            num(round(mean)),
            elbows
        );
        out.push((name.clone(), J::obj(entry)));
    }
    std::fs::write(root.join("parts.json"), J::Obj(out).stringify(1)).expect("cannot write parts.json");
    println!("wrote parts.json");
}
