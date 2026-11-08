//! `viewer WALK [--out FILE] [--title TEXT]`: write a bore as a page you can
//! turn around with the mouse. viewer.py until 2026-11-08. Writes bore.html
//! in the working directory unless --out names a file.
//!
//! `viewer WALK --solidcraft FILE [--volumes] [design switches]` writes the
//! bore as a parametric SolidCraft design instead (solidcraft_design.rs), the
//! switches (--straight=, --bore= and the rest) as bore_split takes them;
//! --volumes prints each piece's volume the design should build to.
use trumpet::bore::{walk_text, Design};

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (mut out, mut title) = ("bore.html".to_string(), "Bore Viewer".to_string());
    let mut sc: Option<String> = None;
    let mut words = vec![];
    let mut i = 0;
    while i < argv.len() {
        if argv[i] == "--solidcraft" {
            let Some(v) = argv.get(i + 1) else {
                eprintln!("--solidcraft needs a file");
                std::process::exit(1);
            };
            sc = Some(v.clone());
            i += 2;
        } else if argv[i] == "--out" || argv[i] == "--title" {
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
    let mut d = Design::default();
    if let Some(f) = sc {
        let vols = words.iter().any(|w| w == "--volumes");
        words.retain(|w| w != "--volumes");
        let rest = d.take_design_switches(&words);
        // a page: its own blocks must be the ones these switches give
        if let Some(pg) = rest.iter().find(|w| w.to_lowercase().ends_with(".html")) {
            let body = std::fs::read_to_string(pg).unwrap_or_else(|e| panic!("{pg}: {e}"));
            let r = lasermade_tools::re::rx(r#"\{"a":\[([-0-9.,e]+)\],"b":\[([-0-9.,e]+)\]"#);
            let nums = |t: &str| -> Vec<f64> { t.split(',').map(|x| x.parse().unwrap()).collect() };
            let page: Vec<(Vec<f64>, Vec<f64>)> = r.captures_iter(&body).map(|c| c.unwrap()).map(|c| (nums(c.get(1).unwrap().as_str()), nums(c.get(2).unwrap().as_str()))).collect();
            let ok = walk_text(pg).and_then(|t| d.specs_for(&t)).map(|sp| {
                let bx = d.block_boxes(&sp.rec);
                bx.len() == page.len() && bx.iter().zip(&page).all(|((lo, hi), (a, b))| (0..3).all(|i| (lo[i] - a[i]).abs() < 1e-6 && (hi[i] - b[i]).abs() < 1e-6))
            });
            if ok != Ok(true) {
                println!("error: {pg} draws other blocks than these switches give ({} on the page)", page.len());
                std::process::exit(1);
            }
        }
        let got = walk_text(&rest.join(" ")).and_then(|t| {
            if vols {
                for v in trumpet::solidcraft_design::volumes(&d, &t)? {
                    println!("{v:.3}");
                }
            }
            trumpet::solidcraft_design::script(&d, &t)
        });
        match got {
            Ok(js) => {
                std::fs::write(&f, js).unwrap_or_else(|e| panic!("{f}: {e}"));
                if !vols {
                    println!("  wrote {}", f.rsplit('/').next().unwrap_or(&f));
                }
            }
            Err(e) => {
                println!("error: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
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
