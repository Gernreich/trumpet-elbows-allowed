//! `nest WALK [--out FILE] [--deepnest FILE] [--tries N] [design switches]`:
//! nest a bore's parts onto sheets, four ways, and report which wins.
//! nest.py until 2026-11-08.
//!
//!   shelf     rows of parts, tallest first
//!   maxrects  free-rectangle packing, best short side fit
//!   skyline   bottom-left against a skyline profile
//!   raster    irregular: the real outline on a 1 mm raster, four rotations
//!
//! Writes one sheet per method beside you -- nest_shelf.svg, nest_maxrects.svg,
//! nest_skyline.svg, nest_raster.svg -- and --out renames the stem rather than
//! the file. The first three pack bounding boxes, so they can never do better
//! than the sum of those boxes. Only `raster` can put a wall inside an L-shaped
//! plate's notch, which is where most of the waste is.
//!
//! nest.py found a free raster spot by FFT convolution and kept the cells whose
//! overlap came out under 0.5; the overlap is a count of whole pixels, so this
//! asks the same question exactly: does any pixel of the part land on a blocked
//! one. Same answer, no rounding to trust.
use lasermade_tools::py;
use std::collections::BTreeMap;
use trumpet::bore::{splitext, sys_exit, walk_text, Design, Part, BED_H, BED_W, DESIGN_HELP};

const GAP: f64 = 4.0;
/// raster pixel, mm
const PX: f64 = 1.0;

struct P {
    part: Part,
    label: String,
}

/// Every part of every section, tagged with its section number.
fn parts_of(d: &Design, text: &str) -> Result<Vec<P>, String> {
    let sp = d.specs_for(text)?;
    let mut out = vec![];
    for (i, p) in sp.plan.iter().enumerate() {
        for part in d.cut(&p.args)? {
            out.push(P { part, label: (i + 1).to_string() });
        }
    }
    Ok(out)
}

/// (sheet, part index, rotation, x, y)
type Placed = (usize, usize, i64, f64, f64);

fn by_desc(ps: &[P], key: impl Fn(&Part) -> f64) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..ps.len()).collect();
    idx.sort_by(|&a, &b| key(&ps[a].part).partial_cmp(&key(&ps[b].part)).unwrap());
    idx
}

fn shelf(ps: &[P], bw: f64, bh: f64) -> Vec<Placed> {
    struct Sh {
        x: f64,
        y: f64,
        h: f64,
    }
    let mut sheets: Vec<Vec<Sh>> = vec![];
    let mut placed = vec![];
    for k in by_desc(ps, |p| -py::max(p.w, p.h)) {
        let (w, h) = (ps[k].part.w, ps[k].part.h);
        let mut done = false;
        'sheets: for (si, sh) in sheets.iter_mut().enumerate() {
            for s in sh.iter_mut() {
                for (r, a, b) in [(0, w, h), (90, h, w)] {
                    if s.x + a <= bw - GAP && b <= s.h {
                        placed.push((si, k, r, s.x, s.y));
                        s.x += a + GAP;
                        done = true;
                        break 'sheets;
                    }
                }
            }
            let (r, a, b) = if w >= h { (0, w, h) } else { (90, h, w) };
            let y = sh.iter().map(|s| s.y + s.h).fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| py::max(m, v)))).unwrap_or(0.0) + GAP;
            if y + b <= bh - GAP && a <= bw - 2.0 * GAP {
                sh.push(Sh { x: GAP + a + GAP, y, h: b });
                placed.push((si, k, r, GAP, y));
                done = true;
                break;
            }
        }
        if !done {
            let (r, a, b) = if w >= h { (0, w, h) } else { (90, h, w) };
            sheets.push(vec![Sh { x: GAP + a + GAP, y: GAP, h: b }]);
            placed.push((sheets.len() - 1, k, r, GAP, GAP));
        }
    }
    placed
}

type Rect = (f64, f64, f64, f64);

fn lt2(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0 < b.0 || (a.0 == b.0 && a.1 < b.1)
}

