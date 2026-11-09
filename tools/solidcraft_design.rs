//! A walk's bore as a parametric SolidCraft design: `viewer WALK
//! --solidcraft FILE` writes it, and SolidCraft builds the STEP file from it.
//!
//! The design is the parts that are cut, not blocks. Every piece is its two
//! face plates and its side walls, each as snakebox draws it for the cut
//! sheet with the burn left out: the plates' fingers, the walls' slots and
//! angled corner fingers, the coupling pins and their slots, laps, flats,
//! ports and mouths. Each part is one body, its outline extruded one ply,
//! set where it sits in the assembled bore. Until 2026-11-08 each piece was
//! one solid body, its blocks less the airway, with no joints.
//!
//! Parameters: the bore and the ply (the block is bore + 2 ply), the
//! straight on a stretched lattice, and the coupling pin's width, play,
//! length and seat. Every point of every outline is a sum of those times
//! fixed numbers; the finger counts are fixed at the defaults' sizes. The
//! numbers are not derived here by hand: snakebox draws the piece at the
//! defaults and again with each parameter nudged, and the differences are
//! the numbers. Before writing, the design is checked against snakebox
//! drawing it a third time, with every parameter moved: each point must
//! land where snakebox puts it.
//!
//! North is +Y, east +X and up +Z, as the 3D page draws it; the page's z
//! points south, so world y = -page z.
use crate::bore::{Design, Rec, Spec};
use crate::snakebox::{Cell, Snake};
use lasermade_tools::solidcraft::{n as nn, Script};

/// The design's independent lengths, in the order Env holds them.
const VARS: [&str; 7] = ["block", "straight", "ply", "pin_w", "pin_play", "pin_len", "pin_seat"];
type Env = [f64; 7];

/// One part in place: its outline and holes in world numbers, the world
/// axis it is extruded along, the face it starts from and which way.
#[derive(Clone, Debug)]
struct Placed {
    name: String,
    normal: usize,
    face: f64,
    dir: f64,
    outline: Vec<[f64; 3]>,
    holes: Vec<Vec<[f64; 3]>>,
}

/// The page frame (x east, y up, z south) to the world (x east, y north, z up).
fn world(p: [f64; 3]) -> [f64; 3] {
    [p[0], -p[2], p[1]]
}
/// A page axis's world axis, and the sign it takes there.
fn world_axis(a: usize) -> (usize, f64) {
    [(0, 1.0), (2, 1.0), (1, -1.0)][a]
}

/// The Design these numbers give: the block, the straight and the sheet.
fn design_at(d: &Design, e: &Env) -> Design {
    let mut d2 = d.clone();
    d2.block = e[0];
    d2.straight = e[1];
    d2.sheet = e[2];
    d2
}

/// snakebox's flags for a piece at these numbers, burn left out.
fn snake_args(d2: &Design, spec: &Spec, e: &Env) -> Vec<String> {
    let mut a = spec.args.clone();
    a.extend(d2.common.iter().cloned());
    a.extend([
        format!("--blocksize={}", e[0]),
        format!("--thickness={}", e[2]),
        "--burn=0".into(),
        format!("--pin_width={}", e[3]),
        format!("--pin_play={}", e[4]),
        format!("--pin_length={}", e[5]),
        format!("--pin_seat={}", e[6]),
    ]);
    if d2.flat {
        a.push("--pin_length=0".into());
    }
    a
}

