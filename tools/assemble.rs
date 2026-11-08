//! Fold a section up as a solid and see whether it closes: assemble.py, Rust
//! since 2026-11-08. A voxel model, flooded from outside. Every region here is
//! a union of axis-aligned boxes, so it is tested as one, with no polygon
//! engine: shapely's contains_xy asked whether a voxel centre lies in the
//! interior of the region, and a point is interior to a union of closed boxes
//! exactly when the four points a hair away from it diagonally all lie in it.
use crate::bore::BoxMm;
use lasermade_tools::py;
use std::collections::{HashSet, VecDeque};

const PX: f64 = 1.0; // voxel, mm
const EPS: f64 = 1e-9;

#[derive(Clone, Copy, Debug)]
struct Bx {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Bx {
    fn new(xs: [f64; 4], ys: [f64; 4]) -> Bx {
        let f = |v: [f64; 4], m: bool| v.iter().copied().reduce(|a, b| if (b < a) == m { b } else { a }).unwrap();
        Bx { x0: f(xs, true), y0: f(ys, true), x1: f(xs, false), y1: f(ys, false) }
    }
    fn holds(&self, x: f64, y: f64) -> bool {
        self.x0 <= x && x <= self.x1 && self.y0 <= y && y <= self.y1
    }
    fn meet(&self, o: &Bx) -> Option<Bx> {
        let b = Bx { x0: self.x0.max(o.x0), y0: self.y0.max(o.y0), x1: self.x1.min(o.x1), y1: self.y1.min(o.y1) };
        (b.x0 < b.x1 && b.y0 < b.y1).then_some(b)
    }
}

/// A region: the union of `plus`, less the closed union of `minus`
#[derive(Clone, Default)]
struct Region {
    plus: Vec<Bx>,
    minus: Vec<Bx>,
}

impl Region {
    fn interior(&self, x: f64, y: f64) -> bool {
        let inu = |bs: &[Bx], x: f64, y: f64| bs.iter().any(|b| b.holds(x, y));
        [(-EPS, -EPS), (-EPS, EPS), (EPS, -EPS), (EPS, EPS)].iter().all(|(dx, dy)| inu(&self.plus, x + dx, y + dy)) && !inu(&self.minus, x, y)
    }
    fn bounds(&self) -> (f64, f64, f64, f64) {
        let p = &self.plus;
        (
            crate::bore::fmin(p.iter().map(|b| b.x0)),
            crate::bore::fmin(p.iter().map(|b| b.y0)),
            crate::bore::fmax(p.iter().map(|b| b.x1)),
            crate::bore::fmax(p.iter().map(|b| b.y1)),
        )
    }
}

/// Axis-aligned band along a boundary run, `depth` mm to one side.
fn band(v: (i64, i64), d: (i64, i64), n: i64, s: f64, depth: f64, outward: bool) -> Bx {
    let o = if outward { (d.1, -d.0) } else { (-d.1, d.0) };
    let (ax, ay) = (v.0 as f64 * s, v.1 as f64 * s);
    let (bx, by) = ((v.0 + d.0 * n) as f64 * s, (v.1 + d.1 * n) as f64 * s);
    let (cx, cy) = (ax + o.0 as f64 * depth, ay + o.1 as f64 * depth);
    let (dx, dy) = (bx + o.0 as f64 * depth, by + o.1 as f64 * depth);
    Bx::new([ax, bx, cx, dx], [ay, by, cy, dy])
}

fn cellbox(c: (i64, i64), s: f64) -> Bx {
    Bx { x0: c.0 as f64 * s, y0: c.1 as f64 * s, x1: (c.0 + 1) as f64 * s, y1: (c.1 + 1) as f64 * s }
}

fn voxels(r: &Region, nx: usize, ny: usize, ox: f64, oy: f64) -> Vec<bool> {
    let mut out = vec![false; nx * ny];
    if r.plus.is_empty() {
        return out;
    }
    for j in 0..ny {
        for i in 0..nx {
            out[j * nx + i] = r.interior((i as f64 + 0.5) * PX + ox, (j as f64 + 0.5) * PX + oy);
        }
    }
    out
}

/// Every voxel of `free` that a flood from the grid's faces reaches.
fn flood(free: &[bool], dims: [usize; 3]) -> Vec<bool> {
    let [nz, ny, nx] = dims;
    let at = |z: usize, y: usize, x: usize| (z * ny + y) * nx + x;
    let mut seen = vec![false; free.len()];
    let mut q = VecDeque::new();
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                if (z == 0 || z == nz - 1 || y == 0 || y == ny - 1 || x == 0 || x == nx - 1) && free[at(z, y, x)] && !seen[at(z, y, x)] {
                    seen[at(z, y, x)] = true;
                    q.push_back((z, y, x));
                }
            }
        }
    }
    while let Some((z, y, x)) = q.pop_front() {
        let mut go = |z2: usize, y2: usize, x2: usize| {
            let k = at(z2, y2, x2);
            if free[k] && !seen[k] {
                seen[k] = true;
                q.push_back((z2, y2, x2));
            }
        };
        if z > 0 {
            go(z - 1, y, x);
        }
        if z + 1 < nz {
            go(z + 1, y, x);
        }
        if y > 0 {
            go(z, y - 1, x);
        }
        if y + 1 < ny {
            go(z, y + 1, x);
        }
        if x > 0 {
            go(z, y, x - 1);
        }
        if x + 1 < nx {
            go(z, y, x + 1);
        }
    }
    seen
}

