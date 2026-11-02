// Can this walk be made shorter without stranding a turn? The rule and the
// measuring are in src/minimal.rs.
//
// Removing the slack is not guaranteed safe: the rule is local, and a shorter
// walk can run into itself or start touching. Treat what this reports as
// candidates and put the result back through bore_split.py.
//
//   target/release/minimal [--terms] [walk-or-file ...]   (default: every walk)

use coilsearch::corpus::{pad_end, pad_start, walks};
use coilsearch::minimal::{report, Report};

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let verbose = argv.iter().any(|a| a == "--terms");
    let args: Vec<&String> = argv.iter().filter(|a| *a != "--terms").collect();
    let targets: Vec<(String, String)> = if args.is_empty() {
        walks()
    } else {
        args.iter()
            .map(|a| {
                let p = std::path::Path::new(a.as_str());
                if p.exists() {
                    let base = p.file_name().unwrap().to_string_lossy().into_owned();
                    let name = base.strip_suffix(".txt").filter(|s| !s.is_empty()).unwrap_or(&base).to_string();
                    (name, coilsearch::corpus::read(p).trim().to_string())
                } else {
                    ("(argument)".to_string(), a.to_string())
                }
            })
            .collect()
    };
    let results: Vec<Report> = targets.iter().map(|(n, w)| report(n, w)).collect();
    for r in &results {
        println!("{}", r.line);
    }
    println!(
        "\n{} of {} are minimal: nothing can be shortened without stranding a turn.",
        results.iter().filter(|r| r.saving == 0.0).count(),
        results.len()
    );
    let loose: Vec<&Report> = results.iter().filter(|r| r.saving > 0.0).collect();
    if !loose.is_empty() && verbose {
        println!("\nterms with slack:");
        for r in loose {
            for x in &r.slack {
                println!(
                    "  {}term {}  {}{}floor {}  -> could drop {}",
                    pad_end(&r.name, 18),
                    pad_start(&(x.i + 1).to_string(), 3),
                    pad_end(&x.term, 4),
                    pad_end(x.kind, 9),
                    x.floor,
                    coilsearch::jsfmt::num(x.slack)
                );
            }
        }
    }
}
