// Copyright (C) 2026 Gernreich
//
//   GPL-3.0-or-later, NOT the CC0 the rest of this repository is under.
//   It is snakeboxvar.py and the part of Boxes.py (Florian Festi's,
//   GPL-3.0-or-later) that it draws through, in Rust, so it is a work
//   derived from both.
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

//! Open-ended tube following a polyomino snake, one unit cube per cell:
//! snakeboxvar.py's SnakeBoxVar, and the turtle, finger joints, polygon wall,
//! path joining and SVG path writing of Boxes.py that it used. Until
//! 2026-11-08 bore_split ran Boxes.py's own `scripts/boxes SnakeBoxVar` for
//! every piece; this draws the same paths, to the byte, without it, but
//! for one deliberate change (2026-11-09): a plain plate border shorter
//! than its inside-corner allowances is drawn straight (see polygon_wall).
//!
//! Both ends are open. The end frame is a square annulus of width
//! `thickness`: the two face plates cover the middle of the top and bottom
//! sides, the two side walls cover the full left and right sides. That
//! partition is only 2-fold symmetric, but the annulus *region* is 4-fold
//! symmetric, so one tab centred on each of the four sides mates with the
//! matching notches in any of the four rotations. One end carries the four
//! tabs, the other the notches.
//!
//! Only what SnakeBoxVar reaches is here: rectangular fingers, square inner
//! corners (`--inner_corners=corner`), no tabs, no labels, no text.
use lasermade_tools::py::{self, mcos as cos, msin as sin, mtan as tan};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::f64::consts::PI;

const EPS: f64 = 1e-4;
const PADDING: f64 = 10.0;

/// Python's % on floats
pub fn pymod(a: f64, b: f64) -> f64 {
    let m = a % b;
    if m != 0.0 {
        if (b < 0.0) != (m < 0.0) {
            return m + b;
        }
        m
    } else {
        0.0f64.copysign(b)
    }
}

/// Python's // on floats (CPython's float_floor_div)
pub fn floordiv(vx: f64, wx: f64) -> f64 {
    let mut m = vx % wx;
    let mut div = (vx - m) / wx;
    if m != 0.0 {
        if (wx < 0.0) != (m < 0.0) {
            m += wx;
            div -= 1.0;
        }
    }
    let _ = m;
    if div != 0.0 {
        let mut f = div.floor();
        if div - f > 0.5 {
            f += 1.0;
        }
        f
    } else {
        0.0f64.copysign(vx / wx)
    }
}

// --- affine ------------------------------------------------------------------

/// The `affine` package's Affine: (a, b, c, d, e, f), x' = a x + b y + c.
#[derive(Clone, Copy, Debug)]
pub struct Affine(f64, f64, f64, f64, f64, f64);

impl Affine {
    fn translation(x: f64, y: f64) -> Affine {
        Affine(1.0, 0.0, x, 0.0, 1.0, y)
    }
    fn scale(sx: f64, sy: f64) -> Affine {
        Affine(sx, 0.0, 0.0, 0.0, sy, 0.0)
    }
    fn rotation(angle: f64) -> Affine {
        let (ca, sa) = cos_sin_deg(angle);
        Affine(ca, -sa, 0.0, sa, ca, 0.0)
    }
    fn mul(&self, o: &Affine) -> Affine {
        let Affine(sa, sb, sc, sd, se, sf) = *self;
        let Affine(oa, ob, oc, od, oe, of) = *o;
        Affine(sa * oa + sb * od, sa * ob + sb * oe, sa * oc + sb * of + sc, sd * oa + se * od, sd * ob + se * oe, sd * oc + se * of + sf)
    }
    fn apply(&self, (vx, vy): (f64, f64)) -> (f64, f64) {
        let Affine(sa, sb, sc, sd, se, sf) = *self;
        (vx * sa + vy * sb + sc, vx * sd + vy * se + sf)
    }
    /// The map back: a page point to the frame this one maps from.
    fn inverse(&self) -> Affine {
        let Affine(a, b, c, d, e, f) = *self;
        let det = a * e - b * d;
        let (ia, ib, id, ie) = (e / det, -b / det, -d / det, a / det);
        Affine(ia, ib, -(ia * c + ib * f), id, ie, -(id * c + ie * f))
    }
}

/// cos and sin of an angle in degrees, exact at the multiples of 90
fn cos_sin_deg(deg: f64) -> (f64, f64) {
    let deg = pymod(deg, 360.0);
    if deg == 90.0 {
        return (0.0, 1.0);
    }
    if deg == 180.0 {
        return (-1.0, 0.0);
    }
    if deg == 270.0 {
        return (0.0, -1.0);
    }
    let rad = py::radians(deg);
    (cos(rad), sin(rad))
}

fn points_equal(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    (x1 - x2).abs() < EPS && (y1 - y2).abs() < EPS
}

// --- the surface: paths, parts, joining ---------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    M,
    L,
    C,
}

/// One path command. `tup` marks one faster_edges rewrote: Python stores
/// those as tuples, and a tuple never equals a list, which is what its
/// duplicate filter compares.
#[derive(Clone, Copy, Debug)]
struct Cmd {
    k: Kind,
    v: [f64; 6],
    tup: bool,
}

