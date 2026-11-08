//! `mcwalk WALK [--out FILE] [--title TEXT]`: draw a walk the way Minecraft
//! draws it, overlaps and all. mcwalk.py until 2026-11-08.
//!
//! bore_split refuses a walk that revisits a cell, which is right for a bore and
//! useless for the question "then why does my build look fine?". Minecraft has
//! no opinion: placing a block in an occupied cell is a no-op, so a walk that
//! crosses itself still comes out one connected tunnel with nothing to mark the
//! fault. This walks permissively, keeps every step in placement order, and
//! draws the cells that result, lighting up the ones placed more than once.
//! Writes mcwalk.html in the working directory unless --out names a file.
mod mcwalk_page;
use lasermade_tools::turn3d::{
    Num::I,
    J,
};
use std::collections::HashMap;
use trumpet::bore::{dirs, parse};
use trumpet::render::{dircol, dirname};

const FAULT: &str = "#ff2d78";

struct Step {
    p: [i64; 3],
    d: char,
}

/// Every step in placement order. No refusals: this is Minecraft's rules.
fn place(text: &str) -> Result<(Vec<Step>, Vec<(usize, char, char)>), String> {
    let toks = parse(text)?;
    let mut h = toks[0].0;
    let mut pos = [0i64; 3];
    let mut steps = vec![Step { p: pos, d: h }];
    let mut reversals = vec![];
    for &(d, n) in &toks {
        if d != h {
            let (a, b) = (dirs(h), dirs(d));
            if a[0] * b[0] + a[1] * b[1] + a[2] * b[2] != 0 {
                reversals.push((steps.len(), h, d));
            }
            h = d;
        }
        for _ in 0..n {
            let v = dirs(h);
            pos = [pos[0] + v[0], pos[1] + v[1], pos[2] + v[2]];
            steps.push(Step { p: pos, d: h });
        }
    }
    Ok((steps, reversals))
}

fn i(x: i64) -> J {
    J::N(I(x))
}
fn s(x: &str) -> J {
    J::S(x.to_string())
}
fn ints(v: &[usize]) -> J {
    J::A(v.iter().map(|&x| i(x as i64)).collect())
}

fn build(text: &str, title: &str) -> Result<String, String> {
    let (steps, reversals) = place(text)?;
    // distinct cells, and which step numbers landed on each
    let mut order: Vec<[i64; 3]> = vec![];
    let mut hits: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
    for (k, st) in steps.iter().enumerate() {
        if !hits.contains_key(&st.p) {
            order.push(st.p);
        }
        hits.entry(st.p).or_default().push(k + 1);
    }
    let faults: Vec<[i64; 3]> = order.iter().copied().filter(|p| hits[p].len() > 1).collect();
    let (placed, distinct) = (steps.len(), order.len());
    let dbl: usize = order.iter().map(|p| hits[p].len() - 1).sum();
    let verdict = if !faults.is_empty() {
        let first = faults.iter().map(|p| hits[p][1]).min().unwrap();
        format!("Minecraft placed <b>{placed}</b> blocks and filled only <b>{distinct}</b> cells. <b>{dbl}</b> of them went into a cell that was already full, starting at block <b>{first}</b> - and a block placed in an occupied cell does nothing at all, so the build never told you.")
    } else {
        format!("{placed} blocks, {distinct} cells, no overlaps. This one is a single tunnel.")
    };
    let mut why = vec!["<p><b>Why the build looks right.</b> Minecraft is filling cells. Re-filling one is a no-op, so the walk simply carries on and you get a connected tunnel with nothing marking the seam.</p>".to_string()];
    if !faults.is_empty() {
        why.push("<p><b>Why it is not a bore.</b> A doubled cell is a junction: the air arrives with two ways out. A trumpet needs one path from mouthpiece to bell, and there is no box section with a hole in four sides.</p>".into());
    }
    if let Some(&(at, from, to)) = reversals.first() {
        why.push(format!("<p><b>And it doubles back.</b> <code>{from} to {to}</code> at block {at} is a 180, so the blocks after it retrace the ones before it exactly - the same cells, filled a second time.</p>"));
    }
    let p3 = |p: &[i64; 3]| J::A(p.iter().map(|&x| i(x)).collect());
    let data = J::obj(vec![
        ("cells", J::A(order.iter().map(|p| J::obj(vec![("p", p3(p)), ("d", s(&steps[hits[p][0] - 1].d.to_string())), ("first", i(hits[p][0] as i64)), ("hits", ints(&hits[p]))])).collect())),
        ("faults", J::A(faults.iter().map(|p| J::obj(vec![("p", p3(p)), ("hits", ints(&hits[p]))])).collect())),
        ("placed", i(placed as i64)),
        ("distinct", i(distinct as i64)),
        ("dircol", J::O("NSEWUD".chars().map(|c| (c.to_string(), s(dircol(c)))).collect())),
        ("fault", s(FAULT)),
        (
            "dirs",
            J::A("NSEWUD".chars().filter(|&c| steps.iter().any(|st| st.d == c)).map(|c| J::obj(vec![("k", s(&c.to_string())), ("name", s(dirname(c))), ("n", i(steps.iter().filter(|st| st.d == c).count() as i64))])).collect()),
        ),
        ("verdict", s(&verdict)),
        ("why", s(&why.concat())),
    ]);
    let mut js = String::new();
    data.dump(&mut js);
    Ok(mcwalk_page::HTML.replace("__DATA__", &js).replace("__TITLE__", title).replace("__WALK__", text.trim()))
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (mut out, mut title, mut words) = ("mcwalk.html".to_string(), "Walk as Minecraft Builds It".to_string(), vec![]);
    let mut k = 0;
    while k < argv.len() {
        if argv[k] == "--out" || argv[k] == "--title" {
            let v = argv.get(k + 1).cloned().unwrap_or_else(|| trumpet::bore::sys_exit("IndexError: list index out of range"));
            if argv[k] == "--out" { out = v } else { title = v }
            k += 2;
        } else {
            words.push(argv[k].clone());
            k += 1;
        }
    }
    match build(&words.join(" "), &title) {
        Ok(html) => {
            let abs = lasermade_tools::text::abspath(&out);
            if let Some(d) = std::path::Path::new(&abs).parent() {
                let _ = std::fs::create_dir_all(d);
            }
            std::fs::write(&out, &html).unwrap_or_else(|e| panic!("{out}: {e}"));
            println!("  wrote {out}  ({:.0} kB)", html.chars().count() as f64 / 1024.0);
        }
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
