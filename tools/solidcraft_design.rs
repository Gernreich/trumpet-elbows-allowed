//! A walk's bore as a parametric SolidCraft design (2026-11-08):
//! `viewer WALK --solidcraft FILE` writes it, and SolidCraft builds the STEP
//! file from it.
//!
//! One body per piece, each piece as the 3D page draws it: its blocks joined,
//! with the airway cut through them -- straight through a straight block,
//! in at one face and out at another through a turn. The bore and the ply are
//! parameters and the block is bore + 2 ply; on a stretched lattice the
//! straight is its own parameter. Every corner is a whole number of half
//! blocks (and half straights) from the mouth, so the whole bore follows
//! them. The walk is the design: a different walk is a different script.
//!
//! The pieces are solid blocks, as the page draws them: no finger joints,
//! laps or flats, no ports, and no mouthpiece or bell (those have their own
//! STEP files in parts/). North is +Y, east +X and up +Z.
use crate::bore::{axis_of, dirs, Design};
use lasermade_tools::solidcraft::{n as nn, Script};

/// A length as half blocks and half straights: b*block/2 + s*straight/2.
#[derive(Clone, Copy, PartialEq, Debug)]
struct L {
    b: i64,
    s: i64,
}

impl L {
    fn add(self, o: L) -> L {
        L { b: self.b + o.b, s: self.s + o.s }
    }
    fn mul(self, k: i64) -> L {
        L { b: self.b * k, s: self.s * k }
    }
    fn v(self, d: &Design) -> f64 {
        (self.b as f64 * d.block + self.s as f64 * d.straight) / 2.0
    }
    /// as an expression, with `extra` (a bore term) added
    fn e(self, extra: &str) -> String {
        let mut t: Vec<String> = vec![];
        if self.b != 0 {
            t.push(format!("{}*block", self.b));
        }
        if self.s != 0 {
            t.push(format!("{}*straight", self.s));
        }
        let half = if t.is_empty() { "0 mm".to_string() } else { format!("({})/2", t.join(" + ")) };
        if extra.is_empty() {
            half
        } else {
            format!("{half} + {extra}")
        }
    }
}

const Z: L = L { b: 0, s: 0 };

/// The page's frame (x east, y up, z south) to SolidCraft's (x east, y
/// north, z up).
fn world(p: [usize; 3]) -> [usize; 3] {
    // world axis k takes the page's axis [0, 2, 1][k]
    [p[0], p[2], p[1]]
}

