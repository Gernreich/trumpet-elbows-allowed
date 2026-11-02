// Composite scoring across the common means: the metrics normalised to (0,1], plus
// the touching count at an explicit weight (SPIRAL_TOUCH_WEIGHT, default 5). The
// scoring is src/score.rs.
//
//   target/release/score [--md] [--clean] [--solid]
//
// --clean ranks only the coils with no touching walls at all, for when "none" is a
// requirement rather than a preference. --solid keeps only coils at least 3 blocks
// thick in every direction: a coil two blocks thick is a ribbon rather than a rod.
// Both filter the field; the normalization is computed over the whole set either
// way -- see tools/iterate.rs for why that distinction matters.

use coilsearch::score::{score, table};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let has = |f: &str| a.iter().any(|x| x == f);
    let sc = score(has("--clean"), has("--solid"));
    print!("{}", table(&sc, has("--md")));
}
