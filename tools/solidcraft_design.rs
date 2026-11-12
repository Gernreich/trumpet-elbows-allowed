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
//! fixed numbers. The numbers are not derived here by hand: snakebox draws
//! the piece at the defaults and again with each parameter nudged, and the
//! differences are the numbers. Before writing, the design is checked
//! against snakebox drawing it a third time, with every parameter moved:
//! each point must land where snakebox puts it.
//!
//! The fingers follow the sizes (since 2026-11-11; fixed at the defaults'
//! counts before). snakebox draws every finger edge straight for the
//! design, so no outline depends on how many fingers fit, and each finger
//! edge becomes one finger (or slot) box patterned along it, its count,
//! width and first offset written as Boxes' calcFingers at the edge's
//! length (finger_exprs). check_fingers holds those against what snakebox
//! draws at the defaults and at sizes well away from them.
//!
//! The ply follows too (since 2026-11-11). The ply and the pin's length are
//! both 3 mm at the defaults, and one corner beside each pin had been
//! fitted to the ply rather than the pin; the two are nudged further apart
//! now, and check_far holds every point against snakebox with each moved
//! a few tenths.
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
    fingers: Vec<Fing>,
}

/// A finger edge in place: the outline edge it runs along (world), the
/// world axis and sign its fingers stand out or cut in along, and what
/// snakebox drew there.
#[derive(Clone, Debug)]
struct Fing {
    a: [f64; 3],
    b: [f64; 3],
    across: (usize, f64),
    fe: crate::snakebox::FingerEdge,
}

/// Inside the closed polygon (even-odd)?
fn inside(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (b.0 - a.0) / (b.1 - a.1) {
            c = !c;
        }
    }
    c
}

