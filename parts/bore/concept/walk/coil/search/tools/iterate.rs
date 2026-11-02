// Iterated ranking -- rank, cut the bottom half, re-rank the survivors, repeat --
// to ask whether the survivors keep their order once the losers leave. The
// reasoning and the arithmetic are in src/iterate.rs.
//
//   target/release/iterate

use coilsearch::corpus::pad_start;
use coilsearch::iterate::results;
use coilsearch::score::rows;

fn main() {
    let rows = rows();
    for r in results(&rows) {
        println!("=== {} ===", r.label);
        for l in &r.log {
            println!(
                "  round {}: {} -> {}   survivors reordered: {}/{}",
                l.round,
                pad_start(&l.from.to_string(), 2),
                pad_start(&l.to.to_string(), 2),
                l.moved,
                l.to
            );
        }
        println!(
            "  winner after elimination: {}   |   ranking all {} once: {}   {}",
            r.winner,
            rows.len(),
            r.ranked_first,
            if r.winner == r.ranked_first { "(same)" } else { "(DIFFERENT)" }
        );
        println!();
    }
}