impl Cmd {
    fn xy(&self) -> (f64, f64) {
        (self.v[0], self.v[1])
    }
    fn same(&self, o: &Cmd) -> bool {
        if self.tup != o.tup || self.k != o.k {
            return false;
        }
        let n = if self.k == Kind::C { 6 } else { 2 };
        self.v[..n] == o.v[..n]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Params {
    rgb: (f64, f64, f64),
    lw: f64,
}

#[derive(Debug)]
struct Path {
    cmds: Vec<Cmd>,
    p: Params,
    /// the part's own frame when it was stroked: render_local() reads the
    /// path back in it, for the SolidCraft design
    frame: Affine,
}

#[derive(Default)]
struct Part {
    pathes: Vec<Path>,
    path: Vec<Cmd>,
}

impl Part {
    fn append(&mut self, c: Cmd) {
        self.path.push(c);
    }
    fn stroke(&mut self, p: Params, frame: Affine) {
        if self.path.is_empty() {
            return;
        }
        let (x0, y0) = self.path[0].xy();
        let (xl, yl) = self.path[self.path.len() - 1].xy();
        if !points_equal(x0, y0, xl, yl) {
            for q in self.pathes.iter_mut().rev() {
                let (x1, y1) = q.cmds[q.cmds.len() - 1].xy();
                if points_equal(x0, y0, x1, y1) && q.p == p {
                    q.cmds.extend_from_slice(&self.path[1..]);
                    self.path.clear();
                    return;
                }
            }
        }
        self.pathes.push(Path { cmds: std::mem::take(&mut self.path), p, frame });
    }
    fn move_to(&mut self, x: f64, y: f64) {
        let m = Cmd { k: Kind::M, v: [x, y, 0.0, 0.0, 0.0, 0.0], tup: false };
        if self.path.is_empty() {
            self.path.push(m);
        } else if self.path[self.path.len() - 1].k == Kind::M {
            let n = self.path.len();
            self.path[n - 1] = m;
        } else {
            let (x0, y0) = self.path[self.path.len() - 1].xy();
            if !points_equal(x0, y0, x, y) {
                self.path.push(m);
            }
        }
    }
}

struct Surface {
    parts: Vec<Part>,
    cur: usize,
    count: usize,
}

impl Surface {
    fn new() -> Surface {
        Surface { parts: vec![Part::default()], cur: 0, count: 0 }
    }
    fn new_part(&mut self) {
        if !self.parts.is_empty() && self.parts[self.parts.len() - 1].pathes.is_empty() {
            return;
        }
        self.parts.push(Part::default());
        self.cur = self.parts.len() - 1;
    }
    fn append(&mut self, c: Cmd) -> Result<(), String> {
        self.count += 1;
        if self.count > 100000 {
            return Err("Too many lines".into());
        }
        self.parts[self.cur].append(c);
        Ok(())
    }
}

// --- the drawing context -----------------------------------------------------

struct Ctx {
    s: Surface,
    stack: Vec<(Affine, (f64, f64), f64, (f64, f64, f64), (f64, f64))>,
    m: Affine,
    xy: (f64, f64),
    mxy: (f64, f64),
    lw: f64,
    rgb: (f64, f64, f64),
    /// the frame the part being drawn is laid out in (see Path::frame)
    frame: Affine,
}

impl Ctx {
    fn save(&mut self) {
        self.stack.push((self.m, self.xy, self.lw, self.rgb, self.mxy));
        self.xy = (0.0, 0.0);
    }
    fn restore(&mut self) {
        let (m, xy, lw, rgb, mxy) = self.stack.pop().expect("restore without save");
        (self.m, self.xy, self.lw, self.rgb, self.mxy) = (m, xy, lw, rgb, mxy);
    }
    fn translate(&mut self, x: f64, y: f64) {
        self.m = self.m.mul(&Affine::translation(x, y));
        self.xy = (0.0, 0.0);
    }
    fn scale(&mut self, sx: f64, sy: f64) {
        self.m = self.m.mul(&Affine::scale(sx, sy));
    }
    fn rotate(&mut self, r: f64) {
        self.m = self.m.mul(&Affine::rotation(180.0 * r / PI));
    }
    fn add_move(&mut self) {
        let (x, y) = self.mxy;
        let c = self.s.cur;
        self.s.parts[c].move_to(x, y);
    }
    fn move_to(&mut self, x: f64, y: f64) {
        self.xy = (x, y);
        self.mxy = self.m.apply(self.xy);
    }
    fn line_to(&mut self, x: f64, y: f64) -> Result<(), String> {
        self.add_move();
        let (x1, y1) = self.mxy;
        self.xy = (x, y);
        self.mxy = self.m.apply(self.xy);
        let (x2, y2) = self.mxy;
        if !points_equal(x1, y1, x2, y2) {
            self.s.append(Cmd { k: Kind::L, v: [x2, y2, 0.0, 0.0, 0.0, 0.0], tup: false })?;
        }
        Ok(())
    }
    fn arc(&mut self, xc: f64, yc: f64, radius: f64, angle1: f64, angle2: f64) -> Result<(), String> {
        if (angle1 - angle2).abs() < EPS || radius < EPS {
            return Ok(());
        }
        let (x1, y1) = (radius * cos(angle1) + xc, radius * sin(angle1) + yc);
        let (x4, y4) = (radius * cos(angle2) + xc, radius * sin(angle2) + yc);
        let (ax, ay, bx, by) = (x1 - xc, y1 - yc, x4 - xc, y4 - yc);
        let q1 = ax * ax + ay * ay;
        let q2 = q1 + ax * bx + ay * by;
        let k2 = 4.0 / 3.0 * (py::powf(2.0 * q1 * q2, 0.5) - q2) / (ax * by - ay * bx);
        let (x2, y2) = (xc + ax - k2 * ay, yc + ay + k2 * ax);
        let (x3, y3) = (xc + bx + k2 * by, yc + by - k2 * bx);
        let (mx2, my2) = self.m.apply((x2, y2));
        let (mx3, my3) = self.m.apply((x3, y3));
        let (mx4, my4) = self.m.apply((x4, y4));
        self.add_move();
        self.s.append(Cmd { k: Kind::C, v: [mx4, my4, mx2, my2, mx3, my3], tup: false })?;
        self.xy = (x4, y4);
        self.mxy = (mx4, my4);
        Ok(())
    }
    fn stroke(&mut self) {
        let p = Params { rgb: self.rgb, lw: self.lw };
        let c = self.s.cur;
        self.s.parts[c].stroke(p, self.frame);
        self.xy = (0.0, 0.0);
    }
}

// --- edges -------------------------------------------------------------------

/// FingerJointSettings, rectangular style: what a deepcopy carries, angle included
#[derive(Clone, Copy, Debug)]
struct Fs {
    thickness: f64,
    space: f64,
    finger: f64,
    surroundingspaces: f64,
    play: f64,
    extra_length: f64,
    angle: f64,
}

impl Fs {
    fn finger_length(&self) -> (f64, f64) {
        let angle = self.angle;
        if angle >= 90.0 || angle <= -90.0 {
            return (self.thickness + self.extra_length, 0.0);
        }
        if angle < 0.0 {
            return (sin(py::radians(-angle)) * self.thickness + self.extra_length, 0.0);
        }
        let a = 90.0 - (180.0 - angle) / 2.0;
        let fl = self.thickness * tan(py::radians(a));
        let b = 90.0 - 2.0 * a;
        let spacerecess = -sin(py::radians(b)) * fl;
        (fl + self.extra_length, spacerecess)
    }
    /// angle=: setValues reaches a plain attribute, and checkValues runs
    fn set_angle(&mut self, a: f64) -> Result<(), String> {
        self.angle = a;
        if (self.space + self.finger).abs() < 0.1 {
            return Err("FingerJointSettings: space + finger must not be close to zero".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Edge {
    /// 'e'
    Plain,
    /// 'f' (positive) or 'F'
    Finger(bool, Fs),
}

impl Edge {
    fn start_width(&self) -> f64 {
        match self {
            Edge::Plain => 0.0,
            Edge::Finger(pos, s) => {
                let w = s.finger_length();
                if *pos {
                    w.1
                } else {
                    w.0
                }
            }
        }
    }
    fn end_width(&self) -> f64 {
        self.start_width()
    }
    fn margin(&self) -> f64 {
        match self {
            Edge::Plain => 0.0,
            Edge::Finger(pos, s) => {
                if *pos {
                    let w = s.finger_length();
                    w.0 - w.1
                } else {
                    0.0
                }
            }
        }
    }
    fn spacing(&self) -> f64 {
        self.start_width() + self.margin()
    }
}

// --- the Boxes turtle ----------------------------------------------------------

pub struct Settings {
    pub thickness: f64,
    pub burn: f64,
    pub spacing: (f64, f64),
}

struct Turtle {
    ctx: Ctx,
    thickness: f64,
    burn: f64,
    spacing: f64,
    fs: Fs,
}

impl Turtle {
    fn new(s: &Settings) -> Turtle {
        let t = s.thickness;
        let fs = Fs { thickness: t, space: 2.0 * t, finger: 2.0 * t, surroundingspaces: 2.0, play: 0.0 * t, extra_length: 0.0 * t, angle: 90.0 };
        let mut ctx = Ctx {
            s: Surface::new(),
            stack: vec![],
            m: Affine::translation(0.0, 0.0),
            xy: (0.0, 0.0),
            mxy: (0.0, 0.0),
            lw: 0.0,
            rgb: (0.0, 0.0, 0.0),
            frame: Affine::translation(0.0, 0.0),
        };
        ctx.mxy = ctx.m.apply(ctx.xy);
        ctx.lw = py::max(2.0 * s.burn, 0.05);
        ctx.rgb = (0.0, 0.0, 0.0);
        let spacing = 2.0 * s.burn + s.spacing.0 * t + s.spacing.1;
        Turtle { ctx, thickness: t, burn: s.burn, spacing, fs }
    }

    fn move_to(&mut self, x: f64, y: f64, degrees: f64) {
        self.ctx.move_to(0.0, 0.0);
        self.ctx.translate(x, y);
        self.ctx.rotate(degrees * PI / 180.0);
        self.ctx.move_to(0.0, 0.0);
    }

    fn continue_direction(&mut self, angle: f64) {
        let (x, y) = self.ctx.xy;
        self.ctx.translate(x, y);
        self.ctx.rotate(angle);
    }

    fn edge(&mut self, length: f64) -> R {
        self.ctx.move_to(0.0, 0.0);
        self.ctx.line_to(length, 0.0)?;
        let (x, y) = self.ctx.xy;
        self.ctx.translate(x, y);
        Ok(())
    }

    fn corner(&mut self, degrees: f64, radius: f64) -> R {
        let rad = degrees * PI / 180.0;
        if (radius > 0.5 * self.burn && degrees.abs() > 36.0) || degrees.abs() > 100.0 {
            let steps = (degrees.abs() / 36.0) as i64 + 1;
            for _ in 0..steps {
                self.corner(degrees / steps as f64, radius)?;
            }
            return Ok(());
        }
        let b = self.burn;
        if degrees > 0.0 {
            self.ctx.arc(0.0, radius + b, radius + b, -0.5 * PI, rad - 0.5 * PI)?;
        } else if radius > b {
            self.ctx.arc(0.0, -(radius - b), radius - b, 0.5 * PI, rad + 0.5 * PI)?;
        } else {
            self.ctx.arc(0.0, b - radius, b - radius, -0.5 * PI, -0.5 * PI + rad)?;
        }
        self.continue_direction(rad);
        Ok(())
    }

    fn polyline(&mut self, args: &[f64]) -> R {
        for (i, &a) in args.iter().enumerate() {
            if i % 2 == 1 {
                self.corner(a, 0.0)?;
            } else {
                self.edge(a)?;
            }
        }
        Ok(())
    }

    fn edge_corner(&mut self, e1: &Edge, e2: &Edge, angle: f64) -> R {
        self.edge(e2.start_width() * tan(py::radians(angle / 2.0)))?;
        self.corner(angle, 0.0)?;
        self.edge(e1.end_width() * tan(py::radians(angle / 2.0)))
    }

    fn draw(&mut self, e: &Edge, length: f64) -> R {
        match e {
            Edge::Plain => self.edge(length),
            Edge::Finger(positive, s) => self.finger(*positive, s, length),
        }
    }

    fn finger(&mut self, positive: bool, st: &Fs, length: f64) -> R {
        let (mut s, mut f) = (st.space, st.finger);
        let thickness = st.thickness;
        let play = st.play;
        // calcFingers
        let mut fingers = floordiv(length - (st.surroundingspaces - 1.0) * s, s + f) as i64;
        if fingers == 0 && length > f + 1.0 * st.thickness {
            fingers = 1;
        }
        if f == 0.0 {
            fingers = 0;
        }
        let mut leftover = length - fingers as f64 * (s + f) + s;
        if fingers <= 0 {
            fingers = 0;
            leftover = length;
        }
        if fingers == 0 && f != 0.0 && leftover > 0.75 * thickness && leftover > 4.0 * play {
            fingers = 1;
            leftover /= 2.0;
            f = leftover;
        }
        if !positive {
            f += play;
            s -= play;
            leftover -= play;
        }
        self.edge(leftover / 2.0)?;
        let (l1, l2) = st.finger_length();
        let h = l1 - l2;
        for i in 0..fingers {
            if i != 0 {
                self.edge(s)?;
            }
            if positive {
                self.polyline(&[0.0, -90.0, h, 90.0, f, 90.0, h, -90.0])?;
            } else {
                self.polyline(&[0.0, 90.0, h, -90.0, f, -90.0, h, 90.0])?;
            }
        }
        self.edge(leftover / 2.0)
    }

    fn new_part(&mut self) {
        self.ctx.s.new_part();
    }

    /// Boxes.move(); returns nothing, since no caller asks for "only"
    fn place(&mut self, x: f64, y: f64, terms: &[&str], before: bool) {
        let (mut x, mut y) = (x + self.spacing, y + self.spacing);
        if terms.contains(&"rotated") {
            (x, y) = (y, x);
        }
        if !before {
            self.ctx.stroke();
            self.ctx.restore();
        }
        for &term in terms {
            let (mx, my, mbp): (f64, f64, Option<bool>) = match term {
                "up" => (0.0, y, Some(false)),
                "down" => (0.0, -y, Some(true)),
                "left" => (-x, 0.0, Some(true)),
                "right" => (x, 0.0, Some(false)),
                _ => (0.0, 0.0, None),
            };
            let mbp_t = mbp == Some(true);
            if mbp_t && before {
                self.move_to(mx, my, 0.0);
            } else if !mbp_t && !before {
                self.move_to(mx, my, 0.0);
            }
        }
        if before {
            self.ctx.save();
            if terms.contains(&"mirror") {
                self.move_to(x, 0.0, 0.0);
                self.ctx.scale(-1.0, 1.0);
            }
            self.move_to(self.spacing / 2.0, self.spacing / 2.0, 0.0);
        }
        self.new_part();
    }

    fn close_polygon_ext(&self, borders: &[f64], edges: &[Edge]) -> (f64, f64, f64, f64) {
        let (mut posx, mut posy) = (0.0f64, 0.0f64);
        let mut ext = [0.0f64; 4];
        let mut angle = 0.0f64;
        let mut nb: Vec<f64> = vec![];
        for (i, &val) in borders.iter().enumerate() {
            if i % 2 == 1 {
                nb.push(val);
            } else {
                let e = edges[(i / 2) % edges.len()];
                let margin = e.margin();
                if margin != 0.0 {
                    nb.extend_from_slice(&[0.0, -90.0, margin, 90.0, val, 90.0, margin, -90.0, 0.0]);
                } else {
                    nb.push(val);
                }
            }
        }
        let cp = |ext: &mut [f64; 4], x: f64, y: f64| {
            ext[0] = py::min(ext[0], x);
            ext[1] = py::min(ext[1], y);
            ext[2] = py::max(ext[2], x);
            ext[3] = py::max(ext[3], y);
        };
        for (i, &b) in nb.iter().enumerate() {
            if i % 2 == 1 {
                angle = pymod(angle + b, 360.0);
                continue;
            }
            posx += b * cos(py::radians(angle));
            posy += b * sin(py::radians(angle));
            cp(&mut ext, posx, posy);
        }
        (ext[0], ext[1], ext[2], ext[3])
    }

    /// polygonWall with a callback by edge number; `mirror` for "mirror right"
    fn polygon_wall(&mut self, borders: &[f64], edges: &[Edge], mirror: bool, mut cb: Option<&mut dyn FnMut(&mut Turtle, usize) -> R>) -> R {
        let t = self.thickness;
        let (minx, miny, maxx, maxy) = self.close_polygon_ext(borders, edges);
        let (tw, th) = (maxx - minx, maxy - miny);
        let terms: &[&str] = if mirror { &["mirror", "right"] } else { &["right"] };
        self.place(tw, th, terms, true);
        self.move_to(-minx, -miny, 0.0);
        self.ctx.frame = self.ctx.m;
        let mut lc = 0.0f64;
        let mut i = 0;
        while i < borders.len() {
            if let Some(f) = cb.as_mut() {
                let b = self.burn;
                self.ctx.save();
                self.move_to(0.0, b, 0.0);
                f(self, i / 2)?;
                self.ctx.restore();
                self.ctx.move_to(0.0, 0.0);
            }
            let lc_before = lc;
            let mut l = borders[i] - lc;
            let next_angle = borders[i + 1];
            lc = if next_angle < 0.0 { t * tan(py::radians(-next_angle / 2.0)) } else { 0.0 };
            l -= lc;
            let e = edges[(i / 2) % edges.len()];
            if l < 0.0 && matches!(e, Edge::Plain) {
                // a plain border shorter than its inside-corner allowances
                // (a 2 mm step beside a pin or a turn's corner): polygonWall
                // drew it back and forth, -1 mm then +3, and the laser cut
                // the 0.93 mm stub that left; drawn straight it ends at the
                // same point
                self.edge(borders[i])?;
            } else {
                self.edge(lc_before)?;
                self.draw(&e, l)?;
                self.edge(lc)?;
            }
            self.corner(next_angle, 0.0)?;
            i += 2;
        }
        self.place(tw, th, terms, false);
        Ok(())
    }

    /// rectangularHole, with @restore and @holeCol round it
    fn rectangular_hole(&mut self, x: f64, y: f64, dx: f64, dy: f64, color: (f64, f64, f64)) -> R {
        self.ctx.save();
        let pt = self.ctx.xy;
        self.ctx.stroke();
        self.ctx.save();
        self.ctx.rgb = color;
        let r = py::min(py::min(0.0, dx / 2.0), dy / 2.0);
        let (xs, ys) = (x, y - dy / 2.0);
        self.move_to(xs, ys + self.burn, 180.0);
        self.edge(dx / 2.0 - r)?;
        for d in [dy, dx, dy, dx / 2.0 + r] {
            self.corner(-90.0, r)?;
            self.edge(d - 2.0 * r)?;
        }
        self.ctx.stroke();
        self.ctx.restore();
        self.ctx.restore();
        self.ctx.move_to(pt.0, pt.1);
        Ok(())
    }

    /// close(): the last stroke, the page coordinates, and each path's d
    fn finish(mut self) -> Vec<(String, Vec<(String, String)>)> {
        self.ctx.stroke();
        let s = &mut self.ctx.s;
        let (mut xmin, mut ymin, mut xmax, mut ymax) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for part in &s.parts {
            for p in &part.pathes {
                for c in &p.cmds {
                    let (x, y) = c.xy();
                    xmin = py::min(xmin, x);
                    xmax = py::max(xmax, x);
                    ymin = py::min(ymin, y);
                    ymax = py::max(ymax, y);
                }
            }
        }
        xmin -= PADDING;
        ymin -= PADDING;
        xmax += PADDING;
        ymax += PADDING;
        let _ = xmax;
        let height = ymax - ymin;
        let mut m = Affine::translation(-xmin, -ymin);
        m = Affine::scale(1.0, -1.0).mul(&m);
        m = Affine::translation(0.0, 1.0 * height).mul(&m);
        let mut out = vec![];
        for (gi, part) in s.parts.iter_mut().enumerate() {
            if part.pathes.is_empty() {
                continue;
            }
            let mut paths = vec![];
            for path in part.pathes.iter_mut() {
                path.p.lw *= 1.0;
                for c in path.cmds.iter_mut() {
                    let (a, b) = m.apply((c.v[0], c.v[1]));
                    c.v[0] = a;
                    c.v[1] = b;
                    if c.k == Kind::C {
                        let (a, b) = m.apply((c.v[2], c.v[3]));
                        c.v[2] = a;
                        c.v[3] = b;
                        let (a, b) = m.apply((c.v[4], c.v[5]));
                        c.v[4] = a;
                        c.v[5] = b;
                    }
                }
                faster_edges(&mut path.cmds, path.p.lw);
                let mut d: Vec<String> = vec![];
                let (mut x, mut y) = (0.0f64, 0.0f64);
                let mut start: Option<usize> = None;
                let mut last: Option<usize> = None;
                for (ci, c) in path.cmds.iter().enumerate() {
                    let (x0, y0) = (x, y);
                    (x, y) = c.xy();
                    match c.k {
                        Kind::M => {
                            if let (Some(st), Some(la)) = (start, last) {
                                let (a, b) = (path.cmds[st].xy(), path.cmds[la].xy());
                                if points_equal(a.0, a.1, b.0, b.1) {
                                    d.push("Z".into());
                                }
                            }
                            start = Some(ci);
                            d.push(format!("M {x:.3} {y:.3}"));
                        }
                        Kind::L => {
                            if (x - x0).abs() < EPS {
                                d.push(format!("V {y:.3}"));
                            } else if (y - y0).abs() < EPS {
                                d.push(format!("H {x:.3}"));
                            } else {
                                d.push(format!("L {x:.3} {y:.3}"));
                            }
                        }
                        Kind::C => {
                            d.push(format!("C {:.3} {:.3} {:.3} {:.3} {x:.3} {y:.3}", c.v[2], c.v[3], c.v[4], c.v[5]));
                        }
                    }
                    last = Some(ci);
                }
                if let (Some(st), Some(la)) = (start, last) {
                    let (a, b) = (path.cmds[st].xy(), path.cmds[la].xy());
                    if st != la && points_equal(a.0, a.1, b.0, b.1) {
                        d.push("Z".into());
                    }
                }
                let (r, g, b) = path.p.rgb;
                let colour = format!("rgb({:.0},{:.0},{:.0})", r * 255.0, g * 255.0, b * 255.0);
                if d.last().is_some_and(|s| s.starts_with('M')) {
                    d.pop();
                }
                if !d.is_empty() {
                    paths.push((d.join(" "), colour));
                }
            }
            out.push((format!("p-{gi}"), paths));
        }
        out
    }
}

type R = Result<(), String>;

fn line_intersection(l1: ((f64, f64), (f64, f64)), l2: ((f64, f64), (f64, f64))) -> Option<(f64, f64)> {
    let xdiff = (l1.0 .0 - l1.1 .0, l2.0 .0 - l2.1 .0);
    let ydiff = (l1.0 .1 - l1.1 .1, l2.0 .1 - l2.1 .1);
    let det = |a: (f64, f64), b: (f64, f64)| a.0 * b.1 - a.1 * b.0;
    let div = det(xdiff, ydiff);
    if div == 0.0 {
        return None;
    }
    let d = (det(l1.0, l1.1), det(l2.0, l2.1));
    let x = det(d, xdiff) / div;
    let y = det(d, ydiff) / div;
    let ok = x + EPS >= py::min(l1.0 .0, l1.1 .0)
        && x + EPS >= py::min(l2.0 .0, l2.1 .0)
        && x - EPS <= py::max(l1.0 .0, l1.1 .0)
        && x - EPS <= py::max(l2.0 .0, l2.1 .0)
        && y + EPS >= py::min(l1.0 .1, l1.1 .1)
        && y + EPS >= py::min(l2.0 .1, l2.1 .1)
        && y - EPS <= py::max(l1.0 .1, l1.1 .1)
        && y - EPS <= py::max(l2.0 .1, l2.1 .1);
    if ok {
        Some((x, y))
    } else {
        None
    }
}

/// Path.faster_edges, inner_corners="corner": a burn arc between two lines
/// becomes the lines' intersection
fn faster_edges(p: &mut Vec<Cmd>, lw: f64) {
    let n = p.len();
    for i in 0..n {
        if p[i].k == Kind::C && i > 1 && i < n - 1 && p[i - 1].k == Kind::L && p[i + 1].k == Kind::L {
            let p11 = p[i - 2].xy();
            let p12 = p[i - 1].xy();
            let p21 = p[i].xy();
            let p22 = p[i + 1].xy();
            if py::powf(p12.0 - p21.0, 2.0) + py::powf(p12.1 - p21.1, 2.0) > py::powf(lw, 2.0) {
                continue;
            }
            if let Some((x, y)) = line_intersection((p11, p12), (p21, p22)) {
                let l = Cmd { k: Kind::L, v: [x, y, 0.0, 0.0, 0.0, 0.0], tup: true };
                p[i - 1] = l;
                p[i] = l;
            }
        }
    }
    if p.len() > 1 {
        let old = p.clone();
        let m = old.len();
        *p = old.iter().enumerate().filter(|(k, c)| !c.same(&old[(k + m - 1) % m])).map(|(_, c)| *c).collect();
    }
}

// --- SnakeBoxVar ------------------------------------------------------------------

/// Every option snakeboxvar.py took, with its defaults.
#[derive(Clone, Debug)]
pub struct Snake {
    pub widths_u: String,
    pub widths_v: String,
    pub origin_u: i64,
    pub origin_v: i64,
    pub blocksize: f64,
    pub path: String,
    pub open_faces: String,
    pub pin_width: f64,
    pub pin_length: f64,
    pub pin_play: f64,
    pub port_in: bool,
    pub port_out: bool,
    pub flat_in: String,
    pub flat_out: String,
    pub plain_in: bool,
    pub plain_out: bool,
    pub port_mirror: bool,
    pub tabs_at_exit: bool,
    pub lap_in: String,
    pub lap_out: String,
    pub mouths: String,
    pub pin_seat: f64,
    pub thickness: f64,
    pub burn: f64,
    pub spacing: (f64, f64),
}

impl Default for Snake {
    fn default() -> Snake {
        Snake {
            widths_u: String::new(),
            widths_v: String::new(),
            origin_u: 0,
            origin_v: 0,
            blocksize: 40.0,
            path: "RRRRRRR".into(),
            open_faces: String::new(),
            pin_width: 12.0,
            pin_length: 3.0,
            pin_play: 0.0,
            port_in: false,
            port_out: false,
            flat_in: String::new(),
            flat_out: String::new(),
            plain_in: false,
            plain_out: false,
            port_mirror: false,
            tabs_at_exit: false,
            lap_in: String::new(),
            lap_out: String::new(),
            mouths: String::new(),
            pin_seat: 0.2,
            thickness: 3.0,
            burn: 0.1,
            spacing: (0.5, 0.0),
        }
    }
}

/// One group of the drawn sheet: its id (p-0, p-1 the plates) and its paths,
/// each (d, stroke colour).
pub type Groups = Vec<(String, Vec<(String, String)>)>;

pub type Cell = (i64, i64);
pub type Run = (Cell, Cell, i64);

const MAGENTA: (f64, f64, f64) = (1.0, 0.0, 1.0);
const BLACK: (f64, f64, f64) = (0.0, 0.0, 0.0);
const MOUTH_ACROSS: f64 = 7.0;
const MOUTH_ALONG: f64 = 14.0;

fn tuple_repr(c: Cell) -> String {
    format!("({}, {})", c.0, c.1)
}

/// repr() of a str
fn r(s: &str) -> String {
    py::str_repr(s)
}

/// Python's "{x}" of a float (str(), which is repr())
fn fl(x: f64) -> String {
    py::repr(x)
}

impl Snake {
    /// Parse the command line Boxes.py would have been given. Only --key=value
    /// and the bare store_true switches; Boxes' own --labels, --reference,
    /// --inner_corners and --output are read and must say what this draws.
    pub fn from_args(args: &[String]) -> Result<Snake, String> {
        let mut s = Snake::default();
        for a in args {
            let (k, v) = match a.split_once('=') {
                Some((k, v)) => (k, Some(v)),
                None => (a.as_str(), None),
            };
            let k = k.trim_start_matches("--");
            let need = || v.ok_or_else(|| format!("argument --{k}: expected one argument"));
            let f = |x: &str| py::float(x).ok_or_else(|| format!("argument --{k}: invalid float value: {}", r(x)));
            let i = |x: &str| x.trim().parse::<i64>().map_err(|_| format!("argument --{k}: invalid int value: {}", r(x)));
            match k {
                "widths_u" => s.widths_u = need()?.into(),
                "widths_v" => s.widths_v = need()?.into(),
                "origin_u" => s.origin_u = i(need()?)?,
                "origin_v" => s.origin_v = i(need()?)?,
                "blocksize" => s.blocksize = f(need()?)?,
                "path" => s.path = need()?.into(),
                "open_faces" => s.open_faces = need()?.into(),
                "pin_width" => s.pin_width = f(need()?)?,
                "pin_length" => s.pin_length = f(need()?)?,
                "pin_play" => s.pin_play = f(need()?)?,
                "pin_seat" => s.pin_seat = f(need()?)?,
                "port_in" => s.port_in = true,
                "port_out" => s.port_out = true,
                "plain_in" => s.plain_in = true,
                "plain_out" => s.plain_out = true,
                "port_mirror" => s.port_mirror = true,
                "tabs_at_exit" => s.tabs_at_exit = true,
                "flat_in" => s.flat_in = need()?.into(),
                "flat_out" => s.flat_out = need()?.into(),
                "lap_in" => s.lap_in = need()?.into(),
                "lap_out" => s.lap_out = need()?.into(),
                "mouths" => s.mouths = need()?.into(),
                "thickness" => s.thickness = f(need()?)?,
                "burn" => s.burn = f(need()?)?,
                "spacing" => {
                    let x = need()?;
                    s.spacing = match py::float(x) {
                        Some(a) => (a, 0.0),
                        None => {
                            let p: Vec<f64> = x.split(':').map(|t| f(t.trim())).collect::<Result<_, _>>()?;
                            (p[0], *p.get(1).unwrap_or(&0.0))
                        }
                    }
                }
                "labels" | "reference" => {
                    if !matches!(need()?, "0" | "false" | "False" | "no") {
                        return Err(format!("--{k} other than 0 is not drawn here"));
                    }
                }
                "inner_corners" => {
                    if need()? != "corner" {
                        return Err("--inner_corners other than corner is not drawn here".into());
                    }
                }
                "output" => {}
                _ => return Err(format!("unrecognized arguments: {a}")),
            }
        }
        Ok(s)
    }

    pub fn cells(&self) -> Result<Vec<Cell>, String> {
        let mut pos = (0i64, 0i64);
        let mut out = vec![(0, 0)];
        let mut seen: HashSet<Cell> = HashSet::from([(0, 0)]);
        for (i, c) in self.path.trim().to_uppercase().chars().enumerate() {
            let st = match c {
                'R' => (1, 0),
                'L' => (-1, 0),
                'U' => (0, 1),
                'D' => (0, -1),
                _ => return Err(format!("move {} is {}; only U, D, L and R are allowed", i + 1, r(&c.to_string()))),
            };
            pos = (pos.0 + st.0, pos.1 + st.1);
            if seen.contains(&pos) {
                return Err(format!("the path revisits cell {} at move {}", tuple_repr(pos), i + 1));
            }
            seen.insert(pos);
            out.push(pos);
        }
        if out.len() < 2 && self.open_faces.trim().is_empty() {
            return Err("need at least one move, or --open_faces to say which two faces of a single cell are open".into());
        }
        Ok(out)
    }

    /// Boundary runs as (start vertex, direction, steps), plus turns.
    pub fn outline(&self, cells: &[Cell]) -> Result<(Vec<Run>, Vec<i64>), String> {
        let s: HashSet<Cell> = cells.iter().copied().collect();
        // dict insertion order matters for nothing but the pinch check
        let mut nxt: HashMap<Cell, Cell> = HashMap::new();
        for &(i, j) in cells {
            for (present, a, b) in [((i, j - 1), (i, j), (i + 1, j)), ((i + 1, j), (i + 1, j), (i + 1, j + 1)), ((i, j + 1), (i + 1, j + 1), (i, j + 1)), ((i - 1, j), (i, j + 1), (i, j))] {
                if !s.contains(&present) {
                    if nxt.contains_key(&a) {
                        return Err(format!("the shape pinches at {}: two cells meet only at a corner, so it cannot be built as a tube", tuple_repr(a)));
                    }
                    nxt.insert(a, b);
                }
            }
        }
        let start = *nxt.keys().min_by_key(|v| (v.1, v.0)).unwrap();
        let mut walk = vec![start];
        let mut v = nxt[&start];
        while v != start {
            walk.push(v);
            v = nxt[&v];
        }
        if walk.len() != nxt.len() {
            return Err("the shape encloses a hole".into());
        }
        let mut runs: Vec<Run> = vec![];
        for k in 0..walk.len() {
            let (a, b) = (walk[k], walk[(k + 1) % walk.len()]);
            let d = ((b.0 - a.0).clamp(-1, 1), (b.1 - a.1).clamp(-1, 1));
            let n = (b.0 - a.0).abs() + (b.1 - a.1).abs();
            if let Some(last) = runs.last_mut() {
                if last.1 == d {
                    last.2 += n;
                    continue;
                }
            }
            runs.push((a, d, n));
        }
        if runs.len() > 1 && runs[0].1 == runs[runs.len() - 1].1 {
            let last = runs.pop().unwrap();
            runs[0] = (last.0, runs[0].1, runs[0].2 + last.2);
        }
        let nr = runs.len();
        let turns: Vec<i64> = (0..nr).map(|k| runs[k].1 .0 * runs[(k + 1) % nr].1 .1 - runs[k].1 .1 * runs[(k + 1) % nr].1 .0).collect();
        let sum: i64 = turns.iter().sum();
        if sum != 4 {
            return Err(format!("AssertionError: turning number {sum}, expected 4"));
        }
        Ok((runs, turns))
    }

    fn normal(d: Cell) -> char {
        match d {
            (1, 0) => 'S',
            (0, 1) => 'E',
            (-1, 0) => 'N',
            _ => 'W',
        }
    }

    /// Indices of the two open runs, the first of which gets the tabs.
    pub fn tip_runs(&self, cells: &[Cell], runs: &[Run]) -> Result<Vec<usize>, String> {
        let up = self.open_faces.trim().to_uppercase().replace(',', " ");
        let faces: Vec<&str> = up.split_whitespace().collect();
        if !faces.is_empty() {
            if cells.len() != 1 {
                return Err("--open_faces is for a single cell only; with a path the two ends are set by where the path ends".into());
            }
            if faces.len() != 2 {
                return Err(format!("--open_faces wants exactly two of N/S/E/W, got {}", py::list_repr(&faces)));
            }
            let mut byface: HashMap<String, usize> = HashMap::new();
            for (k, run) in runs.iter().enumerate() {
                byface.insert(Self::normal(run.1).to_string(), k);
            }
            let mut out = vec![];
            for f in &faces {
                match byface.get(*f) {
                    None => return Err(format!("unknown face {}; use N, S, E or W", r(f))),
                    Some(&k) => out.push(k),
                }
            }
            if out[0] == out[1] {
                return Err("the two open faces must be different".into());
            }
            return Ok(out);
        }
        let s: HashSet<Cell> = cells.iter().copied().collect();
        let adj = |c: Cell| -> Vec<Cell> { [(c.0 + 1, c.1), (c.0 - 1, c.1), (c.0, c.1 + 1), (c.0, c.1 - 1)].into_iter().filter(|n| s.contains(n)).collect() };
        let mut ends: Vec<Cell> = cells.iter().copied().filter(|&c| adj(c).len() == 1).collect();
        if ends.len() != 2 {
            return Err(format!("not a simple snake: {} loose ends", ends.len()));
        }
        ends.sort_by_key(|c| cells.iter().position(|x| x == c).unwrap());
        let mut out = vec![];
        for e in ends {
            let (i, j) = e;
            let a = adj(e)[0];
            let d = (e.0 - a.0, e.1 - a.1);
            let cap = match d {
                (1, 0) => ((i + 1, j), (0, 1)),
                (-1, 0) => ((i, j + 1), (0, -1)),
                (0, 1) => ((i + 1, j + 1), (-1, 0)),
                _ => ((i, j), (1, 0)),
            };
            match runs.iter().position(|&(v, dd, n)| v == cap.0 && dd == cap.1 && n == 1) {
                Some(k) => out.push(k),
                None => return Err(format!("AssertionError: no cap run found for end cell {}", tuple_repr(e))),
            }
        }
        Ok(out)
    }

    /// {lattice index: width} for one in-plane axis.
    fn widths(&self, axis: usize) -> Result<BTreeMap<i64, f64>, String> {
        let raw = if axis == 0 { &self.widths_u } else { &self.widths_v }.trim().to_string();
        let mut out = BTreeMap::new();
        if raw.is_empty() {
            return Ok(out);
        }
        let o = if axis == 0 { self.origin_u } else { self.origin_v };
        for (i, w) in raw.split(',').enumerate() {
            let v = py::float(w).ok_or_else(|| format!("could not convert string to float: {}", r(w)))?;
            out.insert(o + i as i64, v);
        }
        Ok(out)
    }

    fn cubic(&self) -> Result<bool, String> {
        Ok(self.widths(0)?.is_empty() && self.widths(1)?.is_empty())
    }

    /// Real length of a boundary run: the widths of the n cells it crosses.
    fn span(&self, start: Cell, d: Cell, n: i64) -> Result<f64, String> {
        let axis = if d.1 == 0 { 0 } else { 1 };
        let w = self.widths(axis)?;
        if w.is_empty() {
            return Ok(n as f64 * self.blocksize);
        }
        let step = if axis == 0 { d.0 } else { d.1 };
        let i0 = if axis == 0 { start.0 } else { start.1 };
        let idx: Vec<i64> = if step > 0 { (0..n).map(|k| i0 + k).collect() } else { (0..n).map(|k| i0 - 1 - k).collect() };
        let missing: Vec<i64> = idx.iter().copied().filter(|i| !w.contains_key(i)).collect();
        if !missing.is_empty() {
            let have: Vec<i64> = w.keys().copied().collect();
            return Err(format!("--widths_{} has no width for column(s) {}; it covers {}", ['u', 'v'][axis], py::ints_repr(&missing), py::ints_repr(&have)));
        }
        Ok(idx.iter().fold(0.0, |s, i| s + w[i]))
    }

    #[allow(clippy::type_complexity)]
    pub fn geometry(&self) -> Result<(Vec<f64>, Vec<usize>, HashMap<usize, (f64, f64)>, Vec<usize>, usize, HashSet<usize>), String> {
        let t = self.thickness;
        let cells = self.cells()?;
        let (runs, turns) = self.outline(&cells)?;
        let ends = self.tip_runs(&cells, &runs)?;
        let mut ports = vec![];
        if self.port_in {
            ports.push(ends[0]);
        }
        if self.port_out {
            ports.push(ends[1]);
        }
        let tips: Vec<usize> = ends.iter().copied().filter(|k| !ports.contains(k)).collect();
        if tips.is_empty() {
            return Err("both ends cannot be ports: the piece would have no rim opening to couple by".into());
        }
        let tab_run = if self.tabs_at_exit { ends[1] } else { ends[0] };
        let mut plains = HashSet::new();
        if self.plain_in {
            plains.insert(ends[0]);
        }
        if self.plain_out {
            plains.insert(ends[1]);
        }
        let n = runs.len();
        let inset: Vec<f64> = (0..n).map(|k| if tips.contains(&k) { 0.0 } else { t }).collect();
        let mut borders = vec![];
        let mut caps = HashMap::new();
        for k in 0..n {
            let head = turns[(k + n - 1) % n] as f64 * inset[(k + n - 1) % n];
            let tail = turns[k] as f64 * inset[(k + 1) % n];
            let full = self.span(runs[k].0, runs[k].1, runs[k].2)?;
            borders.push(full - head - tail);
            borders.push((90 * turns[k]) as f64);
            if tips.contains(&k) {
                caps.insert(k, (head, full));
            }
        }
        Ok((borders, tips, caps, ports, tab_run, plains))
    }

    /// [(plate, run k, x, y)]: where each requested mouth is drawn.
    fn mouth_spots(&self, cells: &[Cell], runs: &[Run], turns: &[i64], tips: &[usize]) -> Result<Vec<(String, usize, f64, f64)>, String> {
        let spec = self.mouths.trim();
        if spec.is_empty() {
            return Ok(vec![]);
        }
        if !self.cubic()? {
            return Err("a mouth on a non-cubic cell is not implemented: the hole is placed on one pitch and the cell is not.".into());
        }
        let (s, t) = (self.blocksize, self.thickness);
        let n = runs.len();
        let inset: Vec<f64> = (0..n).map(|k| if tips.contains(&k) { 0.0 } else { t }).collect();
        let mut out = vec![];
        for item in spec.replace(' ', "").split(',') {
            let bad = || format!("--mouths entry {} is not CELL:PLATE, e.g. 5:first", r(item));
            let parts: Vec<&str> = item.split(':').collect();
            if parts.len() != 2 {
                return Err(bad());
            }
            let (cell_s, plate) = (parts[0], parts[1]);
            let cell: i64 = cell_s.trim().parse().map_err(|_| bad())?;
            if plate != "first" && plate != "mirror" {
                return Err(format!("--mouths plate {} must be 'first' or 'mirror'", r(plate)));
            }
            if !(0 <= cell && cell < cells.len() as i64) {
                return Err(format!("--mouths names cell {cell} and this piece has {}, numbered 0 to {}", cells.len(), cells.len() as i64 - 1));
            }
            let cell = cell as usize;
            let (ci, cj) = cells[cell];
            let (cx, cy) = (ci as f64 + 0.5, cj as f64 + 0.5);
            let mut found = false;
            for k in 0..n {
                if tips.contains(&k) {
                    continue;
                }
                let ((ax, ay), (dx, dy), m) = runs[k];
                let (ax, ay, dx, dy) = (ax as f64, ay as f64, dx as f64, dy as f64);
                let along = (cx - ax) * dx + (cy - ay) * dy;
                let perp = (cx - ax) * -dy + (cy - ay) * dx;
                if (perp.abs() - 0.5).abs() < 1e-9 && 0.0 < along && along < m as f64 {
                    let ahead = (ci + dx as i64, cj + dy as i64);
                    let behind = (ci - dx as i64, cj - dy as i64);
                    let beside: HashSet<Cell> = [cell as i64 - 1, cell as i64 + 1].iter().filter(|&&x| 0 <= x && x < cells.len() as i64).map(|&x| cells[x as usize]).collect();
                    if !(beside.contains(&ahead) && beside.contains(&behind)) {
                        return Err(format!(
                            "--mouths cell {cell} is {}: a {}mm mouth reaches the cells either side of it along the bore, and there the bore does not continue. Name a cell the airway runs straight through.",
                            if beside.len() < 2 { "an end of the piece" } else { "a turn" },
                            py::g(MOUTH_ALONG)
                        ));
                    }
                    let head = turns[(k + n - 1) % n] as f64 * inset[(k + n - 1) % n];
                    out.push((plate.to_string(), k, along * s - head, s / 2.0 - t));
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("--mouths cell {cell} has no long side to cut into: every side of it is a neighbouring cell or the rim"));
            }
        }
        Ok(out)
    }

    /// {(run, 'l'|'r')} for wall ends that run past an opening as a tongue.
    fn lap_runs(&self, runs: &[Run], ends: &[usize], ports: &[usize]) -> Result<HashSet<(usize, char)>, String> {
        let mut laps = HashSet::new();
        let n = runs.len();
        for (tip, face) in [(ends[0], &self.lap_in), (ends[1], &self.lap_out)] {
            let face = face.trim().to_uppercase();
            if face.is_empty() || ports.contains(&tip) {
                continue;
            }
            let mut ok = false;
            for k in [(tip + n - 1) % n, (tip + 1) % n] {
                if ends.contains(&k) {
                    continue;
                }
                if Self::normal(runs[k].1).to_string() == face {
                    laps.insert((k, if (k + n - 1) % n == tip { 'l' } else { 'r' }));
                    ok = true;
                    break;
                }
            }
            if !ok {
                return Err(format!("no wall faces {face} at that opening; the lap must name one of the two walls beside it"));
            }
        }
        Ok(laps)
    }

    fn lap_profile(&self, tt: &mut Turtle, length: f64) -> R {
        let t = self.thickness;
        tt.corner(-90.0, 0.0)?;
        tt.edge(t)?;
        tt.corner(90.0, 0.0)?;
        tt.edge(length)?;
        tt.corner(90.0, 0.0)?;
        tt.edge(t)?;
        tt.corner(-90.0, 0.0)
    }

    fn end_profile(&self, tt: &mut Turtle, length: f64, pin: bool) -> R {
        if self.pin_length <= 0.0 {
            return tt.edge(length);
        }
        let w = self.pin_width + if pin { 0.0 } else { 2.0 * self.pin_play };
        let p = self.pin_length + if pin { 0.0 } else { self.pin_seat };
        let a = (length - w) / 2.0;
        if a < 0.0 {
            return Err(format!("pin_width {} is too wide for the {length:.1}mm end frame", fl(self.pin_width)));
        }
        let d = if pin { -1.0 } else { 1.0 };
        tt.edge(a)?;
        tt.corner(d * 90.0, 0.0)?;
        tt.edge(p)?;
        tt.corner(-d * 90.0, 0.0)?;
        tt.edge(w)?;
        tt.corner(-d * 90.0, 0.0)?;
        tt.edge(p)?;
        tt.corner(d * 90.0, 0.0)?;
        tt.edge(a)
    }

    /// Borders with the cap runs replaced by a tab / notch profile: (borders,
    /// edge types, starts), starts[k] being run k's first edge.
    #[allow(clippy::type_complexity)]
    fn plate_borders(&self, borders: &[f64], tips: &[usize], caps: &HashMap<usize, (f64, f64)>, tab_run: usize, plains: &HashSet<usize>) -> Result<(Vec<f64>, Vec<char>, Vec<usize>), String> {
        let (mut out, mut et, mut starts) = (vec![], vec![], vec![]);
        let w_pin = self.pin_width;
        let w_slot = self.pin_width + 2.0 * self.pin_play;
        let n = borders.len() / 2;
        for k in 0..n {
            let (length, angle) = (borders[2 * k], borders[2 * k + 1]);
            starts.push(et.len());
            if tips.contains(&k) && (self.pin_length <= 0.0 || plains.contains(&k)) {
                out.extend([length, angle]);
                et.push('e');
            } else if tips.contains(&k) {
                let pin = k == tab_run;
                let w = if pin { w_pin } else { w_slot };
                let p = self.pin_length + if pin { 0.0 } else { self.pin_seat };
                let (head, full) = caps[&k];
                let a = full / 2.0 - head - w / 2.0;
                let a2 = length - w - a;
                if a < 0.0 || a2 < 0.0 {
                    return Err(format!("pin_width {} is too wide for the {length:.1}mm end frame", fl(self.pin_width)));
                }
                let d = if pin { -1.0 } else { 1.0 };
                out.extend([a, d * 90.0, p, -d * 90.0, w, -d * 90.0, p, d * 90.0, a2, angle]);
                et.extend(['e'; 5]);
            } else {
                out.extend([length, angle]);
                et.push('f');
            }
        }
        Ok((out, et, starts))
    }

    /// Walls for every run except the two caps, with pin/slot ends.
    fn snake_walls(&self, tt: &mut Turtle, borders: &[f64], tips: &[usize], laps: &HashSet<(usize, char)>, tab_run: usize, plains: &HashSet<usize>) -> R {
        let t = self.thickness;
        let h = self.blocksize - 2.0 * t;
        let n = borders.len() / 2;
        let bottom = Edge::Finger(false, tt.fs);
        let top = bottom;
        let plain = Edge::Plain;
        let mut lset = tt.fs;
        let mut rset = tt.fs;
        tt.move_to(0.0, bottom.margin(), 0.0);
        for k in 0..n {
            if tips.contains(&k) {
                continue;
            }
            let mut length = borders[2 * k];
            let a_before = borders[(2 * k + borders.len() - 1) % borders.len()];
            let a_after = borders[2 * k + 1];
            let (km, kp) = ((k + n - 1) % n, (k + 1) % n);
            let l_open = tips.contains(&km);
            let r_open = tips.contains(&kp);
            let l_pin = km == tab_run;
            let r_pin = kp == tab_run;
            let l_lap = laps.contains(&(k, 'l'));
            let r_lap = laps.contains(&(k, 'r'));
            if !l_open {
                lset.set_angle(a_before)?;
                if a_before < 0.0 {
                    length -= t * tan(py::radians(-a_before / 2.0));
                }
            }
            if !r_open {
                rset.set_angle(a_after)?;
                if a_after < 0.0 {
                    length -= t * tan(py::radians(-a_after / 2.0));
                }
            }
            let left = if l_open { plain } else { Edge::Finger(true, lset) };
            let right = if r_open { plain } else { Edge::Finger(false, rset) };
            let lx = if l_lap {
                t
            } else if l_open && l_pin && !plains.contains(&km) {
                self.pin_length
            } else {
                0.0
            };
            tt.move_to(left.spacing() + tt.spacing + lx, 0.0, 0.0);
            tt.ctx.save();
            tt.ctx.frame = tt.ctx.m;
            tt.draw(&bottom, length)?;
            tt.edge_corner(&bottom, &right, 90.0)?;
            if r_lap {
                self.lap_profile(tt, h)?;
            } else if r_open && !plains.contains(&kp) {
                self.end_profile(tt, h, r_pin)?;
            } else {
                tt.draw(&right, h)?;
            }
            tt.edge_corner(&right, &top, 90.0)?;
            tt.draw(&top, length)?;
            tt.edge_corner(&top, &left, 90.0)?;
            if l_lap {
                self.lap_profile(tt, h)?;
            } else if l_open && !plains.contains(&km) {
                self.end_profile(tt, h, l_pin)?;
            } else {
                tt.draw(&left, h)?;
            }
            tt.edge_corner(&left, &bottom, 90.0)?;
            tt.ctx.stroke();
            tt.ctx.restore();
            let rx = if r_lap {
                t
            } else if r_open && r_pin && !plains.contains(&kp) {
                self.pin_length
            } else {
                0.0
            };
            tt.move_to(length + right.spacing() + tt.spacing + rx, 0.0, 0.0);
        }
        Ok(())
    }

    /// Draw the piece: the two face plates, then the walls.
    pub fn render(&self) -> Result<Groups, String> {
        Ok(self.draw()?.finish())
    }

    /// The parts as render() draws them, each path read back in its part's
    /// own frame rather than laid out on the sheet: a plate in the snake's
    /// (u, v) frame from its first corner (the mirror plate unmirrored), a
    /// wall along its run from its first corner, up its height. Each path
    /// is its points (one per command) and its colour. For the SolidCraft
    /// design, which places the parts in 3D.
    #[allow(clippy::type_complexity)]
    pub fn render_local(&self) -> Result<Vec<Vec<(Vec<(f64, f64)>, (f64, f64, f64))>>, String> {
        let mut tt = self.draw()?;
        tt.ctx.stroke();
        let mut out = vec![];
        for part in &tt.ctx.s.parts {
            if part.pathes.is_empty() {
                continue;
            }
            let mut paths = vec![];
            for path in &part.pathes {
                if path.cmds.iter().any(|c| c.k == Kind::C) {
                    return Err("render_local: a curve in a part; the design takes straight edges only".into());
                }
                let inv = path.frame.inverse();
                paths.push((path.cmds.iter().map(|c| inv.apply(c.xy())).collect(), path.p.rgb));
            }
            out.push(paths);
        }
        Ok(out)
    }

    fn draw(&self) -> Result<Turtle, String> {
        let mut tt = Turtle::new(&Settings { thickness: self.thickness, burn: self.burn, spacing: self.spacing });
        let (borders, tips, caps, ports, tab_run, plains) = self.geometry()?;
        let (pb, pe, ps) = self.plate_borders(&borders, &tips, &caps, tab_run, &plains)?;
        let cells = self.cells()?;
        let (runs, _) = self.outline(&cells)?;
        let ends = self.tip_runs(&cells, &runs)?;
        let (_, turns) = self.outline(&cells)?;
        let mouths = self.mouth_spots(&cells, &runs, &turns, &tips)?;
        let flat = |which: &str| -> HashSet<usize> { [(ends[0], &self.flat_in), (ends[1], &self.flat_out)].iter().filter(|(_, w)| w.as_str() == which).map(|(e, _)| *e).collect() };
        let (fa, fb) = (flat("first"), flat("mirror"));
        let (ab, ae, astart) = if fa.is_empty() { (pb.clone(), pe.clone(), ps.clone()) } else { self.plate_borders(&borders, &tips, &caps, tab_run, &plains.union(&fa).copied().collect())? };
        let (bb, be, bstart) = if fb.is_empty() { (pb.clone(), pe.clone(), ps.clone()) } else { self.plate_borders(&borders, &tips, &caps, tab_run, &plains.union(&fb).copied().collect())? };
        let edge_of = |c: char, tt: &Turtle| if c == 'f' { Edge::Finger(true, tt.fs) } else { Edge::Plain };
        for (plate, b, e, start) in [("first", &ab, &ae, &astart), ("mirror", &bb, &be, &bstart)] {
            let port_here = !ports.is_empty() && ((plate == "mirror") == self.port_mirror);
            let port_seg: HashMap<usize, f64> = ports.iter().map(|&k| (start[k], borders[2 * k])).collect();
            let mut mine: HashMap<usize, Vec<(f64, f64)>> = HashMap::new();
            for (pl, k, x, y) in &mouths {
                if pl == plate {
                    mine.entry(start[*k]).or_default().push((*x, *y));
                }
            }
            let edges: Vec<Edge> = e.iter().map(|&c| edge_of(c, &tt)).collect();
            let (s, t) = (self.blocksize, self.thickness);
            let cubic = self.cubic()?;
            let mut cb = |tt: &mut Turtle, i: usize| -> R {
                if port_here {
                    if let Some(&el) = port_seg.get(&i) {
                        if !cubic {
                            return Err("a port marker on a non-cubic cell is not implemented: the hole is square in one pitch and the cell is not.".into());
                        }
                        tt.rectangular_hole(el / 2.0, s / 2.0 - t, el + 4.0 * t, s, MAGENTA)?;
                    }
                }
                for &(x, y) in mine.get(&i).map(|v| v.as_slice()).unwrap_or(&[]) {
                    tt.rectangular_hole(x, y, MOUTH_ALONG, MOUTH_ACROSS, BLACK)?;
                }
                Ok(())
            };
            let has = port_here || !mine.is_empty();
            let cbr: Option<&mut dyn FnMut(&mut Turtle, usize) -> R> = if has { Some(&mut cb) } else { None };
            tt.polygon_wall(b, &edges, plate == "mirror", cbr)?;
        }
        let laps = self.lap_runs(&runs, &ends, &ports)?;
        self.snake_walls(&mut tt, &borders, &tips, &laps, tab_run, &plains)?;
        Ok(tt)
    }
}
