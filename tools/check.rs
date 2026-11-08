// Copyright (C) 2026 Gernreich
//
//   GPL-3.0-or-later, NOT the CC0 the rest of this repository is under.
//   It is check.py in Rust, which imported snakeboxvar and through it
//   Boxes.py; this reads the same geometry out of snakebox.rs.
//
//   This program is free software: you can redistribute it and/or modify
//   it under the terms of the GNU General Public License as published by
//   the Free Software Foundation, either version 3 of the License, or
//   (at your option) any later version.
//
//   This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU General Public License for more details.
//
//   You should have received a copy of the GNU General Public License
//   along with this program.  If not, see <http://www.gnu.org/licenses/>.
//   A copy is in tools/LICENSE.GPL-3.0.txt.

//! Check a bore's cut files before any of it goes on the machine: check.py,
//! Rust since 2026-11-08. Every check here exists because something got cut
//! and thrown away; the reasons are in check_main.rs and in NOTES.md. Polygon
//! work is geo's (i_overlay) where it was GEOS's: only verdicts and rounded
//! figures come out of it.
use crate::assemble::{closes, sealed, Section};
use crate::bore::{axis_of, dirs, face2d, hosts, inside, neg, pts, BoxMm, Design, Part, Rec, BED_H, BED_W, MIN_FEATURE, THICKNESS};
use crate::geom::{area, buffer_mitre, buffer_round, region, MP};
use crate::snakebox::{Run, Snake};
use geo::{AffineOps, AffineTransform, BooleanOps, BoundingRect, Centroid, Contains, MultiPolygon, Polygon, Rect};
use lasermade_tools::py;
use std::collections::{BTreeSet, HashMap, HashSet};

/// (ok, section, name, detail), in the order noted
pub struct Results(pub Vec<(bool, String, String, String)>);

impl Results {
    fn note(&mut self, ok: bool, section: impl ToString, name: &str, detail: impl ToString) {
        self.0.push((ok, section.to_string(), name.to_string(), detail.to_string()));
    }
}

fn poly(p: &Part) -> MP {
    region(&p.pts)
}

/// The part as it comes off the bed: its outline with its holes taken out.
fn material(p: &Part) -> MP {
    let mut g = poly(p);
    for h in &p.holes {
        g = g.difference(&poly(h));
    }
    g
}

/// What snakebox.rs worked from, built with the design's own arguments.
struct Geo {
    b: Snake,
    runs: Vec<Run>,
    tips: Vec<usize>,
    ports: Vec<usize>,
}

fn geometry_of(d: &Design, args: &[String]) -> Result<Geo, String> {
    let mut all = args.to_vec();
    all.extend(d.common.iter().cloned());
    if d.flat {
        all.push("--pin_length=0".into());
    }
    let b = Snake::from_args(&all)?;
    let cells = b.cells()?;
    let (runs, _) = b.outline(&cells)?;
    let (_, tips, _, ports, _, _) = b.geometry()?;
    Ok(Geo { b, runs, tips, ports })
}

fn bounds(g: &MP) -> (f64, f64, f64, f64) {
    match g.bounding_rect() {
        Some(r) => (r.min().x, r.min().y, r.max().x, r.max().y),
        None => (f64::NAN, f64::NAN, f64::NAN, f64::NAN),
    }
}

fn translate(g: &MP, x: f64, y: f64) -> MP {
    g.affine_transform(&AffineTransform::translate(x, y))
}

fn affine(g: &MP, a: f64, b: f64, d: f64, e: f64, x: f64, y: f64) -> MP {
    g.affine_transform(&AffineTransform::new(a, b, x, d, e, y))
}

/// affinity.scale(g, sx, sy, origin='center')
fn scale_centre(g: &MP, sx: f64, sy: f64) -> MP {
    let (x0, y0, x1, y1) = bounds(g);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    affine(g, sx, 0.0, 0.0, sy, cx - sx * cx, cy - sy * cy)
}

