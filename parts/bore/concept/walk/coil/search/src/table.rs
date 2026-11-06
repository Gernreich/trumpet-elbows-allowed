//! The metrics tables: shape and cost, and rotation. Judged columns are measured
//! over the interior; blocks and mm describe the whole bore, ends included.

use crate::corpus::{pad_end, unscored, walks};
use crate::jsfmt::{fixed, num, round};
use crate::json::J;
use crate::score::interior;
use crate::spiral_metrics::{metrics, period, Metrics, Period};

pub struct Row {
    pub name: String,
    pub p: J,
    pub full: Metrics,
    pub per: Period,
    pub m: Metrics,
}

/// Every walk, sorted by box. parts.json is optional here: without it the piece
/// columns read "-".
pub fn rows() -> Vec<Row> {
    let parts = std::fs::read_to_string(crate::corpus::root().join("parts.json"))
        .ok()
        .and_then(|t| J::parse(&t).ok())
        .unwrap_or(J::Obj(vec![]));
    let mut rows: Vec<Row> = walks()
        .into_iter()
        .map(|(name, walk)| {
            let p = parts.get(&name).cloned().unwrap_or(J::Obj(vec![]));
            let m = metrics(&walk, interior(&p)).unwrap_or_else(|e| panic!("{e}"));
            Row { full: metrics(&walk, None).unwrap(), per: period(&walk), m, p, name }
        })
        .collect();
    rows.sort_by_key(|r| r.m.vol);
    rows
}

#[derive(Clone, Copy, PartialEq)]
pub enum Which {
    Shape,
    Rot,
    All,
}

/// A value from parts.json as `String(v ?? '-')` writes it.
fn or_dash(p: &J, k: &str) -> String {
    match p.get(k) {
        None | Some(J::Null) => "-".into(),
        Some(J::Num(x)) => num(*x),
        Some(J::Str(s)) => s.clone(),
        Some(other) => other.stringify(0),
    }
}
fn pf(p: &J, k: &str) -> f64 {
    p.get(k).and_then(|v| v.f()).unwrap_or(f64::NAN)
}
fn truthy(p: &J, k: &str) -> bool {
    p.get(k).is_some_and(|v| v.truthy())
}

fn link(r: &Row, md: bool) -> String {
    if md { format!("[`{}`](pages/{}.html)", r.name, r.name) } else { r.name.clone() }
}

fn shape_row(r: &Row, md: bool) -> Vec<String> {
    let j = |a: &[i64]| a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" x ");
    vec![
        link(r, md),
        r.full.blocks.to_string(),
        r.full.mm.to_string(),
        j(&r.m.size),
        r.m.vol.to_string(),
        format!("{} blk / {} mm", j(&r.m.cross), j(&r.m.cross_mm)),
        format!("{} / {} mm", r.m.axis_len, r.m.axis_len_mm),
        or_dash(&r.p, "innerPieces"),
        or_dash(&r.p, "interiorDistinct"),
        match (r.per.terms, r.per.blocks) {
            (Some(t), Some(b)) => format!("{t} tm / {} blk", num(b)),
            _ => "-".into(),
        },
        if truthy(&r.p, "rhythm") {
            format!("{} x{}", num(pf(&r.p, "rhythm")), fixed(pf(&r.p, "repeats"), 1))
        } else {
            "-".into()
        },
        if truthy(&r.p, "meanPlate") { num(round(pf(&r.p, "meanPlate"))) } else { "-".into() },
        r.m.touching.to_string(),
    ]
}

fn rot_row(r: &Row, md: bool) -> Vec<String> {
    let m = &r.m;
    vec![
        link(r, md),
        m.axis_dir.to_string(),
        m.degrees.to_string(),
        fixed(m.turns, 2),
        fixed(m.blocks_per_360_or0(), 1),
        format!("{} / {}", fixed(m.rise_per_360_or0(), 2), num(round(m.rise_mm_per_360.unwrap_or(0.0)))),
        fixed(m.deg_per_block, 1),
        m.turns90.to_string(),
        fixed(m.turns_per_metre, 1),
        format!("{} blk / {} mm", m.longest_straight, m.longest_straight_mm),
    ]
}

const SHAPE_HEAD: [&str; 13] = [
    "spiral", "blocks", "mm", "envelope", "box", "cross-section", "along axis", "pieces", "distinct", "period",
    "rhythm", "mean plate mm2", "touching",
];
const SHAPE_ALIGN: [&str; 13] = ["---", "---:", "---:", "---", "---:", "---", "---:", "---:", "---:", "---:", "---:", "---:", "---:"];
const ROT_HEAD: [&str; 10] = [
    "spiral", "axis", "rotation", "turns", "blocks/360", "rise/360 blk / mm", "deg/block", "90deg turns", "turns/m",
    "longest straight",
];
const ROT_ALIGN: [&str; 10] = ["---", ":-:", "---:", "---:", "---:", "---:", "---:", "---:", "---:", "---:"];

fn emit(out: &mut String, shape: bool, set: &[&Row], md: bool) {
    let (head, align): (&[&str], &[&str]) = if shape { (&SHAPE_HEAD, &SHAPE_ALIGN) } else { (&ROT_HEAD, &ROT_ALIGN) };
    let body: Vec<Vec<String>> = set.iter().map(|r| if shape { shape_row(r, md) } else { rot_row(r, md) }).collect();
    if md {
        *out += &format!("| {} |\n", head.join(" | "));
        *out += &format!("| {} |\n", align.join(" | "));
        for b in &body {
            *out += &format!("| {} |\n", b.join(" | "));
        }
    } else {
        let w: Vec<usize> =
            (0..head.len()).map(|i| body.iter().map(|b| b[i].len()).fold(head[i].len(), usize::max)).collect();
        *out += &(head.iter().enumerate().map(|(i, h)| pad_end(h, w[i])).collect::<Vec<_>>().join("  ") + "\n");
        for b in &body {
            *out += &(b.iter().enumerate().map(|(i, v)| pad_end(v, w[i])).collect::<Vec<_>>().join("  ") + "\n");
        }
    }
}

/// Exactly what table.js printed for these flags.
pub fn render(md: bool, want: Which) -> String {
    let rows = rows();
    let skip = unscored();
    let scored: Vec<&Row> = rows.iter().filter(|r| !skip.contains(&r.name)).collect();
    let unsc: Vec<&Row> = rows.iter().filter(|r| skip.contains(&r.name)).collect();
    let mut out = String::new();
    match want {
        Which::All => {
            emit(&mut out, true, &scored, md);
            out.push('\n');
            emit(&mut out, false, &scored, md);
        }
        Which::Shape => emit(&mut out, true, &scored, md),
        Which::Rot => emit(&mut out, false, &scored, md),
    }
    if !unsc.is_empty() {
        out += if md { "\nNot scored (see `unscored.txt`):\n\n" } else { "\nnot scored:\n" };
        emit(&mut out, want != Which::Rot, &unsc, md);
    }
    out
}
