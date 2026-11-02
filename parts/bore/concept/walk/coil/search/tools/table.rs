// The metrics tables. Piece counts come from parts.json; rebuild it with
// target/release/parts. The tables themselves are src/table.rs.
//
//   target/release/table [--md] [--shape|--rot|--all]

use coilsearch::table::{render, Which};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let has = |f: &str| a.iter().any(|x| x == f);
    let want = if has("--shape") {
        Which::Shape
    } else if has("--rot") {
        Which::Rot
    } else {
        Which::All
    };
    print!("{}", render(has("--md"), want));
}