fn union_all(gs: Vec<MP>) -> MP {
    gs.into_iter().reduce(|a, b| a.union(&b)).unwrap_or_else(|| MultiPolygon::new(vec![]))
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> MP {
    MultiPolygon::new(vec![Rect::new(geo::coord! {x: x0, y: y0}, geo::coord! {x: x1, y: y1}).to_polygon()])
}

fn check_section(r: &mut Results, d: &Design, i: usize, args: &[String], parts: &[Part], flat: &(String, String)) -> Result<(), String> {
    let gg = geometry_of(d, args)?;
    let b = &gg.b;
    let plates: Vec<&Part> = parts.iter().filter(|p| p.role == 'P').collect();
    let walls: Vec<&Part> = parts.iter().filter(|p| p.role == 'W').collect();
    let want = gg.runs.len() - gg.tips.len();
    r.note(plates.len() == 2 && walls.len() == want, i, "part count", format!("{} plates + {} walls, expected 2 + {want}", plates.len(), walls.len()));

    if b.pin_length <= 0.0 {
        let mut d2 = d.clone();
        d2.flat = false;
        let coupled = d2.cut(args)?;
        let couplings = [b.plain_in, b.plain_out].iter().filter(|x| !**x).count();
        let differ = parts.iter().zip(&coupled).filter(|(p, q)| p.d != q.d).count();
        let simpler = parts.iter().zip(&coupled).all(|(p, q)| p.pts.len() <= q.pts.len());
        r.note(
            parts.len() == coupled.len() && simpler && if couplings > 0 { differ > 0 } else { differ == 0 },
            i,
            "the flat ends dropped their couplings",
            format!("{differ} of {} part(s) differ from the coupled drawing, {couplings} coupling(s) there to drop", parts.len()),
        );
    }

    for p in parts {
        let g = material(p);
        r.note(g.0.len() == 1, i, "part is one closed piece", format!("{:.1}x{:.1}", p.w, p.h));
        let w = MIN_FEATURE / 2.0;
        let thin = g.difference(&buffer_mitre(&buffer_mitre(&g, -w), w));
        let bad = thin.0.iter().filter(|x| geo::Area::unsigned_area(*x) > 0.02).count();
        r.note(
            bad == 0,
            i,
            &format!("no feature under {} mm", py::g(MIN_FEATURE)),
            if bad == 0 { String::new() } else { format!("{bad} on a {:.1}x{:.1} part", p.w, p.h) },
        );
        r.note(py::max(p.w, p.h) <= BED_W && py::min(p.w, p.h) <= BED_H, i, "part fits the bed", format!("{:.1}x{:.1}", p.w, p.h));
    }

    if plates.len() == 2 {
        let (a, bb) = (poly(plates[0]), poly(plates[1]));
        let ba = bounds(&a);
        let a = translate(&a, -ba.0, -ba.1);
        let bbb = bounds(&bb);
        let bb = translate(&bb, -bbb.0, -bbb.1);
        let mut same = (area(&a) - area(&bb)).abs() < 0.5;
        if same {
            let best = [bb.clone(), scale_centre(&bb, -1.0, 1.0), scale_centre(&bb, 1.0, -1.0), scale_centre(&bb, -1.0, -1.0)]
                .iter()
                .map(|c| {
                    let bc = bounds(c);
                    area(&translate(c, -bc.0, -bc.1).xor(&a))
                })
                .reduce(|x, y| if y < x { y } else { x })
                .unwrap();
            same = best < 1.0;
        }
        let nflat = [&flat.0, &flat.1].iter().filter(|f| !f.is_empty()).count();
        if nflat > 0 {
            let gap = (area(&a) - area(&bb)).abs();
            r.note(gap <= 40.0 * nflat as f64 + 4.0, i, "the plates differ only by the dropped coupling", format!("{gap:.1} mm2 apart, {nflat} end(s) flattened"));
        } else if !gg.ports.is_empty() {
            let gap = (area(&a) - area(&bb)).abs();
            let cell = py::powf(d.block, 2.0);
            r.note(gap <= cell + 4.0, i, "the plates differ only by the ported cell", format!("{gap:.1} mm2 apart, a cell is {cell:.0} mm2"));
        } else {
            r.note(same, i, "the two plates are one part mirrored", format!("{:.1} vs {:.1} mm2", area(&a), area(&bb)));
        }
    }

    let cells = b.cells()?;
    let plate_cells: Option<Vec<(i64, i64)>> = if gg.ports.is_empty() {
        None
    } else {
        let ends = b.tip_runs(&cells, &gg.runs)?;
        Some(if gg.ports.contains(&ends[0]) { cells[1..].to_vec() } else { cells[..cells.len() - 1].to_vec() })
    };
    let (ok, leak, bore) = closes(&cells, &gg.runs, &gg.tips, b.blocksize, b.thickness, plate_cells.as_deref());
    r.note(ok, i, "the section closes round its bore", format!("{leak:.0} mm3 of the {bore:.0} mm3 bore is open to the outside"));
    r.note(
        gg.ports.is_empty(),
        i,
        "no wall finger left unengaged",
        if gg.ports.is_empty() { "" } else { "a port removes a plate over one cell, orphaning the fingers there" },
    );
    Ok(())
}

type Fit = (i64, i64, i64, i64, f64, f64);

/// Every placement of the piece's cells that fits this drawn plate.
fn cells_on(plate: &Part, cells: &[(i64, i64)], s: f64) -> Vec<Fit> {
    let body = poly(plate);
    let bb = bounds(&body);
    let bba = [bb.0, bb.1, bb.2, bb.3];
    let cell_sq = union_all(cells.iter().map(|&(i, j)| rect(i as f64 * s, j as f64 * s, (i + 1) as f64 * s, (j + 1) as f64 * s)).collect());
    let mut fits: Vec<(f64, Fit)> = vec![];
    for (a, b, d, e) in [(1, 0, 0, 1), (-1, 0, 0, 1), (1, 0, 0, -1), (-1, 0, 0, -1), (0, 1, 1, 0), (0, -1, 1, 0), (0, 1, -1, 0), (0, -1, -1, 0)] {
        let u = affine(&cell_sq, a as f64, b as f64, d as f64, e as f64, 0.0, 0.0);
        let ub = bounds(&u);
        let uba = [ub.0, ub.1, ub.2, ub.3];
        for kx in [0, 2] {
            for ky in [1, 3] {
                let x = bba[kx] - uba[kx];
                let y = bba[ky] - uba[ky];
                let miss = area(&translate(&u, x, y).xor(&body));
                fits.push((miss, (a, b, d, e, x, y)));
            }
        }
    }
    let best = fits.iter().map(|f| f.0).reduce(|x, y| if y < x { y } else { x }).unwrap();
    let mut out: Vec<Fit> = vec![];
    for (m, f) in fits {
        if m <= best + 5.0 && !out.iter().any(|g| g.0 == f.0 && g.1 == f.1 && g.2 == f.2 && g.3 == f.3 && g.4 == f.4 && g.5 == f.5) {
            out.push(f);
        }
    }
    out.sort_by(|p, q| (p.0, p.1, p.2, p.3).cmp(&(q.0, q.1, q.2, q.3)).then(p.4.partial_cmp(&q.4).unwrap()).then(p.5.partial_cmp(&q.5).unwrap()));
    out
}

fn sorted_repr(s: &BTreeSet<usize>) -> String {
    py::ints_repr(&s.iter().copied().collect::<Vec<_>>())
}

/// Each mouth is over the block it names, on its cheek, and opens onto bore.
fn check_mouths(r: &mut Results, d: &Design, i: usize, group: &[usize], b: &Snake, parts: &[Part]) -> Result<(), String> {
    let mut asked = d.mouth_at.clone().unwrap_or_default();
    asked.sort();
    let mine: Vec<i64> = asked.iter().copied().filter(|&blk| group[0] as i64 <= blk - 1 && blk - 1 <= group[group.len() - 1] as i64).collect();
    if mine.is_empty() {
        return Ok(());
    }
    let (s, t) = (b.blocksize, b.thickness);
    let cells = b.cells()?;
    let plates: Vec<&Part> = parts.iter().filter(|p| p.role == 'P').collect();
    if plates.len() != 2 {
        return Ok(());
    }
    let k = std::cmp::max(1, cells.len() as i64 - 1) as usize;
    let channel = union_all(
        (0..k)
            .map(|k| {
                let pr = &cells[k..(k + 2).min(cells.len())];
                let mnx = pr.iter().map(|c| c.0).min().unwrap();
                let mny = pr.iter().map(|c| c.1).min().unwrap();
                let mxx = pr.iter().map(|c| c.0).max().unwrap();
                let mxy = pr.iter().map(|c| c.1).max().unwrap();
                rect(mnx as f64 * s + t, mny as f64 * s + t, (mxx + 1) as f64 * s - t, (mxy + 1) as f64 * s - t)
            })
            .collect(),
    );
    let fits: Vec<Vec<Fit>> = plates.iter().map(|p| cells_on(p, &cells, s)).collect();
    let over = |pi: usize, hole: &MP| -> BTreeSet<usize> {
        let c = hole.centroid().unwrap();
        let (cx, cy) = (c.x(), c.y());
        let mut out = BTreeSet::new();
        for f in &fits[pi] {
            for (n, &(ci, cj)) in cells.iter().enumerate() {
                let corners = [(ci as f64 * s, cj as f64 * s), ((ci + 1) as f64 * s, (cj + 1) as f64 * s)];
                let m: Vec<(f64, f64)> = corners.iter().map(|&(x, y)| (f.0 as f64 * x + f.1 as f64 * y + f.4, f.2 as f64 * x + f.3 as f64 * y + f.5)).collect();
                let (x0, x1) = (m[0].0.min(m[1].0), m[0].0.max(m[1].0));
                let (y0, y1) = (m[0].1.min(m[1].1), m[0].1.max(m[1].1));
                if x0 < cx && cx < x1 && y0 < cy && cy < y1 {
                    out.insert(n);
                }
            }
        }
        out
    };
    let holes: Vec<Vec<MP>> = plates.iter().map(|p| p.holes.iter().map(poly).collect()).collect();
    let mut found: HashMap<i64, Vec<usize>> = HashMap::new();
    for &blk in &mine {
        let cell = (blk - 1 - group[0] as i64) as usize;
        let want = asked.iter().position(|&x| x == blk).unwrap() % 2;
        let at: Vec<usize> = (0..2).flat_map(|pi| holes[pi].iter().filter(|h| over(pi, h).contains(&cell)).map(move |_| pi).collect::<Vec<_>>()).collect();
        found.insert(blk, at.clone());
        let wh: Vec<String> = (0..2)
            .map(|pi| {
                let l: Vec<String> = holes[pi].iter().map(|h| sorted_repr(&over(pi, h))).collect();
                format!("P{} has holes over cell(s) {}", pi + 1, if l.is_empty() { "none".to_string() } else { l.join(", ") })
            })
            .collect();
        r.note(at == vec![want], i, "mouth is over the block it names", format!("block {blk} is cell {cell}, asked on P{}; {}", want + 1, wh.join("; ")));
    }
    for pi in 0..2 {
        for h in &holes[pi] {
            let hp = &h.0[0];
            let ok = fits[pi].iter().any(|f| buffer_round(&affine(&channel, f.0 as f64, f.1 as f64, f.2 as f64, f.3 as f64, f.4, f.5), 0.05).contains(hp));
            r.note(ok, i, "mouth opens onto the bore", format!("the hole on P{} over cell(s) {} runs past the airway into a wall or the rim", pi + 1, sorted_repr(&over(pi, h))));
        }
    }
    for w in mine.windows(2) {
        let (p0, p1) = (w[0], w[1]);
        let (f0, f1) = (&found[&p0], &found[&p1]);
        let pl = |v: &Vec<usize>| py::ints_repr(&v.iter().map(|x| x + 1).collect::<Vec<_>>());
        r.note(f0.len() == 1 && f1.len() == 1 && f0 != f1, i, "mouths in a piece on opposite cheeks", format!("blocks {p0} and {p1} on P{} and P{}", pl(f0), pl(f1)));
    }
    let n_holes: usize = holes.iter().map(|h| h.len()).sum();
    r.note(n_holes == mine.len(), i, "one hole per mouth, no more", format!("{n_holes} hole(s) against {} mouth(s)", mine.len()));
    Ok(())
}

/// The two ends that meet must be opposite genders, on the same centre.
fn check_seam(r: &mut Results, d: &Design, i: usize, a_args: &[String], b_args: &[String]) -> Result<(), String> {
    let ga = geometry_of(d, a_args)?;
    let gb = geometry_of(d, b_args)?;
    let (ba, bb) = (&ga.b, &gb.b);
    let ends_a = ba.tip_runs(&ba.cells()?, &ga.runs)?;
    let ends_b = bb.tip_runs(&bb.cells()?, &gb.runs)?;
    let tab_a = if ba.tabs_at_exit { ends_a[1] } else { ends_a[0] };
    let tab_b = if bb.tabs_at_exit { ends_b[1] } else { ends_b[0] };
    let exit_tabs = ends_a[1] == tab_a;
    let entry_tabs = ends_b[0] == tab_b;
    let plain_a = ba.plain_out || ba.port_out;
    let plain_b = bb.plain_in || bb.port_in;
    let sec = format!("{i}-{}", i + 1);
    if ba.pin_length <= 0.0 || bb.pin_length <= 0.0 {
        r.note(ba.pin_length <= 0.0 && bb.pin_length <= 0.0, &sec, "seam is a glued butt joint, flat both sides", "one side has a coupling and the other does not");
    } else {
        let ok = (plain_a && plain_b) || (!plain_a && !plain_b && exit_tabs != entry_tabs);
        r.note(
            ok,
            &sec,
            "seam is one tab side and one slot side",
            format!("exit {}, entry {}", if exit_tabs { "tabs" } else { "notches" }, if entry_tabs { "tabs" } else { "notches" }),
        );
    }
    r.note(
        ba.pin_width == bb.pin_width && ba.blocksize == bb.blocksize && ba.thickness == bb.thickness,
        &sec,
        "both sides agree on bore and tab size",
        format!("{}/{} vs {}/{}", py::repr(ba.blocksize), py::repr(ba.pin_width), py::repr(bb.blocksize), py::repr(bb.pin_width)),
    );
    let at: Vec<String> = [(format!("section {i}"), &ga.ports), (format!("section {}", i + 1), &gb.ports)].iter().filter(|(_, p)| !p.is_empty()).map(|(w, _)| w.clone()).collect();
    let at = at.join(" and ");
    r.note(
        ga.ports.is_empty() && gb.ports.is_empty(),
        &sec,
        "seam has no port",
        if at.is_empty() { String::new() } else { format!("{at} opens through a face plate here, so the joint is a glued butt with no tab") },
    );
    Ok(())
}

/// Each section as (boxes, openings) where the bore actually puts it.
fn sections_3d(d: &Design, rec: &[Rec], groups: &[Vec<usize>]) -> Vec<Section> {
    let allb = d.block_boxes(rec);
    groups
        .iter()
        .map(|gr| {
            let boxes: Vec<BoxMm> = gr.iter().map(|&k| allb[k]).collect();
            let n = boxes.len();
            let ins = neg(dirs(rec[gr[0]].inn));
            (boxes, HashSet::from([(0, ins), (n - 1, dirs(rec[gr[gr.len() - 1]].out))]))
        })
        .collect()
}

/// Put the sections where they belong and test the joints between them.
fn check_seams_3d(r: &mut Results, d: &Design, rec: &[Rec], groups: &[Vec<usize>], t: f64) {
    let secs = sections_3d(d, rec, groups);
    for i in 0..secs.len().saturating_sub(1) {
        let (a, b) = (&secs[i], &secs[i + 1]);
        let outer = [(a.0[0], neg(dirs(rec[groups[i][0]].inn))), (b.0[b.0.len() - 1], dirs(rec[groups[i + 1][groups[i + 1].len() - 1]].out))];
        let (ok, leak, _bore, parts) = sealed(&[a.clone(), b.clone()], t, &outer, 1.0);
        let sec = format!("{}-{}", i + 1, i + 2);
        r.note(ok, &sec, "the joint is closed", format!("{leak:.0} mm3 open to the outside"));
        r.note(parts == 1, &sec, "the bore carries on through the joint", format!("{parts} separate regions"));
    }
    let mouth = (secs[0].0[0], neg(dirs(rec[0].inn)));
    let last = &secs[secs.len() - 1].0;
    let far = (last[last.len() - 1], dirs(rec[rec.len() - 1].out));
    let (ok, leak, bore, parts) = sealed(&secs, t, &[mouth, far], 1.0);
    let s = d.block;
    let turns = rec.iter().filter(|r| r.inn != r.out).count();
    // DELIBERATELY a second implementation of the block length: see check.py's
    // note, kept in NOTES.md. It must not call extent().
    let block_length = |r: &Rec| if r.inn == r.out { d.straight } else { d.block };
    let want = rec.iter().fold(0.0, |acc, r| acc + py::powf(s - 2.0 * t, 2.0) * block_length(r)) + turns as f64 * t * t * (s - 2.0 * t);
    r.note(ok && parts == 1, "all", "the assembled bore is one sealed passage", format!("{leak:.0} mm3 leaking, {parts} regions"));
    r.note((bore - want).abs() < 0.005 * want, "all", "bore volume matches the walk", format!("{bore:.0} mm3 against {want:.0} for {} blocks and {turns} turns", rec.len()));
}

/// Every tab must meet a notch, and every missing side a flat one.
fn check_pairing(r: &mut Results, rec: &[Rec], groups: &[Vec<usize>], norms: &[usize], flats: &[(String, String)], laps: &[(String, String)]) {
    let frame = |gi: usize, which: &str| -> Vec<([i64; 3], String)> {
        let (gr, k) = (&groups[gi], norms[gi]);
        let bore_axis = axis_of(if which == "in" { rec[gr[0]].inn } else { rec[gr[gr.len() - 1]].out });
        let mut sides: Vec<([i64; 3], String)> = vec![];
        for a in 0..3 {
            if a == bore_axis {
                continue;
            }
            for sgn in [-1, 1] {
                let mut dd = [0; 3];
                dd[a] = sgn;
                sides.push((dd, if a == k { "plate" } else { "wall" }.into()));
            }
        }
        let set = |sides: &mut Vec<([i64; 3], String)>, dd: [i64; 3], v: &str| {
            if let Some(e) = sides.iter_mut().find(|e| e.0 == dd) {
                e.1 = v.into();
            }
        };
        if gr.len() == 1 && rec[gr[0]].inn != rec[gr[0]].out {
            let other = if which == "in" { dirs(rec[gr[0]].out) } else { neg(dirs(rec[gr[0]].inn)) };
            set(&mut sides, other, "absent");
        }
        let end = if which == "in" { 0 } else { 1 };
        let want_lap = if end == 0 { &laps[gi].0 } else { &laps[gi].1 };
        if !want_lap.is_empty() {
            for e in sides.iter_mut() {
                if e.1 != "wall" {
                    continue;
                }
                let f2: Vec<i64> = (0..3).filter(|&j| j != k).map(|j| e.0[j]).collect();
                if face2d((f2[0], f2[1])).map(|c| c.to_string()).as_deref() == Some(want_lap.as_str()) {
                    e.1 = "lap".into();
                }
            }
        }
        let want = if end == 0 { &flats[gi].0 } else { &flats[gi].1 };
        if !want.is_empty() {
            for e in sides.iter_mut() {
                if e.1 == "plate" && (0..3).find(|&j| e.0[j] != 0).unwrap() == k {
                    let sgn = if k == 0 || k == 2 { 1 } else { -1 };
                    if (e.0[k] == sgn) == (want == "first") {
                        e.1 = "flat".into();
                    }
                }
            }
        }
        sides
    };
    for i in 0..groups.len().saturating_sub(1) {
        let (a, b) = (frame(i, "out"), frame(i + 1, "in"));
        let mut bad = vec![];
        for (dd, av) in &a {
            let bv = b.iter().find(|e| e.0 == *dd).map(|e| e.1.clone()).unwrap_or_else(|| "absent".into());
            let pair: HashSet<&str> = [av.as_str(), bv.as_str()].into_iter().collect();
            let sub = |s: &[&str]| pair.iter().all(|x| s.contains(x));
            let eq = |s: &[&str]| pair.len() == s.len() && sub(s);
            let ok = sub(&["plate", "wall"]) || eq(&["flat", "absent"]) || eq(&["lap", "absent"]) || eq(&["absent"]) || eq(&["flat"]);
            if !ok {
                let ax = ['x', 'y', 'z'][(0..3).find(|&j| dd[j] != 0).unwrap()];
                let sgn = if dd.iter().sum::<i64>() > 0 { '+' } else { '-' };
                bad.push(format!("{sgn}{ax}: {av} against {bv}"));
            }
        }
        r.note(bad.is_empty(), format!("{}-{}", i + 1, i + 2), "every side of the seam has a mate", bad.join("; "));
    }
}

// --- reading the written sheets ------------------------------------------------

use crate::xml::{parse as parse_xml, El};

type Xf = Box<dyn Fn(f64, f64) -> (f64, f64)>;

fn xf(g: &El) -> Option<(f64, f64, i64)> {
    let r = lasermade_tools::re::rx(r"^translate\(([-\d.]+),([-\d.]+)\)(?:\s*rotate\((\d+)\))?");
    let t = g.get("transform").unwrap_or("");
    let c = lasermade_tools::re::first(&r, t)?;
    let f = |k: usize| lasermade_tools::re::g(&c, k).parse::<f64>().unwrap();
    let rot = c.get(3).map(|m| m.as_str().parse::<i64>().unwrap()).unwrap_or(0);
    Some((f(1), f(2), rot))
}

fn collect(n: &El, f: &dyn Fn(f64, f64) -> (f64, f64), p: &mut Vec<Vec<(f64, f64)>>, l: &mut Vec<Vec<(f64, f64)>>) -> Result<(), String> {
    for ch in &n.kids {
        let tag = ch.local();
        if tag == "g" {
            let t = xf(ch);
            let g2: Xf = match t {
                None => Box::new(|x, y| (x, y)),
                Some((dx, dy, r)) => Box::new(move |x: f64, y: f64| {
                    let (x, y) = match r {
                        90 => (-y, x),
                        180 => (-x, -y),
                        270 => (y, -x),
                        _ => (x, y),
                    };
                    (x + dx, y + dy)
                }),
            };
            collect(ch, &|x, y| {
                let (a, b) = g2(x, y);
                f(a, b)
            }, p, l)?;
        } else if tag == "path" && ch.get("stroke") == Some("#000000") {
            let q: Vec<(f64, f64)> = pts(ch.get("d").unwrap_or(""))?.into_iter().map(|(x, y)| f(x, y)).collect();
            if q.len() > 2 {
                p.push(if q[0] == q[q.len() - 1] { q[..q.len() - 1].to_vec() } else { q });
            }
        } else if tag == "polyline" && ch.get("stroke") == Some("#0000ff") {
            l.push(
                ch.get("points")
                    .unwrap_or("")
                    .split_whitespace()
                    .map(|t| {
                        let (a, b) = t.split_once(',').unwrap();
                        f(py::float(a).unwrap(), py::float(b).unwrap())
                    })
                    .collect(),
            );
        }
    }
    Ok(())
}

fn basename(p: &str) -> String {
    p.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string()
}

/// The written sheets, as the machine will see them.
fn check_sheets(r: &mut Results, d: &Design, folder: &str, want_sections: usize) -> Result<(), String> {
    let mut files: Vec<String> = std::fs::read_dir(folder)
        .map(|it| it.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".svg")).collect())
        .unwrap_or_default();
    files.sort();
    let pat = lasermade_tools::re::rx(r"^(bore[\d.]+-|\d\d_|nest_|recut_)");
    let (mut seen, mut names, mut holes_seen) = (0, vec![], 0);
    let fname = basename(&lasermade_tools::text::normpath(folder));
    for name in files {
        if !lasermade_tools::re::has(&pat, &name) {
            continue;
        }
        seen += 1;
        names.push(name.clone());
        let src = std::fs::read_to_string(crate::bore::pjoin(folder, &name)).map_err(|e| e.to_string())?;
        let root = parse_xml(&src)?;
        let wh = |k: &str| -> f64 {
            let v = root.get(k).unwrap_or("");
            py::float(&v[..v.len().saturating_sub(2)]).unwrap_or(f64::NAN)
        };
        let (w, h) = (wh("width"), wh("height"));
        let (mut p, mut l) = (vec![], vec![]);
        collect(&root, &|x, y| (x, y), &mut p, &mut l)?;
        r.note(w <= BED_W + 1e-6 && h <= BED_H + 1e-6, &name, "sheet fits the bed", format!("{w:.0}x{h:.0}"));
        let gs: Vec<MP> = p.iter().map(|q| region(q)).collect();
        let hole: Vec<bool> = hosts(&p).iter().map(|h| h.is_some()).collect();
        let parts: Vec<usize> = (0..gs.len()).filter(|&i| !hole[i]).collect();
        let holes: Vec<usize> = (0..gs.len()).filter(|&i| hole[i]).collect();
        holes_seen += holes.len();
        let bbs: Vec<(f64, f64, f64, f64)> = gs.iter().map(bounds).collect();
        let mut ov = 0;
        for a in 0..parts.len() {
            for b in a + 1..parts.len() {
                let (x, y) = (bbs[parts[a]], bbs[parts[b]]);
                if x.2.min(y.2) <= x.0.max(y.0) || x.3.min(y.3) <= x.1.max(y.1) {
                    continue;
                }
                if area(&gs[parts[a]].intersection(&gs[parts[b]])) > 1e-6 {
                    ov += 1;
                }
            }
        }
        r.note(ov == 0, &name, "no two parts overlap", format!("{ov} pairs{}", if holes.is_empty() { String::new() } else { format!(", {} hole(s) inside their parts", holes.len()) }));
        let off = l.iter().flatten().filter(|&&(x, y)| !parts.iter().any(|&i| inside(&p[i], x, y)) || holes.iter().any(|&i| inside(&p[i], x, y))).count();
        r.note(off == 0, &name, "engraving on material", format!("{off} points off"));
    }
    r.note(seen > 0, &fname, "folder holds cut files", format!("{seen} matched *.svg"));
    if let Some(m) = &d.mouth_at {
        r.note(holes_seen == m.len(), &fname, "every mouth asked for is cut", format!("{holes_seen} hole(s) against {} --mouth-at block(s)", m.len()));
    }
    if want_sections > 0 {
        let rx = lasermade_tools::re::rx(r"-(\d+)of(\d+)-");
        let (mut nums, mut tot) = (BTreeSet::new(), BTreeSet::new());
        for name in &names {
            if let Some(c) = lasermade_tools::re::first(&rx, name) {
                nums.insert(lasermade_tools::re::g(&c, 1).parse::<i64>().unwrap());
                tot.insert(lasermade_tools::re::g(&c, 2).parse::<i64>().unwrap());
            }
        }
        let all: BTreeSet<i64> = (1..=want_sections as i64).collect();
        let ok = tot == BTreeSet::from([want_sections as i64]) && nums == all;
        let missing: Vec<i64> = all.difference(&nums).copied().collect();
        r.note(
            ok,
            &fname,
            "the sheets are this walk's sections",
            format!(
                "{} of {want_sections}{}{}",
                nums.len(),
                if tot != BTreeSet::from([want_sections as i64]) { format!(", file names say {} sections", py::ints_repr(&tot.iter().copied().collect::<Vec<_>>())) } else { String::new() },
                if missing.is_empty() { String::new() } else { format!(", missing {}", py::ints_repr(&missing)) }
            ),
        );
    }
    Ok(())
}