/// Connected regions of `on`, six-connected, as ndimage.label counts them.
fn count_parts(on: &[bool], dims: [usize; 3]) -> usize {
    let [nz, ny, nx] = dims;
    let mut seen = vec![false; on.len()];
    let mut n = 0;
    let mut stack = vec![];
    for k0 in 0..on.len() {
        if !on[k0] || seen[k0] {
            continue;
        }
        n += 1;
        seen[k0] = true;
        stack.push(k0);
        while let Some(k) = stack.pop() {
            let (z, y, x) = (k / (ny * nx), (k / nx) % ny, k % nx);
            let mut nb = vec![];
            if z > 0 {
                nb.push(k - ny * nx);
            }
            if z + 1 < nz {
                nb.push(k + ny * nx);
            }
            if y > 0 {
                nb.push(k - nx);
            }
            if y + 1 < ny {
                nb.push(k + nx);
            }
            if x > 0 {
                nb.push(k - 1);
            }
            if x + 1 < nx {
                nb.push(k + 1);
            }
            for m in nb {
                if on[m] && !seen[m] {
                    seen[m] = true;
                    stack.push(m);
                }
            }
        }
    }
    n
}

pub type Run = ((i64, i64), (i64, i64), i64);

/// (sealed, leak_volume_mm3, bore_volume_mm3) for one section.
pub fn closes(cells: &[(i64, i64)], runs: &[Run], tips: &[usize], s: f64, t: f64, plate_cells: Option<&[(i64, i64)]>) -> (bool, f64, f64) {
    let solid = Region { plus: cells.iter().map(|&c| cellbox(c, s)).collect(), minus: vec![] };
    let mut walls = Region::default();
    for (k, &(v, d, n)) in runs.iter().enumerate() {
        if tips.contains(&k) {
            continue;
        }
        let b = band(v, d, n, s, t, false);
        walls.plus.extend(solid.plus.iter().filter_map(|c| b.meet(c)));
    }
    let near = solid.clone();
    let far = match plate_cells {
        None => solid.clone(),
        Some(pc) => Region { plus: pc.iter().map(|&c| cellbox(c, s)).collect(), minus: vec![] },
    };
    let (minx, miny, maxx, maxy) = solid.bounds();
    let m = 3.0;
    let (ox, oy) = (minx - m, miny - m);
    let nx = ((maxx - minx + 2.0 * m) / PX) as usize;
    let ny = ((maxy - miny + 2.0 * m) / PX) as usize;
    let nz = (s / PX) as usize;
    let nn = voxels(&near, nx, ny, ox, oy);
    let ff = voxels(&far, nx, ny, ox, oy);
    let ww = voxels(&walls, nx, ny, ox, oy);
    let ss = voxels(&solid, nx, ny, ox, oy);
    let plane = nx * ny;
    let mut mat = vec![false; nz * plane];
    let ti = py::round(t / PX).max(0) as usize;
    for z in 0..nz {
        let lay = &mut mat[z * plane..(z + 1) * plane];
        if z < ti {
            lay.copy_from_slice(&nn);
        }
        if ti > 0 && z >= nz.saturating_sub(ti) || ti == 0 {
            lay.copy_from_slice(&ff);
        }
        for k in 0..plane {
            lay[k] |= ww[k];
        }
    }
    let mut plug2 = vec![false; plane];
    for &k in tips {
        let (v, d, n) = runs[k];
        let r = Region { plus: vec![band(v, d, n, s, m, true)], minus: solid.plus.clone() };
        for (a, b) in plug2.iter_mut().zip(voxels(&r, nx, ny, ox, oy)) {
            *a |= b;
        }
    }
    let free: Vec<bool> = (0..nz * plane).map(|k| !(mat[k] || plug2[k % plane])).collect();
    let out = flood(&free, [nz, ny, nx]);
    let (mut leaked, mut bore) = (0usize, 0usize);
    for k in 0..nz * plane {
        if !mat[k] && ss[k % plane] {
            bore += 1;
            if out[k] {
                leaked += 1;
            }
        }
    }
    let v = py::powf(PX, 3.0);
    (leaked == 0, leaked as f64 * v, bore as f64 * v)
}

