//! `repro [--update]`: do the cut files on disk match what the code draws
//! today? repro.py until 2026-11-08.
//!
//! regress runs check over the sheets in cut-files/. It measures what is
//! committed; it never redraws it. So every invariant can hold while the
//! shipped SVG is one the current code would no longer produce. Two kinds of
//! folder, checked in opposite directions:
//!
//!   LIVE      redraw it and the bytes must match. Ordinary reproduction.
//!   AS-BUILT  its as-cut/ holds the sheets the wood was cut from, drawn at
//!             the 3.0 mm and 0.1 mm of the time, before the kerf and stub
//!             changes -- see fold2-long-straight/CLAUDE.md. A redraw SHOULD
//!             differ from them, so what must not change is the file itself:
//!             pinned by hash in as-built.sha256. Its cut-files/ is LIVE like
//!             any other (until 2026-11-12 the record WAS its cut-files/, and
//!             the folder had no sheet anyone should cut).
//!
//! `--update` re-pins them, and is the explicit act the prose asks for when it
//! says to know that you are overwriting the record of the instrument.
//!
//! The output path is part of every filename -- folder_stack() borrows the
//! family directory into the slug -- so each design is redrawn into a MIRROR of
//! its own path under a temp root, not into the temp directory itself.
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::process::Output;
use trumpet::designs::{designs, here, pool, sibling, walk_of};

const AS_BUILT: [&str; 1] = ["parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-1.5t"];

/// The folder as it is written in as-built.sha256: repo-relative, no '..'.
fn rel_of(folder: &str) -> String {
    lasermade_tools::text::normpath(&format!("tools/{folder}"))
}

fn sha(path: &str) -> String {
    let out = std::process::Command::new("shasum").args(["-a", "256", path]).output().expect("shasum");
    String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap_or("").to_string()
}

fn svgs(dir: &str) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir).map(|it| it.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|f| f.ends_with(".svg")).collect()).unwrap_or_default();
    v.sort();
    v
}