/// check.main(text, folder): every check, in the order run
pub fn run(d: &Design, text: &str, folder: Option<&str>, o: Option<&mut String>) -> Result<Results, String> {
    let sp = d.specs_for(text)?;
    let mut r = Results(vec![]);
    if let Some(o) = o {
        o.push_str(&format!("{}\n{} blocks, {} sections\n\n", text.trim(), sp.rec.len(), sp.groups.len()));
    }
    for (i, p) in sp.plan.iter().enumerate() {
        let i = i + 1;
        let parts = d.cut(&p.args)?;
        check_section(&mut r, d, i, &p.args, &parts, &p.flat)?;
        let gg = geometry_of(d, &p.args)?;
        check_mouths(&mut r, d, i, &sp.groups[i - 1], &gg.b, &parts)?;
    }
    for i in 0..sp.plan.len().saturating_sub(1) {
        check_seam(&mut r, d, i + 1, &sp.plan[i].args, &sp.plan[i + 1].args)?;
    }
    check_seams_3d(&mut r, d, &sp.rec, &sp.groups, THICKNESS);
    let norms: Vec<usize> = sp.plan.iter().map(|p| p.norm).collect();
    let flats: Vec<(String, String)> = sp.plan.iter().map(|p| p.flat.clone()).collect();
    let laps: Vec<(String, String)> = sp.plan.iter().map(|p| p.lap.clone()).collect();
    check_pairing(&mut r, &sp.rec, &sp.groups, &norms, &flats, &laps);
    if let Some(f) = folder {
        check_sheets(&mut r, d, f, sp.groups.len())?;
    }
    Ok(r)
}

