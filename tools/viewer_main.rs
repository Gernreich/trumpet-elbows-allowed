//! `viewer WALK [--out FILE] [--title TEXT]`: write a bore as a page you can
//! turn around with the mouse. viewer.py until 2026-11-08. Writes bore.html
//! in the working directory unless --out names a file.
use trumpet::bore::{walk_text, Design};

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (mut out, mut title) = ("bore.html".to_string(), "Bore Viewer".to_string());
    let mut words = vec![];
    let mut i = 0;
    while i < argv.len() {
        if argv[i] == "--out" || argv[i] == "--title" {
            let Some(v) = argv.get(i + 1) else {
                eprintln!("IndexError: list index out of range");
                std::process::exit(1);
            };
            if argv[i] == "--out" { out = v.clone() } else { title = v.clone() }
            i += 2;
        } else {
            words.push(argv[i].clone());
            i += 1;
        }
    }
    let d = Design::default();
    match walk_text(&words.join(" ")).and_then(|t| trumpet::viewer::build(&d, &t, &title)) {
        Ok(html) => {
            std::fs::write(&out, &html).unwrap_or_else(|e| panic!("{out}: {e}"));
            println!("  wrote {out}  ({:.0} kB, no external assets but the fonts)", html.chars().count() as f64 / 1024.0);
        }
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
