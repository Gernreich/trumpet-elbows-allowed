//! Running bore_split.py, which decides how a walk splits into pieces. It needs the
//! Boxes.py venv's python.

use crate::corpus::gen;
use std::process::{Command, Stdio};

const CANDIDATES: [&str; 2] = ["~/Software/boxes", "~/boxes"]; // as bore_split.py searches

/// The venv interpreter. BORE_PY is checked too: taken on trust, a wrong one fails
/// later and deeper, as a spawn error against a walk rather than a bad setting.
pub fn py() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let found = std::env::var("BORE_PY").ok().filter(|s| !s.is_empty()).or_else(|| {
        CANDIDATES
            .iter()
            .map(|c| format!("{}/venv/bin/python", c.replacen('~', &home, 1)))
            .find(|p| std::path::Path::new(p).exists())
    });
    match found {
        Some(p) if std::path::Path::new(&p).exists() => p,
        Some(p) => {
            eprintln!("BORE_PY is set to {p}, which does not exist.");
            std::process::exit(1)
        }
        None => {
            eprintln!(
                "no Boxes.py venv found. Looked in {}.\n  Set BORE_PY=/path/to/venv/bin/python.",
                CANDIDATES.join(", ")
            );
            std::process::exit(1)
        }
    }
}

/// `bore_split.py --no-write WALK`, run in the bore toolchain. Ok(stdout) when it
/// succeeds; Err((stderr, stdout)) when it refuses. `show_stderr` passes its
/// warnings through to ours, as Node's execSync did when not told otherwise.
pub fn run_split(walk: &str, show_stderr: bool) -> Result<String, (String, String)> {
    let out = Command::new(py())
        .args(["bore_split.py", "--no-write", walk])
        .current_dir(gen())
        .stdin(Stdio::null())
        .stderr(if show_stderr { Stdio::inherit() } else { Stdio::piped() })
        .output()
        .unwrap_or_else(|e| {
            eprintln!("cannot run bore_split.py: {e}");
            std::process::exit(1)
        });
    let so = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(so)
    } else {
        Err((String::from_utf8_lossy(&out.stderr).into_owned(), so))
    }
}

#[derive(Clone, Debug)]
pub struct Split {
    pub ok: bool,
    pub elbows: i64,
    pub pieces: f64,
    pub blocks: f64,
    pub oversize: bool,
    pub err: String,
}

/// The first capture of `re` as a number, NaN when absent (JavaScript's `+undefined`).
pub fn cap_num(re: &regex::Regex, text: &str) -> f64 {
    re.captures(text).and_then(|c| c[1].parse().ok()).unwrap_or(f64::NAN)
}

/// Piece kinds counted off the assembly list, in the order first seen.
pub fn kinds(out: &str) -> Vec<(String, i64)> {
    let re = regex::Regex::new(r"^\s+\d+\s+\d+-\d+\s+(\w+)\s").unwrap();
    let mut k: Vec<(String, i64)> = Vec::new();
    for l in out.split('\n') {
        if let Some(c) = re.captures(l) {
            match k.iter_mut().find(|(n, _)| n == &c[1]) {
                Some(e) => e.1 += 1,
                None => k.push((c[1].to_string(), 1)),
            }
        }
    }
    k
}

pub fn split(walk: &str) -> Split {
    match run_split(walk, false) {
        Ok(out) => {
            let k = kinds(&out);
            let elbows = k.iter().find(|(n, _)| n == "elbow").map(|e| e.1).unwrap_or(0);
            Split {
                ok: true,
                elbows,
                pieces: cap_num(&regex::Regex::new(r"(\d+) pieces to assemble").unwrap(), &out),
                blocks: cap_num(&regex::Regex::new(r"(\d+) blocks").unwrap(), &out),
                oversize: regex::Regex::new(r"(?i)does not fit|too big").unwrap().is_match(&out),
                err: String::new(),
            }
        }
        Err((se, so)) => {
            let text = if se.is_empty() { so } else { se };
            let first: String = text.split('\n').next().unwrap_or("").chars().take(70).collect();
            Split { ok: false, elbows: 0, pieces: f64::NAN, blocks: f64::NAN, oversize: false, err: first }
        }
    }
}

impl Split {
    /// As reduce.js wrote it into reduced.json: { ok, elbows, pieces, blocks, oversize }.
    pub fn json(&self) -> crate::json::J {
        use crate::json::J;
        J::obj(vec![
            ("ok", J::Bool(self.ok)),
            ("elbows", J::Num(self.elbows as f64)),
            ("pieces", J::Num(self.pieces)),
            ("blocks", J::Num(self.blocks)),
            ("oversize", J::Bool(self.oversize)),
        ])
    }
}