fn maxrects(ps: &[P], bw: f64, bh: f64) -> Vec<Placed> {
    let split = |f: Rect, r: Rect| -> Vec<Rect> {
        let (fx, fy, fw, fh) = f;
        let (rx, ry, rw, rh) = r;
        if rx >= fx + fw || rx + rw <= fx || ry >= fy + fh || ry + rh <= fy {
            return vec![f];
        }
        let mut out = vec![];
        if ry > fy {
            out.push((fx, fy, fw, ry - fy));
        }
        if ry + rh < fy + fh {
            out.push((fx, ry + rh, fw, fy + fh - ry - rh));
        }
        if rx > fx {
            out.push((fx, fy, rx - fx, fh));
        }
        if rx + rw < fx + fw {
            out.push((rx + rw, fy, fx + fw - rx - rw, fh));
        }
        out
    };
    let prune = |fs: &[Rect]| -> Vec<Rect> {
        let mut keep = vec![];
        for (i, a) in fs.iter().enumerate() {
            if a.2 <= 0.0 || a.3 <= 0.0 {
                continue;
            }
            if fs.iter().enumerate().any(|(j, b)| b.2 > 0.0 && b.3 > 0.0 && i != j && a.0 >= b.0 && a.1 >= b.1 && a.0 + a.2 <= b.0 + b.2 && a.1 + a.3 <= b.1 + b.3) {
                continue;
            }
            keep.push(*a);
        }
        keep
    };
    let mut bins: Vec<Vec<Rect>> = vec![];
    let mut placed = vec![];
    for k in by_desc(ps, |p| -p.w * p.h) {
        let (w, h) = (ps[k].part.w, ps[k].part.h);
        let mut best: Option<((f64, f64), usize, Rect, i64, f64, f64)> = None;
        for (bi, free) in bins.iter().enumerate() {
            for &f in free {
                for (r, a, b) in [(0, w + GAP, h + GAP), (90, h + GAP, w + GAP)] {
                    if a <= f.2 && b <= f.3 {
                        let key = (py::min(f.2 - a, f.3 - b), py::max(f.2 - a, f.3 - b));
                        if best.as_ref().map_or(true, |bb| lt2(key, bb.0)) {
                            best = Some((key, bi, f, r, a, b));
                        }
                    }
                }
            }
        }
        let (bi, f, r, a, b) = match best {
            None => {
                let f = (0.0, 0.0, bw - 2.0 * GAP, bh - 2.0 * GAP);
                bins.push(vec![f]);
                let (r, a, b) = if w >= h { (0, w + GAP, h + GAP) } else { (90, h + GAP, w + GAP) };
                (bins.len() - 1, f, r, a, b)
            }
            Some((_, bi, f, r, a, b)) => (bi, f, r, a, b),
        };
        let (x, y) = (f.0, f.1);
        placed.push((bi, k, r, x + GAP, y + GAP));
        let new: Vec<Rect> = bins[bi].iter().flat_map(|&g| split(g, (x, y, a, b))).collect();
        bins[bi] = prune(&new);
    }
    placed
}

type Seg = (f64, f64, f64);

fn skyline(ps: &[P], bw: f64, bh: f64) -> Vec<Placed> {
    let fit = |sky: &[Seg], a: f64, b: f64| -> Option<(f64, f64)> {
        let mut best: Option<((f64, f64), f64, f64)> = None;
        for i in 0..sky.len() {
            let x = sky[i].0;
            if x + a > bw {
                continue;
            }
            let (mut rem, mut y, mut j) = (a, 0.0, i);
            while rem > 1e-9 && j < sky.len() {
                y = py::max(y, sky[j].1);
                rem -= sky[j].2;
                j += 1;
            }
            if rem > 1e-9 || y + b > bh {
                continue;
            }
            let key = (y + b, x);
            if best.as_ref().map_or(true, |bb| lt2(key, bb.0)) {
                best = Some((key, x, y));
            }
        }
        best.map(|b| (b.1, b.2))
    };
    let add = |sky: &[Seg], x: f64, y: f64, a: f64, b: f64| -> Vec<Seg> {
        let mut out: Vec<Seg> = vec![];
        for &(sx, sy, sw) in sky {
            if sx + sw <= x || sx >= x + a {
                out.push((sx, sy, sw));
                continue;
            }
            if sx < x {
                out.push((sx, sy, x - sx));
            }
            if sx + sw > x + a {
                out.push((x + a, sy, sx + sw - x - a));
            }
        }
        out.push((x, y + b, a));
        out.sort_by(|p, q| p.partial_cmp(q).unwrap());
        let mut merged = vec![out[0]];
        for seg in &out[1..] {
            let m = merged.last_mut().unwrap();
            if (seg.1 - m.1).abs() < 1e-9 {
                m.2 += seg.2;
            } else {
                merged.push(*seg);
            }
        }
        merged
    };
    let mut sheets: Vec<Vec<Seg>> = vec![];
    let mut placed = vec![];
    for k in by_desc(ps, |p| -py::max(p.w, p.h)) {
        let (w, h) = (ps[k].part.w, ps[k].part.h);
        let mut done = false;
        'sheets: for si in 0..sheets.len() {
            for (r, a, b) in [(0, w + GAP, h + GAP), (90, h + GAP, w + GAP)] {
                if let Some((x, y)) = fit(&sheets[si], a, b) {
                    placed.push((si, k, r, x + GAP / 2.0, y + GAP / 2.0));
                    sheets[si] = add(&sheets[si], x, y, a, b);
                    done = true;
                    break 'sheets;
                }
            }
        }
        if !done {
            sheets.push(vec![(0.0, 0.0, bw)]);
            let (r, a, b) = if w >= h { (0, w + GAP, h + GAP) } else { (90, h + GAP, w + GAP) };
            placed.push((sheets.len() - 1, k, r, GAP / 2.0, GAP / 2.0));
            let n = sheets.len() - 1;
            sheets[n] = add(&sheets[n], 0.0, 0.0, a, b);
        }
    }
    placed
}

