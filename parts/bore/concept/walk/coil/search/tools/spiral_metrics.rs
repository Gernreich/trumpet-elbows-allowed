// Rotation metrics for one walk, as JSON. The measuring itself is
// src/spiral_metrics.rs, which the other tools share.
//
//   target/release/spiral_metrics "<walk>"

use coilsearch::spiral_metrics::metrics;

fn main() {
    // Run bare, this used to die inside the measuring rather than say what it wanted.
    let Some(walk) = std::env::args().nth(1).filter(|w| !w.is_empty()) else {
        eprintln!("usage: target/release/spiral_metrics \"<walk>\"");
        eprintln!("  Prints the metrics for one walk. The other tools share the code.");
        std::process::exit(2);
    };
    match metrics(&walk, None) {
        Ok(m) => println!("{}", m.json().stringify(2)),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
