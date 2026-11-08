//! `sizes WALK_NAME OUT.html`: one page holding a design at every block size it
//! is cut at. sizes.py until 2026-11-08.
//!
//! Nothing here is cut at two sizes: every design is on the 16mm block, so this
//! writes no page that is kept. It is left in place because the next design cut
//! at two pitches will want it.
use lasermade_tools::py;
use trumpet::bore::{axis_of, pytitle, walk_text, Design};

/// label, the --blocksize it is cut at; the default first
const SIZES: [(&str, f64); 1] = [("10mm", 16.0)];

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 2 {
        trumpet::bore::sys_exit("usage: sizes WALK_NAME OUT.html");
    }
    let (walk_name, out) = (&a[0], &a[1]);
    let here = format!("{}/tools", trumpet::common::repo());
    let path = format!("{here}/walks/{walk_name}.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| trumpet::bore::sys_exit(&format!("FileNotFoundError: {e}: {}", py::str_repr(&path)))).trim().to_string();
    let run = || -> Result<String, String> {
        let mut d = Design::default();
        let (mut sets, mut stats) = (vec![], vec![]);
        for (label, block) in SIZES {
            // each set built while its own size is set
            d.set_blocksize(block);
            sets.push((label.to_string(), text.clone(), trumpet::viewer::data_for(&d, &text)?));
            let sp = d.specs_for(&walk_text(&text)?)?;
            let mm = sp.rec.iter().fold(0.0, |s, r| s + d.extent(r, axis_of(r.out)));
            stats.push((label, block, sp.rec.len(), mm, sp.groups.len()));
        }
        let title = format!("{} Bore", pytitle(&walk_name.replace('_', " ")));
        std::fs::write(out, trumpet::viewer::build_sets(&sets, &title)?).map_err(|e| e.to_string())?;
        let mut o = format!("  {}\n", out.rsplit('/').next().unwrap());
        for (label, block, n, mm, g) in stats {
            o.push_str(&format!("    {label:>5}  block {}mm  {n} blocks  {}mm  {g} sections\n", py::g(block), py::g(mm)));
        }
        Ok(o)
    };
    match run() {
        Ok(o) => print!("{o}"),
        Err(e) => trumpet::bore::sys_exit(&format!("ValueError: {e}")),
    }
}