/// A section: its blocks as mm boxes, and its openings as (box index, direction)
pub type Section = (Vec<BoxMm>, HashSet<(usize, [i64; 3])>);

struct Grid {
    org: [f64; 3],
    shape: [usize; 3],
    px: f64,
}

impl Grid {
    /// voxel range of the mm interval [a, b] along `axis`, clipped as a slice is
    fn sl(&self, a: f64, b: f64, axis: usize) -> (usize, usize) {
        let n = self.shape[axis] as i64;
        let fix = |v: i64| if v < 0 { (v + n).max(0) } else { v.min(n) } as usize;
        (fix(py::round((a - self.org[axis]) / self.px)), fix(py::round((b - self.org[axis]) / self.px)))
    }
    fn at(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.shape[1] + y) * self.shape[0] + x
    }
    fn len(&self) -> usize {
        self.shape[0] * self.shape[1] * self.shape[2]
    }
    fn each(&self, r: [(usize, usize); 3], mut f: impl FnMut(usize, [usize; 3])) {
        for z in r[2].0..r[2].1 {
            for y in r[1].0..r[1].1 {
                for x in r[0].0..r[0].1 {
                    f(self.at(x, y, z), [x, y, z]);
                }
            }
        }
    }
}

/// Voxel the given sections where the bore actually puts them:
/// (material, bore, grid)
fn build(sections: &[Section], t: f64, px: f64) -> (Vec<bool>, Vec<bool>, Grid) {
    let allb: Vec<&BoxMm> = sections.iter().flat_map(|s| s.0.iter()).collect();
    let lo: Vec<f64> = (0..3).map(|i| crate::bore::fmin(allb.iter().map(|b| b.0[i]))).collect();
    let hi: Vec<f64> = (0..3).map(|i| crate::bore::fmax(allb.iter().map(|b| b.1[i]))).collect();
    let m = 2.0 * py::max(t, 1.0) * 8.0;
    let org = [lo[0] - m, lo[1] - m, lo[2] - m];
    let shape = [0, 1, 2].map(|i| py::round((hi[i] - lo[i] + 2.0 * m) / px).max(0) as usize);
    let gr = Grid { org, shape, px };
    let mut mat = vec![false; gr.len()];
    let mut bore = vec![false; gr.len()];
    let region = |b: &BoxMm| [0, 1, 2].map(|i| gr.sl(b.0[i], b.1[i], i));
    for (boxes, openings) in sections {
        for (n, bx) in boxes.iter().enumerate() {
            let r = region(bx);
            gr.each(r, |k, _| bore[k] = true);
            for axis in 0..3 {
                for sign in [-1i64, 1] {
                    let mut d = [0i64; 3];
                    d[axis] = sign;
                    if openings.contains(&(n, d)) {
                        continue;
                    }
                    let mut face = r;
                    face[axis] = if sign < 0 { gr.sl(bx.0[axis], bx.0[axis] + t, axis) } else { gr.sl(bx.1[axis] - t, bx.1[axis], axis) };
                    let mut covers: Vec<[(usize, usize); 3]> = vec![];
                    for (mm, other) in boxes.iter().enumerate() {
                        if mm == n {
                            continue;
                        }
                        let touches = if sign < 0 { (other.1[axis] - bx.0[axis]).abs() < 1e-9 } else { (other.0[axis] - bx.1[axis]).abs() < 1e-9 };
                        if !touches {
                            continue;
                        }
                        let mut cover = Some(face);
                        for i in 0..3 {
                            if i == axis {
                                continue;
                            }
                            let a0 = py::max(bx.0[i], other.0[i]);
                            let a1 = py::min(bx.1[i], other.1[i]);
                            if a1 - a0 <= 1e-9 {
                                cover = None;
                                break;
                            }
                            cover.as_mut().unwrap()[i] = gr.sl(a0, a1, i);
                        }
                        if let Some(c) = cover {
                            covers.push(c);
                        }
                    }
                    gr.each(face, |k, p| {
                        if !covers.iter().any(|c| (0..3).all(|i| c[i].0 <= p[i] && p[i] < c[i].1)) {
                            mat[k] = true;
                        }
                    });
                }
            }
        }
    }
    for k in 0..bore.len() {
        bore[k] = bore[k] && !mat[k];
    }
    (mat, bore, gr)
}