/// Every part of every piece, placed, at these numbers.
fn placed(d: &Design, text: &str, e: &Env) -> Result<Vec<Placed>, String> {
    let d2 = design_at(d, e);
    let sp = d2.specs_for(text)?;
    let boxes = d2.block_boxes(&sp.rec);
    let t = e[2];
    let mut out = vec![];
    for (pi, spec) in sp.plan.iter().enumerate() {
        let snake = Snake::from_args(&snake_args(&d2, spec, e))?;
        let parts = snake.render_local()?;
        let cells = snake.cells()?;
        let (runs, turns) = snake.outline(&cells)?;
        let (_, tips, _, _, _, _) = snake.geometry()?;
        let n = runs.len();
        let inset: Vec<f64> = (0..n).map(|k| if tips.contains(&k) { 0.0 } else { t }).collect();
        let k = spec.norm;
        let ax: Vec<usize> = (0..3).filter(|&j| j != k).collect();
        let frame = Frame::of(&sp.rec, &spec.group, &boxes, k, &ax, e[0])?;
        // a snake point in mm, and a height above the piece's low face, to the page
        let page = |p: (f64, f64), h: f64| -> [f64; 3] {
            let (a, b) = frame.to_page(p);
            let mut v = [0.0; 3];
            v[ax[0]] = a;
            v[ax[1]] = b;
            v[k] = frame.lo_k + h;
            v
        };
        let vert = |c: Cell| frame.vertex(c);
        if runs[0].1 != (1, 0) || runs[n - 1].1 != (0, -1) {
            return Err(format!("piece {}: the outline does not start at a bottom-left corner", pi + 1));
        }
        // plates: "first" on the side the snake's u x v points to
        let first_hi = k != 1;
        let p0 = {
            let a = vert(runs[0].0);
            (a.0 + inset[n - 1], a.1 + inset[0])
        };
        let (wk, ws) = world_axis(k);
        for (which, paths) in parts.iter().take(2).enumerate() {
            let hi = (which == 0) == first_hi;
            let low = if hi { frame.block_k - t } else { 0.0 };
            let lift = |q: &(f64, f64)| world(page((p0.0 + q.0, p0.1 + q.1), low));
            let (outer, rest) = split_outline(paths)?;
            // extruded from the face at `low` up one ply along page k
            let base = world(page(p0, low))[wk];
            out.push(Placed {
                name: format!("piece {} {} plate", pi + 1, ["first", "mirror"][which]),
                normal: wk,
                face: base,
                dir: ws,
                outline: outer.iter().map(lift).collect(),
                holes: rest.iter().map(|h| h.iter().map(lift).collect()).collect(),
            });
        }
        // walls: one per run that is not a tip, in run order
        let walls: Vec<usize> = (0..n).filter(|r| !tips.contains(r)).collect();
        if parts.len() != 3 || parts[2].len() != walls.len() {
            return Err(format!("piece {}: {} wall outlines for {} walls", pi + 1, parts.get(2).map_or(0, |p| p.len()), walls.len()));
        }
        for (w, &r) in walls.iter().enumerate() {
            let km = (r + n - 1) % n;
            let head = turns[km] as f64 * inset[km];
            let concave = turns[km] < 0 && !tips.contains(&km);
            let along0 = head + if concave { t } else { 0.0 };
            let (a, d, _) = runs[r];
            let a = vert(a);
            let dv = (d.0 as f64, d.1 as f64);
            let m = (-dv.1, dv.0); // inward
            let pts = open(&parts[2][w].0);
            let at = |x: f64, y: f64, off: f64| world(page((a.0 + (along0 + x) * dv.0 + off * m.0, a.1 + (along0 + x) * dv.1 + off * m.1), y));
            // the wall's own normal: the world axis its inward step moves along
            // (a lone turn's frame is turned, so the snake's axes are not the page's)
            let (o, i) = (at(0.0, 0.0, 0.0), at(0.0, 0.0, 1.0));
            let mw = (0..3).find(|&j| (i[j] - o[j]).abs() > 0.5).unwrap();
            let outline: Vec<[f64; 3]> = pts.iter().map(|q| at(q.0, q.1, 0.0)).collect();
            out.push(Placed {
                name: format!("piece {} wall {}", pi + 1, w + 1),
                normal: mw,
                face: o[mw],
                dir: (i[mw] - o[mw]).signum(),
                outline,
                holes: vec![],
            });
        }
    }
    Ok(out)
}

/// The closed path's corners: no repeated last point, no point closer than
/// TINY to the one before it, and no point in line with its neighbours. An
/// edge snakebox draws zero long at the defaults is gone from its drawing
/// there and back the moment a parameter moves, and a there-and-back spike
/// (the flat end draws one) is no outline SolidCraft can extrude; neither
/// changes the part.
fn open(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    const TINY: f64 = 0.02;
    let mut v: Vec<(f64, f64)> = vec![];
    for &q in p {
        if v.last().map_or(true, |l: &(f64, f64)| (l.0 - q.0).hypot(l.1 - q.1) >= TINY) {
            v.push(q);
        }
    }
    while v.len() > 1 && (v[0].0 - v[v.len() - 1].0).hypot(v[0].1 - v[v.len() - 1].1) < TINY {
        v.pop();
    }
    loop {
        let n = v.len();
        let Some(i) = (0..n).find(|&i| {
            let (a, b, c) = (v[(i + n - 1) % n], v[i], v[(i + 1) % n]);
            ((b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)).abs() < 1e-9
        }) else {
            break;
        };
        v.remove(i);
        // a spike leaves two equal neighbours behind
        let n = v.len();
        let j = i % n;
        let k = (j + n - 1) % n;
        if (v[j].0 - v[k].0).hypot(v[j].1 - v[k].1) < TINY {
            v.remove(j);
        }
    }
    v
}

