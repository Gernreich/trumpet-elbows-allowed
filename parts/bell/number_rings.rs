//! Engrave a hex index inside each ring of a bell sheet, smallest ring 0.
//!
//!     target/release/number_rings SHEET.svg
//!     target/release/number_rings SHEET.svg --cap=3.0          # smaller digits than the wall allows
//!     target/release/number_rings SHEET.svg --order=document   # a profile that doubles back
//!     target/release/number_rings SHEET.svg --start=26         # continue an existing stack
//!     target/release/number_rings SHEET.svg --mark=no          # no orientation mark
//!
//! Rust since 2026-11-07 (number_rings.py until then; the move changed no
//! byte of any sheet it numbers). Why each ring carries its own number, why
//! hex, why a tick beside the last character, why the digits are strokes and
//! how the size is fitted is all in NOTES.md, [Numbering a sheet]; in short:
//!
//! A RING HAS NO TOP, so every label carries a mark: a short horizontal tick
//! on the baseline, to the right of the last character, the seven-segment
//! display's own answer to 6 and 9. It is a tick, not a dot, because a dot
//! would be a zero-length path an engraver may not draw.
//!
//! SMALLEST FIRST ONLY WHERE THAT IS ALSO ASSEMBLY ORDER. A mouthpiece narrows
//! to the throat and opens again, so sorting by size shuffles its backbore into
//! its cup. When the sizes are not monotonic in the file this refuses to guess;
//! --order=document numbers them as written.
//!
//! BLUE ENGRAVES (#0000ff), written before the black that frees the part.
//!
//! FITTING IS MEASURED, NOT ASSUMED. Each digit is placed at the bottom of its
//! ring and grown to the largest size where EVERY point of it still lands
//! between that ring's aperture and its outer edge, measured against the real
//! curves. A digit that cannot be fitted is reported and nothing is written.
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, hypot, matan2 as atan2, mcos as cos, msin as sin};
use lasermade_tools::re;
use std::f64::consts::PI;

const BLUE: &str = "#0000ff"; // blue engraves
const MARGIN: f64 = 0.30; // clear of both edges, mm
const CAP: f64 = 4.0; // no digit larger than this however wide the wall
const GROUP: &str = "ring-numbers"; // replaced wholesale if the sheet already has one

type P = (f64, f64);

