//! Composite scoring across the common means.
//!
//! The metrics in METRICS below, each normalized to (0,1] with 1 = best in the set,
//! plus the touching count carried the same way and given an explicit weight. Then
//! the power-mean family and three means that sit outside it.
//!
//! blocks/360 is deliberately absent: it is anti-correlated with turns/m by
//! construction -- a tighter spiral must turn more often -- so carrying both let
//! them cancel and made the composite quieter about coiling than the columns are.

use crate::corpus::{pad_end, parts, unscored, walks};
use crate::jsfmt::fixed;
use crate::spiral_metrics::{metrics, Metrics};
use std::collections::HashMap;

pub struct Metric {
    pub k: &'static str,
    pub lo_better: bool,
    pub label: &'static str,
}

// Two of these are scored per block rather than absolutely. Whole periods of
// different lengths cannot all reach the same total, so the bores differ by about
// 9% -- and box and piece count both grow with tube, so comparing them absolutely
// hands the shorter coils an advantage they did nothing to earn. Cross-section,
// mean plate, distinct shapes and the rates are length-independent already, and
// touching stays absolute because the requirement is none of it, at any length.
pub const METRICS: [Metric; 8] = [
    Metric { k: "volPerBlock", lo_better: true, label: "box/block" },
    Metric { k: "crossArea", lo_better: true, label: "cross area" },
    Metric { k: "piecesPerBlock", lo_better: true, label: "pieces/block" },
    Metric { k: "distinct", lo_better: true, label: "distinct" },
    Metric { k: "risePer360", lo_better: true, label: "rise/360" },
    Metric { k: "turnsPerMetre", lo_better: true, label: "turns/m" },
    Metric { k: "longestStraight", lo_better: false, label: "longest str" },
    Metric { k: "meanPlate", lo_better: false, label: "mean plate" },
];

pub const EPS: f64 = 0.01; // a floor, so one worst-in-set value cannot zero a product

/// Touching walls are an aesthetic requirement, not just a cost, so it is weighted
/// heaviest of anything here: SPIRAL_TOUCH_WEIGHT, default 5.
pub fn touch_weight() -> f64 {
    match std::env::var("SPIRAL_TOUCH_WEIGHT") {
        Ok(s) if !s.is_empty() => crate::jsfmt::number(&s),
        _ => 5.0,
    }
}

