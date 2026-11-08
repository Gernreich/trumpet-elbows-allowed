//! `coils [OUT.html]`: one page holding every 10x10x30 coil, with a control to
//! switch between them. coils.py until 2026-11-08. A bare run rewrites the
//! design's own coils.html.
//!
//! The four coils are the same walk truncated at its N spacers, so comparing
//! them means comparing lengths of one thing rather than four designs. Not a
//! second viewer: the same page and drawing as the per-coil pages, with more
//! than one set in it.
use lasermade_tools::py;
use trumpet::bore::{axis_of, walk_text, Design};

/// label, walk file; shortest first, the order they truncate in
const COILS: [(&str, &str); 4] = [("¼", "walks/coil-0.75t.txt"), ("1¼", "walks/coil_fold2.txt"), ("2", "walks/coil-2.25t.txt"), ("2¾", "walks/coil-3t.txt")];
const TITLE: &str = "10x10x30 Coils";

fn main() {
    let here = format!("{}/tools", trumpet::common::repo());
    let a: Vec<String> = std::env::args().skip(1).collect();
    let out = a.first().cloned().unwrap_or_else(|| format!("{here}/../parts/bore/concept/walk/coil/fold2-long-straight/coils.html"));
    // the geometry has to be set before any walk is read: the cells carry mm
    let mut d = Design::default();
    d.set_bore(10.0);
    d.straight = 30.0;
    let items: Vec<(String, String)> = COILS
        .iter()
        .map(|(lab, f)| (lab.to_string(), std::fs::read_to_string(format!("{here}/{f}")).unwrap_or_else(|e| panic!("{f}: {e}")).trim().to_string()))
        .collect();
    let run = || -> Result<String, String> {
        let sets: Vec<(String, String, lasermade_tools::turn3d::J)> = items.iter().map(|(l, t)| Ok((l.clone(), t.clone(), trumpet::viewer::data_for(&d, t)?))).collect::<Result<_, String>>()?;
        let html = trumpet::viewer::build_sets(&sets, TITLE)?;
        std::fs::write(&out, html).map_err(|e| e.to_string())?;
        let mut o = format!("  {}\n", out.rsplit('/').next().unwrap());
        for (lab, text) in &items {
            let sp = d.specs_for(&walk_text(text)?)?;
            let mm = sp.rec.iter().fold(0.0, |s, r| s + d.extent(r, axis_of(r.out)));
            o.push_str(&format!("    {lab:>2} turns   {:>2} blocks  {:>5}mm  {:>2} sections\n", sp.rec.len(), py::g(mm), sp.groups.len()));
        }
        Ok(o)
    };
    match run() {
        Ok(o) => print!("{o}"),
        Err(e) => trumpet::bore::sys_exit(&format!("ValueError: {e}")),
    }
}