fn cut60(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn main() {
    let update = std::env::args().skip(1).any(|a| a == "--update");
    let here = here();
    let pins_path = format!("{here}/as-built.sha256");
    let mut pins: BTreeMap<String, String> = BTreeMap::new();
    if let Ok(s) = std::fs::read_to_string(&pins_path) {
        for line in s.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let (h, name) = line.split_once("  ").unwrap_or((line, ""));
                pins.insert(name.to_string(), h.to_string());
            }
        }
    }
    let tmp = std::env::temp_dir().join(format!("repro-{}", std::process::id()));
    let tmp = tmp.to_string_lossy().into_owned();
    let all = designs();
    let isdir = |p: &str| std::path::Path::new(p).is_dir();
    // every live design redrawn at once, into its mirror under tmp
    let jobs: Vec<Option<(String, Vec<String>)>> = all
        .iter()
        .map(|&(_, walk, folder, sw)| {
            let f = folder?;
            let rel = rel_of(f);
            if !isdir(&format!("{here}/{f}/cut-files")) {
                return None;
            }
            let dest = format!("{tmp}/{rel}");
            let mut a = vec!["--write".to_string(), dest.clone()];
            a.extend(walk_of(walk).split_whitespace().map(String::from));
            a.extend(sw.iter().map(|s| s.to_string()));
            Some((dest, a))
        })
        .collect();
    let bs = sibling("bore_split");
    let runs: Vec<Option<Output>> = pool(jobs, |j| {
        j.map(|(dest, a)| {
            std::fs::create_dir_all(&dest).expect("mkdir");
            std::process::Command::new(&bs).args(&a).current_dir(&here).output().expect("run bore_split")
        })
    });
    let (mut bad, mut same, mut frozen) = (0, 0, 0);
    let mut lines = vec![];
    let mut honoured = HashSet::new();
    let mut said = HashSet::new();
    let mut o = String::new();
    for (&(name, _, folder, _), run) in all.iter().zip(runs) {
        let Some(folder) = folder else { continue };
        let rel = rel_of(folder);
        let sheets = format!("{here}/{folder}/cut-files");
        if !isdir(&sheets) {
            o.push_str(&format!("  FAIL      {name}: no cut-files/ under {folder}\n"));
            bad += 1;
            continue;
        }
        let committed = svgs(&sheets);
        if AS_BUILT.contains(&rel.as_str()) {
            let record = format!("{here}/{folder}/as-cut");
            let kept = if isdir(&record) { svgs(&record) } else { vec![] };
            for f in &kept {
                let key = format!("{rel}/as-cut/{f}");
                let got = sha(&format!("{record}/{f}"));
                honoured.insert(key.clone());
                if update {
                    lines.push(format!("{got}  {key}"));
                    frozen += 1;
                } else if !pins.contains_key(&key) {
                    o.push_str(&format!("  FAIL      {name}: {} is not pinned\n", cut60(f, 60)));
                    bad += 1;
                } else if pins[&key] != got {
                    o.push_str(&format!("  CHANGED   {name}: {}\n            this file describes wood that was cut. If that was deliberate, re-pin with repro --update.\n", cut60(f, 60)));
                    bad += 1;
                } else {
                    frozen += 1;
                }
            }
            if !update {
                o.push_str(&format!("  frozen    {name:<22} {} as cut\n", kept.len()));
            }
        }
        let r = run.expect("a live design is always redrawn");
        let out = format!("{tmp}/{rel}/cut-files");
        if !r.status.success() || !isdir(&out) {
            let both = format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
            o.push_str(&format!("  ERROR     {name}: {}\n", cut60(both.trim().lines().last().unwrap_or("no output"), 90)));
            bad += 1;
            continue;
        }
        for line in String::from_utf8_lossy(&r.stderr).trim().lines() {
            let line = line.trim();
            if !line.is_empty() && said.insert(line.to_string()) {
                o.push_str(&format!("  note      {}\n", cut60(line, 100)));
            }
        }
        let drawn = svgs(&out);
        let (c, d): (BTreeSet<&String>, BTreeSet<&String>) = (committed.iter().collect(), drawn.iter().collect());
        for f in c.symmetric_difference(&d) {
            let where_ = if c.contains(f) { "only committed" } else { "only drawn" };
            o.push_str(&format!("  NAME      {name}: {where_} {}\n", cut60(f, 56)));
            bad += 1;
        }
        let mut n = 0;
        for f in c.intersection(&d) {
            if std::fs::read(format!("{sheets}/{f}")).ok() == std::fs::read(format!("{out}/{f}")).ok() {
                same += 1;
                n += 1;
            } else {
                o.push_str(&format!("  DIFFERS   {name}: {}\n", cut60(f, 60)));
                bad += 1;
            }
        }
        o.push_str(&format!("  ok        {name:<22} {n}/{}\n", committed.len()));
    }
    let _ = std::fs::remove_dir_all(&tmp);
    if !update {
        for key in pins.keys().filter(|k| !honoured.contains(*k)) {
            o.push_str(&format!("  GONE      {}\n            pinned as cut, and no such file is there now. If the sheet was deleted on purpose, re-pin with repro --update.\n", cut60(key, 70)));
            bad += 1;
        }
    }
    if update {
        lines.sort();
        let head = "# The cut files that describe wood which has already been cut.\n#\n# These were NOT regenerated on 2026-09-09 with the rest of the tree, when\n# the sheet was calipered at 2.94mm and the kerf at 0.13mm. They are drawn\n# at the old 3.0mm and 0.1mm because they record an instrument that exists.\n# Since 2026-11-12 they live in as-cut/, beside a cut-files/ redrawn today.\n# repro checks them against these hashes instead of redrawing them, so a\n# regenerate sweep that overwrites the record fails the gate rather than\n# passing every invariant in silence.\n#\n# Rewrite with: repro --update -- and only when you have decided to\n# overwrite the record, which is what fold2-long-straight/CLAUDE.md asks.\n";
        std::fs::write(&pins_path, format!("{head}{}", lines.iter().map(|l| format!("{l}\n")).collect::<String>())).expect("write pins");
        o.push_str(&format!("\n  pinned {frozen} as-built files\n"));
        print!("{o}");
        return;
    }
    if bad > 0 {
        o.push_str(&format!("\n  {same} reproduce, {frozen} frozen as cut, {bad} failing\n"));
    } else {
        o.push_str(&format!("\n  {same} reproduce, {frozen} frozen as cut\n"));
    }
    print!("{o}");
    std::process::exit(if bad > 0 { 1 } else { 0 });
}
