//! `regress [PATTERN]`: run the pre-cut gate over every design we have built or
//! tried, or those whose name contains PATTERN. regress.py until 2026-11-08.
//! `regress --count` prints how many designs the table holds.
//!
//! Designs diverge more than they look: the first trumpet never called the lap
//! code, so a crash in it went unseen until a spiral needed one, and a volume
//! check that passed the trumpet at +0.35% was 6% out on a spiral of identical
//! construction. A change is only known good against the whole set. The table
//! is tools/designs.rs; every design carries the switches it is cut with.
use lasermade_tools::py;
use lasermade_tools::turn3d::J;
use std::process::Output;
use trumpet::bore::{axis_of, walk_text, Design};
use trumpet::designs::{designs, here, pool, sibling, walk_of};

/// The 3D page beside the cut files, which nothing else looks at.
///
/// check reads sheets and geometry and never opens the viewer, so a render can
/// be wrong while every check passes -- it has been: the page drew a uniform
/// lattice for a stretched one, and 392 checks said nothing. This does not
/// judge the picture. It asserts the page was handed the walk it sits beside:
/// the block count and the centreline it prints.
fn check_page(folder: &str, text: &str, switches: &[&str]) -> Option<String> {
    let mut d = Design::default();
    let sw: Vec<String> = switches.iter().map(|s| s.to_string()).collect();
    let left = d.take_design_switches(&sw);
    if !left.is_empty() {
        return Some(format!("{folder}: check_page does not know {}, so it cannot measure this design", left.join(" ")));
    }
    let sp = match walk_text(text).and_then(|t| d.specs_for(&t)) {
        Ok(s) => s,
        Err(e) => return Some(format!("{folder}: {e}")),
    };
    let want_blocks = sp.rec.len() as i64;
    let want_mm = py::round(sp.rec.iter().fold(0.0, |s, r| s + d.extent(r, axis_of(r.out))));
    let dir = format!("{}/{folder}", here());
    let mut pages: Vec<String> = std::fs::read_dir(&dir).map(|it| it.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|f| f.ends_with(".html")).collect()).unwrap_or_default();
    if pages.len() != 1 {
        return Some(format!("{} html pages in {folder}, expected 1", pages.len()));
    }
    let page = pages.pop().unwrap();
    let body = std::fs::read_to_string(format!("{dir}/{page}")).unwrap_or_default();
    let r = lasermade_tools::re::rx(r"(?s)const SETS = (\[.*?\]);\n");
    let Some(m) = lasermade_tools::re::first(&r, &body) else {
        return Some(format!("{page}: no SETS block -- not a viewer page?"));
    };
    let sets = J::parse(lasermade_tools::re::g(&m, 1)).unwrap_or_else(|e| panic!("{page}: {e}"));
    let sets = sets.arr();
    if sets.len() != 1 {
        return Some(format!("{page}: {} coils in a per-coil page", sets.len()));
    }
    let d0 = sets[0].get("d").unwrap();
    let blocks = d0.get("blocks").unwrap().num();
    if blocks.f() != want_blocks as f64 {
        return Some(format!("{page}: page says {} blocks, the walk has {want_blocks}", blocks.repr()));
    }
    let mm = d0.get("mm").unwrap().num();
    if mm.f() != want_mm as f64 {
        return Some(format!("{page}: page says {}mm of centreline, the walk gives {want_mm}", mm.repr()));
    }
    if sets[0].get("walk").unwrap().str() != text.trim() {
        return Some(format!("{page}: the page carries a different walk"));
    }
    None
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let all = designs();
    if argv.first().map(|s| s.as_str()) == Some("--count") {
        println!("{}", all.len());
        return;
    }
    let pattern = argv.first().cloned();
    let here = here();
    let isdir = |p: &str| std::path::Path::new(&format!("{here}/{p}")).is_dir();
    let chosen: Vec<_> = all.iter().filter(|e| pattern.as_ref().map_or(true, |p| e.0.to_lowercase().contains(&p.to_lowercase()))).collect();
    // every design's check started at once: each is its own process, writes
    // nothing and reads only its own sheets
    let jobs: Vec<Option<Vec<String>>> = chosen
        .iter()
        .map(|&&(_, walk, folder, sw)| {
            if folder.is_some_and(|f| !isdir(&format!("{f}/cut-files"))) {
                return None;
            }
            let mut a = vec![walk_of(walk)];
            a.extend(sw.iter().map(|s| s.to_string()));
            if let Some(f) = folder {
                a.extend(["--files".to_string(), format!("{f}/cut-files")]);
            }
            Some(a)
        })
        .collect();
    let check = sibling("check");
    let results: Vec<Option<Output>> = pool(jobs, |a| a.map(|a| std::process::Command::new(&check).args(&a).current_dir(&here).output().expect("run check")));
    let (mut bad, mut o) = (0, String::new());
    for (&&(name, walk, folder, sw), r) in chosen.iter().zip(results) {
        let text = walk_of(walk);
        if let Some(f) = folder {
            if !isdir(f) {
                o.push_str(&format!("  FAIL  {name:<18} names {f}, which is not there\n"));
                bad += 1;
                continue;
            }
            if !isdir(&format!("{f}/cut-files")) {
                o.push_str(&format!("  FAIL  {name:<18} has no cut-files/ under {f}\n"));
                bad += 1;
                continue;
            }
        }
        let r = r.unwrap();
        let stdout = String::from_utf8_lossy(&r.stdout);
        let stderr = String::from_utf8_lossy(&r.stderr);
        let last = stdout.trim().lines().last().unwrap_or("no output").to_string();
        let page = folder.and_then(|f| check_page(f, &text, sw));
        let ok = r.status.success() && last.contains("0 failed") && page.is_none();
        bad += !ok as usize;
        o.push_str(&format!("  {}  {name:<18} {last}{}\n", if ok { "pass" } else { "FAIL" }, if page.is_none() { "" } else { "   + page" }));
        if let Some(p) = &page {
            o.push_str(&format!("          page: {p}\n"));
        }
        if !ok {
            for line in stdout.lines() {
                if line.contains("FAIL") || line.starts_with("      ") {
                    o.push_str(&format!("          {}\n", line.trim()));
                }
            }
            if let Some(l) = stderr.trim().lines().last() {
                o.push_str(&format!("          {l}\n"));
            }
        }
    }
    if chosen.is_empty() {
        o.push_str(&format!("  no design name contains {}. {} designs are in the table.\n\n  0 designs run\n", py::str_repr(pattern.as_deref().unwrap_or("")), all.len()));
        print!("{o}");
        std::process::exit(1);
    }
    o.push_str(if bad > 0 { "\n" } else { "\n  all designs pass\n" });
    if bad > 0 {
        o.push_str(&format!("  {bad} design(s) failing\n"));
    }
    print!("{o}");
    std::process::exit(if bad > 0 { 1 } else { 0 });
}
