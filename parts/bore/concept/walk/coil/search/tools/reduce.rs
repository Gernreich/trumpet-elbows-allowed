// The reduction pass: shorten every leg that can be shortened, without losing
// the coil.
//
// tools/minimal.rs reports slack against a purely local rule -- a term's floor
// in its own window of three. Taking all of it at once usually destroys the
// design, because coiling is global: the period has to close laterally, drifting
// on one axis and returning to where it started on the other two. Shorten one
// leg and not its opposite and the period stops closing, so the walk wanders
// diagonally instead of coiling and its envelope explodes. coil_3x3_47 reduced
// naively goes from a box of 423 to 10452.
//
// So this enumerates the reductions rather than applying them, keeps only those
// that still close and still wind a whole turn, takes the shortest, rebuilds at
// comparable length and puts the result back through bore_split.py. What it emits
// is standardised like everything else, so a reduction can be compared with the
// coil it came from without any other difference in the way. The pass itself is
// src/reduce.rs.
//
//   target/release/reduce [--write]      (--write: reduced.json)

use coilsearch::corpus::{pad_end, pad_start, root};
use coilsearch::jsfmt::{fixed, num};
use coilsearch::json::J;
use coilsearch::reduce::results;

fn main() {
    let write = std::env::args().any(|a| a == "--write");
    let res = results();
    for r in &res {
        let Some(x) = &r.r else {
            println!("{}{}", pad_end(&r.name, 16), r.status);
            continue;
        };
        let (b, a) = (&x.before, &x.after);
        println!(
            "{}box {} -> {}   blocks {} -> {}   touching {} -> {}   blk/360 {} -> {}   {} pieces   -> {}",
            pad_end(&r.name, 16),
            pad_start(&b.vol.to_string(), 4),
            pad_start(&a.vol.to_string(), 4),
            pad_start(&b.blocks.to_string(), 3),
            pad_start(&a.blocks.to_string(), 3),
            pad_start(&b.touching.to_string(), 2),
            pad_start(&a.touching.to_string(), 2),
            fixed(b.blocks_per_360_or0(), 1),
            fixed(a.blocks_per_360_or0(), 1),
            num(x.s.pieces),
            match &x.lands {
                Some(l) if *l == r.name => "itself",
                Some(l) => l.as_str(),
                None => "NOT IN walks/",
            }
        );
    }
    let won: Vec<_> = res.iter().filter(|r| r.r.is_some()).collect();
    let stray: Vec<&str> = won.iter().filter(|r| r.r.as_ref().unwrap().lands.is_none()).map(|r| r.r.as_ref().unwrap().red.as_str()).collect();
    println!(
        "\n{} of {} reduced while staying a coil; {} came out smaller.",
        won.len(),
        res.len(),
        won.iter().filter(|r| { let x = r.r.as_ref().unwrap(); x.after.vol < x.before.vol }).count()
    );
    // Said out loud, because "the set is closed under reduction" was a claim on the
    // page and nothing here was in a position to contradict it.
    let mut distinct: Vec<&str> = Vec::new();
    for p in &stray {
        if !distinct.contains(p) {
            distinct.push(p);
        }
    }
    if stray.is_empty() {
        println!("every reduction lands on a coil already in walks/.");
    } else {
        println!(
            "{} land on {} period(s) that walks/ does not have: {}",
            stray.len(),
            distinct.len(),
            distinct.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")
        );
    }
    if write {
        let j = J::Arr(won.iter().map(|r| r.json()).collect());
        std::fs::write(root().join("reduced.json"), j.stringify(1)).expect("cannot write reduced.json");
    }
}
