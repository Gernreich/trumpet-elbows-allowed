//! Rotation metrics for a lattice bore walk.
//!
//! A coil turns about one axis while advancing along it. The lateral projection
//! -- the walk with the advancing axis dropped -- is what actually rotates, and
//! on a cubic lattice it can only ever turn in 90 degree steps, so the winding
//! is counted in quarter turns and multiplied up. Steps along the coil axis
//! project to nothing and are skipped; they are advance, not rotation.

use crate::json::J;
use std::collections::HashMap;

/// The block pitch in millimetres: a 10mm bore in 3mm stock. This is a SECOND
/// copy of bore_split.py's default, not an import, and every mm figure in the
/// scoring and the tables comes from this line -- so a pitch changed there has
/// to be changed here, and a run that disagrees will not say so.
pub const MM: i64 = 16;
const AXNAME: [&str; 3] = ["x (east/west)", "y (up/down)", "z (north/south)"];
const ALONG: [(&str, &str); 3] = [("E", "W"), ("U", "D"), ("S", "N")]; // +axis, -axis (north is -z)

pub fn v(d: u8) -> [i64; 3] {
    match d {
        b'N' => [0, 0, -1],
        b'S' => [0, 0, 1],
        b'E' => [1, 0, 0],
        b'W' => [-1, 0, 0],
        b'U' => [0, 1, 0],
        b'D' => [0, -1, 0],
        _ => panic!("not a direction: {}", d as char),
    }
}