// --- raster ----------------------------------------------------------------------

/// shapely's contains: strictly inside, a point on the outline is not.
fn contains(ring: &[(f64, f64)], x: f64, y: f64) -> bool {
    let n = ring.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        let cross = (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0);
        if cross == 0.0 && x >= a.0.min(b.0) && x <= a.0.max(b.0) && y >= a.1.min(b.1) && y <= a.1.max(b.1) {
            return false;
        }
        if (a.1 > y) != (b.1 > y) {
            let xc = a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1);
            if x < xc {
                inside = !inside;
            }
        }
    }
    inside
}

/// A part's real outline as a pixel mask, row by row: (rotation, rows, nx).
/// Each row is the runs [c0, c1) of pixels whose centre the outline holds.
struct Mask {
    rot: i64,
    nx: usize,
    ny: usize,
    rows: Vec<Vec<(usize, usize)>>,
}

/// Masks of the real outline at 0/90/180/270, true size. The gap is kept by
/// growing what is already placed, not the part being placed: a grown mask
/// can overhang the sheet edge even when the part itself fits.
fn raster_masks(p: &Part) -> Vec<Mask> {
    let pts = if p.pts[0] == p.pts[p.pts.len() - 1] { &p.pts[..p.pts.len() - 1] } else { &p.pts[..] };
    let poly: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x - p.x0, y - p.y0)).collect();
    let mut out = vec![];
    for rot in [0i64, 90, 180, 270] {
        // shapely.affinity.rotate about the origin, its near-zero terms zeroed
        let (c, s) = match rot {
            0 => (1.0, 0.0),
            90 => (0.0, 1.0),
            180 => (-1.0, 0.0),
            _ => (0.0, -1.0),
        };
        let g: Vec<(f64, f64)> = poly.iter().map(|&(x, y)| (c * x - s * y, s * x + c * y)).collect();
        let minx = trumpet::bore::fmin(g.iter().map(|q| q.0));
        let miny = trumpet::bore::fmin(g.iter().map(|q| q.1));
        let maxx = trumpet::bore::fmax(g.iter().map(|q| q.0));
        let maxy = trumpet::bore::fmax(g.iter().map(|q| q.1));
        let g: Vec<(f64, f64)> = g.iter().map(|&(x, y)| (x - minx, y - miny)).collect();
        let nx = 1.max(((maxx - minx) / PX).ceil() as usize);
        let ny = 1.max(((maxy - miny) / PX).ceil() as usize);
        let rows = (0..ny)
            .map(|iy| {
                let mut runs = vec![];
                let mut start: Option<usize> = None;
                for ix in 0..=nx {
                    let on = ix < nx && contains(&g, (ix as f64 + 0.5) * PX, (iy as f64 + 0.5) * PX);
                    match (on, start) {
                        (true, None) => start = Some(ix),
                        (false, Some(s0)) => {
                            runs.push((s0, ix));
                            start = None;
                        }
                        _ => {}
                    }
                }
                runs
            })
            .collect();
        out.push(Mask { rot, nx, ny, rows });
    }
    out
}

