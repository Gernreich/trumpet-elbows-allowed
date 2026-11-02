//! Standard form for a coil, so two coils differ only where they really differ:
//! north forward, counter-clockwise, the lexicographically smallest representation,
//! whole periods nearest a common target, one block in and one block out.
//! tools/standardise.rs applies it to the corpus; reduce.rs builds with it.

use crate::jsfmt::{num, round};
use crate::spiral_metrics::v;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Term {
    pub d: u8,
    pub n: f64,
}
impl Term {
    pub fn text(&self) -> String {
        format!("{}{}", self.d as char, num(self.n))
    }
}
pub fn text(t: &[Term]) -> String {
    t.iter().map(Term::text).collect::<Vec<_>>().join(" ")
}

/// `{ d: t[0], n: t.length > 1 ? +t.slice(1) : 1 }` per term.
pub fn parse(w: &str) -> Vec<Term> {
    w.split_whitespace()
        .map(|t| Term {
            d: t.as_bytes()[0],
            n: if t.len() > 1 { crate::jsfmt::number(&t[1..]) } else { 1.0 },
        })
        .collect()
}

fn name_of(w: [i64; 3]) -> Option<u8> {
    b"NSEWUD".iter().copied().find(|&d| v(d) == w)
}

/// The 48 signed axis permutations: rotations and mirrors both, because some coils
/// are the wrong way round and only a reflection fixes handedness. In order.
fn transforms() -> Vec<([usize; 3], [i64; 3])> {
    let mut t = Vec::new();
    for p in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        for s in [[1, 1, 1], [1, 1, -1], [1, -1, 1], [1, -1, -1], [-1, 1, 1], [-1, 1, -1], [-1, -1, 1], [-1, -1, -1]] {
            t.push((p, s));
        }
    }
    t
}

pub fn net_of(t: &[Term]) -> [f64; 3] {
    t.iter().fold([0.0; 3], |a, m| {
        let w = v(m.d);
        [a[0] + w[0] as f64 * m.n, a[1] + w[1] as f64 * m.n, a[2] + w[2] as f64 * m.n]
    })
}
pub fn blocks_of(t: &[Term]) -> f64 {
    t.iter().fold(0.0, |a, m| a + m.n)
}

/// Counter-clockwise seen looking ALONG forward. Forward is -z, so the viewer looks
/// down -z with +x right and +y up, and CCW is +x turning toward +y.
fn winding_xy(t: &[Term]) -> i64 {
    let proj: Vec<[i64; 2]> = t.iter().map(|m| v(m.d)).filter(|w| w[0] != 0 || w[1] != 0).map(|w| [w[0], w[1]]).collect();
    let mut q = 0;
    for i in 0..proj.len() {
        let (u, w) = (proj[i], proj[(i + 1) % proj.len()]);
        q += u[0] * w[1] - u[1] * w[0];
    }
    q
}

/// Every representation that travels north, turns CCW and opens on a north term;
/// the lexicographically smallest, so equal coils are identical strings.
pub fn orient(period: &[Term]) -> Option<Vec<Term>> {
    let mut cands: Vec<String> = Vec::new();
    for (p, s) in transforms() {
        let t: Vec<Option<Term>> = period
            .iter()
            .map(|m| {
                let w = v(m.d);
                name_of([s[0] * w[p[0]], s[1] * w[p[1]], s[2] * w[p[2]]]).map(|d| Term { d, n: m.n })
            })
            .collect();
        if t.iter().any(|m| m.is_none()) {
            continue;
        }
        let t: Vec<Term> = t.into_iter().map(Option::unwrap).collect();
        let net = net_of(&t);
        if net[0] != 0.0 || net[1] != 0.0 || net[2] >= 0.0 {
            continue; // must travel north
        }
        if winding_xy(&t) <= 0 {
            continue; // must be CCW
        }
        for i in 0..t.len() {
            if t[i].d != b'N' {
                continue; // must open on north
            }
            let rot: Vec<Term> = t[i..].iter().chain(t[..i].iter()).copied().collect();
            cands.push(text(&rot));
        }
    }
    cands.sort();
    cands.first().map(|c| parse(c))
}

/// The repeating unit of a walk, its lead-in and lead-out runs left off; None when
/// the walk is too short to show one.
pub fn period_of(t: &[Term]) -> Option<Vec<Term>> {
    if t.len() < 2 {
        return None;
    }
    let body: Vec<String> = t[1..t.len() - 1].iter().map(Term::text).collect();
    let mut p = 1;
    while p <= body.len().saturating_sub(p) {
        if (0..body.len() - p).all(|i| body[i] == body[i + p]) {
            return Some(body[..p].iter().map(|s| parse(s)[0]).collect());
        }
        p += 1;
    }
    None
}

pub struct Built {
    pub walk: String,
    pub oriented: Vec<Term>,
    pub k: f64,
    pub period_blocks: f64,
}

/// Orient a period and repeat it the number of times that lands nearest the target.
/// The walk opens on its first numbered term, a north term by orient(), and leaves
/// facing the last; block 1 is the one block in, the closing run's last block the
/// one block out.
pub fn build_walk(period: &[Term], target: f64) -> Option<Built> {
    let o = orient(period)?;
    let pb = blocks_of(&o);
    let k = crate::jsfmt::max(1.0, round(target / pb));
    let one = text(&o);
    let walk = vec![one; k as usize].join(" ");
    Some(Built { walk, oriented: o, k, period_blocks: pb })
}

pub struct Source {
    pub name: String,
    pub walk: String,
    pub per: Option<Vec<Term>>,
}

pub fn sources() -> Vec<Source> {
    crate::corpus::walks()
        .into_iter()
        .map(|(name, walk)| {
            let per = period_of(&parse(&walk));
            Source { name, walk, per }
        })
        .collect()
}

/// Whole periods of different lengths cannot all reach the same total, so the
/// target is chosen rather than assumed: the one from 120 to 220 blocks that makes
/// the tube lengths most alike, taking the NEAREST whole-period count to it.
/// Returns (target, spread, the period lengths it was chosen over).
pub fn target(src: &[Source]) -> (f64, f64, Vec<f64>) {
    let mut periods: Vec<f64> = Vec::new();
    for s in src {
        if let Some(p) = &s.per {
            let b = blocks_of(p);
            if !periods.contains(&b) {
                periods.push(b);
            }
        }
    }
    let (mut best_l, mut best) = (f64::NAN, f64::INFINITY);
    for l in 120..=220 {
        let len = lengths_for(&periods, l as f64);
        let spread = len.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            / len.iter().copied().fold(f64::INFINITY, f64::min)
            - 1.0;
        if spread < best - 1e-12 {
            best = spread;
            best_l = l as f64;
        }
    }
    (best_l, best, periods)
}

pub fn lengths_for(periods: &[f64], l: f64) -> Vec<f64> {
    periods.iter().map(|&p| crate::jsfmt::max(1.0, round(l / p)) * p).collect()
}
