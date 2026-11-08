//! Check a bell sheet that has been edited by hand, before it is cut. Rust
//! since 2026-11-07 (verify_bell.py until then).
//!
//!     target/release/verify_bell SHEET.svg [REFERENCE.svg]
//!
//! With one argument it checks the sheet against itself: ring count, the lap,
//! what is nested inside what, whether any two outlines cross, and whether the
//! cut stage is one colour. With a second it also compares every ring to the
//! reference's sizes -- useful after the paths have been converted to curves,
//! when a byte-for-byte diff no longer says anything. Position-independent, so
//! it still answers after the parts have been nested. Square bells only: a
//! sheet with arcs is the mouthpiece or a round bell, and is skipped.
//!
//! Sizes are measured per command; quadratic control points are counted into
//! a box, a superset of the true one, the safe way to be wrong when telling a
//! ring from a digit.
#[path = "../common.rs"]
mod common;
use common::*;
use lasermade_tools::py::{self, g};
use lasermade_tools::re;

/// Separates rings from engraved digits: the smallest aperture is the bore
/// and every digit is under 3mm.
const RING_MIN: f64 = 5.0;
/// Each ring overhangs the next aperture by this, unless the sheet says otherwise.
const LAP: f64 = 3.0;
/// mm of centre-to-centre slop between an aperture and its own outline
const CONCENTRIC: f64 = 0.5;

type P = (f64, f64);
type B = (f64, f64, f64, f64);

