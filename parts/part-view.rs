//! A bell or a mouthpiece you can turn: rings stacked, drag to rotate. Rust
//! since 2026-11-07 (part-view.py until then; the move changed no byte of
//! either turn page).
//!
//!     target/release/part-view bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
//!     target/release/part-view mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
//!
//! bell-view and mouthpiece-view draw one fixed isometric each, right for a
//! page and no use for looking at the object. This is the same geometry as a
//! solid you can turn, in the family of the bore viewers in the swept-curve
//! repository (ribbon_view.rs). It reads the ring sizes with bell-view's own
//! sections(), from views.rs, so the two cannot come to disagree about what is
//! on a sheet. DISPLAY ONLY. Not a cut file.
#[path = "common.rs"]
mod common;
#[path = "part_view_page.rs"]
mod page;
#[path = "views.rs"]
mod views;
use lasermade_tools::text;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let files: Vec<&String> = a.iter().filter(|x| !x.starts_with('-')).collect();
    if files.is_empty() {
        println!("target/release/part-view bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg");
        std::process::exit(1);
    }
    let mut n = 28;
    for x in &a {
        if let Some(v) = x.strip_prefix("--facets=") {
            n = common::int(v);
        }
    }
    let src = files[0];
    let d = views::data_for(src, n.max(0) as usize);
    let base = text::pathstr(src).rsplit('/').next().unwrap_or("").replace("-cut-files.svg", "");
    let kind = if base.contains("mouthpiece") { "Mouthpiece" } else { "Bell" };
    let title = format!("{kind}: {base}");
    // Beside the part's GENERATOR, not beside the source: cut-files/ holds cut
    // files and nothing else, and a turn page is not one.
    let out = match files.get(1) {
        Some(o) => o.to_string(),
        None => {
            let abs = text::abspath(src);
            let dir = abs.rsplit_once('/').map(|(p, _)| p.to_string()).unwrap_or_default();
            let parent = if dir.rsplit('/').next() == Some("cut-files") { dir.rsplit_once('/').map(|(p, _)| p.to_string()).unwrap_or_default() } else { dir };
            format!("{parent}/{base}-turn.html")
        }
    };
    let mut data = String::new();
    d.dump(&mut data);
    common::write(&out, &page::HTML.replace("__TITLE__", &title).replace("__DATA__", &data));
    let quads = d.get("Q").map_or(0, |q| q.arr().len());
    let rings = d.get("rings").map_or(0, |r| r.num().f() as i64);
    let shown = out.rsplit('/').next().unwrap_or("");
    println!("  {shown:<58}{rings:>3} rings, {quads} quads, drag to turn");
}