/// A plate's outline (the black path with the widest extent) and its holes.
#[allow(clippy::type_complexity)]
fn split_outline(paths: &[(Vec<(f64, f64)>, (f64, f64, f64))]) -> Result<(Vec<(f64, f64)>, Vec<Vec<(f64, f64)>>), String> {
    let span = |p: &Vec<(f64, f64)>| {
        let xs = p.iter().map(|q| q.0);
        let ys = p.iter().map(|q| q.1);
        (xs.clone().fold(f64::MIN, f64::max) - xs.fold(f64::MAX, f64::min)) + (ys.clone().fold(f64::MIN, f64::max) - ys.fold(f64::MAX, f64::min))
    };
    let mut best = None;
    for (i, (p, rgb)) in paths.iter().enumerate() {
        if *rgb == (0.0, 0.0, 0.0) && best.map_or(true, |b: usize| span(p) > span(&paths[b].0)) {
            best = Some(i);
        }
    }
    let b = best.ok_or("a plate with no outline")?;
    let holes = paths.iter().enumerate().filter(|(i, _)| *i != b).map(|(_, (p, _))| open(p)).collect();
    Ok((open(&paths[b].0), holes))
}

/// Where a piece's snake frame sits on the page.
struct Frame {
    /// page mm of each snake lattice line, u then v
    us: std::collections::BTreeMap<i64, f64>,
    vs: std::collections::BTreeMap<i64, f64>,
    lo_k: f64,
    block_k: f64,
    /// a lone turn's snake frame is its cell turned q quarter turns
    q: i64,
    cell_lo: (f64, f64),
    s: f64,
}

impl Frame {
    fn of(rec: &[Rec], group: &[usize], boxes: &[([f64; 3], [f64; 3])], k: usize, ax: &[usize], s: f64) -> Result<Frame, String> {
        let (mut us, mut vs) = (std::collections::BTreeMap::new(), std::collections::BTreeMap::new());
        let base = rec[group[0]].pos;
        for &j in group {
            let (lo, hi) = boxes[j];
            let (i0, i1) = (rec[j].pos[ax[0]] - base[ax[0]], rec[j].pos[ax[1]] - base[ax[1]]);
            us.insert(i0, lo[ax[0]]);
            us.insert(i0 + 1, hi[ax[0]]);
            vs.insert(i1, lo[ax[1]]);
            vs.insert(i1 + 1, hi[ax[1]]);
        }
        let (lo, hi) = boxes[group[0]];
        let mut q = 0;
        if group.len() == 1 && rec[group[0]].inn != rec[group[0]].out {
            q = crate::bore::lone_turn_quarters(rec, group[0], k);
        }
        Ok(Frame { us, vs, lo_k: lo[k], block_k: hi[k] - lo[k], q, cell_lo: (lo[ax[0]], lo[ax[1]]), s })
    }
    /// A snake lattice vertex, in snake mm from the first cell's low corner.
    fn vertex(&self, c: Cell) -> (f64, f64) {
        if self.q != 0 {
            return (c.0 as f64 * self.s, c.1 as f64 * self.s);
        }
        (self.us[&c.0] - self.us[&0], self.vs[&c.1] - self.vs[&0])
    }
    /// Snake mm to the page's two in-plane coordinates.
    fn to_page(&self, p: (f64, f64)) -> (f64, f64) {
        if self.q == 0 {
            return (self.us[&0] + p.0, self.vs[&0] + p.1);
        }
        // turn back by q quarter turns about the cell's centre
        let h = self.s / 2.0;
        let (mut x, mut y) = (p.0 - h, p.1 - h);
        for _ in 0..self.q.rem_euclid(4) {
            (x, y) = (y, -x);
        }
        (self.cell_lo.0 + h + x, self.cell_lo.1 + h + y)
    }
}

// --- lengths as sums of the design's lengths ---------------------------------

/// c[0] + sum c[i+1] * VARS[i]
#[derive(Clone, Debug, PartialEq)]
struct Lin([f64; 8]);

impl Lin {
    fn val(&self, e: &Env) -> f64 {
        self.0[0] + (0..7).map(|i| self.0[i + 1] * e[i]).sum::<f64>()
    }
    fn e(&self) -> String {
        let mut t: Vec<String> = (0..7).filter(|&i| self.0[i + 1] != 0.0).map(|i| format!("{}*{}", nn(self.0[i + 1]), VARS[i])).collect();
        if self.0[0] != 0.0 {
            t.push(format!("{} mm", nn(self.0[0])));
        }
        if t.is_empty() {
            "0 mm".into()
        } else {
            format!("({})", t.join(" + "))
        }
    }
}

