//! What the bell and mouthpiece generators share: their --name=value options
//! read as Python's dict() read them, sys.exit(), float() and int() of an
//! option, where a generated sheet goes, and the ring numbering every sheet
//! generator runs as its last step.
#![allow(dead_code)]
use lasermade_tools::py;
use lasermade_tools::text;

/// sys.exit("message"): the message on stderr, exit status 1
pub fn exit(msg: impl std::fmt::Display) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

/// dict(a[2:].split("=", 1) for every --name=value): a repeated name keeps
/// its first place and takes its last value
pub struct Opts(pub Vec<(String, String)>);

impl Opts {
    pub fn of(argv: &[String]) -> Opts {
        let mut v: Vec<(String, String)> = Vec::new();
        for a in argv {
            if let Some(r) = a.strip_prefix("--") {
                if let Some((k, x)) = r.split_once('=') {
                    match v.iter_mut().find(|(n, _)| n == k) {
                        Some(e) => e.1 = x.to_string(),
                        None => v.push((k.to_string(), x.to_string())),
                    }
                }
            }
        }
        Opts(v)
    }
    pub fn get(&self, k: &str) -> Option<&str> {
        self.0.iter().find(|(n, _)| n == k).map(|(_, x)| x.as_str())
    }
    pub fn has(&self, k: &str) -> bool {
        self.get(k).is_some()
    }
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }
    /// float(opts.get(k, default))
    pub fn float(&self, k: &str, default: f64) -> f64 {
        match self.get(k) {
            None => default,
            Some(v) => float(v),
        }
    }
    /// int(opts.get(k, default))
    pub fn int(&self, k: &str, default: i64) -> i64 {
        match self.get(k) {
            None => default,
            Some(v) => int(v),
        }
    }
    pub fn str<'a>(&'a self, k: &str, default: &'a str) -> &'a str {
        self.get(k).unwrap_or(default)
    }
}

/// float(s), or the ValueError Python would have died of
pub fn float(s: &str) -> f64 {
    py::float(s).unwrap_or_else(|| exit(format!("ValueError: could not convert string to float: {}", py::str_repr(s))))
}

/// int(s), or the ValueError Python would have died of
pub fn int(s: &str) -> i64 {
    let t = s.trim().replace('_', "");
    t.parse::<i64>().unwrap_or_else(|_| exit(format!("ValueError: invalid literal for int() with base 10: {}", py::str_repr(s))))
}

/// The arguments, without the program
pub fn argv() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// The first --switch given with no "=", if any
pub fn bare(argv: &[String]) -> Option<&String> {
    argv.iter().find(|a| a.starts_with("--") && !a.contains('='))
}

/// The arguments that are not --options
pub fn positional(argv: &[String]) -> Vec<String> {
    argv.iter().filter(|a| !a.starts_with("--")).cloned().collect()
}

/// This repository's root, symlinks resolved, as Path(__file__).resolve() sees it
pub fn repo() -> String {
    let m = env!("CARGO_MANIFEST_DIR");
    std::fs::canonicalize(m).map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| m.to_string())
}

/// A generated sheet goes in cut-files/ beside the generator (`sub` is its
/// folder under parts/, "bell" or "mouthpiece"), reported relative to where
/// the caller stands.
pub fn in_cut_files(sub: &str, name: &str) -> String {
    let d = format!("{}/parts/{sub}/cut-files", repo());
    let _ = std::fs::create_dir(&d);
    let cwd = std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    text::relpath(&format!("{d}/{name}"), &cwd)
}

/// Run number_rings on a written sheet, --order=document: Ok on success, or
/// Err(stdout + stderr) of the refusal.
pub fn number_rings(path: &str) -> Result<(), String> {
    number_rings_with(path, &[])
}