#[derive(Clone, Debug)]
pub struct Vals {
    pub vol: f64,
    pub cross_area: f64,
    pub vol_per_block: f64,
    pub pieces_per_block: f64,
    pub pieces: f64,
    pub distinct: f64,
    pub rise_per_360: f64,
    pub turns_per_metre: f64,
    pub longest_straight: f64,
    pub mean_plate: f64,
    pub touching: f64,
}
impl Vals {
    pub fn get(&self, k: &str) -> f64 {
        match k {
            "volPerBlock" => self.vol_per_block,
            "crossArea" => self.cross_area,
            "piecesPerBlock" => self.pieces_per_block,
            "distinct" => self.distinct,
            "risePer360" => self.rise_per_360,
            "turnsPerMetre" => self.turns_per_metre,
            "longestStraight" => self.longest_straight,
            "meanPlate" => self.mean_plate,
            _ => panic!("no metric {k}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub name: String,
    pub m: Metrics,
    pub full: Metrics,
    pub v: Vals,
    /// score under each mean, in MEANS order
    pub s: Vec<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub x: f64,
    pub w: f64,
    pub label: &'static str,
}

/// parts.json's number for a coil, NaN when missing.
fn pnum(p: &crate::json::J, k: &str) -> f64 {
    p.get(k).and_then(|v| v.f()).unwrap_or(f64::NAN)
}

/// The interior block span parts.json recorded for a coil.
pub fn interior(p: &crate::json::J) -> Option<(i64, i64)> {
    let a = p.get("interiorBlocks")?.arr()?;
    Some((a.first()?.f()? as i64, a.get(1)?.f()? as i64))
}

/// Every scored coil. Nothing is judged on the bore's mouth and exit: m is measured
/// over the interior only, and pieces and shapes are counted there too.
pub fn rows() -> Vec<Row> {
    let parts = parts();
    let skip = unscored();
    walks()
        .into_iter()
        .filter(|(n, _)| !skip.contains(n))
        .map(|(name, walk)| {
            let p = parts.get(&name).unwrap_or_else(|| {
                eprintln!("parts.json has no entry for {name}; rebuild it with tools/parts");
                std::process::exit(1)
            });
            let m = metrics(&walk, interior(p)).unwrap_or_else(|e| panic!("{e}"));
            let full = metrics(&walk, None).unwrap();
            let inner = pnum(p, "innerPieces");
            let v = Vals {
                vol: m.vol as f64,
                cross_area: (m.cross[0] * m.cross[1]) as f64,
                vol_per_block: m.vol as f64 / m.blocks as f64,
                pieces_per_block: inner / m.blocks as f64,
                pieces: inner,
                distinct: pnum(p, "interiorDistinct"),
                rise_per_360: m.rise_per_360_or0(),
                turns_per_metre: m.turns_per_metre,
                longest_straight: m.longest_straight as f64,
                mean_plate: pnum(p, "meanPlate"),
                touching: m.touching as f64,
            };
            Row { name, m, full, v, s: vec![] }
        })
        .collect()
}

fn js_min(xs: &[f64]) -> f64 {
    xs.iter().copied().fold(f64::INFINITY, crate::jsfmt::min)
}
fn js_max(xs: &[f64]) -> f64 {
    xs.iter().copied().fold(f64::NEG_INFINITY, crate::jsfmt::max)
}

/// Each metric's lowest and highest value across the set.
pub fn bounds(set: &[&Row]) -> Vec<(f64, f64)> {
    METRICS
        .iter()
        .map(|mt| {
            let xs: Vec<f64> = set.iter().map(|r| r.v.get(mt.k)).collect();
            (js_min(&xs), js_max(&xs))
        })
        .collect()
}

/// Every input on one scale: (0,1], 1 = best. Touching included the same way rather
/// than as a special case, so each mean sees the same numbers.
pub fn inputs(r: &Row, b: &[(f64, f64)], tw: f64) -> Vec<Input> {
    let mut out: Vec<Input> = METRICS
        .iter()
        .zip(b)
        .map(|(mt, &(lo, hi))| {
            let span = hi - lo;
            let t = if span == 0.0 { 1.0 } else { (r.v.get(mt.k) - lo) / span };
            let good = if mt.lo_better { 1.0 - t } else { t };
            Input { x: EPS + (1.0 - EPS) * good, w: 1.0, label: mt.label }
        })
        .collect();
    out.push(Input { x: 1.0 / (1.0 + r.v.touching), w: tw, label: "touching" });
    out
}

fn wsum(vals: &[Input], f: impl Fn(&Input) -> f64) -> f64 {
    vals.iter().fold(0.0, |a, v| a + f(v))
}

/// The power mean of order p. p = 1 arithmetic, p -> 0 geometric, p = -1 harmonic,
/// p = 2 quadratic (RMS), p = 3 cubic. All weighted by w.
pub fn power(vals: &[Input], p: f64) -> f64 {
    let w = wsum(vals, |v| v.w);
    if p == 0.0 {
        return (wsum(vals, |v| v.w * v.x.ln()) / w).exp();
    }
    (wsum(vals, |v| v.w * v.x.powf(p)) / w).powf(1.0 / p)
}

/// Weighted properly, not by copying each value w times; at an integer weight it
/// returns exactly what the expansion returned.
pub fn median(vals: &[Input]) -> f64 {
    let mut xs = vals.to_vec();
    xs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let w = wsum(&xs, |v| v.w);
    let mut seen = 0.0;
    for i in 0..xs.len() {
        seen += xs[i].w;
        if seen > w / 2.0 {
            return xs[i].x;
        }
        // landing exactly on the half-way mark: the two neighbours share the middle
        if seen == w / 2.0 {
            return (xs[i].x + xs[i + 1].x) / 2.0;
        }
    }
    xs[xs.len() - 1].x
}

/// (max + min) / 2: no weight can reach it.
pub fn midrange(vals: &[Input]) -> f64 {
    let xs: Vec<f64> = vals.iter().map(|v| v.x).collect();
    (js_max(&xs) + js_min(&xs)) / 2.0
}

pub fn contra(vals: &[Input]) -> f64 {
    wsum(vals, |v| v.w * v.x * v.x) / wsum(vals, |v| v.w * v.x)
}

pub struct Mean {
    pub k: &'static str,
    pub label: &'static str,
}
pub const MEANS: [Mean; 8] = [
    Mean { k: "harm", label: "harmonic" },
    Mean { k: "geo", label: "geometric" },
    Mean { k: "arith", label: "arithmetic" },
    Mean { k: "quad", label: "quadratic" },
    Mean { k: "cubic", label: "cubic" },
    Mean { k: "median", label: "median" },
    Mean { k: "midr", label: "midrange" },
    Mean { k: "contra", label: "contraharmonic" },
];
pub fn mean_index(k: &str) -> usize {
    MEANS.iter().position(|m| m.k == k).unwrap()
}

fn apply(i: usize, v: &[Input]) -> f64 {
    match i {
        0 => power(v, -1.0),
        1 => power(v, 0.0),
        2 => power(v, 1.0),
        3 => power(v, 2.0),
        4 => power(v, 3.0),
        5 => median(v),
        6 => midrange(v),
        _ => contra(v),
    }
}

pub struct Scored {
    pub rows: Vec<Row>,
    /// indices into rows of what --clean / --solid leave
    pub shown: Vec<usize>,
    /// per mean, name -> rank among `shown`
    pub ranks: Vec<HashMap<String, usize>>,
    pub bounds: Vec<(f64, f64)>,
    pub touch_weight: f64,
}

impl Scored {
    pub fn rank(&self, mean: &str, name: &str) -> usize {
        self.ranks[mean_index(mean)][name]
    }
    pub fn inputs(&self, r: &Row) -> Vec<Input> {
        inputs(r, &self.bounds, self.touch_weight)
    }
}

/// Score every coil. The normalization is computed over the whole set either way;
/// --clean and --solid filter the field, they do not rescale it.
pub fn score(clean_only: bool, solid_only: bool) -> Scored {
    let mut rows = rows();
    let tw = touch_weight();
    let b = bounds(&rows.iter().collect::<Vec<_>>());
    for r in rows.iter_mut() {
        let v = inputs(r, &b, tw);
        r.s = (0..MEANS.len()).map(|i| apply(i, &v)).collect();
    }
    let mut shown: Vec<usize> = (0..rows.len()).collect();
    if clean_only {
        shown.retain(|&i| rows[i].v.touching == 0.0);
    }
    if solid_only {
        shown.retain(|&i| *rows[i].m.size.iter().min().unwrap() >= 3);
    }
    let ranks = (0..MEANS.len())
        .map(|k| {
            let mut s = shown.clone();
            s.sort_by(|&a, &b| rows[b].s[k].partial_cmp(&rows[a].s[k]).unwrap());
            s.iter().enumerate().map(|(i, &r)| (rows[r].name.clone(), i + 1)).collect()
        })
        .collect();
    Scored { rows, shown, ranks, bounds: b, touch_weight: tw }
}

/// The table score.js printed: markdown, or plain text in padded columns.
pub fn table(sc: &Scored, md: bool) -> String {
    let mut head = vec!["spiral".to_string(), "touch".to_string()];
    for m in &MEANS {
        head.push(m.label.to_string());
        head.push("#".to_string());
    }
    let mut order = sc.shown.clone();
    order.sort_by_key(|&i| sc.rank("geo", &sc.rows[i].name));
    let body: Vec<Vec<String>> = order
        .iter()
        .map(|&i| {
            let r = &sc.rows[i];
            let mut b = vec![r.name.clone(), crate::jsfmt::num(r.v.touching)];
            for (k, m) in MEANS.iter().enumerate() {
                b.push(fixed(r.s[k], 4));
                b.push(sc.rank(m.k, &r.name).to_string());
            }
            b
        })
        .collect();
    let mut out = String::new();
    if md {
        out += &format!("| {} |\n", head.join(" | "));
        let al: Vec<&str> = (0..head.len()).map(|i| if i == 0 { "---" } else { "---:" }).collect();
        out += &format!("| {} |\n", al.join(" | "));
        for b in &body {
            out += &format!("| {} |\n", b.join(" | "));
        }
    } else {
        let w: Vec<usize> =
            (0..head.len()).map(|i| body.iter().map(|b| b[i].len()).fold(head[i].len(), usize::max)).collect();
        out += &(head.iter().enumerate().map(|(i, h)| pad_end(h, w[i])).collect::<Vec<_>>().join("  ") + "\n");
        for b in &body {
            out += &(b.iter().enumerate().map(|(i, v)| pad_end(v, w[i])).collect::<Vec<_>>().join("  ") + "\n");
        }
    }
    out
}