/// A finger edge in a part's own frame to world: its ends, and the way its
/// fingers go (out of the outline for fingers, into it for slots).
fn fing(fe: &crate::snakebox::FingerEdge, poly: &[(f64, f64)], to_w: &dyn Fn((f64, f64)) -> [f64; 3]) -> Result<Fing, String> {
    let (a, b) = (fe.a, fe.b);
    let l = (b.0 - a.0).hypot(b.1 - a.1);
    let d = ((b.0 - a.0) / l, (b.1 - a.1) / l);
    let nl = (-d.1, d.0);
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let left_in = inside((mid.0 + 0.25 * nl.0, mid.1 + 0.25 * nl.1), poly);
    let right_in = inside((mid.0 - 0.25 * nl.0, mid.1 - 0.25 * nl.1), poly);
    if left_in == right_in {
        return Err("a finger edge that is not on its part's outline".into());
    }
    // fingers stand out of the part, slots cut into it
    let side = if left_in != fe.positive { nl } else { (-nl.0, -nl.1) };
    let (wa, ws) = (to_w(a), to_w((a.0 + side.0, a.1 + side.1)));
    let o = (0..3).find(|&j| (ws[j] - wa[j]).abs() > 0.5).ok_or("a finger edge at a slant")?;
    Ok(Fing { a: wa, b: to_w(b), across: (o, (ws[o] - wa[o]).signum()), fe: *fe })
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
        // every finger edge drawn straight: the outlines no longer depend
        // on how many fingers fit, and each edge's fingers are its own feature
        let (parts, fedges) = snake.render_local_with(true)?;
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
            let fingers = fedges[which].iter().map(|(_, fe)| fing(fe, &outer, &|q| lift(&q))).collect::<Result<Vec<_>, _>>()?;
            // extruded from the face at `low` up one ply along page k
            let base = world(page(p0, low))[wk];
            out.push(Placed {
                name: format!("piece {} {} plate", pi + 1, ["first", "mirror"][which]),
                normal: wk,
                face: base,
                dir: ws,
                outline: outer.iter().map(lift).collect(),
                holes: rest.iter().map(|h| h.iter().map(lift).collect()).collect(),
                fingers,
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
            let fingers = fedges[2].iter().filter(|(pi, _)| *pi == w).map(|(_, fe)| fing(fe, &pts, &|q| at(q.0, q.1, 0.0))).collect::<Result<Vec<_>, _>>()?;
            out.push(Placed {
                name: format!("piece {} wall {}", pi + 1, w + 1),
                normal: mw,
                face: o[mw],
                dir: (i[mw] - o[mw]).signum(),
                outline,
                holes: vec![],
                fingers,
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

impl Lin {
    fn minus(&self, o: &Lin) -> Lin {
        Lin(std::array::from_fn(|i| self.0[i] - o.0[i]))
    }
    fn times(&self, k: f64) -> Lin {
        Lin(std::array::from_fn(|i| if self.0[i] == 0.0 { 0.0 } else { self.0[i] * k }))
    }
}

// --- fingers -------------------------------------------------------------------

/// Boxes' calcFingers for an edge L long in ply t, as the design's
/// expressions write it (finger_exprs): fingers 2t wide and 2t apart, as
/// many as fit with a space at each end, the leftover split between the
/// ends; one 2t finger on an edge longer than 3t that holds none, and one
/// L/2 wide on an edge from 0.75t to 3t. (n, width, first finger's offset.)
fn finger_rule(l: f64, t: f64) -> (i64, f64, f64) {
    let n0 = ((l - 2.0 * t) / (4.0 * t) + 1e-9).floor();
    let n = n0.max(1f64.min((l / (0.75 * t) - 1e-9).floor()));
    let w = (2.0 * t).min((l / 2.0).max(2.0 * t * 1f64.min((l / (3.0 * t) - 1e-9).floor())));
    (n as i64, w, (l - n * (2.0 * t + w) + 2.0 * t) / 2.0)
}

/// finger_rule as SolidCraft expressions of an edge's length.
fn finger_exprs(l: &str) -> (String, String, String) {
    let n = format!("max(floor(({l} - 2*ply)/(4*ply) + 1e-9), min(1, floor({l}/(0.75*ply) - 1e-9)))");
    let w = format!("min(2*ply, max({l}/2, 2*ply*min(1, floor({l}/(3*ply) - 1e-9))))");
    let off = format!("({l} - {n}*(2*ply + {w}) + 2*ply)/2");
    (n, w, off)
}

/// The world axis a finger edge runs along, and which way.
fn along(f: &Fing) -> (usize, f64) {
    let a = (0..3).max_by(|&i, &j| (f.b[i] - f.a[i]).abs().total_cmp(&(f.b[j] - f.a[j]).abs())).unwrap();
    (a, (f.b[a] - f.a[a]).signum())
}

/// Check the design's fingers against snakebox's at the defaults and at
/// sizes well away from them (another ply, bore and straight): each edge's
/// length as the design writes it, and finger_rule's count, width and
/// offset at that length, must be what snakebox draws. A part whose
/// outline changes shape at a size is left out there; the defaults and the
/// small moves fit() checks always count.
fn check_fingers(d: &Design, f: &Fitted, e0: &Env, dr: &mut Draws) -> Result<(), String> {
    let cubic = d.cubic();
    let mut envs = vec![*e0];
    for (dt, db, ds) in [(-0.5, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 4.0, 0.0), (0.0, -2.0, 0.0), (-1.0, -3.0, 0.0), (0.0, 0.0, 7.0)] {
        if cubic && ds != 0.0 {
            continue;
        }
        let mut e = *e0;
        e[2] += dt;
        e[0] += db;
        e[1] = if cubic { e[0] } else { e[1] + ds };
        envs.push(e);
    }
    for (k, e) in envs.iter().enumerate() {
        let Some(ps) = dr.at(e).cloned() else {
            if k == 0 {
                return Err("snakebox cannot draw the defaults".into());
            }
            continue;
        };
        for (pi, p) in f.parts.iter().enumerate() {
            if !same_part(p, &ps[pi]) {
                if k == 0 {
                    return Err(format!("{}: drawn twice, two shapes", p.name));
                }
                continue;
            }
            for (j, g) in ps[pi].fingers.iter().enumerate() {
                let (ax, sg) = along(&p.fingers[j]);
                let [la, lb] = &f.fingers[pi][j];
                let l = lb[ax].minus(&la[ax]).times(sg).val(e);
                let drawn = (g.b[ax] - g.a[ax]) * sg;
                let t = e[2];
                let (n, w, off) = finger_rule(l, t);
                let fe = &g.fe;
                let bad = (l - drawn).abs() > 1e-7 || n != fe.n || n < 1 || (w - fe.w).abs() > 1e-7 || (off - fe.off).abs() > 1e-7 || (fe.depth - t).abs() > 1e-9 || (fe.space - 2.0 * t).abs() > 1e-9;
                if bad {
                    return Err(format!(
                        "{} finger edge {}: at ply {t} block {}, the design gives {n} fingers {w} wide from {off} on {l} mm, snakebox {} {} wide from {} on {drawn} mm",
                        p.name,
                        j + 1,
                        e[0],
                        fe.n,
                        fe.w,
                        fe.off
                    ));
                }
            }
        }
    }
    Ok(())
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
    /// each finger edge's two ends
    fingers: Vec<Vec<[[Lin; 3]; 2]>>,
}

fn same_part(p: &Placed, q: &Placed) -> bool {
    p.name == q.name && p.normal == q.normal && p.dir == q.dir && p.outline.len() == q.outline.len() && p.holes.len() == q.holes.len() && p.holes.iter().zip(&q.holes).all(|(h, g)| h.len() == g.len())
        && p.fingers.len() == q.fingers.len()
        && p.fingers.iter().zip(&q.fingers).all(|(f, g)| f.across == g.across && f.fe.positive == g.fe.positive)
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
    for f in &p.fingers {
        v.extend(f.a);
        v.extend(f.b);
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
    // 0.001: the stretched lattice's widths reach snakebox as %g, six
    // figures. The ply and the pin's length 0.05: both are 3 mm at the
    // defaults, so beside each pin snakebox draws a step pin_len - ply
    // long, zero there, and open() drops a step under 0.02 mm. Nudged
    // less, the step stayed hidden and the corner past it was fitted to the
    // ply instead of the pin (until 2026-11-11: a 3.3 mm ply put eleven
    // points of the first trumpet 0.3 mm wrong)
    let delta_of = |i: usize| if i == 2 || i == 5 { 0.05 } else { 0.001 };
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
    let (mut face, mut pts, mut holes, mut fingers) = (vec![], vec![], vec![], vec![]);
    for (pi, p) in b.iter().enumerate() {
        let fb = flatten(p);
        let mut coef = vec![[0.0f64; 8]; fb.len()];
        let mut signs = [0.0f64; 7];
        for &i in &free {
            let mut got = None;
            for sg in [1.0, -1.0] {
                if let Some(ps) = dr.at(&moved(&e0, i, sg * delta_of(i))) {
                    if same_part(p, &ps[pi]) {
                        got = Some((sg, flatten(&ps[pi])));
                        break;
                    }
                }
            }
            let (sg, fp) = got.ok_or_else(|| format!("{}: {} cannot move either way without a finger count changing", p.name, VARS[i]))?;
            signs[i] = sg;
            for (j, c) in coef.iter_mut().enumerate() {
                let x = (fp[j] - fb[j]) / (sg * delta_of(i));
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
            e2 = moved(&e2, i, signs[i] * 3.0 * delta_of(i));
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
        fingers.push(p.fingers.iter().map(|_| [[nx(), nx(), nx()], [nx(), nx(), nx()]]).collect());
    }
    let f = Fitted { parts: b, face, pts, holes, fingers };
    check_fingers(d, &f, &e0, &mut dr)?;
    check_far(d, &f, &e0, &mut dr)?;
    Ok((f, e0))
}

/// Check every point against snakebox with the ply and the pin's length
/// moved well apart, one at a time: the ply 0.3 mm either way (the bore
/// held, so the block moves twice that) and the pin 0.5 mm. A step zero
/// long at the defaults is tenths of a millimetre long there, so a corner
/// fitted to the wrong one of the two shows.
fn check_far(d: &Design, f: &Fitted, e0: &Env, dr: &mut Draws) -> Result<(), String> {
    for (i, by) in [(2, 0.3), (2, -0.3), (5, 0.5), (5, -0.5)] {
        let mut e = *e0;
        e[i] += by;
        if i == 2 {
            e[0] += 2.0 * by;
            if d.cubic() {
                e[1] = e[0];
            }
        }
        let ps = dr.at(&e).cloned().ok_or_else(|| format!("snakebox cannot draw the bore with {} moved {by} mm", VARS[i]))?;
        for (pi, p) in f.parts.iter().enumerate() {
            if !same_part(p, &ps[pi]) {
                return Err(format!("{}: moving {} {by} mm changes its shape", p.name, VARS[i]));
            }
            let drawn = flatten(&ps[pi]);
            let mut lins: Vec<&Lin> = vec![&f.face[pi]];
            lins.extend(f.pts[pi].iter().flatten());
            lins.extend(f.holes[pi].iter().flatten().flatten());
            lins.extend(f.fingers[pi].iter().flatten().flatten());
            for (j, l) in lins.iter().enumerate() {
                if (l.val(&e) - drawn[j]).abs() > 1e-7 {
                    return Err(format!("{} number {j}: with {} moved {by} mm the design puts it at {} and snakebox at {}", p.name, VARS[i], l.val(&e), drawn[j]));
                }
            }
        }
    }
    Ok(())
}

/// The two in-plane world axes of a part, in the order that makes the
/// sketch's normal +normal.
fn plane(normal: usize) -> (usize, usize) {
    [(1, 2), (2, 0), (0, 1)][normal]
}

fn unit_s(a: usize, s: f64) -> [f64; 3] {
    let mut v = [0.0; 3];
    v[a] = s;
    v
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
        // each finger edge: one finger (or slot) as a box, patterned along
        // the edge as many times as finger_exprs counts at its length
        let nk = p.normal;
        let lo = if p.dir > 0.0 { f.face[i].e() } else { format!("{} - ply", f.face[i].e()) };
        for (j, g) in p.fingers.iter().enumerate() {
            let (ax, sg) = along(g);
            let [la, lb] = &f.fingers[i][j];
            let (n, w, off) = finger_exprs(&lb[ax].minus(&la[ax]).times(sg).e());
            let (o, so) = g.across;
            let cut = !g.fe.positive;
            // a slot's box runs 1 mm past the faces it opens, so no face of it lies on the part's
            let over = if cut { 1.0 } else { 0.0 };
            let mut corner: [String; 3] = Default::default();
            let mut size: [String; 3] = Default::default();
            corner[ax] = if sg > 0.0 { format!("{} + {off}", la[ax].e()) } else { format!("{} - {off} - {w}", la[ax].e()) };
            size[ax] = w.clone();
            // across: out of the edge for a finger, in for a slot, one ply;
            // a finger also sinks 0.5 mm into its part and a slot runs 1 mm out
            let back = if cut { over } else { 0.5 };
            corner[o] = if so > 0.0 { format!("{} - {} mm", la[o].e(), nn(back)) } else { format!("{} - ply", la[o].e()) };
            size[o] = format!("ply + {} mm", nn(back));
            // through the ply: 1 mm proud of both faces, fingers too, which
            // the trim below takes back off. A join flush with both faces can
            // fail on a face already split many times ("join cannot preserve
            // the original faces": the expanding spiral at a 16 mm bore,
            // where a finger lines up with two across the plate).
            corner[nk] = format!("{lo} - 1 mm");
            size[nk] = "ply + 2 mm".into();
            let name = format!("{} fingers {}", p.name, j + 1);
            s.bx(&name, corner, size, if cut { "cut" } else { "join" }, &[&p.name]);
            s.pattern(&name, unit_s(ax, sg), format!("max(1, {n})"), format!("2*ply + {w}"));
        }
        if p.fingers.iter().any(|g| g.fe.positive) {
            // the fingers' 1 mm either side of the ply, cut back off
            for (side, at) in [("below", format!("{lo} - 2 mm")), ("above", format!("{lo} + ply"))] {
                let mut corner: [String; 3] = std::array::from_fn(|_| "-5000 mm".to_string());
                let mut size: [String; 3] = std::array::from_fn(|_| "10000 mm".to_string());
                corner[nk] = at;
                size[nk] = "2 mm".into();
                s.bx(&format!("{} trim {side}", p.name), corner, size, "cut", &[&p.name]);
            }
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
            // fingers add their area, slots take theirs away
            let f: f64 = p.fingers.iter().map(|g| g.fe.n as f64 * g.fe.w * g.fe.depth * if g.fe.positive { 1.0 } else { -1.0 }).sum();
            (p.name.clone(), (crate::geom::area(&r) + f) * d.sheet)
        })
        .collect())
}
