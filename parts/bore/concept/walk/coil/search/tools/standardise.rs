// Put every coil in the same orientation, so two coils differ only where they
// really differ.
//
//   forward   north for all of them, and the walk opens on a north term
//   rotation  counter-clockwise seen looking along the bore from the mouth
//   length    whole periods only, the count that lands nearest a common target
//   ends      one block in, one block out, and no partial period between them
//
// A bore opens at both ends, so the first and last piece cannot be got rid of:
// they are bounded by the mouth and the exit rather than by a neighbour. What
// can be done is to make them the same on every walk, which is what this does,
// and the judged metrics ignore them anyway. The rules are src/standardise.rs.
//
//   target/release/standardise [--write]     (--write: standardised.json)

use coilsearch::bore::split;
use coilsearch::corpus::{pad_end, pad_start, root};
use coilsearch::jsfmt::{fixed, num};
use coilsearch::json::J;
use coilsearch::standardise::{build_walk, lengths_for, orient, sources, target, text};

fn main() {
    let write = std::env::args().any(|a| a == "--write");
    let src = sources();
    let (tgt, spread, periods) = target(&src);
    let span = lengths_for(&periods, tgt);
    let lo = span.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = span.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!(
        "target {} blocks, chosen to minimise the spread: lengths {}-{}, {}%\n",
        num(tgt),
        num(lo),
        num(hi),
        fixed(spread * 100.0, 1)
    );

    let mut out: Vec<(String, String)> = Vec::new(); // (name, walk)
    for s in &src {
        let mut per = s.per.clone();
        if per.is_none() {
            // too short to show its own period; borrow a twin's
            per = src
                .iter()
                .find(|k| k.name != s.name && k.per.as_ref().is_some_and(|p| s.walk.contains(&text(p))))
                .and_then(|k| k.per.clone());
            if per.is_none() {
                println!("{}no period; skipped", pad_end(&s.name, 20));
                continue;
            }
        }
        let per = per.unwrap();
        if orient(&per).is_none() {
            println!("{}could not orient", pad_end(&s.name, 20));
            continue;
        }
        let built = build_walk(&per, tgt).unwrap();
        let (pb, k) = (built.period_blocks, built.k);
        let r = split(&built.walk);
        println!(
            "{}period {} x{} = {}  -> {} blocks, {}",
            pad_end(&s.name, 20),
            pad_start(&num(pb), 2),
            num(k),
            pad_start(&num(pb * k), 3),
            pad_start(&if r.ok { num(r.blocks) } else { "?".into() }, 3),
            if r.ok {
                format!("{} pieces, elbows {}", num(r.pieces), r.elbows)
            } else {
                format!("REFUSED {}", r.err)
            }
        );
        if r.ok && r.elbows == 0 && !r.oversize {
            out.push((s.name.clone(), built.walk));
        }
    }
    // standardising can reveal that two walks were the same coil all along
    let mut by_walk: Vec<(String, Vec<String>)> = Vec::new();
    for (name, walk) in &out {
        match by_walk.iter_mut().find(|(w, _)| w == walk) {
            Some(e) => e.1.push(name.clone()),
            None => by_walk.push((walk.clone(), vec![name.clone()])),
        }
    }
    println!("\n{} standardised, {} distinct", out.len(), by_walk.len());
    for (_, names) in &by_walk {
        if names.len() > 1 {
            println!("  same coil: {}", names.join(", "));
        }
    }
    if write {
        let j = J::Arr(
            by_walk
                .iter()
                .map(|(w, n)| J::obj(vec![("walk", J::s(w)), ("names", J::Arr(n.iter().map(|x| J::s(x)).collect()))]))
                .collect(),
        );
        std::fs::write(root().join("standardised.json"), j.stringify(1)).expect("cannot write standardised.json");
    }
}