/// One sheet's pixels, what the gap blocks around them, and per row a running
/// count of blocked pixels, so a run's overlap is two lookups.
struct Sheet {
    nx: usize,
    ny: usize,
    on: Vec<bool>,
    count: Vec<Vec<u32>>,
}

impl Sheet {
    fn new(nx: usize, ny: usize) -> Sheet {
        Sheet { nx, ny, on: vec![false; nx * ny], count: vec![vec![0; nx + 1]; ny] }
    }
    fn put(&mut self, m: &Mask, ix: usize, iy: usize) {
        for (r, runs) in m.rows.iter().enumerate() {
            for &(c0, c1) in runs {
                for c in c0..c1 {
                    self.on[(iy + r) * self.nx + ix + c] = true;
                }
            }
        }
        // binary_dilation by an n x n square, n = 2 ceil(GAP/PX) + 3
        let k = (GAP / PX).ceil() as usize + 1; // half of n
        let (nx, ny) = (self.nx, self.ny);
        // along each row, then down each column, by running counts
        let mut hz = vec![false; nx * ny];
        for y in 0..ny {
            let mut pre = vec![0u32; nx + 1];
            for x in 0..nx {
                pre[x + 1] = pre[x] + self.on[y * nx + x] as u32;
            }
            for x in 0..nx {
                hz[y * nx + x] = pre[(x + k + 1).min(nx)] != pre[x.saturating_sub(k)];
            }
        }
        for x in 0..nx {
            let mut pre = vec![0u32; ny + 1];
            for y in 0..ny {
                pre[y + 1] = pre[y] + hz[y * nx + x] as u32;
            }
            for y in 0..ny {
                let b = pre[(y + k + 1).min(ny)] != pre[y.saturating_sub(k)];
                self.count[y][x + 1] = b as u32;
            }
        }
        for y in 0..ny {
            for x in 0..nx {
                self.count[y][x + 1] += self.count[y][x];
            }
        }
    }
    /// The first free (row, column) for this mask, rows first.
    fn first_free(&self, m: &Mask) -> Option<(usize, usize)> {
        if m.ny > self.ny || m.nx > self.nx {
            return None;
        }
        for iy in 0..=self.ny - m.ny {
            'x: for ix in 0..=self.nx - m.nx {
                for (r, runs) in m.rows.iter().enumerate() {
                    let row = &self.count[iy + r];
                    for &(c0, c1) in runs {
                        if row[ix + c1] != row[ix + c0] {
                            continue 'x;
                        }
                    }
                }
                return Some((iy, ix));
            }
        }
        None
    }
}

fn raster(ps: &[P], bw: f64, bh: f64, order: Option<&[usize]>) -> Vec<Placed> {
    let (nx, ny) = (((bw - 2.0 * GAP) / PX) as usize, ((bh - 2.0 * GAP) / PX) as usize);
    let default = by_desc(ps, |p| -p.w * p.h);
    let order = order.unwrap_or(&default);
    let mut sheets: Vec<Sheet> = vec![];
    let mut placed = vec![];
    for &k in order {
        let p = &ps[k].part;
        let masks = raster_masks(p);
        let mut done = false;
        for si in 0..sheets.len() {
            let mut best: Option<((usize, usize), usize)> = None;
            for (mi, m) in masks.iter().enumerate() {
                if let Some(at) = sheets[si].first_free(m) {
                    if best.map_or(true, |b| at < b.0) {
                        best = Some((at, mi));
                    }
                }
            }
            if let Some(((iy, ix), mi)) = best {
                sheets[si].put(&masks[mi], ix, iy);
                placed.push((si, k, masks[mi].rot, GAP + ix as f64 * PX, GAP + iy as f64 * PX));
                done = true;
                break;
            }
        }
        if !done {
            sheets.push(Sheet::new(nx, ny));
            let si = sheets.len() - 1;
            let Some(m) = masks.iter().find(|m| m.ny <= ny && m.nx <= nx) else {
                sys_exit(&format!("a part is {:.0}x{:.0} mm and will not fit the bed in any rotation", p.w, p.h));
            };
            sheets[si].put(m, 0, 0);
            placed.push((si, k, m.rot, GAP, GAP));
        }
    }
    placed
}

