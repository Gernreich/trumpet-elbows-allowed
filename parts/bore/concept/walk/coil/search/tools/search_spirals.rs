// Tightest BEND-ONLY spiral. Corpus rule, stated in bore-designs/README:
// consecutive terms are always on different axes (a reversal is refused), so a
// window of three terms names three axes exactly when the outer two differ ---
// and then the middle term must be >= 3. If the outer two share an axis it is a
// fold, and a fold can be as tight as you like.
//
//   target/release/search_spirals        (writes cands.json where it is run)

use coilsearch::corpus::pad_start;
use coilsearch::jsfmt::{fixed, num};
use coilsearch::json::J;
use coilsearch::minimal::axis;
use coilsearch::spiral_metrics::v;
use std::collections::HashSet;

const DIRS: [u8; 6] = *b"NSEWUD";
const TARGET: usize = 196;
const MAXK: usize = 8;
const MAXL: i64 = 4;

type Terms = Vec<(u8, i64)>;

fn cells(terms: &Terms, reps: usize) -> Vec<[i64; 3]> {
    let mut p = [0i64; 3];
    let mut out = vec![p];
    for _ in 0..reps {
        for &(d, l) in terms {
            let w = v(d);
            for _ in 0..l {
                p = [p[0] + w[0], p[1] + w[1], p[2] + w[2]];
                out.push(p);
            }
        }
    }
    out
}

/// the >=3 rule, applied cyclically
fn ok3(terms: &Terms) -> bool {
    let k = terms.len();
    (0..k).all(|i| {
        let (a, b, c) = (terms[(i + k - 1) % k], terms[i], terms[(i + 1) % k]);
        let need = if axis(a.0) != axis(c.0) {
            3 // coil: leaves the plane
        } else if a.0 == c.0 {
            1 // step: free
        } else {
            2 // hairpin: doubles back
        };
        b.1 >= need
    })
}

fn quarter(terms: &Terms, ax: usize) -> i64 {
    let lat: Vec<usize> = (0..3).filter(|&i| i != ax).collect();
    let proj: Vec<[i64; 2]> =
        terms.iter().map(|t| v(t.0)).filter(|w| w[lat[0]] != 0 || w[lat[1]] != 0).map(|w| [w[lat[0]], w[lat[1]]]).collect();
    let mut q = 0;
    for i in 0..proj.len() {
        let (u, w) = (proj[i], proj[(i + 1) % proj.len()]);
        q += u[0] * w[1] - u[1] * w[0];
    }
    q
}

struct Found {
    terms: String,
    per: i64,
    rise: i64,
    turns: i64,
    size: String,
    vol: i64,
    dens: f64,
}

fn dfs(terms: &mut Terms, pos: [i64; 3], occ: &mut HashSet<[i64; 3]>, res: &mut Vec<Found>) {
    let k = terms.len();
    if k >= 4 && ok3(terms) {
        for ax in 0..3 {
            let lat: Vec<usize> = (0..3).filter(|&i| i != ax).collect();
            if pos[lat[0]] == 0 && pos[lat[1]] == 0 && pos[ax] != 0 {
                let q = quarter(terms, ax);
                if q != 0 && q % 4 == 0 {
                    let per: i64 = terms.iter().map(|t| t.1).sum();
                    let reps = 2.max((TARGET as f64 / per as f64).ceil() as usize);
                    let c = cells(terms, reps);
                    let mut seen = HashSet::new();
                    if c.iter().all(|q2| seen.insert(*q2)) {
                        let used = &c[..TARGET.min(c.len())];
                        let mut lo = [1_000_000_000i64; 3];
                        let mut hi = [-1_000_000_000i64; 3];
                        for q2 in used {
                            for i in 0..3 {
                                lo[i] = lo[i].min(q2[i]);
                                hi[i] = hi[i].max(q2[i]);
                            }
                        }
                        let mut sz = [hi[0] - lo[0] + 1, hi[1] - lo[1] + 1, hi[2] - lo[2] + 1];
                        let vol = sz[0] * sz[1] * sz[2];
                        sz.sort();
                        res.push(Found {
                            terms: terms.iter().map(|t| format!("{}{}", t.0 as char, t.1)).collect::<Vec<_>>().join(" "),
                            per,
                            rise: pos[ax].abs(),
                            turns: q / 4,
                            size: sz.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("x"),
                            vol,
                            dens: TARGET as f64 / vol as f64,
                        });
                    }
                }
            }
        }
    }
    if k == MAXK {
        return;
    }
    let dirs: &[u8] = if k == 0 { b"N" } else { &DIRS };
    for &d in dirs {
        if k > 0 && axis(d) == axis(terms[k - 1].0) {
            continue; // no repeat, no reversal
        }
        for l in 1..=MAXL {
            let w = v(d);
            let mut p = pos;
            let mut added = Vec::new();
            let mut clash = false;
            for _ in 0..l {
                p = [p[0] + w[0], p[1] + w[1], p[2] + w[2]];
                if !occ.insert(p) {
                    clash = true;
                    break;
                }
                added.push(p);
            }
            if !clash {
                terms.push((d, l));
                dfs(terms, p, occ, res);
                terms.pop();
            }
            for key in added {
                occ.remove(&key);
            }
        }
    }
}

fn main() {
    let mut res = Vec::new();
    let mut occ = HashSet::from([[0i64, 0, 0]]);
    dfs(&mut Vec::new(), [0, 0, 0], &mut occ, &mut res);
    let mut seen = HashSet::new();
    let mut list: Vec<&Found> = res.iter().filter(|r| seen.insert(format!("{}|{}|{}", r.size, r.vol, r.per))).collect();
    list.sort_by_key(|r| r.vol);
    let j = J::Arr(
        list.iter()
            .map(|r| {
                J::obj(vec![
                    ("terms", J::s(&r.terms)),
                    ("per", J::Num(r.per as f64)),
                    ("rise", J::Num(r.rise as f64)),
                    ("turns", J::Num(r.turns as f64)),
                    ("size", J::s(&r.size)),
                    ("vol", J::Num(r.vol as f64)),
                    ("dens", J::Num(r.dens)),
                ])
            })
            .collect(),
    );
    std::fs::write("cands.json", j.stringify(0)).expect("cannot write cands.json");
    println!("bend-only helices found: {}  distinct: {}", res.len(), list.len());
    println!("\nenvelope   vol   fill   per  rise  turns   terms");
    for r in list.iter().take(12) {
        println!(
            "{} {} {}% {} {} {}    {}",
            pad_start(&r.size, 9),
            pad_start(&r.vol.to_string(), 5),
            pad_start(&fixed(r.dens * 100.0, 0), 5),
            pad_start(&r.per.to_string(), 5),
            pad_start(&r.rise.to_string(), 5),
            pad_start(&num(r.turns as f64), 6),
            r.terms
        );
    }
}
