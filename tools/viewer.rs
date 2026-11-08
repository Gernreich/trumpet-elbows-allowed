//! Write a bore as a self-contained page you can turn around with the mouse:
//! viewer.py, Rust since 2026-11-08. The page itself (viewer_page.rs) is the
//! browser code and stays JavaScript; this fills in its data.
use crate::bore::{dirs, neg, Design};
use crate::render::{dircol, dirname, wheel};
use lasermade_tools::py;
use lasermade_tools::turn3d::{
    Num::{F, I},
    J,
};

fn n(x: f64) -> J {
    J::N(F(x))
}
fn i(x: i64) -> J {
    J::N(I(x))
}
fn s(x: &str) -> J {
    J::S(x.to_string())
}
fn dumps(j: &J) -> String {
    let mut o = String::new();
    j.dump(&mut o);
    o
}

const ORDER: [[i64; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

/// The one dict the page draws from.
pub fn data_for(d: &Design, text: &str) -> Result<J, String> {
    let sp = d.specs_for(text)?;
    let rec = &sp.rec;
    let mut sec_of = vec![0usize; rec.len()];
    for (k, gr) in sp.groups.iter().enumerate() {
        for &j in gr {
            sec_of[j] = k;
        }
    }
    let pal = wheel(sp.groups.len());
    let boxes = d.block_boxes(rec);
    let cells: Vec<J> = rec
        .iter()
        .enumerate()
        .map(|(k, r)| J::obj(vec![("a", J::A(boxes[k].0.iter().map(|&x| n(x)).collect())), ("b", J::A(boxes[k].1.iter().map(|&x| n(x)).collect())), ("d", s(&r.out.to_string())), ("s", i(sec_of[k] as i64)), ("i0", i(k as i64))]))
        .collect();
    let fidx = |v: [i64; 3]| ORDER.iter().position(|o| *o == v).unwrap();
    let openings = J::O(vec![
        (format!("0:{}", fidx(neg(dirs(rec[0].inn)))), s("IN")),
        (format!("{}:{}", rec.len() - 1, fidx(dirs(rec[rec.len() - 1].out))), s("OUT")),
    ]);
    let mm = rec.iter().fold(0.0, |acc, r| acc + d.extent(r, crate::bore::axis_of(r.out)));
    Ok(J::obj(vec![
        ("cells", J::A(cells)),
        ("blocks", i(rec.len() as i64)),
        ("mm", i(py::round(mm))),
        ("dircol", J::O("NSEWUD".chars().map(|c| (c.to_string(), s(dircol(c)))).collect())),
        (
            "dirs",
            J::A(
                "NSEWUD"
                    .chars()
                    .filter(|&c| rec.iter().any(|r| r.out == c))
                    .map(|c| J::obj(vec![("k", s(&c.to_string())), ("name", s(dirname(c))), ("n", i(rec.iter().filter(|r| r.out == c).count() as i64))]))
                    .collect(),
            ),
        ),
        (
            "sections",
            J::A(sp.groups.iter().zip(&sp.plan).enumerate().map(|(k, (gr, p))| J::obj(vec![("col", s(&pal[k % pal.len()])), ("kind", s(&p.kind)), ("n", i(gr.len() as i64))])).collect()),
        ),
        ("openings", openings),
        ("inCol", s("#1c1c20")),
        ("outCol", s("#e0457b")),
    ]))
}

/// The mouthpiece and the bell, drawn on every page at IN and OUT: their ring
/// stacks read from their own cut files, 24 facets round.
fn parts_dir() -> String {
    format!("{}/parts", crate::common::repo())
}
const END_PARTS: [(&str, &str, &str); 2] = [
    ("bell", "bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg", "#d8b445"),
    ("mouthpiece", "mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg", "#8a5a2b"),
];

pub fn ends_json() -> String {
    let mut out = vec![];
    for (name, svg, col) in END_PARTS {
        let dd = crate::views::data_for(&format!("{}/{svg}", parts_dir()), 24);
        let v = dd.get("V").unwrap().arr().iter().map(|p| J::A(p.arr().iter().map(|c| n(py::round_dp(c.num().f(), 2))).collect())).collect();
        let q = dd
            .get("Q")
            .unwrap()
            .arr()
            .iter()
            .map(|q| {
                let mut vs: Vec<J> = q.get("v").unwrap().arr().to_vec();
                vs.push(q.get("f").unwrap().clone());
                J::A(vs)
            })
            .collect();
        out.push((name.to_string(), J::obj(vec![("V", J::A(v)), ("Q", J::A(q)), ("col", s(col))])));
    }
    out.push(("shade".into(), J::A(vec![n(1.0), n(0.92), n(0.55), n(0.7)])));
    dumps(&J::O(out))
}

pub fn build(d: &Design, text: &str, title: &str) -> Result<String, String> {
    build_sets(&[(String::new(), text.to_string(), data_for(d, text)?)], title)
}

/// items is [(label, walk, data)], with the data already computed.
pub fn build_sets(items: &[(String, String, J)], title: &str) -> Result<String, String> {
    let sets: Vec<J> = items.iter().map(|(lab, txt, dd)| J::obj(vec![("label", s(lab)), ("walk", s(txt.trim())), ("d", dd.clone())])).collect();
    let first_walk = items[0].1.trim().to_string();
    let blocks = items[0].2.get("blocks").unwrap().num();
    Ok(crate::viewer_page::HTML
        .replace("__SETS__", &dumps(&J::A(sets)))
        .replace("__ENDS__", &ends_json())
        .replace("__TITLE__", title)
        .replace("__WALK__", &first_walk)
        .replace("__N__", &blocks.repr()))
}