// --- output ----------------------------------------------------------------------

fn emit(d: &Design, ps: &[P], placed: &[Placed], path: &str, tag: &str) -> Vec<(String, f64, f64, usize)> {
    let mut bysheet: BTreeMap<usize, Vec<(usize, i64, f64, f64)>> = BTreeMap::new();
    for &(si, k, rot, x, y) in placed {
        bysheet.entry(si).or_default().push((k, rot, x, y));
    }
    let mut out = vec![];
    let nsheets = bysheet.len();
    for (si, here) in &bysheet {
        let (mut body, mut xs, mut ys) = (vec![], vec![], vec![]);
        for &(k, rot, x, y) in here {
            let p = &ps[k].part;
            let (w, h) = (p.w, p.h);
            let t = match rot {
                0 => format!("translate({x:.3},{y:.3})"),
                90 => format!("translate({:.3},{y:.3}) rotate(90)", x + h),
                180 => format!("translate({:.3},{:.3}) rotate(180)", x + w, y + h),
                _ => format!("translate({x:.3},{:.3}) rotate(270)", y + w),
            };
            let inner: String = p.holes.iter().map(|q| format!("<path d=\"{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.2\"/>", q.d)).collect();
            let local = format!("<g transform=\"translate({:.3},{:.3})\">{inner}<path d=\"{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.2\"/></g>", -p.x0, -p.y0, p.d);
            let lbl = d.part_labels(p, &ps[k].label, 1);
            body.push(format!("<g transform=\"{t}\">{local}{lbl}</g>"));
            let (ww, hh) = if rot == 90 || rot == 270 { (h, w) } else { (w, h) };
            xs.extend([x, x + ww]);
            ys.extend([y, y + hh]);
        }
        let (ww, hh) = (trumpet::bore::fmax(xs.into_iter()) + GAP, trumpet::bore::fmax(ys.into_iter()) + GAP);
        let (stem, ext) = splitext(path);
        let this = if nsheets > 1 { format!("{stem}_{tag}_{}{ext}", si + 1) } else { format!("{stem}_{tag}{ext}") };
        let svg = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{ww:.2}mm\" height=\"{hh:.2}mm\" viewBox=\"0 0 {ww:.2} {hh:.2}\">\n<!-- nested by {tag}, {GAP:.0} mm apart; each part carries its section number -->\n{}\n</svg>\n",
            body.join("\n")
        );
        std::fs::write(&this, svg).unwrap_or_else(|e| sys_exit(&format!("{this}: {e}")));
        out.push((this, ww, hh, here.len()));
    }
    out
}

/// Shift an absolute path's coordinates, keeping every curve exact. Nesters
/// vary in how well they apply a <g transform>, so the file hands them plain
/// top-level paths with nothing left to interpret.
fn bake(d: &str, dx: f64, dy: f64) -> Result<String, String> {
    let r = lasermade_tools::re::rx(r"[MLHVCSQTAZmlhvcsqtaz]|-?\d*\.?\d+");
    let tok: Vec<&str> = lasermade_tools::re::all(&r, d).iter().map(|c| c.get(0).unwrap().as_str()).collect();
    let is_cmd = |t: &str| t.len() == 1 && "MLHVCSQTAZmlhvcsqtaz".contains(t);
    let (mut out, mut i) = (vec![], 0);
    while i < tok.len() {
        let c = tok[i];
        if c == "Z" || c == "z" {
            out.push("Z".to_string());
            i += 1;
            continue;
        }
        if !(c.len() == 1 && "MLHVCSQTA".contains(c)) {
            return Err(format!("relative or unknown command {}; this only shifts absolute paths", py::str_repr(c)));
        }
        let step = match c {
            "M" | "L" | "T" => 2,
            "H" | "V" => 1,
            "C" => 6,
            "S" | "Q" => 4,
            _ => 7,
        };
        i += 1;
        out.push(c.to_string());
        let mut vals = vec![];
        while i < tok.len() && !is_cmd(tok[i]) {
            vals.push(py::float(tok[i]).unwrap());
            i += 1;
        }
        for g in vals.chunks(step) {
            let g: Vec<f64> = match c {
                "H" => vec![g[0] + dx],
                "V" => vec![g[0] + dy],
                "A" if g.len() == 7 => {
                    let mut v = g[..5].to_vec();
                    v.extend([g[5] + dx, g[6] + dy]);
                    v
                }
                _ => g.iter().enumerate().map(|(k, v)| v + if k % 2 == 0 { dx } else { dy }).collect(),
            };
            out.push(g.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(" "));
        }
    }
    Ok(out.join(" "))
}

