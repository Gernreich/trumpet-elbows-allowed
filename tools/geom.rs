//! The polygon work shapely did for the trumpet tools, over geo (whose
//! boolean operations and buffers are i_overlay's). Only decisions and
//! rounded figures are taken from it: no cut path in a gated design passes
//! through here, which is what lets a second geometry engine stand in for
//! GEOS. The one route that writes geometry from it is a port (subtract
//! below), and no shipped design has a port.
use crate::bore::{open_ring, Part};
use geo::algorithm::buffer::{Buffer, BufferStyle, LineJoin};
use geo::{Area, BooleanOps, Coord, LineString, MultiPolygon, Polygon};
use std::f64::consts::PI;

pub type MP = MultiPolygon<f64>;

pub fn poly(q: &[(f64, f64)]) -> Polygon<f64> {
    let q = open_ring(q);
    Polygon::new(LineString::from(q.iter().map(|&(x, y)| Coord { x, y }).collect::<Vec<_>>()), vec![])
}

/// Polygon(q).buffer(0): the ring as a valid area
pub fn region(q: &[(f64, f64)]) -> MP {
    let p = MultiPolygon::new(vec![poly(q)]);
    p.union(&MultiPolygon::new(vec![]))
}

/// buffer(d, join_style=mitre, mitre_limit=20)
pub fn buffer_mitre(g: &MP, d: f64) -> MP {
    if g.0.is_empty() {
        return g.clone();
    }
    // a mitre limit of 20 allows any corner sharper than 2 asin(1/20)
    g.buffer_with_style(BufferStyle::new(d).line_join(LineJoin::Miter(2.0 * (1.0f64 / 20.0).asin())))
}

/// buffer(d) with round joins, sixteen segments a quarter circle
pub fn buffer_round(g: &MP, d: f64) -> MP {
    if g.0.is_empty() {
        return g.clone();
    }
    g.buffer_with_style(BufferStyle::new(d).line_join(LineJoin::Round(PI / 32.0)))
}

pub fn area(g: &MP) -> f64 {
    g.unsigned_area()
}

/// Trim slivers the cut leaves behind, wherever they are.
pub fn shave_stubs(mut p: MP, minw: f64) -> MP {
    let w = minw / 4.0;
    for _ in 0..6 {
        let opened = buffer_mitre(&buffer_mitre(&p, -w), w);
        let thin = p.difference(&opened);
        let mut took = false;
        for b in thin.0.iter() {
            let bm = MultiPolygon::new(vec![b.clone()]);
            if b.unsigned_area() <= 0.02 {
                continue;
            }
            let trimmed = p.difference(&buffer_round(&bm, 0.01));
            if trimmed.0.len() == 1 && area(&p) - area(&trimmed) < 1.5 * b.unsigned_area() + 0.5 {
                p = trimmed;
                took = true;
            }
        }
        if !took {
            break;
        }
    }
    p
}

fn ring_of(ls: &LineString<f64>, role: char) -> Part {
    let mut pts: Vec<(f64, f64)> = ls.0.iter().map(|c| (c.x, c.y)).collect();
    if pts.len() > 1 && pts[0] == pts[pts.len() - 1] {
        pts.pop();
    }
    let d = format!("M {} Z", pts.iter().map(|(x, y)| format!("{x:.3} {y:.3}")).collect::<Vec<_>>().join(" L "));
    pts.push(pts[0]);
    Part::of(d, pts, role)
}

/// Cut the marked region out of a part, keeping its remaining edges.
pub fn subtract(part: &Part, mark: &Part, _thickness: f64) -> Part {
    let left = region(&part.pts).difference(&buffer_round(&region(&mark.pts), 0.2));
    let biggest = left.0.iter().cloned().reduce(|a, b| if b.unsigned_area() > a.unsigned_area() { b } else { a });
    let Some(biggest) = biggest else { return part.clone() };
    let left = shave_stubs(MultiPolygon::new(vec![biggest]), 3.0);
    let Some(shape) = left.0.first() else { return part.clone() };
    let mut out = ring_of(shape.exterior(), part.role);
    out.holes = part.holes.clone();
    out.holes.extend(shape.interiors().iter().map(|r| ring_of(r, part.role)));
    out.name = part.name.clone();
    out
}