/// The nearest multiple of 1/64, if the number is that close to one.
fn snap(x: f64) -> Option<f64> {
    let r = (x * 64.0).round() / 64.0;
    if (x - r).abs() < 1e-6 {
        Some(if r == 0.0 { 0.0 } else { r })
    } else {
        None
    }
}

/// Every number of every part as a Lin.
struct Fitted {
    parts: Vec<Placed>,
    face: Vec<Lin>,
    pts: Vec<Vec<[Lin; 3]>>,
    holes: Vec<Vec<Vec<[Lin; 3]>>>,
}

fn same_part(p: &Placed, q: &Placed) -> bool {
    p.name == q.name && p.normal == q.normal && p.dir == q.dir && p.outline.len() == q.outline.len() && p.holes.len() == q.holes.len() && p.holes.iter().zip(&q.holes).all(|(h, g)| h.len() == g.len())
}

/// A part's numbers in order: its face, its outline's points, its holes'.
fn flatten(p: &Placed) -> Vec<f64> {
    let mut v = vec![p.face];
    for q in &p.outline {
        v.extend(q);
    }
    for h in &p.holes {
        for q in h {
            v.extend(q);
        }
    }
    v
}

fn base_env(d: &Design) -> Env {
    [d.block, d.straight, d.sheet, d.pin_width(), d.pin_play(), 3.0, 0.2]
}

/// The parts at these numbers, drawn once per distinct set of numbers.
struct Draws<'a> {
    d: &'a Design,
    text: &'a str,
    seen: Vec<(Env, Option<Vec<Placed>>)>,
}

impl Draws<'_> {
    fn at(&mut self, e: &Env) -> Option<&Vec<Placed>> {
        let i = match self.seen.iter().position(|(f, _)| f == e) {
            Some(i) => i,
            None => {
                self.seen.push((*e, placed(self.d, self.text, e).ok()));
                self.seen.len() - 1
            }
        };
        self.seen[i].1.as_ref()
    }
}

/// Fit every part's numbers. A finger count sits on the edge of changing
/// wherever a wall's length is a whole number of finger pitches, so a
/// parameter is nudged up, or down where up would change that part's
/// count; each part takes the way that keeps its own.
fn fit(d: &Design, text: &str) -> Result<(Fitted, Env), String> {
    let e0 = base_env(d);
    let cubic = d.cubic();
    let free: Vec<usize> = (0..7).filter(|&i| !(cubic && i == 1)).collect();
    // 0.001: the stretched lattice's widths reach snakebox as %g, six figures
    let delta = 0.001;
    let mut dr = Draws { d, text, seen: vec![] };
    let b = dr.at(&e0).cloned().ok_or_else(|| placed(d, text, &e0).unwrap_err())?;
    let moved = |e: &Env, i: usize, by: f64| -> Env {
        let mut e = *e;
        e[i] += by;
        if cubic && i == 0 {
            e[1] = e[0];
        }
        e
    };
    let (mut face, mut pts, mut holes) = (vec![], vec![], vec![]);
    for (pi, p) in b.iter().enumerate() {
        let fb = flatten(p);
        let mut coef = vec![[0.0f64; 8]; fb.len()];
        let mut signs = [0.0f64; 7];
        for &i in &free {
            let mut got = None;
            for sg in [1.0, -1.0] {
                if let Some(ps) = dr.at(&moved(&e0, i, sg * delta)) {
                    if same_part(p, &ps[pi]) {
                        got = Some((sg, flatten(&ps[pi])));
                        break;
                    }
                }
            }
            let (sg, fp) = got.ok_or_else(|| format!("{}: {} cannot move either way without a finger count changing", p.name, VARS[i]))?;
            signs[i] = sg;
            for (j, c) in coef.iter_mut().enumerate() {
                let x = (fp[j] - fb[j]) / (sg * delta);
                c[i + 1] = snap(x).ok_or_else(|| format!("{} number {j}: {} has a coefficient of {x}, not a simple number", p.name, VARS[i]))?;
            }
        }
        if cubic {
            // the straight is the block: its part is in the block's
            for c in coef.iter_mut() {
                c[2] = 0.0;
            }
        }
        for (j, c) in coef.iter_mut().enumerate() {
            let rest: f64 = (0..7).map(|i| c[i + 1] * e0[i]).sum();
            let k = fb[j] - rest;
            c[0] = if k.abs() < 1e-9 { 0.0 } else { (k * 1e6).round() / 1e6 };
        }
        // the check: every parameter moved at once, each the way it could
        let mut e2 = e0;
        for &i in &free {
            e2 = moved(&e2, i, signs[i] * 3.0 * delta);
        }
        let q = dr.at(&e2).map(|ps| ps[pi].clone()).filter(|q| same_part(p, q)).ok_or_else(|| format!("{}: moving every parameter at once changes a finger count", p.name))?;
        let f2 = flatten(&q);
        for (j, c) in coef.iter().enumerate() {
            let l = Lin(*c);
            if (l.val(&e2) - f2[j]).abs() > 1e-7 || (l.val(&e0) - fb[j]).abs() > 1e-7 {
                return Err(format!("{} number {j}: the design puts it at {} and snakebox at {}", p.name, l.val(&e2), f2[j]));
            }
        }
        let mut it = coef.into_iter().map(Lin);
        let mut nx = || it.next().unwrap();
        face.push(nx());
        pts.push(p.outline.iter().map(|_| [nx(), nx(), nx()]).collect());
        holes.push(p.holes.iter().map(|h| h.iter().map(|_| [nx(), nx(), nx()]).collect()).collect());
    }
    Ok((Fitted { parts: b, face, pts, holes }, e0))
}