/// Sheet and parts as plain closed paths, in two files: the sheet has to be
/// marked as such, and on its own it is the only thing there to mark.
/// Engraved numbers are left out: a nester imports open paths as parts.
fn deepnest_input(ps: &[P], path: &str, bw: f64, bh: f64) -> Result<(String, String, f64, f64), String> {
    let (stem, ext) = splitext(path);
    let (sheet_f, parts_f) = (format!("{stem}_sheet{ext}"), format!("{stem}_parts{ext}"));
    std::fs::write(
        &sheet_f,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{bw:.2}mm\" height=\"{bh:.2}mm\" viewBox=\"0 0 {bw:.2} {bh:.2}\">\n<!-- The sheet: one {bw:.0} x {bh:.0} mm rectangle and nothing else, so it is the only thing to mark. -->\n<path d=\"M 0 0 L {bw:.2} 0 L {bw:.2} {bh:.2} L 0 {bh:.2} Z\" fill=\"none\" stroke=\"#ff0000\" stroke-width=\"0.2\"/>\n</svg>\n"
        ),
    )
    .map_err(|e| e.to_string())?;
    let (mut body, mut x, mut y, mut rh) = (vec![], 0.0, 0.0, 0.0f64);
    for k in by_desc(ps, |p| -p.w * p.h) {
        let p = &ps[k].part;
        if x > 0.0 && x + p.w > bw * 2.0 {
            (x, y, rh) = (0.0, y + rh + 10.0, 0.0);
        }
        for q in std::iter::once(p).chain(p.holes.iter()) {
            body.push(format!("<path d=\"{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.2\"/>", bake(&q.d, x - p.x0, y - p.y0)?));
        }
        x += p.w + 10.0;
        rh = py::max(rh, p.h);
    }
    let (ww, hh) = (bw * 2.0 + 20.0, y + rh + 20.0);
    std::fs::write(
        &parts_f,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{ww:.2}mm\" height=\"{hh:.2}mm\" viewBox=\"0 0 {ww:.2} {hh:.2}\">\n<!-- {} parts, each one closed path at top level with its holes inside it, no groups, no transforms, no engraving. -->\n{}\n</svg>\n",
            ps.len(),
            body.join("\n")
        ),
    )
    .map_err(|e| e.to_string())?;
    Ok((sheet_f, parts_f, ww, hh))
}

fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap()
}

const USAGE: &str = "usage: nest [-h] [--out OUT] [--deepnest FILE] [--tries N] walk [walk ...]";