/// The hex alphabet, as polylines in a unit box: (0,0) bottom-left, (1,1)
/// top-right. Kept in step with bore_split's G table by hand.
fn glyph(ch: char) -> Vec<Vec<P>> {
    let g: &[&[P]] = match ch {
        '0' => &[&[(0.5, 1.0), (0.85, 0.8), (0.85, 0.2), (0.5, 0.0), (0.15, 0.2), (0.15, 0.8), (0.5, 1.0)]],
        '1' => &[&[(0.3, 0.78), (0.52, 1.0), (0.52, 0.0)], &[(0.28, 0.0), (0.78, 0.0)]],
        '2' => &[&[(0.1, 0.78), (0.3, 1.0), (0.7, 1.0), (0.9, 0.78), (0.9, 0.6), (0.1, 0.0), (0.9, 0.0)]],
        '3' => &[&[(0.1, 1.0), (0.9, 1.0), (0.45, 0.55)], &[(0.45, 0.55), (0.9, 0.55), (0.9, 0.16), (0.72, 0.0), (0.28, 0.0), (0.1, 0.16)]],
        '4' => &[&[(0.7, 0.0), (0.7, 1.0), (0.12, 0.32), (0.92, 0.32)]],
        '5' => &[&[(0.85, 1.0), (0.2, 1.0), (0.15, 0.55), (0.5, 0.62), (0.8, 0.5), (0.88, 0.28), (0.75, 0.06), (0.4, 0.0), (0.15, 0.12)]],
        '6' => &[&[
            (0.82, 0.92),
            (0.55, 1.0),
            (0.25, 0.85),
            (0.15, 0.45),
            (0.15, 0.18),
            (0.35, 0.0),
            (0.62, 0.0),
            (0.85, 0.18),
            (0.85, 0.38),
            (0.62, 0.55),
            (0.3, 0.55),
            (0.15, 0.45),
        ]],
        '7' => &[&[(0.12, 1.0), (0.9, 1.0), (0.42, 0.0)]],
        '8' => &[&[
            (0.5, 0.55),
            (0.22, 0.68),
            (0.22, 0.87),
            (0.5, 1.0),
            (0.78, 0.87),
            (0.78, 0.68),
            (0.5, 0.55),
            (0.18, 0.4),
            (0.18, 0.14),
            (0.5, 0.0),
            (0.82, 0.14),
            (0.82, 0.4),
            (0.5, 0.55),
        ]],
        '9' => &[&[
            (0.18, 0.08),
            (0.45, 0.0),
            (0.75, 0.15),
            (0.85, 0.55),
            (0.85, 0.82),
            (0.65, 1.0),
            (0.38, 1.0),
            (0.15, 0.82),
            (0.15, 0.62),
            (0.38, 0.45),
            (0.7, 0.45),
            (0.85, 0.55),
        ]],
        'A' => &[&[(0.1, 0.0), (0.5, 1.0), (0.9, 0.0)], &[(0.26, 0.4), (0.74, 0.4)]],
        'B' => &[
            &[(0.15, 0.0), (0.15, 1.0), (0.68, 1.0), (0.88, 0.83), (0.88, 0.68), (0.68, 0.55), (0.15, 0.55)],
            &[(0.15, 0.55), (0.72, 0.55), (0.9, 0.4), (0.9, 0.16), (0.7, 0.0), (0.15, 0.0)],
        ],
        'C' => &[&[(0.9, 0.8), (0.7, 1.0), (0.3, 1.0), (0.1, 0.8), (0.1, 0.2), (0.3, 0.0), (0.7, 0.0), (0.9, 0.2)]],
        'D' => &[&[(0.15, 0.0), (0.15, 1.0), (0.58, 1.0), (0.88, 0.74), (0.88, 0.26), (0.58, 0.0), (0.15, 0.0)]],
        'E' => &[&[(0.9, 1.0), (0.15, 1.0), (0.15, 0.0), (0.9, 0.0)], &[(0.15, 0.5), (0.68, 0.5)]],
        'F' => &[&[(0.88, 1.0), (0.15, 1.0), (0.15, 0.0)], &[(0.15, 0.52), (0.68, 0.52)]],
        _ => exit(format!("KeyError: {}", py::str_repr(&ch.to_string()))),
    };
    g.iter().map(|s| s.to_vec()).collect()
}