fn subpaths(d: &str) -> Vec<Vec<P>> {
    let num = re::rx(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?");
    let (mut out, mut cur): (Vec<Vec<P>>, Option<Vec<P>>) = (vec![], None);
    let (mut x, mut y, mut sx, mut sy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for m in re::all(&re::rx(r"([MmLlHhVvQqTtCcSsAaZz])([^A-Za-z]*)"), d) {
        let cmd = re::g(&m, 1).chars().next().unwrap();
        let n: Vec<f64> = re::findall1(&num, re::g(&m, 2), 0).iter().map(|v| float(v)).collect();
        let (up, rel) = (cmd.to_ascii_uppercase(), cmd.is_ascii_lowercase());
        match up {
            'M' => {
                if let Some(c) = cur.take() {
                    if !c.is_empty() {
                        out.push(c);
                    }
                }
                (x, y) = if rel { (x + n[0], y + n[1]) } else { (n[0], n[1]) };
                (sx, sy) = (x, y);
                let mut c = vec![(x, y)];
                let mut i = 2;
                while i < n.len() {
                    (x, y) = if rel { (x + n[i], y + n[i + 1]) } else { (n[i], n[i + 1]) };
                    c.push((x, y));
                    i += 2;
                }
                cur = Some(c);
            }
            'L' => {
                let c = need(cur.as_mut());
                let mut i = 0;
                while i < n.len() {
                    (x, y) = if rel { (x + n[i], y + n[i + 1]) } else { (n[i], n[i + 1]) };
                    c.push((x, y));
                    i += 2;
                }
            }
            'H' => {
                let c = need(cur.as_mut());
                for &v in &n {
                    x = if rel { x + v } else { v };
                    c.push((x, y));
                }
            }
            'V' => {
                let c = need(cur.as_mut());
                for &v in &n {
                    y = if rel { y + v } else { v };
                    c.push((x, y));
                }
            }
            'Q' | 'T' | 'C' | 'S' | 'A' => {
                let c = need(cur.as_mut());
                let step = match up {
                    'Q' => 4,
                    'T' => 2,
                    'C' => 6,
                    'S' => 4,
                    _ => 7,
                };
                let mut i = 0i64;
                while i < n.len() as i64 - step as i64 + 1 {
                    let seg = &n[i as usize..i as usize + step];
                    if up == 'A' {
                        (x, y) = if rel { (x + seg[5], y + seg[6]) } else { (seg[5], seg[6]) };
                        c.push((x, y));
                    } else {
                        let pts: Vec<P> = (0..step).step_by(2).map(|k| (seg[k], seg[k + 1])).collect();
                        for &(dx, dy) in &pts {
                            c.push(if rel { (x + dx, y + dy) } else { (dx, dy) });
                        }
                        let (lx, ly) = pts[pts.len() - 1];
                        (x, y) = if rel { (x + lx, y + ly) } else { (lx, ly) };
                    }
                    i += step as i64;
                }
            }
            'Z' => (x, y) = (sx, sy),
            _ => {}
        }
    }
    if let Some(c) = cur {
        if !c.is_empty() {
            out.push(c);
        }
    }
    out
}

fn bx(pts: &[P]) -> B {
    let xs = || pts.iter().map(|p| p.0);
    let ys = || pts.iter().map(|p| p.1);
    (fmin(xs()), fmin(ys()), fmax(xs()), fmax(ys()))
}

fn span(pts: &[P]) -> f64 {
    let (x0, y0, x1, y1) = bx(pts);
    py::max(x1 - x0, y1 - y0)
}

fn centre(b: B) -> P {
    ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
}

#[derive(Clone)]
struct Ring {
    i: i64,
    outer: B,
    ap: B,
    o: f64,
    a: f64,
    c: Option<String>,
}

#[derive(Clone)]
struct Mark {
    i: i64,
    bx: B,
    c: Option<String>,
    solo: bool,
}

struct Loose {
    i: i64,
    b: B,
    s: f64,
    c: Option<String>,
}

/// Pair single-outline apertures with their own outlines, by CONCENTRICITY and
/// size, smallest first: not by colour, not by file order, not by containment.
fn pair_by_centre(loose: &[Loose]) -> (Vec<Ring>, Vec<usize>) {
    let mut rings = vec![];
    let mut used = vec![false; loose.len()];
    let mut order: Vec<usize> = (0..loose.len()).collect();
    order.sort_by(|&a, &b| loose[a].s.partial_cmp(&loose[b].s).unwrap());
    for &k in &order {
        if used[k] {
            continue;
        }
        let a = &loose[k];
        let ac = centre(a.b);
        let mut mate = None;
        for &j in &order {
            if used[j] || j == k || loose[j].s <= a.s + 1e-6 {
                continue;
            }
            let bc = centre(loose[j].b);
            if (bc.0 - ac.0).abs() <= CONCENTRIC && (bc.1 - ac.1).abs() <= CONCENTRIC {
                mate = Some(j);
                break;
            }
        }
        let Some(mate) = mate else { continue };
        used[k] = true;
        used[mate] = true;
        let o = &loose[mate];
        // The ring's cut stage is the OUTLINE's colour: the cut that frees the part.
        rings.push(Ring { i: o.i, outer: o.b, ap: a.b, o: o.s, a: a.s, c: o.c.clone() });
    }
    (rings, (0..loose.len()).filter(|&k| !used[k]).collect())
}

/// (rings, marks). A ring is an aperture and the outline concentric with it,
/// in one path as two subpaths or in two paths of two colour groups.
fn read(path: &str) -> (Vec<Ring>, Vec<Mark>) {
    let src = lasermade_tools::text::read_strict(path).unwrap_or_else(|e| exit(format!("FileNotFoundError: {e}: {}", py::str_repr(path))));
    let hue = re::rx(r"stroke\s*[:=]\s*.?(#[0-9a-fA-F]{6})");
    let dre = re::rx(r#"\bd="([^"]+)""#);
    let (mut rings, mut marks, mut loose) = (vec![], vec![], vec![]);
    let (mut gstroke, mut stack, mut i): (Option<String>, Vec<Option<String>>, i64) = (None, vec![], -1);
    for m in re::all(&re::rx(r"<g\b[^>]*>|</g\s*>|<path\b[^>]*>"), &src) {
        let el = re::g(&m, 0);
        if el.starts_with("</g") {
            gstroke = stack.pop().unwrap_or(None);
            continue;
        }
        if el.starts_with("<g") {
            stack.push(gstroke.clone());
            if let Some(h) = re::first(&hue, el) {
                gstroke = Some(re::g(&h, 1).to_lowercase());
            }
            continue;
        }
        i += 1;
        let attrs = &el[5..el.len() - 1];
        let Some(d) = re::first(&dre, attrs) else { continue };
        let sp = subpaths(re::g(&d, 1));
        let colour = re::first(&hue, attrs).map(|h| re::g(&h, 1).to_lowercase()).or_else(|| gstroke.clone());
        let mut big: Vec<&Vec<P>> = sp.iter().filter(|p| span(p) >= RING_MIN).collect();
        big.sort_by(|a, b| span(a).partial_cmp(&span(b)).unwrap());
        if big.len() == 2 {
            rings.push(Ring { i, outer: bx(big[1]), ap: bx(big[0]), o: span(big[1]), a: span(big[0]), c: colour });
        } else if big.len() == 1 {
            loose.push(Loose { i, b: bx(big[0]), s: span(big[0]), c: colour });
        } else {
            let all: Vec<P> = sp.iter().flatten().copied().collect();
            marks.push(Mark { i, bx: bx(&all), c: colour, solo: false });
        }
    }
    let (paired, leftover) = pair_by_centre(&loose);
    rings.extend(paired);
    // A big outline with no aperture is a SOLID DISC where a ring should be.
    marks.extend(leftover.iter().map(|&k| Mark { i: loose[k].i, bx: loose[k].b, c: loose[k].c.clone(), solo: true }));
    rings.sort_by_key(|r| r.i);
    marks.sort_by_key(|r| r.i);
    (rings, marks)
}

fn inside(b: B, o: B) -> bool {
    let pad = 0.01;
    b.0 >= o.0 - pad && b.1 >= o.1 - pad && b.2 <= o.2 + pad && b.3 <= o.3 + pad
}

fn crosses(a: B, b: B) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// Square bells only: arcs mean the mouthpiece or a round bell, about which
/// this tool has nothing true to say.
fn is_bell_sheet(path: &str) -> (bool, String) {
    let src = lasermade_tools::text::read_strict(path).unwrap_or_else(|e| exit(format!("FileNotFoundError: {e}: {}", py::str_repr(path))));
    let ds = re::findall1(&re::rx(r#"\bd="([^"]+)""#), &src, 1);
    if ds.iter().any(|d| d.contains(['A', 'a'])) {
        return (false, "rings are drawn with arcs, so they are round — not a bell sheet".into());
    }
    let (rings, _) = read(path);
    if rings.len() < 2 {
        return (false, format!("only {} concentric-square ring(s) — a section drawing or a sheet of something else", rings.len()));
    }
    (true, String::new())
}

/// repr() of a list of floats
fn flist(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|&x| py::repr(x)).collect::<Vec<_>>().join(", "))
}
/// repr() of a list of colours, None where a path had none
fn clist(v: &[Option<String>]) -> String {
    format!("[{}]", v.iter().map(|c| c.as_deref().map(py::str_repr).unwrap_or_else(|| "None".into())).collect::<Vec<_>>().join(", "))
}
/// sorted() of colours; Python cannot order None against a string
fn csorted(mut v: Vec<Option<String>>) -> Vec<Option<String>> {
    if v.iter().any(|c| c.is_none()) && v.iter().any(|c| c.is_some()) {
        exit("TypeError: '<' not supported between instances of 'NoneType' and 'str'");
    }
    v.sort();
    v
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 2 {
        exit("  usage: verify_bell SHEET.svg [REFERENCE.svg]");
    }
    let sheet = &a[1];
    let (ok, why) = is_bell_sheet(sheet);
    if !ok {
        println!("  skipped {}: {why}", name_of(sheet));
        return;
    }
    let (rings, marks) = read(sheet);
    let mut fails: Vec<String> = vec![];
    let solo: Vec<&Mark> = marks.iter().filter(|m| m.solo).collect();
    println!("  rings {}   other paths {}", rings.len(), marks.len());
    if !solo.is_empty() {
        for m in &solo {
            println!("    ✗ path #{} is {:.1}mm across with nothing concentric inside it — a solid disc, not a ring", m.i, m.bx.2 - m.bx.0);
        }
        fails.push(format!("{} outline(s) have no aperture: the sheet would cut {} solid disc(s)", solo.len(), solo.len()));
    }

    // Ordered by APERTURE: the airway only opens, so aperture order is assembly
    // order however the sheet was nested. Ring 0's flange is wider than the rings above it.
    let text = lasermade_tools::text::read_strict(sheet).unwrap();
    let lap = match re::first(&re::rx(r"(?:lapping the ring below by|seating on the ring below over)\s*([\d.]+)"), &text) {
        Some(m) => float(re::g(&m, 1)),
        None => LAP,
    };
    let mut order = rings.clone();
    order.sort_by(|x, y| x.a.partial_cmp(&y.a).unwrap());
    println!("\n  the {}mm lap, ring to ring:", g(lap));
    let laps: Vec<f64> = order.windows(2).map(|w| py::round_dp((w[0].o - w[1].a) / 2.0, 3)).collect();
    // The flange joint is deliberately deeper, so the first only has to be at least the lap.
    let mut bad: Vec<f64> = laps.iter().skip(1).copied().filter(|v| (v - lap).abs() > 0.01).collect();
    if !laps.is_empty() && laps[0] < lap - 0.01 {
        bad.insert(0, laps[0]);
    }
    let mut uniq = laps.clone();
    uniq.sort_by(|x, y| x.partial_cmp(y).unwrap());
    uniq.dedup();
    println!("    {}mm per side{}", flist(&uniq), if bad.is_empty() { String::new() } else { format!("   OFF: {}", flist(&bad)) });
    if !bad.is_empty() {
        fails.push(format!("lap is not {}mm at every joint", g(lap)));
    } else if !laps.is_empty() && laps[0] > lap + 0.01 {
        println!("    the flange joint is deeper, {}mm — it covers the bore's end face", g(laps[0]));
    }

    // One colour for every ring is one stage; a ramp must rise with ring size.
    println!("\n  cut stage:");
    let mut mcs: Vec<Option<String>> = marks.iter().map(|m| m.c.clone()).collect();
    mcs.dedup();
    let mut mc = csorted(mcs);
    mc.dedup();
    let mut by_size = rings.clone();
    by_size.sort_by(|x, y| x.o.partial_cmp(&y.o).unwrap());
    let cols: Vec<Option<String>> = by_size.iter().map(|r| r.c.clone()).collect();
    let mut set_cols = cols.clone();
    set_cols.sort();
    set_cols.dedup();
    let cs = |c: &Option<String>| c.clone().unwrap_or_else(|| "None".into());
    if set_cols.len() == 1 {
        println!("    rings  one stage, {}", cs(&cols[0]));
    } else {
        println!(
            "    rings  {} stages, {} (ø{:.1}) → {} (ø{:.1})",
            set_cols.len(),
            cs(&cols[0]),
            by_size[0].o,
            cs(&cols[cols.len() - 1]),
            by_size[by_size.len() - 1].o
        );
        if set_cols.len() != cols.len() {
            let dupe: Vec<Option<String>> = set_cols.iter().filter(|c| cols.iter().filter(|x| x == c).count() > 1).cloned().collect();
            fails.push(format!("rings share a stage, so their order is undecided: {}", clist(&csorted(dupe))));
        }
        let keys: Vec<i64> = cols
            .iter()
            .map(|c| {
                let c = c.as_deref().unwrap_or_else(|| exit("TypeError: 'NoneType' object is not subscriptable"));
                i64::from_str_radix(&c[1..], 16).unwrap()
            })
            .collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        if keys != sorted_keys {
            fails.push("ring colours do not rise with ring size — the ramp orders the cut wrongly".into());
        } else {
            println!("    ramp rises with size, so the smallest cuts first and the rim last");
        }
    }
    println!("    labels {}", clist(&mc));
    if mc.len() > 1 {
        fails.push(format!("marks use {} shades, so they will not all land in one stage: {}", mc.len(), clist(&mc)));
    }
    let shared: Vec<Option<String>> = mc.iter().filter(|c| set_cols.contains(c)).cloned().collect();
    if !shared.is_empty() {
        fails.push(format!("a label shares a stage with a ring: {}", clist(&csorted(shared))));
    }

    // A note, not a failure: document order is only the cut order on an
    // importer that follows it, and most do not.
    println!("\n  nesting:");
    let (mut nested, mut late) = (0, 0);
    for (ia, ra) in rings.iter().enumerate() {
        for (ib, rb) in rings.iter().enumerate() {
            if ia == ib || !inside(ra.outer, rb.ap) {
                continue;
            }
            nested += 1;
            if ra.i > rb.i {
                late += 1;
                println!("    – ø{:.1} (path #{}) sits in ø{:.1} (path #{}) and is written after it", ra.o, ra.i, rb.o, rb.i);
            }
        }
    }
    println!("    {nested} parts nested, {} written before the ring that frees them", nested - late);
    if late > 0 {
        println!("    {late} written after. That only decides anything if your software cuts in");
        println!("    document order — most order inner shapes first regardless.");
    }

    let mut clash = 0;
    for i in 0..rings.len() {
        for j in i + 1..rings.len() {
            let (ra, rb) = (&rings[i], &rings[j]);
            if crosses(ra.outer, rb.outer) && !inside(ra.outer, rb.ap) && !inside(rb.outer, ra.ap) {
                clash += 1;
                println!("    ✗ ø{:.1} and ø{:.1} overlap and neither is nested", ra.o, rb.o);
            }
        }
    }
    if clash > 0 {
        fails.push(format!("{clash} pair(s) of outlines cross"));
    }
    println!("    {clash} overlapping outlines");

    if a.len() > 2 {
        let (rf, _) = read(&a[2]);
        println!("\n  against {}:", name_of(&a[2]));
        let mut x: Vec<f64> = rings.iter().map(|r| py::round_dp(r.a, 2)).collect();
        let mut y: Vec<f64> = rf.iter().map(|r| py::round_dp(r.a, 2)).collect();
        x.sort_by(|p, q| p.partial_cmp(q).unwrap());
        y.sort_by(|p, q| p.partial_cmp(q).unwrap());
        let same = x == y && rings.len() == rf.len();
        println!("    {} rings vs {}; apertures {}", rings.len(), rf.len(), if same { "identical" } else { "DIFFER" });
        if !same {
            fails.push("ring sizes differ from the reference".into());
        }
    }

    println!();
    for f in &fails {
        println!("  ✗ {f}");
    }
    println!("  {}", if fails.is_empty() { "✓ sheet is ready to cut".to_string() } else { format!("{} problem(s)", fails.len()) });
    std::process::exit(if fails.is_empty() { 0 } else { 1 });
}

/// x, where Python would have died appending to None
fn need<T>(o: Option<T>) -> T {
    o.unwrap_or_else(|| crate::common::exit("AttributeError: 'NoneType' object has no attribute 'append'"))
}
