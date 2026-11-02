//! Iterated ranking: rank, cut the bottom half, re-rank the survivors, repeat.
//!
//! The question is whether the survivors keep their order when the losers leave.
//! They do not, if the normalization is computed over the set -- min-max reads its
//! lo and hi off whoever is present, so removing alternatives rescales every
//! metric and reorders coils that did not change. That is an independence-of-
//! irrelevant-alternatives failure, and it is why a cut should be made on a
//! property decided in advance, not on composite score.
//!
//! The other end of it: make the normalization set-independent and give it a
//! geometric mean, and the whole procedure is provably a no-op, because scaling
//! metric i by c_i multiplies every score by the same (prod c_i)^(1/n).

use crate::score::{bounds, touch_weight, Row, EPS, METRICS};

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// what SCORING.md ranks with: set-relative, floored
    MinMax,
    /// ratio-to-best with no floor: nothing depends on which other coils are present
    Pure,
}

fn score(set: &[&Row], mode: Mode, harm: bool, tw: f64) -> Vec<String> {
    let b = bounds(set);
    let mut scored: Vec<(String, f64)> = set
        .iter()
        .map(|r| {
            let mut vals: Vec<(f64, f64)> = METRICS
                .iter()
                .zip(&b)
                .map(|(mt, &(lo, hi))| {
                    let x = r.v.get(mt.k);
                    if mode == Mode::MinMax {
                        let span = hi - lo;
                        let t = if span == 0.0 { 1.0 } else { (x - lo) / span };
                        let good = if mt.lo_better { 1.0 - t } else { t };
                        (EPS + (1.0 - EPS) * good, 1.0)
                    } else {
                        (if mt.lo_better { lo / x } else { x / hi }, 1.0)
                    }
                })
                .collect();
            // The touching term is the same in both modes and in score: 1/(1+t) is
            // absolute, so it is not the thing that moves when the set shrinks.
            vals.push((1.0 / (1.0 + r.v.touching), if mode == Mode::MinMax { tw } else { 1.0 }));
            let w = vals.iter().fold(0.0, |a, v| a + v.1);
            let s = if harm {
                w / vals.iter().fold(0.0, |a, v| a + v.1 / v.0)
            } else {
                (vals.iter().fold(0.0, |a, v| a + v.1 * v.0.ln()) / w).exp()
            };
            (r.name.clone(), s)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().map(|x| x.0).collect()
}

pub struct Round {
    pub round: usize,
    pub from: usize,
    pub to: usize,
    pub moved: usize,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

pub struct Run {
    pub label: &'static str,
    pub log: Vec<Round>,
    pub winner: String,
    pub ranked_first: String,
}

fn eliminate(rows: &[Row], mode: Mode, harm: bool, tw: f64) -> (Vec<Round>, String, String) {
    let mut set: Vec<&Row> = rows.iter().collect();
    let (mut round, mut first) = (0, Vec::new());
    let mut log = Vec::new();
    while set.len() > 1 {
        let ranked = score(&set, mode, harm, tw);
        if round == 0 {
            first = ranked.clone();
        }
        let keep: Vec<String> = ranked[..ranked.len().div_ceil(2)].to_vec();
        let before: Vec<String> = ranked.iter().filter(|n| keep.contains(n)).cloned().collect();
        let kept: Vec<&Row> = set.iter().copied().filter(|r| keep.contains(&r.name)).collect();
        let after = score(&kept, mode, harm, tw);
        let moved = before.iter().enumerate().filter(|(i, n)| after.get(*i) != Some(n)).count();
        log.push(Round { round, from: set.len(), to: keep.len(), moved, before, after });
        set = kept;
        round += 1;
    }
    (log, set[0].name.clone(), first[0].clone())
}

pub fn results(rows: &[Row]) -> Vec<Run> {
    let tw = touch_weight();
    let runs: [(Mode, bool, &'static str); 3] = [
        (Mode::MinMax, true, "harmonic, min-max (what SCORING.md uses)"),
        (Mode::MinMax, false, "geometric, min-max"),
        (Mode::Pure, false, "geometric, pure ratio-to-best"),
    ];
    runs.iter()
        .map(|&(mode, harm, label)| {
            let (log, winner, ranked_first) = eliminate(rows, mode, harm, tw);
            Run { label, log, winner, ranked_first }
        })
        .collect()
}
