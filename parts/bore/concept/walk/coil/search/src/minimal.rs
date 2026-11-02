//! Can this walk be made shorter without stranding a turn?
//!
//! A term's floor is set by the window of three around it -- outer A, middle m,
//! outer C. Consecutive terms are always on different axes, so:
//!
//!   A and C same axis, same direction      a step     m >= 1
//!   A and C same axis, opposite direction  a hairpin  m >= 2
//!   A and C different axes                 a coil     m >= 3
//!
//! A walk where every term already sits on its floor cannot be shortened at all
//! by this rule: every block left in it is load-bearing.

use crate::corpus::{pad_end, pad_start};

pub fn axis(d: u8) -> u8 {
    match d {
        b'N' | b'S' => 2,
        b'E' | b'W' => 0,
        _ => 1,
    }
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub i: usize,
    pub term: String,
    pub n: f64,
    pub kind: &'static str,
    pub floor: f64,
    pub slack: f64,
}

/// parseInt(s, 10): leading digits, NaN when there are none.
fn parse_int(s: &str) -> f64 {
    let d: String = s.trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
    d.parse().unwrap_or(f64::NAN)
}

pub fn floors(walk: &str) -> Vec<Floor> {
    let t: Vec<(u8, f64, &str)> = walk
        .split_whitespace()
        .map(|s| (s.as_bytes()[0], if s.len() > 1 { parse_int(&s[1..]) } else { 1.0 }, s))
        .collect();
    let mut out = Vec::new();
    // the first and last term are the lead-in and the lead-out runs; they have no
    // window of their own and are not the design's to choose, so are left alone
    for i in 1..t.len().saturating_sub(1) {
        let (a, m, c) = (t[i - 1], t[i], t[i + 1]);
        let kind = if axis(a.0) != axis(c.0) {
            "coil"
        } else if a.0 == c.0 {
            "step"
        } else {
            "hairpin"
        };
        let floor = match kind {
            "coil" => 3.0,
            "step" => 1.0,
            _ => 2.0,
        };
        out.push(Floor { i, term: m.2.to_string(), n: m.1, kind, floor, slack: m.1 - floor });
    }
    out
}

pub struct Report {
    pub name: String,
    pub saving: f64,
    pub slack: Vec<Floor>,
    pub line: String,
}

pub fn report(name: &str, walk: &str) -> Report {
    let f = floors(walk);
    let slack: Vec<Floor> = f.iter().filter(|x| x.slack > 0.0).cloned().collect();
    let saving = slack.iter().fold(0.0, |a, x| a + x.slack);
    let mut by_kind: Vec<(&str, usize)> = Vec::new();
    for x in &f {
        match by_kind.iter_mut().find(|(k, _)| *k == x.kind) {
            Some(e) => e.1 += 1,
            None => by_kind.push((x.kind, 1)),
        }
    }
    // Object.entries(...).sort(): compared as the strings "kind,count"
    let mut entries: Vec<String> = by_kind.iter().map(|(k, v)| format!("{k},{v}")).collect();
    entries.sort();
    let kinds: Vec<String> = entries
        .iter()
        .map(|e| {
            let (k, v) = e.split_once(',').unwrap();
            format!("{v} {k}")
        })
        .collect();
    let line = format!(
        "{}{} terms   {}{}",
        pad_end(name, 18),
        pad_start(&f.len().to_string(), 4),
        pad_end(&kinds.join(", "), 32),
        if saving == 0.0 {
            "MINIMAL".to_string()
        } else {
            format!("{} blocks removable in {} terms", crate::jsfmt::num(saving), slack.len())
        }
    );
    Report { name: name.to_string(), saving, slack, line }
}

/// Every walk in walks/, as gen_readme reads it.
pub fn results() -> Vec<Report> {
    crate::corpus::walks().iter().map(|(n, w)| report(n, w)).collect()
}