/// Results grouped by check name, first appearance first: (name, ok, failures)
fn by_name(r: &Results) -> Vec<(String, bool, Vec<String>)> {
    let mut by: Vec<(String, bool, Vec<String>)> = vec![];
    for (ok, sec, name, detail) in &r.0 {
        let k = match by.iter().position(|e| &e.0 == name) {
            Some(k) => k,
            None => {
                by.push((name.clone(), true, vec![]));
                by.len() - 1
            }
        };
        if !ok {
            by[k].1 = false;
            by[k].2.push(format!("section {sec}: {detail}"));
        }
    }
    by
}

/// check.main(text, folder, report=False): (checks, failed, failing names)
pub fn summary(d: &Design, text: &str, folder: &str) -> Result<crate::bore::CheckSummary, String> {
    let r = run(d, text, Some(folder), None)?;
    let bad = r.0.iter().filter(|x| !x.0).count();
    Ok((r.0.len(), bad, by_name(&r).into_iter().filter(|e| !e.1).map(|e| (e.0, e.2)).collect()))
}

/// check.main(text, folder): the report; returns the exit status
pub fn report(d: &Design, text: &str, folder: Option<&str>, o: &mut String) -> Result<i32, String> {
    let r = run(d, text, folder, Some(o))?;
    let bad = r.0.iter().filter(|x| !x.0).count();
    o.push_str(&format!("{:<40}result\n", "check"));
    for (name, ok, fails) in by_name(&r) {
        o.push_str(&format!("  {name:<38}{}\n", if ok { "pass" } else { "FAIL" }));
        for f in fails.iter().take(4) {
            o.push_str(&format!("      {f}\n"));
        }
    }
    o.push_str(&format!("\n{} checks, {bad} failed\n", r.0.len()));
    Ok(if bad > 0 { 1 } else { 0 })
}

#[allow(dead_code)]
fn _unused(_: Polygon<f64>) {}
