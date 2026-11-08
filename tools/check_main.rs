// Copyright (C) 2026 Gernreich. GPL-3.0-or-later, like check.rs, which it runs;
// the licence text is in tools/LICENSE.GPL-3.0.txt.

//! `check "N10 U2 W2 S7 U2 E4 N9 W2 D2 N4" [--files DIR] [design switches]`:
//! check a bore's cut files before any of it goes on the machine. check.py
//! until 2026-11-08. Every check exists because something got cut and thrown
//! away; NOTES.md [check] has what each one caught.
use trumpet::bore::{walk_text, Design, DESIGN_HELP};

const USAGE: &str = "usage: check [-h] [--files DIR] walk [walk ...]";

fn main() {
    let mut d = Design::default();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let rest = d.take_design_switches(&argv);
    let (mut walk, mut files) = (vec![], None);
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a == "-h" || a == "--help" {
            println!("{USAGE}\n\nCheck a bore's cut files before any of it goes on the machine.\n\npositional arguments:\n  walk\n\noptions:\n  -h, --help   show this help message and exit\n  --files DIR  also check the sheets written into DIR\n\n{DESIGN_HELP}");
            return;
        } else if a == "--files" {
            let Some(v) = rest.get(i + 1) else {
                eprintln!("{USAGE}\ncheck: error: argument --files: expected one argument");
                std::process::exit(2);
            };
            files = Some(v.clone());
            i += 2;
            continue;
        } else if let Some(v) = a.strip_prefix("--files=") {
            files = Some(v.to_string());
        } else if a.starts_with('-') && a.len() > 1 && !a[1..].starts_with(|c: char| c.is_ascii_digit()) {
            eprintln!("{USAGE}\ncheck: error: unrecognized arguments: {a}");
            std::process::exit(2);
        } else {
            walk.push(a.clone());
        }
        i += 1;
    }
    if walk.is_empty() {
        eprintln!("{USAGE}\ncheck: error: the following arguments are required: walk");
        std::process::exit(2);
    }
    let mut o = String::new();
    let r = walk_text(&walk.join(" ")).and_then(|t| trumpet::check::report(&d, &t, files.as_deref(), &mut o));
    match r {
        Ok(code) => {
            print!("{o}");
            std::process::exit(code);
        }
        Err(e) => {
            print!("{o}");
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
