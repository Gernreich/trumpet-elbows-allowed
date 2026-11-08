//! Where things are, and reading the corpus: the walks, parts.json, unscored.txt.

use crate::json::J;
use std::collections::HashSet;
use std::path::PathBuf;

/// The search directory. The programs live in its target/release/, so it is found
/// from the running program rather than from wherever it was started.
pub fn root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(r) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            if r.join("walks").is_dir() {
                return r.to_path_buf();
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The bore toolchain (tools/, and bore_split beside it in target/release), six levels up since the 2026-09-05 restructure.
pub fn gen() -> PathBuf {
    let mut p = root();
    for _ in 0..6 {
        p.push("..");
    }
    p.push("tools");
    p
}

pub fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", path.display());
        std::process::exit(1)
    })
}

/// Names in a directory, sorted by byte as Node's readdirSync returns them.
pub fn list_dir(dir: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| {
            eprintln!("cannot list {}: {e}", dir.display());
            std::process::exit(1)
        })
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// Every walk in walks/, as (name, walk), in file order.
pub fn walks() -> Vec<(String, String)> {
    let dir = root().join("walks");
    list_dir(&dir)
        .into_iter()
        .filter(|f| f.ends_with(".txt"))
        .map(|f| {
            let walk = read(&dir.join(&f)).trim().to_string();
            (f.trim_end_matches(".txt").to_string(), walk)
        })
        .collect()
}

pub fn parts() -> J {
    let p = root().join("parts.json");
    J::parse(&read(&p)).unwrap_or_else(|e| {
        eprintln!("parts.json: {e}");
        std::process::exit(1)
    })
}

/// Walks listed in unscored.txt, comments stripped.
pub fn unscored() -> HashSet<String> {
    let p = root().join("unscored.txt");
    let text = std::fs::read_to_string(p).unwrap_or_default();
    text.split('\n')
        .map(|l| l.split('#').next().unwrap().trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// JavaScript's `s.padEnd(n)` / `padStart(n)` for ASCII text.
pub fn pad_end(s: &str, n: usize) -> String {
    format!("{s:<n$}")
}
pub fn pad_start(s: &str, n: usize) -> String {
    format!("{s:>n$}")
}