/// The two in-plane world axes of a part, in the order that makes the
/// sketch's normal +normal.
fn plane(normal: usize) -> (usize, usize) {
    [(1, 2), (2, 0), (0, 1)][normal]
}

fn unit(a: usize) -> [f64; 3] {
    let mut v = [0.0; 3];
    v[a] = 1.0;
    v
}

pub fn script(d: &Design, text: &str) -> Result<String, String> {
    let (f, e0) = fit(d, text)?;
    let mut s = Script::new();
    let bore = d.block - 2.0 * d.sheet;
    s.len("bore", format!("{} mm", nn(bore)), "the airway, square, wall face to wall face");
    s.len("ply", format!("{} mm", nn(d.sheet)), "the ply every part is cut from, nominal");
    s.len("block", "bore + 2*ply", "read only: the lattice's pitch, a block's side");
    if d.cubic() {
        s.len("straight", "block", "read only: a straight block is a block on this lattice");
    } else {
        s.len("straight", format!("{} mm", nn(d.straight)), "a straight block's length along the airway; turns stay a block");
    }
    s.len("pin_w", format!("{} mm", nn(e0[3])), "the coupling pin's width (bore_split: 0.48 bore, at least 2 ply, at most bore - 4 mm)");
    s.len("pin_play", format!("{} mm", nn(e0[4])), "the pin slot's clearance per side, measured for the 10 mm bore");
    s.len("pin_len", format!("{} mm", nn(e0[5])), "how far a coupling pin stands out of its wall");
    s.len("pin_seat", format!("{} mm", nn(e0[6])), "how much deeper a pin's slot is than the pin");
    for (i, p) in f.parts.iter().enumerate() {
        let (a, b) = plane(p.normal);
        let sk = s.sketch_on([0.0; 3], unit(a), unit(b), Some(&f.face[i].e()), &format!("{} sketch", p.name));
        let v: Vec<[String; 2]> = f.pts[i].iter().map(|q| [q[a].e(), q[b].e()]).collect();
        s.poly(&v);
        s.finish();
        s.extrude(&sk, "ply", p.dir < 0.0, "new", &[], &p.name);
        for (h, hole) in f.holes[i].iter().enumerate() {
            let hn = format!("{} hole {}", p.name, h + 1);
            let sk = s.sketch_on([0.0; 3], unit(a), unit(b), Some(&f.face[i].e()), &format!("{hn} sketch"));
            let v: Vec<[String; 2]> = hole.iter().map(|q| [q[a].e(), q[b].e()]).collect();
            s.poly(&v);
            s.finish();
            s.extrude(&sk, "ply", p.dir < 0.0, "cut", &[&p.name], &hn);
        }
    }
    s.measure();
    Ok(s.text())
}

/// Each part's volume, as the script should build it: its outline's area,
/// less its holes inside the outline, times the ply.
pub fn volumes(d: &Design, text: &str) -> Result<Vec<(String, f64)>, String> {
    use geo::BooleanOps;
    let ps = placed(d, text, &base_env(d))?;
    Ok(ps
        .iter()
        .map(|p| {
            let (a, b) = plane(p.normal);
            let flat = |q: &Vec<[f64; 3]>| q.iter().map(|v| (v[a], v[b])).collect::<Vec<_>>();
            let mut r = crate::geom::region(&flat(&p.outline));
            for h in &p.holes {
                r = r.difference(&crate::geom::region(&flat(h)));
            }
            (p.name.clone(), crate::geom::area(&r) * d.sheet)
        })
        .collect())
}
