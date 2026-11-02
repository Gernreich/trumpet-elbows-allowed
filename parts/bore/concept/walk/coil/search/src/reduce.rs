//! The reduction pass: shorten every leg that can be shortened, without losing the
//! coil. See tools/reduce.rs for why it enumerates rather than applies.

use crate::bore::{split, Split};
use crate::json::J;
use crate::minimal::axis;
use crate::spiral_metrics::{metrics, v, Metrics};
use crate::standardise::{build_walk, parse, period_of, sources, target, text, Term};

fn floor_of(t: &[Term], i: usize) -> f64 {
    let k = t.len();
    let (a, c) = (t[(i + k - 1) % k], t[(i + 1) % k]);
    if axis(a.d) != axis(c.d) {
        3.0
    } else if a.d == c.d {
        1.0
    } else {
        2.0
    }
}

fn net(t: &[Term]) -> [f64; 3] {
    crate::standardise::net_of(t)
}

/// A whole number of revolutions per period, not necessarily one: the staircase
/// coil's period turns three times, and demanding exactly one rejected it.
fn winds_whole(t: &[Term], ax: usize) -> bool {
    let lat: Vec<usize> = (0..3).filter(|&i| i != ax).collect();
    let proj: Vec<[i64; 2]> =
        t.iter().map(|m| v(m.d)).filter(|w| w[lat[0]] != 0 || w[lat[1]] != 0).map(|w| [w[lat[0]], w[lat[1]]]).collect();
    let mut q = 0i64;
    for i in 0..proj.len() {
        let (u, w) = (proj[i], proj[(i + 1) % proj.len()]);
        q += u[0] * w[1] - u[1] * w[0];
    }
    q != 0 && q.abs() % 4 == 0
}

/// still a coil: travels on exactly one axis, and turns a whole revolution about it
fn is_coil(t: &[Term]) -> bool {
    let p = net(t);
    let nz: Vec<usize> = (0..3).filter(|&i| p[i] != 0.0).collect();
    nz.len() == 1 && winds_whole(t, nz[0])
}

pub struct Reduced {
    pub red: String,
    pub walk: String,
    pub s: Split,
    pub before: Metrics,
    pub after: Metrics,
    pub period_before: String,
    pub lands: Option<String>,
}

pub struct Res {
    pub name: String,
    pub status: &'static str,
    pub r: Option<Reduced>,
}

impl Res {
    /// One entry of reduced.json, keys in reduce.js's order.
    pub fn json(&self) -> J {
        let r = self.r.as_ref().unwrap();
        J::obj(vec![
            ("name", J::s(&self.name)),
            ("status", J::s(self.status)),
            ("red", J::s(&r.red)),
            ("walk", J::s(&r.walk)),
            ("s", r.s.json()),
            ("before", r.before.json()),
            ("after", r.after.json()),
            ("periodBefore", J::s(&r.period_before)),
            ("lands", r.lands.as_deref().map(J::s).unwrap_or(J::Null)),
        ])
    }
}

/// Periods are matched up to rotation: a coil and the same coil entered a term later
/// are the same coil, and the standardiser is free to write either.
fn rotations(p: &str) -> Vec<String> {
    let t: Vec<&str> = p.split(' ').collect();
    (0..t.len()).map(|i| t[i..].iter().chain(t[..i].iter()).copied().collect::<Vec<_>>().join(" ")).collect()
}

pub fn results() -> Vec<Res> {
    let src = sources();
    let (tgt, _, _) = target(&src);
    // WHERE A REDUCTION LANDS, against every period in walks/
    let catalogue: Vec<(String, String)> =
        src.iter().filter_map(|s| s.per.as_ref().map(|p| (s.name.clone(), text(p)))).collect();
    let lands_on = |red: &str| catalogue.iter().find(|(_, per)| rotations(per).iter().any(|r| r == red)).map(|c| c.0.clone());

    let mut out = Vec::new();
    for s in &src {
        let name = s.name.clone();
        let Some(per) = period_of(&parse(&s.walk)) else {
            out.push(Res { name, status: "no clean period", r: None });
            continue;
        };
        let slack: Vec<(usize, f64, f64)> = (0..per.len())
            .map(|i| (i, floor_of(&per, i), per[i].n))
            .filter(|x| x.2 > x.1)
            .collect();
        if slack.is_empty() {
            out.push(Res { name, status: "already minimal", r: None });
            continue;
        }

        // every combination of reductions, shortest first
        let mut combos: Vec<Vec<(usize, f64)>> = vec![vec![]];
        for &(i, floor, n) in &slack {
            let mut next = Vec::new();
            for c in &combos {
                let mut v = floor;
                while v <= n {
                    let mut c2 = c.clone();
                    c2.push((i, v));
                    next.push(c2);
                    v += 1.0;
                }
            }
            combos = next;
            if combos.len() > 50000 {
                break;
            }
        }
        let total = |c: &Vec<(usize, f64)>| c.iter().fold(0.0, |x, y| x + y.1);
        combos.sort_by(|a, b| total(a).partial_cmp(&total(b)).unwrap());

        let mut found: Option<(String, String, Split)> = None;
        for c in &combos {
            let mut t = per.clone();
            for &(i, v) in c {
                t[i].n = v;
            }
            if (0..t.len()).any(|i| t[i].n < floor_of(&t, i)) {
                continue; // floors shift as legs change
            }
            if !is_coil(&t) {
                continue;
            }
            let red = text(&t);
            let Some(built) = build_walk(&t, tgt) else { continue };
            let r = split(&built.walk);
            if r.ok && r.elbows == 0 && !r.oversize {
                found = Some((red, built.walk, r));
                break;
            }
        }
        let Some((red, walk, sp)) = found else {
            out.push(Res { name, status: "no reduction keeps the coil", r: None });
            continue;
        };
        let before = metrics(&s.walk, None).unwrap_or_else(|e| panic!("{e}"));
        let after = metrics(&walk, None).unwrap_or_else(|e| panic!("{e}"));
        let lands = lands_on(&red);
        out.push(Res {
            name,
            status: "reduced",
            r: Some(Reduced { red, walk, s: sp, before, after, period_before: text(&per), lands }),
        });
    }
    out
}