pub fn script(d: &Design, text: &str) -> Result<String, String> {
    let sp = d.specs_for(text)?;
    let rec = &sp.rec;
    let stretched = !d.cubic();
    let mut s = Script::new();
    let bore = d.block - 2.0 * crate::bore::THICKNESS;
    s.len("bore", format!("{} mm", nn(bore)), "the airway, square, wall face to wall face");
    s.len("ply", format!("{} mm", nn(crate::bore::THICKNESS)), "the ply every wall is cut from, nominal");
    s.len("block", "bore + 2*ply", "read only: the lattice's pitch, a block's side");
    if stretched {
        s.len("straight", format!("{} mm", nn(d.straight)), "a straight block's length along the airway; turns stay a block");
    } else {
        s.len("straight", "block", "read only: a straight block is a block on this lattice");
    }
    // each block as (lo, hi) in half units, page frame, as block_boxes() works it
    let half = L { b: 1, s: 0 };
    let ext = |r: &crate::bore::Rec, a: usize| -> L {
        if r.inn == r.out && axis_of(r.out) == a {
            L { b: 0, s: 2 }
        } else {
            L { b: 2, s: 0 }
        }
    };
    let mut p = [Z; 3];
    let mut boxes: Vec<([L; 3], [L; 3])> = vec![];
    for r in rec {
        let (din, dout) = (dirs(r.inn), dirs(r.out));
        let a_in = axis_of(r.inn);
        let mut centre = [Z; 3];
        let mut hw = [half; 3];
        if r.inn != r.out {
            for i in 0..3 {
                centre[i] = p[i].add(half.mul(din[i]));
            }
        } else {
            let run = ext(r, a_in);
            for i in 0..3 {
                // centre = p + din*run/2: run is in half units, so halve it
                centre[i] = p[i].add(L { b: run.b / 2 * din[i], s: run.s / 2 * din[i] });
            }
            hw[a_in] = L { b: run.b / 2, s: run.s / 2 };
        }
        let lo = [0, 1, 2].map(|i| centre[i].add(hw[i].mul(-1)));
        let hi = [0, 1, 2].map(|i| centre[i].add(hw[i]));
        boxes.push((lo, hi));
        let a_out = axis_of(r.out);
        let out = if r.inn == r.out { ext(r, a_out) } else { L { b: 2, s: 0 } };
        for i in 0..3 {
            p[i] = centre[i].add(L { b: out.b / 2 * dout[i], s: out.s / 2 * dout[i] });
        }
    }
    // check against the page's own boxes
    let want = d.block_boxes(rec);
    for (k, ((lo, hi), (wl, wh))) in boxes.iter().zip(&want).enumerate() {
        for i in 0..3 {
            if (lo[i].v(d) - wl[i]).abs() > 1e-9 || (hi[i].v(d) - wh[i]).abs() > 1e-9 {
                return Err(format!("block {} of the design is off the page's", k + 1));
            }
        }
    }
    // the corner and size of a box, in SolidCraft's frame; a page z is south,
    // so world y = -page z and the box's corner takes the page's hi
    let place = |lo: [String; 3], hi: [String; 3], size: [String; 3]| -> ([String; 3], [String; 3]) {
        let w = world([0, 1, 2]);
        let corner = [lo[w[0]].clone(), format!("-({})", hi[w[1]]), lo[w[2]].clone()];
        (corner, [size[w[0]].clone(), size[w[1]].clone(), size[w[2]].clone()])
    };
    for (k, (gr, plan)) in sp.groups.iter().zip(&sp.plan).enumerate() {
        let body = format!("piece {} ({})", k + 1, plan.kind);
        for (j, &c) in gr.iter().enumerate() {
            let (lo, hi) = &boxes[c];
            let size = [0, 1, 2].map(|i| hi[i].add(lo[i].mul(-1)).e(""));
            let (corner, sz) = place([0, 1, 2].map(|i| lo[i].e("")), [0, 1, 2].map(|i| hi[i].e("")), size);
            let name = format!("{body} block {}", c + 1);
            if j == 0 {
                s.bx(&body, corner, sz, "new", &[]);
            } else {
                s.bx(&name, corner, sz, "join", &[&body]);
            }
        }
        // the airway: each block's, through its in face and its out face
        for &c in gr {
            let r = &rec[c];
            let (lo, hi) = &boxes[c];
            let centre: Vec<L> = (0..3).map(|i| L { b: lo[i].b + hi[i].b, s: lo[i].s + hi[i].s }).collect(); // twice the centre
            // a run from the face on side `sg` of axis `a` to the centre and bore/2 past it
            let mut runs: Vec<(usize, i64, bool)> = vec![];
            if r.inn == r.out {
                runs.push((axis_of(r.out), 0, false));
            } else {
                let (din, dout) = (dirs(r.inn), dirs(r.out));
                let ai = axis_of(r.inn);
                let ao = axis_of(r.out);
                runs.push((ai, -din[ai], true));
                runs.push((ao, dout[ao], true));
            }
            for (m, (a, sg, part)) in runs.into_iter().enumerate() {
                let mut l: [String; 3] = Default::default();
                let mut h: [String; 3] = Default::default();
                let mut sz: [String; 3] = Default::default();
                for i in 0..3 {
                    // centre[i] is twice the centre in half units: c/2
                    let cen = L { b: centre[i].b, s: centre[i].s };
                    let ce = |extra: &str| -> String {
                        let mut t: Vec<String> = vec![];
                        if cen.b != 0 {
                            t.push(format!("{}*block", cen.b));
                        }
                        if cen.s != 0 {
                            t.push(format!("{}*straight", cen.s));
                        }
                        let base = if t.is_empty() { "0 mm".into() } else { format!("({})/4", t.join(" + ")) };
                        format!("{base} {extra}")
                    };
                    if i != a {
                        l[i] = ce("- bore/2");
                        h[i] = ce("+ bore/2");
                        sz[i] = "bore".into();
                    } else if !part {
                        l[i] = lo[i].e("");
                        h[i] = hi[i].e("");
                        sz[i] = hi[i].add(lo[i].mul(-1)).e("");
                    } else if sg > 0 {
                        l[i] = ce("- bore/2");
                        h[i] = hi[i].e("");
                        sz[i] = format!("{} - ({}) + bore/2", hi[i].e(""), ce(""));
                    } else {
                        l[i] = lo[i].e("");
                        h[i] = ce("+ bore/2");
                        sz[i] = format!("{} + bore/2 - ({})", ce(""), lo[i].e(""));
                    }
                }
                let (corner, size) = place(l, h, sz);
                s.bx(&format!("{body} air {}{}", c + 1, ["", "b"][m]), corner, size, "cut", &[&body]);
            }
        }
    }
    s.measure();
    Ok(s.text())
}

/// Each piece's volume, as the script should build it: blocks less airway.
pub fn volumes(d: &Design, text: &str) -> Result<Vec<f64>, String> {
    let sp = d.specs_for(text)?;
    let bore = d.block - 2.0 * crate::bore::THICKNESS;
    let boxes = d.block_boxes(&sp.rec);
    Ok(sp
        .groups
        .iter()
        .map(|gr| {
            gr.iter()
                .map(|&c| {
                    let (lo, hi) = boxes[c];
                    let r = &sp.rec[c];
                    let len = if r.inn == r.out { hi[axis_of(r.out)] - lo[axis_of(r.out)] } else { d.block };
                    (0..3).map(|i| hi[i] - lo[i]).product::<f64>() - bore * bore * len
                })
                .sum()
        })
        .collect())
}