fn main() {
    let mut d = Design::default();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let rest = d.take_design_switches(&argv);
    let (mut walk, mut out, mut deep, mut tries) = (vec![], "nest.svg".to_string(), None, 0i64);
    let bad = |m: String| -> ! {
        eprintln!("{USAGE}\nnest: error: {m}");
        std::process::exit(2)
    };
    let mut i = 0;
    while i < rest.len() {
        let a = rest[i].as_str();
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k, Some(v.to_string())),
            _ => (a, None),
        };
        match key {
            "-h" | "--help" => {
                println!("{USAGE}\n\nNest a bore's parts onto sheets, four ways, and report which wins.\n\n{DESIGN_HELP}");
                return;
            }
            "--out" | "--deepnest" | "--tries" => {
                let v = match inline {
                    Some(v) => v,
                    None => {
                        i += 1;
                        rest.get(i).cloned().unwrap_or_else(|| bad(format!("argument {key}: expected one argument")))
                    }
                };
                match key {
                    "--out" => out = v,
                    "--deepnest" => deep = Some(v),
                    _ => tries = trumpet::bore::pyint(&v).unwrap_or_else(|| bad(format!("argument --tries: invalid int value: {}", py::str_repr(&v)))),
                }
            }
            _ if a.starts_with('-') && a.len() > 1 && !a[1..].starts_with(|c: char| c.is_ascii_digit()) => bad(format!("unrecognized arguments: {a}")),
            _ => walk.push(a.to_string()),
        }
        i += 1;
    }
    if walk.is_empty() {
        bad("the following arguments are required: walk".into());
    }
    let ps = match walk_text(&walk.join(" ")).and_then(|t| parts_of(&d, &t)) {
        Ok(p) => p,
        Err(e) => sys_exit(&format!("ValueError: {e}")),
    };
    if let Some(f) = deep {
        let (sf, pf, ww, hh) = deepnest_input(&ps, &f, BED_W, BED_H).unwrap_or_else(|e| sys_exit(&format!("ValueError: {e}")));
        println!("sheet -> {}  ({BED_W:.0} x {BED_H:.0} mm, one rectangle)", basename(&sf));
        println!("{} parts -> {}  ({ww:.0} x {hh:.0} mm)", ps.len(), basename(&pf));
        return;
    }
    let area = |q: &[(f64, f64)]| -> f64 {
        let n = q.len();
        (0..n).fold(0.0, |s, i| s + (q[i].0 * q[(i + 1) % n].1 - q[(i + 1) % n].0 * q[i].1)).abs() / 2.0
    };
    let poly = ps.iter().fold(0.0, |s, p| s + area(&p.part.pts));
    let bed = BED_W * BED_H;
    let mut o = format!("{} parts, {poly:.0} mm2 of material = {:.2} sheets\n\n", ps.len(), poly / bed);
    o.push_str(&format!("{:<10}{:>7}{:>8}   files\n", "method", "sheets", "used"));
    print!("{o}");
    let mut results: Vec<(&str, usize)> = vec![];
    let mut raster_n = 0;
    type Nester = fn(&[P], f64, f64) -> Vec<Placed>;
    let methods: [(&str, Nester); 4] = [("shelf", shelf), ("maxrects", maxrects), ("skyline", skyline), ("raster", |p, w, h| raster(p, w, h, None))];
    for (name, f) in methods {
        let placed = f(&ps, BED_W, BED_H);
        if name == "raster" {
            raster_n = placed.iter().map(|q| q.0).collect::<std::collections::HashSet<_>>().len();
        }
        let files = emit(&d, &ps, &placed, &out, name);
        let n = files.len();
        let used = poly / (n as f64 * bed);
        results.push((name, n));
        println!("{name:<10}{n:>7}{:>7.0}%   {}", 100.0 * used, files.iter().map(|f| basename(&f.0)).collect::<Vec<_>>().join(", "));
    }
    let best = results.iter().fold(results[0], |b, r| if r.1 < b.1 { *r } else { b });
    println!("\nbest: {} at {} sheet(s)", best.0, best.1);

    if tries > 0 {
        // unseeded, as nest.py's was: the search is not meant to repeat
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|t| t.as_nanos() as i64).unwrap_or(0);
        let mut rng = trumpet::pyrandom::Mt::new(seed);
        let base = by_desc(&ps, |p| -p.w * p.h);
        let (mut champ, mut champ_n, mut champ_last): (Option<Vec<Placed>>, usize, Option<usize>) = (None, raster_n, None);
        let t0 = std::time::Instant::now();
        println!("\nsearching {tries} orderings for the raster nester");
        for i in 0..tries {
            let mut order = base.clone();
            let k = rng.randint(1, 2.max(order.len() as i64 / 2));
            for _ in 0..k {
                let x = rng.randrange(order.len());
                let y = rng.randrange(order.len());
                order.swap(x, y);
            }
            let placed = raster(&ps, BED_W, BED_H, Some(&order));
            let n = placed.iter().map(|q| q.0).collect::<std::collections::HashSet<_>>().len();
            let last = placed.iter().filter(|q| q.0 == n - 1).count();
            if n < champ_n || (n == champ_n && champ_last.is_some_and(|c| last < c)) {
                (champ, champ_n, champ_last) = (Some(placed), n, Some(last));
                println!("   try {}: {n} sheets, {last} parts on the last one", i + 1);
            }
        }
        println!("   {tries} tries in {:.0}s; best {champ_n} sheet(s)", t0.elapsed().as_secs_f64());
        if let Some(c) = champ {
            for f in emit(&d, &ps, &c, &out, "rastersearch") {
                println!("   wrote {}", basename(&f.0));
            }
        }
    }
}