/// number_rings with more options after --order=document
pub fn number_rings_with(path: &str, more: &[&str]) -> Result<(), String> {
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("number_rings")));
    let exe = exe.unwrap_or_else(|| exit("cannot find number_rings beside this program"));
    let r = std::process::Command::new(&exe)
        .args([path, "--order=document"])
        .args(more)
        .output()
        .unwrap_or_else(|e| exit(format!("{}: {e}", exe.display())));
    if r.status.success() {
        Ok(())
    } else {
        Err(format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr)))
    }
}

/// max() and min() of a run of floats: the first unless a later one is
/// strictly beyond it, as Python's do
pub fn fmax(v: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = v.into_iter();
    let f = it.next().unwrap_or_else(|| exit("ValueError: max() arg is an empty sequence"));
    it.fold(f, |a, b| if b > a { b } else { a })
}
pub fn fmin(v: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = v.into_iter();
    let f = it.next().unwrap_or_else(|| exit("ValueError: min() arg is an empty sequence"));
    it.fold(f, |a, b| if b < a { b } else { a })
}

/// f'{x:.Nf}'
pub fn f(x: f64, n: usize) -> String {
    format!("{:.*}", n, x)
}

/// Path(p).name
pub fn name_of(p: &str) -> String {
    let s = text::pathstr(p);
    s.rsplit('/').next().unwrap_or("").to_string()
}

/// Path(p).write_text(s), or the error Python would have died of
pub fn write(p: &str, s: &str) {
    std::fs::write(p, s).unwrap_or_else(|e| exit(format!("{p}: {e}")));
}

/// --help prints the docstring and draws nothing
pub fn help(argv: &[String], doc: &str) {
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("{doc}");
        std::process::exit(0);
    }
}

/// How far a rounded square of half-width h and corner radius c reaches in a
/// direction, s = |cos| + |sin| of it: 1 through a flat, root2 through a corner.
pub fn support(h: f64, c: f64, s: f64) -> f64 {
    (h - c) * s + c
}

/// A square of half-width h with its corners rounded to radius c: c = 0 is a
/// sharp square in H and V, c = h a true circle in four arcs.
pub fn rounded(cx: f64, cy: f64, h: f64, c: f64) -> String {
    let c = py::min(py::max(c, 0.0), h);
    if c <= 1e-9 {
        return format!("M {:.4},{:.4} H {:.4} V {:.4} H {:.4} Z", cx - h, cy - h, cx + h, cy + h, cx - h);
    }
    let s = h - c;
    let arc = |x: f64, y: f64| format!("A {c:.4},{c:.4} 0 0 1 {x:.4},{y:.4}");
    let run = |cmd: &str, v: f64| if s > 1e-9 { format!("{cmd} {v:.4} ") } else { String::new() };
    format!(
        "M {:.4},{:.4} {}{} {}{} {}{} {}{} Z",
        cx - s,
        cy - h,
        run("H", cx + s),
        arc(cx + h, cy - s),
        run("V", cy + s),
        arc(cx + s, cy + h),
        run("H", cx - s),
        arc(cx - h, cy + s),
        run("V", cy - s),
        arc(cx - s, cy - h)
    )
}

/// A circle of diameter d as four arcs, as mouthpiece.rs and mouthpiece-cup.rs draw it
pub fn circle(cx: f64, cy: f64, d: f64) -> String {
    let r = d / 2.0;
    format!(
        "M {:.4},{cy:.4} A {r:.4},{r:.4} 0 0 1 {cx:.4},{:.4} A {r:.4},{r:.4} 0 0 1 {:.4},{cy:.4} A {r:.4},{r:.4} 0 0 1 {cx:.4},{:.4} A {r:.4},{r:.4} 0 0 1 {:.4},{cy:.4} Z",
        cx + r,
        cy + r,
        cx - r,
        cy - r,
        cx + r
    )
}

/// The refusal every sheet generator gives when its rings cannot be numbered,
/// after removing the sheet: a sheet on disk gets cut.
pub fn unnumbered(path: &str, out: &str) -> ! {
    let _ = std::fs::remove_file(path);
    exit(format!("  {path} NOT written: the rings could not be numbered.\n{out}  Fix that, or pass --numbers=no if you meant a bare sheet."))
}