/// Is the bore reachable from outside anywhere but through `plug`?
/// (sealed, leak mm3, bore mm3, separate regions of bore)
pub fn sealed(sections: &[Section], t: f64, plug: &[(BoxMm, [i64; 3])], px: f64) -> (bool, f64, f64, usize) {
    let (mat, bore, gr) = build(sections, t, px);
    let mut stop = mat;
    for (bx, d) in plug {
        let axis = (0..3).find(|&i| d[i] != 0).unwrap();
        let sign = d[axis];
        let mut b3 = [0, 1, 2].map(|i| gr.sl(bx.0[i], bx.1[i], i));
        b3[axis] = if sign < 0 { gr.sl(bx.0[axis] - t, bx.0[axis], axis) } else { gr.sl(bx.1[axis], bx.1[axis] + t, axis) };
        gr.each(b3, |k, _| stop[k] = true);
    }
    let free: Vec<bool> = stop.iter().map(|x| !x).collect();
    let out = flood(&free, [gr.shape[2], gr.shape[1], gr.shape[0]]);
    let (mut leak, mut nb) = (0usize, 0usize);
    for k in 0..bore.len() {
        if bore[k] {
            nb += 1;
            if out[k] {
                leak += 1;
            }
        }
    }
    let parts = count_parts(&bore, [gr.shape[2], gr.shape[1], gr.shape[0]]);
    let v = py::powf(px, 3.0);
    (leak == 0, leak as f64 * v, nb as f64 * v, parts)
}