/// One direction per block: the way the bore travels through it. You start IN
/// block 1 facing the first term, so the blocks are 1 + the sum of the numbers --
/// "N5" is six blocks -- and the count matches bore_split.py.
pub fn block_dirs(walk: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for t in walk.split_whitespace() {
        let b = t.as_bytes();
        let ok = b.len() >= 2 && b"NSEWUD".contains(&b[0]) && b[1..].iter().all(|c| c.is_ascii_digit());
        if !ok {
            return Err(format!("bad term: {t}"));
        }
        let n: u64 = t[1..].parse().map_err(|_| format!("bad term: {t}"))?;
        if out.is_empty() {
            out.push(b[0]); // block 1: where you start
        }
        for _ in 0..n {
            out.push(b[0]);
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub blocks: i64,
    pub mm: i64,
    pub size: [i64; 3],
    pub vol: i64,
    pub cross: [i64; 2],
    pub cross_mm: [i64; 2],
    pub axis_len: i64,
    pub axis_len_mm: i64,
    pub turns90: i64,
    pub turns_per_metre: f64,
    pub touching: i64,
    pub touching_per_block: f64,
    pub longest_straight: i64,
    pub longest_straight_mm: i64,
    pub axis: &'static str,
    pub axis_dir: &'static str,
    pub hand: &'static str,
    pub degrees: i64,
    pub turns: f64,
    pub blocks_per_360: Option<f64>,
    pub rise_per_360: Option<f64>,
    pub rise_mm_per_360: Option<f64>,
    pub deg_per_block: f64,
    pub rise: i64,
    pub rise_mm: i64,
}

impl Metrics {
    /// The object the JavaScript spiral_metrics returned, keys in its order.
    pub fn json(&self) -> J {
        let n = |x: i64| J::Num(x as f64);
        J::obj(vec![
            ("blocks", n(self.blocks)),
            ("mm", n(self.mm)),
            ("size", J::nums(self.size.iter().map(|&x| x as f64))),
            ("vol", n(self.vol)),
            ("cross", J::nums(self.cross.iter().map(|&x| x as f64))),
            ("crossMM", J::nums(self.cross_mm.iter().map(|&x| x as f64))),
            ("axisLen", n(self.axis_len)),
            ("axisLenMM", n(self.axis_len_mm)),
            ("turns90", n(self.turns90)),
            ("turnsPerMetre", J::Num(self.turns_per_metre)),
            ("touching", n(self.touching)),
            ("touchingPerBlock", J::Num(self.touching_per_block)),
            ("longestStraight", n(self.longest_straight)),
            ("longestStraightMM", n(self.longest_straight_mm)),
            ("axis", J::s(self.axis)),
            ("axisDir", J::s(self.axis_dir)),
            ("hand", J::s(self.hand)),
            ("degrees", n(self.degrees)),
            ("turns", J::Num(self.turns)),
            ("blocksPer360", J::opt(self.blocks_per_360)),
            ("risePer360", J::opt(self.rise_per_360)),
            ("riseMMPer360", J::opt(self.rise_mm_per_360)),
            ("degPerBlock", J::Num(self.deg_per_block)),
            ("rise", n(self.rise)),
            ("riseMM", n(self.rise_mm)),
        ])
    }
    /// What JavaScript arithmetic made of these: null counts as 0.
    pub fn rise_per_360_or0(&self) -> f64 {
        self.rise_per_360.unwrap_or(0.0)
    }
    pub fn blocks_per_360_or0(&self) -> f64 {
        self.blocks_per_360.unwrap_or(0.0)
    }
}

/// `array.slice(start, end)` bounds, with JavaScript's negative-index rule.
pub fn js_slice(len: usize, start: i64, end: Option<i64>) -> (usize, usize) {
    let len = len as i64;
    let fix = |x: i64| if x < 0 { (len + x).max(0) } else { x.min(len) };
    let a = fix(start);
    let b = end.map(fix).unwrap_or(len);
    (a as usize, b.max(a) as usize)
}

/// `range`, when given, is a 1-indexed inclusive block span: measure only that part
/// of the bore. Used to leave the mouth and exit pieces out of every judgement.
pub fn metrics(walk: &str, range: Option<(i64, i64)>) -> Result<Metrics, String> {
    let mut s = block_dirs(walk)?;
    if let Some((a, b)) = range {
        let (x, y) = js_slice(s.len(), a - 1, Some(b));
        s = s[x..y].to_vec();
    }
    let mut p = [0i64; 3];
    let mut cells = vec![p];
    for i in 0..s.len().saturating_sub(1) {
        // n-1 moves between n blocks
        let d = v(s[i]);
        p = [p[0] + d[0], p[1] + d[1], p[2] + d[2]];
        cells.push(p);
    }

    let mut lo = [1_000_000_000i64; 3];
    let mut hi = [-1_000_000_000i64; 3];
    for c in &cells {
        for i in 0..3 {
            lo[i] = lo[i].min(c[i]);
            hi[i] = hi[i].max(c[i]);
        }
    }
    let size = [hi[0] - lo[0] + 1, hi[1] - lo[1] + 1, hi[2] - lo[2] + 1];

    // the coil axis is the one the walk actually travels down
    let net = p;
    let mut ax = 0;
    for i in 1..3 {
        if net[i].abs() > net[ax].abs() {
            ax = i;
        }
    }
    let lat: Vec<usize> = (0..3).filter(|&i| i != ax).collect();

    // winding of the lateral projection, in quarter turns
    let proj: Vec<[i64; 2]> = s
        .iter()
        .map(|&d| v(d))
        .map(|w| [w[lat[0]], w[lat[1]]])
        .filter(|w| w[0] != 0 || w[1] != 0)
        .collect();
    let mut q = 0i64;
    for i in 0..proj.len().saturating_sub(1) {
        let (u, w) = (proj[i], proj[i + 1]);
        q += u[0] * w[1] - u[1] * w[0]; // +1 one way, -1 the other, 0 straight on
    }

    // 90 degree turns along the bore, and the longest run without one
    let (mut turns90, mut run, mut longest) = (0i64, 1i64, 1i64);
    for i in 1..s.len() {
        if s[i] == s[i - 1] {
            run += 1;
            longest = longest.max(run);
        } else {
            turns90 += 1;
            run = 1;
        }
    }

    let deg = q * 90;
    let turns = deg.abs() as f64 / 360.0;
    let rise = net[ax].abs();
    let blocks = cells.len() as i64;

    // Blocks that touch without being joined along the bore: where two runs of the
    // bore share a wall, that wall is all that separates them.
    let mut at: HashMap<[i64; 3], usize> = HashMap::new();
    for (i, c) in cells.iter().enumerate() {
        at.insert(*c, i);
    }
    const NB: [[i64; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    let mut touching = 0i64;
    for (i, c) in cells.iter().enumerate() {
        for n in NB {
            if let Some(&j) = at.get(&[c[0] + n[0], c[1] + n[1], c[2] + n[2]]) {
                if j > i && j != i + 1 {
                    touching += 1; // count each pair once
                }
            }
        }
    }

    // The cross-section is the envelope with the coil axis taken out: how fat the
    // coil is, which is what decides whether it fits inside anything.
    let mut cross = [size[lat[0]], size[lat[1]]];
    cross.sort();

    let bf = blocks as f64;
    // Handedness is a property of the helix, not of where you stand. The lateral
    // pair (lat0,lat1,ax) is a right-handed frame for ax = x and z but a
    // left-handed one for ax = y, and advancing down the negative axis flips it
    // again; fold both in so the answer means the same thing every time.
    let hand = if deg == 0 {
        "none"
    } else {
        let frame = if ax == 1 { -1 } else { 1 };
        let along = if net[ax] >= 0 { 1 } else { -1 };
        if q * frame * along > 0 { "right-handed" } else { "left-handed" }
    };
    let per = |x: f64| if turns != 0.0 { Some(x / turns) } else { None };
    Ok(Metrics {
        blocks,
        mm: blocks * MM,
        size,
        vol: size[0] * size[1] * size[2],
        cross,
        cross_mm: [cross[0] * MM, cross[1] * MM],
        axis_len: size[ax],
        axis_len_mm: size[ax] * MM,
        turns90,
        turns_per_metre: turns90 as f64 / ((blocks * MM) as f64 / 1000.0),
        touching,
        touching_per_block: touching as f64 / bf,
        longest_straight: longest,
        longest_straight_mm: longest * MM,
        axis: AXNAME[ax],
        axis_dir: if net[ax] > 0 { ALONG[ax].0 } else { ALONG[ax].1 },
        hand,
        degrees: deg.abs(),
        turns,
        blocks_per_360: per(bf),
        rise_per_360: per(rise as f64),
        rise_mm_per_360: per((rise * MM) as f64),
        deg_per_block: deg.abs() as f64 / bf,
        rise,
        rise_mm: rise * MM,
    })
}

/// The repeating unit: how many terms, and how many blocks.
pub struct Period {
    pub terms: Option<i64>,
    pub blocks: Option<f64>,
}

pub fn period(walk: &str) -> Period {
    let all: Vec<&str> = walk.split_whitespace().collect();
    let (a, b) = js_slice(all.len(), 2, Some(-2));
    let t = &all[a..b];
    let mut p = 1;
    while p <= t.len().saturating_sub(p) && p <= t.len() {
        if (0..t.len() - p).all(|i| t[i] == t[i + p]) {
            // `+x.slice(1) || 1`: a bare letter, or a zero, counts one block
            let blocks = t[..p].iter().fold(0.0, |acc, x| {
                let n = crate::jsfmt::number(&x[1..]);
                acc + if n == 0.0 || n.is_nan() { 1.0 } else { n }
            });
            return Period { terms: Some(p as i64), blocks: Some(blocks) };
        }
        p += 1;
    }
    Period { terms: None, blocks: None }
}