fn nums(arg: &str) -> Vec<f64> {
    let r = re::rx(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?");
    re::findall1(&r, arg, 0).iter().map(|v| float(v)).collect()
}

/// Every subpath as a closed ring of points, arcs sampled and Z honoured.
fn flatten(d: &str) -> Vec<Vec<P>> {
    let per = 64;
    let tok = re::rx(r"([MmLlHhVvAaZz])([^A-Za-z]*)");
    let mut out: Vec<Vec<P>> = Vec::new();
    let mut cur: Option<Vec<P>> = None;
    let (mut x, mut y, mut sx, mut sy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for m in re::all(&tok, d) {
        let cmd = re::g(&m, 1);
        let n = nums(re::g(&m, 2));
        match cmd {
            "M" => {
                if let Some(c) = cur.take() {
                    if !c.is_empty() {
                        out.push(c);
                    }
                }
                x = n[0];
                y = n[1];
                sx = x;
                sy = y;
                cur = Some(vec![(x, y)]);
            }
            // Straight runs are sampled, not just stepped to: a radius-by-angle
            // lookup interpolates between whatever points it is given.
            "H" => {
                let c = cur.as_mut().unwrap_or_else(|| exit("TypeError: unsupported operand type(s) for +=: 'NoneType' and 'list'"));
                for &v in &n {
                    c.extend((1..=per).map(|i| (x + (v - x) * i as f64 / per as f64, y)));
                    x = v;
                }
            }
            "V" => {
                let c = cur.as_mut().unwrap_or_else(|| exit("TypeError: unsupported operand type(s) for +=: 'NoneType' and 'list'"));
                for &v in &n {
                    c.extend((1..=per).map(|i| (x, y + (v - y) * i as f64 / per as f64)));
                    y = v;
                }
            }
            "A" => {
                let c = cur.as_mut().unwrap_or_else(|| exit("TypeError: unsupported operand type(s) for +=: 'NoneType' and 'list'"));
                let mut i = 0;
                while i < n.len() {
                    let (r, la, sw, ex, ey) = (n[i], n[i + 3], n[i + 4], n[i + 5], n[i + 6]);
                    c.extend(arc(x, y, r, la.trunc() as i64, sw.trunc() as i64, ex, ey, per));
                    x = ex;
                    y = ey;
                    i += 7;
                }
            }
            "Z" => {
                x = sx;
                y = sy;
            }
            _ => {}
        }
    }
    if let Some(c) = cur {
        if !c.is_empty() {
            out.push(c);
        }
    }
    out.into_iter().map(close).collect()
}

#[allow(clippy::too_many_arguments)]
fn arc(x0: f64, y0: f64, r: f64, large: i64, sweep: i64, x1: f64, y1: f64, per: usize) -> Vec<P> {
    let mut r = r;
    let (dx, dy) = ((x0 - x1) / 2.0, (y0 - y1) / 2.0);
    let lam = (dx * dx + dy * dy) / (r * r);
    if lam > 1.0 {
        r *= lam.sqrt();
    }
    let mut k = py::max((r * r * r * r - r * r * (dy * dy + dx * dx)) / (r * r * (dy * dy + dx * dx)), 0.0).sqrt();
    if large == sweep {
        k *= -1.0;
    }
    let (cx, cy) = (k * r * dy / r + (x0 + x1) / 2.0, -k * r * dx / r + (y0 + y1) / 2.0);
    let a0 = atan2(y0 - cy, x0 - cx);
    let a1 = atan2(y1 - cy, x1 - cx);
    let mut da = a1 - a0;
    if sweep != 0 && da < 0.0 {
        da += 2.0 * PI;
    }
    if sweep == 0 && da > 0.0 {
        da -= 2.0 * PI;
    }
    (1..=per)
        .map(|i| {
            let t = a0 + da * i as f64 / per as f64;
            (cx + r * cos(t), cy + r * sin(t))
        })
        .collect()
}

/// Fill in the segment Z leaves implicit, or a sharp square is three sides and a gap.
fn close(mut pts: Vec<P>) -> Vec<P> {
    let per = 64;
    let (ax, ay) = pts[pts.len() - 1];
    let (bx, by) = pts[0];
    if hypot(bx - ax, by - ay) < 1e-9 {
        return pts;
    }
    pts.extend((1..per).map(|i| (ax + (bx - ax) * i as f64 / per as f64, ay + (by - ay) * i as f64 / per as f64)));
    pts
}

fn polar(pts: &[P], cx: f64, cy: f64) -> Vec<P> {
    let mut v: Vec<P> = pts.iter().map(|&(px, py_)| (atan2(py_ - cy, px - cx), hypot(px - cx, py_ - cy))).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

fn radius(tab: &[P], th: f64) -> f64 {
    let (mut lo, mut hi) = (0usize, tab.len() - 1);
    if th <= tab[0].0 || th >= tab[tab.len() - 1].0 {
        return (tab[0].1 + tab[tab.len() - 1].1) / 2.0;
    }
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if tab[mid].0 <= th {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let ((t0, r0), (t1, r1)) = (tab[lo], tab[hi]);
    if t1 == t0 {
        r0
    } else {
        r0 + (r1 - r0) * (th - t0) / (t1 - t0)
    }
}

/// One character as polylines, top-left at (x, y), as SVG path data.
fn digit(ch: char, x: f64, y: f64, w: f64, h: f64) -> Vec<String> {
    glyph(ch)
        .iter()
        .map(|st| {
            let pts: Vec<String> = st.iter().map(|&(px, py_)| format!("{:.4},{:.4}", x + px * w, y + (1.0 - py_) * h)).collect();
            format!("M {}", pts.join(" L "))
        })
        .collect()
}

/// Ring index in hex, upper case.
fn hexlabel(i: i64) -> String {
    if i < 16 {
        format!("{i:X}")
    } else {
        format!("{i:02X}")
    }
}

/// (d, stroke) for every path, taking its colour from the element or its group.
fn paths_with_stroke(src: &str) -> Vec<(String, Option<String>)> {
    let pat = re::rx(r"<g\b[^>]*>|</g\s*>|<path\b[^>]*>");
    let hue = re::rx(r#"stroke\s*[:=]\s*"?\s*(#[0-9a-fA-F]{6})"#);
    let dre = re::rx(r#"\bd="([^"]+)""#);
    let (mut out, mut gstroke, mut stack): (Vec<(String, Option<String>)>, Option<String>, Vec<Option<String>>) = (vec![], None, vec![]);
    for m in re::all(&pat, src) {
        let el = re::g(&m, 0);
        if el.starts_with("</g") {
            gstroke = stack.pop().unwrap_or(None);
        } else if el.starts_with("<g") {
            stack.push(gstroke.clone());
            if let Some(c) = re::first(&hue, el) {
                gstroke = Some(re::g(&c, 1).to_lowercase());
            }
        } else if let Some(d) = re::first(&dre, el) {
            let c = re::first(&hue, el).map(|c| re::g(&c, 1).to_lowercase());
            out.push((re::g(&d, 1).to_string(), c.or_else(|| gstroke.clone())));
        }
    }
    out
}

fn points_of(paths: &[String]) -> Vec<P> {
    let r = re::rx(r"(-?[\d.]+),(-?[\d.]+)");
    let mut out = Vec::new();
    for p in paths {
        for m in re::all(&r, p) {
            out.push((float(re::g(&m, 1)), float(re::g(&m, 2))));
        }
    }
    out
}

struct Ring {
    ap: Vec<P>,
    out: Vec<P>,
    cx: f64,
    cy: f64,
    o: f64,
}

fn span(p: &[P]) -> f64 {
    py::max(fmax(p.iter().map(|q| q.0)) - fmin(p.iter().map(|q| q.0)), fmax(p.iter().map(|q| q.1)) - fmin(p.iter().map(|q| q.1)))
}
fn mid(p: &[P]) -> P {
    ((fmin(p.iter().map(|q| q.0)) + fmax(p.iter().map(|q| q.0))) / 2.0, (fmin(p.iter().map(|q| q.1)) + fmax(p.iter().map(|q| q.1))) / 2.0)
}

fn main() {
    let a = argv();
    let args = positional(&a);
    let opts = Opts::of(&a);
    let cap = opts.float("cap", CAP);
    // An extension sheet continues the stack it will be glued to, so its first
    // ring is not ring 0.
    let start = opts.int("start", 0);
    let mark = opts.str("mark", "yes").to_string();
    if mark != "yes" && mark != "no" {
        exit(format!("  --mark: yes or no, not {}", py::str_repr(&mark)));
    }
    for k in opts.keys() {
        if !["cap", "order", "start", "mark"].contains(&k) {
            exit(format!("  unknown option --{k}: cap, order, start or mark"));
        }
    }
    // A switch with no "=" was once dropped on the floor; it is refused.
    if let Some(b) = bare(&a) {
        exit(format!("  {b} takes a value: write it as {b}=VALUE. Nothing has been engraved."));
    }
    if args.is_empty() {
        exit("  usage: number_rings SHEET.svg [--cap=MM] [--order=document] [--start=N] [--mark=no]");
    }
    let src_path = &args[0];
    let sname = name_of(src_path);
    let mut src = lasermade_tools::text::read_strict(src_path)
        .unwrap_or_else(|e| exit(format!("FileNotFoundError: {e}: {}", py::str_repr(src_path))));

    let old = re::rx(&format!(r#"(?s)\s*<g id="{GROUP}".*?</g>"#));
    if let Some(m) = re::first(&old, &src) {
        let g0 = m.get(0).unwrap();
        src = format!("{}{}", &src[..g0.start()], &src[g0.end()..]);
    }

    // Blue is engraving, never a part, so blue paths are not candidates.
    let elements = paths_with_stroke(&src);
    let existing = elements.iter().filter(|(_, s)| s.as_deref() == Some(BLUE)).count();
    if existing > 0 {
        exit(format!(
            "  {sname} already carries {existing} engraved mark(s) in {BLUE}.\n  Numbering it would put a second set of labels on the same parts. Those\n  are hand-placed — clear them in Inkscape first if you want these instead."
        ));
    }

    // Refuse what cannot be read rather than misread it.
    let cuts: Vec<String> = elements.iter().map(|(d, _)| d.clone()).collect();
    let mut unknown: Vec<char> = cuts.iter().flat_map(|d| d.chars().filter(|c| c.is_ascii_alphabetic())).filter(|c| !"MHVAZ".contains(*c)).collect();
    unknown.sort();
    unknown.dedup();
    if !unknown.is_empty() {
        let u: Vec<String> = unknown.iter().map(|c| c.to_string()).collect();
        exit(format!(
            "  {sname} uses path commands this cannot read: {}\n  Relative commands and curves mean the sheet has been through an editor.\n  The generators write M, H, V, A and Z only.",
            u.join(" ")
        ));
    }

    // A ring is an outline and the aperture inside it, as two subpaths of one
    // path or as two paths; paired on their shared centre either way, and put
    // back in file order, which is assembly order.
    let mut loose: Vec<(usize, Vec<P>)> = Vec::new();
    let mut rings: Vec<(usize, Ring)> = Vec::new();
    let ring = |ap: &[P], out: &[P]| {
        let (cx, cy) = mid(out);
        Ring { ap: polar(ap, cx, cy), out: polar(out, cx, cy), cx, cy, o: span(out) }
    };
    for (pos, d) in cuts.iter().enumerate() {
        let mut sp = flatten(d);
        if sp.len() == 2 {
            sp.sort_by(|a, b| span(a).partial_cmp(&span(b)).unwrap());
            rings.push((pos, ring(&sp[0], &sp[1])));
        } else if sp.len() == 1 {
            loose.push((pos, sp.pop().unwrap()));
        }
    }
    let mut taken: Vec<usize> = Vec::new();
    let mut order_l: Vec<usize> = (0..loose.len()).collect();
    order_l.sort_by(|&a, &b| span(&loose[b].1).partial_cmp(&span(&loose[a].1)).unwrap());
    for li in order_l {
        let (pos, out) = (&loose[li].0, &loose[li].1);
        let (cx, cy) = mid(out);
        let so = span(out);
        let inner: Vec<usize> = (0..loose.len())
            .filter(|&qi| {
                let (qp, q) = (&loose[qi].0, &loose[qi].1);
                !taken.contains(qp) && qi != li && span(q) < so && (mid(q).0 - cx).abs() < 0.01 && (mid(q).1 - cy).abs() < 0.01
            })
            .collect();
        if inner.is_empty() {
            continue;
        }
        let best = inner[1..].iter().fold(inner[0], |a, &b| if span(&loose[b].1) > span(&loose[a].1) { b } else { a });
        taken.push(loose[best].0);
        rings.push((*pos, ring(&loose[best].1, out)));
    }
    rings.sort_by_key(|t| t.0);
    let mut rings: Vec<Ring> = rings.into_iter().map(|t| t.1).collect();
    if rings.is_empty() {
        exit(format!("  no rings in {sname}"));
    }
    if rings.len() > 256 {
        exit(format!("  {} rings needs more than two hex characters", rings.len()));
    }

    // A number is only worth engraving if it is the ASSEMBLY order.
    let spans: Vec<f64> = rings.iter().map(|r| r.o).collect();
    let rising = spans.windows(2).all(|w| w[1] > w[0]);
    let order = match opts.get("order") {
        Some(o) => o.to_string(),
        None => {
            if rising {
                "size".to_string()
            } else {
                exit(
                    "  these rings are not in size order in the file. A mouthpiece doubles\n  back through the same diameters; a bell's flange ring is wider than the\n  rings just above it; a hand-nested sheet can be in any order at all.\n  Numbering by size would not be assembly order. Pass --order=document to\n  number them as the file lists them, which IS assembly order for a\n  generated sheet, or --order=size to confirm you meant smallest-first.",
                )
            }
        }
    };
    if order != "size" && order != "document" {
        exit(format!("  --order: size or document, not {}", py::str_repr(&order)));
    }
    if order == "size" {
        rings.sort_by(|a, b| a.o.partial_cmp(&b.o).unwrap());
    }

    let mut marks: Vec<String> = Vec::new();
    let mut report: Vec<(usize, String, f64, f64, f64)> = Vec::new();
    let mut fails: Vec<String> = Vec::new();
    for (i, r) in rings.iter().enumerate() {
        let label = hexlabel(start + i as i64);
        let n = label.chars().count() as f64;
        let down = PI / 2.0; // SVG y grows downward, so this is the bottom
        let (inner, outer) = (radius(&r.ap, down), radius(&r.out, down));
        let mut placed: Option<(Vec<String>, f64)> = None;
        for step in 0..40 {
            let h = cap - step as f64 * (cap - 0.4) / 39.0;
            let gap = 0.34 * h;
            let (mgap, mlen) = if mark == "yes" { (0.16 * h, 0.22 * h) } else { (0.0, 0.0) };
            let w = 0.55 * h * n + gap * (n - 1.0) + mgap + mlen;
            let mid_ = (inner + outer) / 2.0;
            let (x0, y0) = (r.cx - w / 2.0, r.cy + mid_ - h / 2.0);
            let mut paths: Vec<String> = Vec::new();
            let mut xoff = x0;
            for ch in label.chars() {
                paths.extend(digit(ch, xoff, y0, 0.55 * h, h));
                xoff += 0.55 * h + gap;
            }
            if mark == "yes" {
                let (mx, my) = (xoff - gap + mgap, y0 + h);
                paths.push(format!("M {:.4},{:.4} L {:.4},{:.4}", mx, my, mx + mlen, my));
            }
            let ok = points_of(&paths).iter().all(|&(px, py_)| {
                let th = atan2(py_ - r.cy, px - r.cx);
                let d = hypot(px - r.cx, py_ - r.cy);
                radius(&r.ap, th) + MARGIN <= d && d <= radius(&r.out, th) - MARGIN
            });
            if ok {
                placed = Some((paths, h));
                break;
            }
        }
        match placed {
            None => fails.push(format!("ring {i} ({label}): no digit fits between ø{:.1} and ø{:.1}", 2.0 * inner, 2.0 * outer)),
            Some((p, h)) => {
                marks.extend(p);
                report.push((i, label, r.o, outer - inner, h));
            }
        }
    }

    if !fails.is_empty() {
        for f in &fails {
            println!("  x {f}");
        }
        exit(format!("  {sname} not written: {} ring(s) unlabelled", fails.len()));
    }

    let body: Vec<String> = marks.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect();
    let group = format!("\n  <g id=\"{GROUP}\" fill=\"none\" stroke=\"{BLUE}\" stroke-width=\"0.1\">\n{}\n  </g>", body.join("\n"));
    let at = src.find("  <g ").unwrap_or_else(|| exit("ValueError: substring not found"));
    let out = format!("{}{}\n{}", &src[..at], group.trim_start_matches('\n'), &src[at..]);

    let dre = re::rx(r#"\bd="([^"]+)""#);
    let before = re::findall1(&dre, &src, 1);
    let after = re::findall1(&dre, &out, 1);
    if after.get(marks.len()..).map(|s| s.to_vec()) != Some(before) {
        exit("  cut path data changed — refusing to write");
    }

    write(src_path, &out);
    println!(
        "  {sname}: {} rings numbered {}-{}, {} engraved strokes in {BLUE}",
        rings.len(),
        report[0].1,
        report[report.len() - 1].1,
        marks.len()
    );
    for (i, label, o, wall, h) in &report {
        println!("    ring {i:>2}  '{label}'  ø{o:>6.2}  wall {wall:>5.2}mm  digit {h:.2}mm");
    }
}
