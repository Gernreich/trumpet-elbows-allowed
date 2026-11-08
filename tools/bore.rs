//! Split a bore written in the agreed notation into cuttable part files:
//! bore_split.py's working, in Rust since 2026-11-08. The notation, the
//! reasoning behind every constant and every refusal are in bore_split_main.rs
//! (the docstring) and in NOTES.md; this file keeps the comments that say why
//! a line is the way it is.
//!
//! Each piece is drawn by snakebox.rs, which is GPL-3.0-or-later. This file is
//! CC0 like the rest of the repository; a program built from both is
//! distributed under the GPL.
use crate::snakebox::Snake;
use geo::{Area, Contains, Polygon};
use lasermade_tools::py::{self, g};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

pub const BED_W: f64 = 600.0; // xTool P2S work area, mm (cut-tested 2026-10-19)
pub const BED_H: f64 = 305.0;
pub const PIN: f64 = 1.5; // tab reach
pub const THICKNESS: f64 = 3.0; // ply, NOMINAL: what the lattice is dimensioned on
pub const PIN_FRAC: f64 = 0.48;
pub const MIN_SHOULDER: f64 = 2.0;
pub const MIN_FEATURE: f64 = 1.5;
pub const PLAY_UNMEASURED: f64 = 0.025;
const KERF0: f64 = 0.15;
const BLOCK0: f64 = 16.0;
const SHEET0: f64 = 3.0;
/// A turn folded into a bend wherever one can be.
const FOLD_TURNS: bool = true;

/// PLAY_BY_BORE: the measured joint clearance per side, by bore
fn play_by_bore(bore: f64) -> Option<f64> {
    if bore == 10.0 {
        Some(0.025)
    } else {
        None
    }
}

/// Every design switch: bore_split.py's module globals.
#[derive(Clone, Debug)]
pub struct Design {
    pub block: f64,
    pub straight: f64,
    pub sheet: f64,
    pub kerf: f64,
    pub burn: f64,
    pub flat: bool,
    pub allow_ports: bool,
    pub refuse_elbows: bool,
    pub mouth_at: Option<Vec<i64>>,
    pub tag: String,
    pub play_override: Option<f64>,
    pub notch: Option<f64>,
    pub title: Option<String>,
    pub common: Vec<String>,
}

thread_local! {
    static UNMEASURED_SAID: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// sys.exit(message): the message on stderr, exit 1
pub fn sys_exit(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

impl Default for Design {
    fn default() -> Design {
        let mut d = Design {
            block: BLOCK0,
            straight: BLOCK0,
            sheet: SHEET0,
            kerf: KERF0,
            burn: KERF0 / 2.0,
            flat: false,
            allow_ports: false,
            refuse_elbows: false,
            mouth_at: None,
            tag: String::new(),
            play_override: None,
            notch: None,
            title: None,
            common: vec![],
        };
        d.common = d.make_common();
        d
    }
}

impl Design {
    /// The measured joint clearance for this bore, or the fallback -- out loud.
    pub fn pin_play(&self) -> f64 {
        if let Some(p) = self.play_override {
            return p;
        }
        let bore = py::round_dp(self.block - 2.0 * THICKNESS, 3);
        if let Some(p) = play_by_bore(bore) {
            return p;
        }
        let key = py::repr(bore);
        let first = UNMEASURED_SAID.with(|s| s.borrow_mut().insert(key));
        if first {
            eprintln!(
                "note: the {}mm bore has no measured joint clearance. Using {}mm per side, the small-joint value, which is the safe direction but a guess. Cut a coupon, assemble it, and add the row to PLAY_BY_BORE.",
                g(bore),
                g(PLAY_UNMEASURED)
            );
        }
        PLAY_UNMEASURED
    }

    /// The coupling tab, which must never be the weakest feature on the sheet.
    pub fn pin_width(&self) -> f64 {
        let tooth = 2.0 * self.sheet;
        let frame = self.block - 2.0 * THICKNESS;
        if let Some(notch) = self.notch {
            let w = notch - 2.0 * self.pin_play();
            if w <= 0.0 {
                sys_exit(&format!("--notch must be wider than the {}mm of play", g(2.0 * self.pin_play())));
            }
            let thin = py::min((frame - notch) / 2.0, (frame - w) / 2.0);
            if thin < MIN_FEATURE {
                sys_exit(&format!("--notch={} leaves {thin:.2}mm beside it in a {}mm frame, under the {}mm the gate allows", g(notch), g(frame), g(MIN_FEATURE)));
            }
            return w;
        }
        py::min(py::max(PIN_FRAC * frame, tooth), frame - 2.0 * MIN_SHOULDER)
    }

    fn make_common(&self) -> Vec<String> {
        vec![
            format!("--blocksize={}", g(self.block)),
            format!("--thickness={}", g(self.sheet)),
            format!("--burn={}", g(self.burn)),
            format!("--pin_width={}", g(self.pin_width())),
            format!("--pin_play={}", g(self.pin_play())),
            "--labels=0".into(),
            "--reference=0".into(),
            "--inner_corners=corner".into(),
            "--spacing=0.5".into(),
        ]
    }

    pub fn set_blocksize(&mut self, mm: f64) {
        let was_cubic = self.straight == self.block;
        self.block = mm;
        if was_cubic {
            self.straight = self.block;
        }
        self.common = self.make_common();
    }
    pub fn set_bore(&mut self, mm: f64) {
        self.set_blocksize(mm + 2.0 * THICKNESS);
    }
    pub fn set_play(&mut self, mm: f64) {
        self.play_override = Some(mm);
        self.common = self.make_common();
    }
    pub fn set_notch(&mut self, mm: f64) {
        self.notch = Some(mm);
        self.common = self.make_common();
    }
    pub fn cubic(&self) -> bool {
        self.straight == self.block
    }
    /// One block's outer size along a world axis.
    pub fn extent(&self, r: &Rec, axis: usize) -> f64 {
        if r.inn == r.out && axis_of(r.out) == axis {
            return self.straight;
        }
        self.block
    }

    /// Set the design switches in argv and return the arguments left over.
    pub fn take_design_switches(&mut self, argv: &[String]) -> Vec<String> {
        const BARE: [&str; 3] = ["flat", "ports", "refuse-elbows"];
        const VALUED: [&str; 9] = ["mouth-at", "blocksize", "bore", "straight", "sheet", "kerf", "tag", "play", "notch"];
        let mut got: HashMap<String, String> = HashMap::new();
        let mut rest = vec![];
        let mut i = 0;
        while i < argv.len() {
            let x = &argv[i];
            let (name, eq, mut v) = if let Some(s) = x.strip_prefix("--") {
                match s.split_once('=') {
                    Some((a, b)) => (a.to_string(), true, b.to_string()),
                    None => (s.to_string(), false, String::new()),
                }
            } else {
                (String::new(), false, String::new())
            };
            if BARE.contains(&name.as_str()) && !eq {
                v = "True".into();
            } else if VALUED.contains(&name.as_str()) {
                if !eq {
                    if i + 1 >= argv.len() {
                        sys_exit(&format!("error: --{name} needs a value."));
                    }
                    i += 1;
                    v = argv[i].clone();
                }
            } else {
                rest.push(x.clone());
                i += 1;
                continue;
            }
            if got.contains_key(&name) {
                sys_exit(&format!("error: --{name} is given twice."));
            }
            got.insert(name, v);
            i += 1;
        }
        let number = |k: &str| -> f64 { py::float(&got[k]).unwrap_or_else(|| sys_exit(&format!("error: --{k}={} is not a number.", got[k]))) };
        if got.contains_key("flat") {
            self.flat = true;
        }
        if got.contains_key("ports") {
            self.allow_ports = true;
        }
        if got.contains_key("refuse-elbows") {
            self.refuse_elbows = true;
        }
        if let Some(m) = got.get("mouth-at") {
            let mut v = vec![];
            for b in m.split(',') {
                if b.is_empty() {
                    continue;
                }
                match pyint(b) {
                    Some(n) => v.push(n),
                    None => sys_exit(&format!("error: --mouth-at={m} is not a comma-separated list of block numbers.")),
                }
            }
            if v.is_empty() {
                sys_exit("error: --mouth-at= names no block at all.");
            }
            self.mouth_at = Some(v);
        }
        if got.contains_key("bore") && got.contains_key("blocksize") {
            let want = number("bore") + 2.0 * THICKNESS;
            if (want - number("blocksize")).abs() > 1e-9 {
                sys_exit(&format!(
                    "error: --bore={} means --blocksize={} at {}mm ply, and --blocksize={} was asked for as well. They are two spellings of one number. Pass one.",
                    got["bore"],
                    g(want),
                    g(THICKNESS),
                    got["blocksize"]
                ));
            }
        }
        if got.contains_key("blocksize") {
            self.set_blocksize(number("blocksize"));
        }
        if got.contains_key("bore") {
            self.set_bore(number("bore"));
        }
        if got.contains_key("sheet") {
            self.sheet = number("sheet");
            self.common = self.make_common();
        }
        if got.contains_key("kerf") {
            self.kerf = number("kerf");
            self.burn = self.kerf / 2.0;
            self.common = self.make_common();
        }
        if let Some(t) = got.get("tag") {
            self.tag = t.clone();
        }
        if got.contains_key("play") {
            self.set_play(number("play"));
        }
        if got.contains_key("straight") {
            self.straight = number("straight");
        }
        if got.contains_key("notch") {
            self.set_notch(number("notch"));
        }
        rest
    }
}

/// int(s) as Python reads it: surrounding whitespace, a sign, underscores
pub fn pyint(s: &str) -> Option<i64> {
    let t = s.trim();
    let u = t.strip_prefix('+').unwrap_or(t);
    let (neg, body) = match u.strip_prefix('-') {
        Some(b) if !t.starts_with('+') => (true, b),
        _ => (false, u),
    };
    if body.is_empty() || body.starts_with('_') || body.ends_with('_') || body.contains("__") || !body.chars().all(|c| c.is_ascii_digit() || c == '_') {
        return None;
    }
    let n: i64 = body.replace('_', "").parse().ok()?;
    Some(if neg { -n } else { n })
}

pub const DESIGN_HELP: &str = "design switches, read by bore_split.take_design_switches() and
so the same in bore_split.py, check.py, nest.py and regress.py:
  --bore=MM        the square airway; the block is that plus a wall each side
  --blocksize=MM   the block pitch instead -- one of these two, not both
  --straight=MM    length of a straight block; turns stay cubic
  --flat           plain butt ends, no tabs and no notches
  --ports          let a piece open through a face plate
  --refuse-elbows  stop, writing nothing, if any turn cannot fold into a bend
  --mouth-at=B,B   a mouth through a face plate at each block, 1-based
  --sheet=MM       the ply as measured
  --kerf=MM        the cut width as measured
  --play=MM        joint clearance per side, for cutting a coupon
  --notch=MM       size the joint from the notch
  --tag=X          engraved beside every part number";

// --- the walk ----------------------------------------------------------------

/// Facing north, east is to your right. North is away from you, south toward you.
pub fn dirs(c: char) -> [i64; 3] {
    match c {
        'E' => [1, 0, 0],
        'W' => [-1, 0, 0],
        'U' => [0, 1, 0],
        'D' => [0, -1, 0],
        'S' => [0, 0, 1],
        'N' => [0, 0, -1],
        _ => panic!("no direction {c}"),
    }
}
pub fn axis_of(c: char) -> usize {
    match c {
        'E' | 'W' => 0,
        'U' | 'D' => 1,
        _ => 2,
    }
}
pub fn neg(v: [i64; 3]) -> [i64; 3] {
    [-v[0], -v[1], -v[2]]
}
pub fn face2d(v: (i64, i64)) -> Option<char> {
    match v {
        (0, -1) => Some('S'),
        (1, 0) => Some('E'),
        (0, 1) => Some('N'),
        (-1, 0) => Some('W'),
        _ => None,
    }
}
fn path2d(v: (i64, i64)) -> char {
    match v {
        (1, 0) => 'R',
        (-1, 0) => 'L',
        (0, 1) => 'U',
        _ => 'D',
    }
}
fn vec2d(c: char) -> (i64, i64) {
    match c {
        'S' => (0, -1),
        'E' => (1, 0),
        'N' => (0, 1),
        _ => (-1, 0),
    }
}
/// v turned a quarter turn anticlockwise, q times.
fn turn2d(mut v: (i64, i64), q: i64) -> (i64, i64) {
    for _ in 0..q.rem_euclid(4) {
        v = (-v.1, v.0);
    }
    v
}

#[derive(Clone, Debug)]
pub struct Rec {
    pub pos: [i64; 3],
    pub inn: char,
    pub out: char,
}

pub fn tuple3(p: [i64; 3]) -> String {
    format!("({}, {}, {})", p[0], p[1], p[2])
}

pub fn parse(text: &str) -> Result<Vec<(char, i64)>, String> {
    let t = text.trim().to_uppercase();
    let bad: String = t.chars().filter(|c| !("NSEWUD0123456789.,".contains(*c) || c.is_whitespace())).collect();
    if !bad.is_empty() {
        return Err(format!("unexpected characters: {}", py::str_repr(&bad)));
    }
    if t.contains('.') {
        return Err("half blocks are no longer written: the first term already puts you at the centre of block 1, facing the way it points".into());
    }
    let cs: Vec<char> = t.chars().collect();
    let mut toks: Vec<(char, String)> = vec![];
    let mut i = 0;
    while i < cs.len() {
        if "NSEWUD".contains(cs[i]) {
            let mut j = i + 1;
            while j < cs.len() && cs[j].is_ascii_digit() {
                j += 1;
            }
            toks.push((cs[i], cs[i + 1..j].iter().collect()));
            i = j;
        } else {
            i += 1;
        }
    }
    if toks.is_empty() {
        return Err("need at least one term: a direction and how many blocks to travel".into());
    }
    let out: Vec<(char, i64)> = toks.iter().map(|(d, n)| (*d, if n.is_empty() { 0 } else { n.parse().unwrap() })).collect();
    for (i, &(d, n)) in out.iter().enumerate() {
        if n != 0 {
            continue;
        }
        if i == 0 {
            return Err(format!("\"{d}\" carries no distance. The way you came in is no longer a term: you enter facing the first term. Drop it if the term after it points the same way -- if it does not, it was turning block 1, and that stranded turn cannot be written now."));
        }
        if i == out.len() - 1 {
            return Err(format!("\"{d}\" carries no distance. The way you leave is no longer a term: you leave facing the last term. Drop it if it repeats the term before it -- if it does not, it was turning the last block, and that stranded turn cannot be written now."));
        }
        return Err(format!("\"{d}\" in the middle does nothing. Turning without travelling either repeats the term after it or tries to bend one block twice. Give it a distance, or drop it."));
    }
    Ok(out)
}

/// Walk the tunnel. One record per block: position, heading in, heading out.
pub fn walk(text: &str) -> Result<Vec<Rec>, String> {
    let toks = parse(text)?;
    let mut heading = toks[0].0;
    let mut pos = [0i64; 3];
    let mut rec = vec![Rec { pos, inn: heading, out: heading }];
    let mut seen: HashMap<[i64; 3], usize> = HashMap::from([(pos, 1)]);
    for &(d, n) in &toks {
        if d != heading {
            let (a, b) = (dirs(heading), dirs(d));
            if a[0] * b[0] + a[1] * b[1] + a[2] * b[2] != 0 {
                return Err(format!("{heading} -> {d} reverses the bore, not a turn"));
            }
            let last = rec.last().unwrap();
            if last.inn != last.out {
                return Err(format!(
                    "\"{d}{}\" turns again in a block that already turns {} to {}. A block bends once; give the previous term at least one block of travel.",
                    if n != 0 { n.to_string() } else { String::new() },
                    last.inn,
                    last.out
                ));
            }
            rec.last_mut().unwrap().out = d;
            heading = d;
        }
        for _ in 0..n {
            let v = dirs(heading);
            pos = [pos[0] + v[0], pos[1] + v[1], pos[2] + v[2]];
            if let Some(&s) = seen.get(&pos) {
                return Err(format!(
                    "block {} runs into block {s}: the bore is back at {}. Two blocks cannot share a cell - the walk has to go round, not through.",
                    rec.len() + 1,
                    tuple3(pos)
                ));
            }
            seen.insert(pos, rec.len() + 1);
            rec.push(Rec { pos, inn: heading, out: heading });
        }
    }
    Ok(rec)
}

pub type BoxMm = ([f64; 3], [f64; 3]);

impl Design {
    /// Every block as a real (lo, hi) box in mm, by following the chain.
    pub fn block_boxes(&self, rec: &[Rec]) -> Vec<BoxMm> {
        let half = self.block / 2.0;
        let mut boxes = vec![];
        let mut p = [0.0f64; 3];
        for r in rec {
            let (din, dout) = (dirs(r.inn), dirs(r.out));
            let a_in = axis_of(r.inn);
            let run = self.extent(r, a_in);
            let mut centre = [0.0; 3];
            for i in 0..3 {
                centre[i] = p[i] + (din[i] as f64 * run) / 2.0;
            }
            let mut hw = [half; 3];
            hw[a_in] = run / 2.0;
            if r.inn != r.out {
                hw = [half; 3];
                for i in 0..3 {
                    centre[i] = p[i] + din[i] as f64 * half;
                }
            }
            let mut lo = [0.0; 3];
            let mut hi = [0.0; 3];
            for i in 0..3 {
                lo[i] = centre[i] - hw[i];
                hi[i] = centre[i] + hw[i];
            }
            boxes.push((lo, hi));
            let a_out = axis_of(r.out);
            let out = if r.inn == r.out { self.extent(r, a_out) } else { self.block };
            for i in 0..3 {
                p[i] = centre[i] + (dout[i] as f64 * out) / 2.0;
            }
        }
        boxes
    }

    /// --widths_* for a piece, in SnakeBox's cell frame.
    pub fn piece_widths(&self, rec: &[Rec], group: &[usize], k: usize) -> Result<Vec<String>, String> {
        if self.cubic() {
            return Ok(vec![]);
        }
        let ax: Vec<usize> = (0..3).filter(|&j| j != k).collect();
        let base: Vec<i64> = ax.iter().map(|&j| rec[group[0]].pos[j]).collect();
        let mut out = vec![];
        for (a, (&axis, &b)) in ax.iter().zip(&base).enumerate() {
            let mut w: BTreeMap<i64, f64> = BTreeMap::new();
            for &j in group {
                let i = rec[j].pos[axis] - b;
                let e = self.extent(&rec[j], axis);
                let have = *w.entry(i).or_insert(e);
                if have != e {
                    return Err(format!(
                        "piece at blocks {}-{} needs column {}={i} to be both {} and {}mm wide. A block is long only along the axis it runs straight on, so a straight and a turn sharing a column cannot both fit. Shorten --straight to the block size, or change the walk.",
                        group[0] + 1,
                        group[group.len() - 1] + 1,
                        ['u', 'v'][a],
                        g(have),
                        g(e)
                    ));
                }
            }
            let (lo, hi) = (*w.keys().next().unwrap(), *w.keys().last().unwrap());
            let vals: Vec<String> = (lo..=hi).map(|i| g(*w.get(&i).unwrap_or(&self.block))).collect();
            out.push(format!("--widths_{}={}", ['u', 'v'][a], vals.join(",")));
            out.push(format!("--origin_{}={lo}", ['u', 'v'][a]));
        }
        Ok(out)
    }
}

/// Blocks that sit face to face without being neighbours along the bore.
pub fn touching(d: &Design, rec: &[Rec]) -> Vec<(usize, usize)> {
    let boxes = d.block_boxes(rec);
    let mut out = vec![];
    for i in 0..boxes.len() {
        for j in i + 2..boxes.len() {
            let ((a0, a1), (b0, b1)) = (boxes[i], boxes[j]);
            let mut abut = 0;
            for k in 0..3 {
                if (a1[k] - b0[k]).abs() < 1e-9 || (b1[k] - a0[k]).abs() < 1e-9 {
                    abut += 1;
                } else if py::min(a1[k], b1[k]) - py::max(a0[k], b0[k]) <= 1e-9 {
                    abut = -1;
                    break;
                }
            }
            if abut == 1 {
                out.push((i + 1, j + 1));
            }
        }
    }
    out
}

/// Do these 2D cells form one path, rather than a fork or a ring?
pub fn simple_snake(cells: &[(i64, i64)]) -> bool {
    let s: HashSet<(i64, i64)> = cells.iter().copied().collect();
    if s.len() == 1 {
        return true;
    }
    let deg = |c: (i64, i64)| [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|d| s.contains(&(c.0 + d.0, c.1 + d.1))).count();
    if s.iter().any(|&c| deg(c) > 2) {
        return false;
    }
    if s.iter().filter(|&&c| deg(c) == 1).count() != 2 {
        return false;
    }
    for &c in &s {
        for (dx, dy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            if s.contains(&(c.0 + dx, c.1 + dy)) && !s.contains(&(c.0 + dx, c.1)) && !s.contains(&(c.0, c.1 + dy)) {
                return false;
            }
        }
    }
    true
}

/// (k, port_in, port_out) per piece
pub type PiecePlan = (usize, bool, bool);

impl Design {
    pub fn plate_span_mm(&self, rec: &[Rec], group: &[usize], k: usize) -> (f64, f64) {
        let mut out = vec![];
        for axis in (0..3).filter(|&j| j != k) {
            let mut w: BTreeMap<i64, f64> = BTreeMap::new();
            for &j in group {
                w.insert(rec[j].pos[axis], self.extent(&rec[j], axis));
            }
            let (lo, hi) = (*w.keys().next().unwrap(), *w.keys().last().unwrap());
            out.push((lo..=hi).fold(0.0, |s, i| s + *w.get(&i).unwrap_or(&self.block)));
        }
        (out[0], out[1])
    }
    pub fn plate_mm(&self, s: (f64, f64)) -> (f64, f64) {
        (s.0 + 2.0 * PIN + self.burn * 2.0, s.1 + 2.0 * PIN + self.burn * 2.0)
    }
    pub fn plate_size(&self, rec: &[Rec], group: &[usize], k: usize) -> ((i64, i64), (f64, f64)) {
        let cells: Vec<[i64; 3]> = group.iter().map(|&j| rec[j].pos).collect();
        let ax: Vec<usize> = (0..3).filter(|&j| j != k).collect();
        let span = |j: usize| cells.iter().map(|c| c[j]).max().unwrap() - cells.iter().map(|c| c[j]).min().unwrap() + 1;
        ((span(ax[0]), span(ax[1])), self.plate_mm(self.plate_span_mm(rec, group, k)))
    }

    /// (k, port_in, port_out) for blocks a..b, or None if it cannot be cut.
    pub fn piece_plan(&self, rec: &[Rec], a: usize, b: usize) -> Option<PiecePlan> {
        let pos: Vec<[i64; 3]> = (a..=b).map(|i| rec[i].pos).collect();
        let consts: Vec<usize> = (0..3).filter(|&j| pos.iter().all(|p| p[j] == pos[0][j])).collect();
        let single = a == b;
        let mut best: Option<(i64, usize, bool, bool)> = None;
        let group: Vec<usize> = (a..=b).collect();
        for k in consts {
            let flat: Vec<(i64, i64)> = pos
                .iter()
                .map(|p| {
                    let v: Vec<i64> = (0..3).filter(|&j| j != k).map(|j| p[j]).collect();
                    (v[0], v[1])
                })
                .collect();
            if !simple_snake(&flat) {
                continue;
            }
            if !fits_bed(self.plate_mm(self.plate_span_mm(rec, &group, k))) {
                continue;
            }
            let pi = axis_of(rec[a].inn) == k;
            let po = axis_of(rec[b].out) == k;
            if pi && po {
                continue;
            }
            if (pi || po) && !self.allow_ports {
                continue;
            }
            if !single {
                if rec[a].inn != rec[a].out && !pi {
                    continue;
                }
                if rec[b].inn != rec[b].out && !po {
                    continue;
                }
            } else if pi || po {
                continue;
            }
            let cand = (pi as i64 + po as i64, k, pi, po);
            if best.is_none() || cand < best.unwrap() {
                best = Some(cand);
            }
        }
        best.map(|c| (c.1, c.2, c.3))
    }

    /// Split the walk into the fewest cuttable pieces.
    pub fn coplanar_pieces(&self, text: &str) -> Result<(Vec<Rec>, Vec<Vec<usize>>, Vec<PiecePlan>), String> {
        let rec = walk(text)?;
        let n = rec.len();
        let inf = f64::INFINITY;
        let mut best = vec![(inf, inf, inf); n + 1];
        best[0] = (0.0, 0.0, 0.0);
        let mut cut: Vec<Option<(usize, PiecePlan)>> = vec![None; n + 1];
        for i in 1..=n {
            for a in (0..i).rev() {
                let Some(plan) = self.piece_plan(&rec, a, i - 1) else { continue };
                let lone = (i - a == 1 && rec[a].inn != rec[a].out) as i64 as f64;
                let head = if FOLD_TURNS { (best[a].0 + lone, best[a].1 + 1.0) } else { (best[a].0 + 1.0, best[a].1 + lone) };
                let cost = (head.0, head.1, best[a].2 + plan.1 as i64 as f64 + plan.2 as i64 as f64);
                if cost <= best[i] {
                    best[i] = cost;
                    cut[i] = Some((a, plan));
                }
            }
        }
        if cut[n].is_none() {
            return Err("no way to cut this bore into flat pieces".into());
        }
        let (mut groups, mut plans, mut i) = (vec![], vec![], n);
        while i > 0 {
            let (a, plan) = cut[i].expect("TypeError: cannot unpack non-iterable NoneType object");
            groups.push((a..i).collect::<Vec<usize>>());
            plans.push(plan);
            i = a;
        }
        groups.reverse();
        plans.reverse();
        Ok((rec, groups, plans))
    }
}

pub fn fits_bed(mm: (f64, f64)) -> bool {
    let (a, b) = (py::max(mm.0, mm.1), py::min(mm.0, mm.1));
    a <= BED_W && b <= BED_H
}

fn flat3(v: [i64; 3], k: usize) -> (i64, i64) {
    let w: Vec<i64> = (0..3).filter(|&j| j != k).map(|j| v[j]).collect();
    (w[0], w[1])
}

/// Which plate, at which end, loses its coupling.
pub fn flat_sides(rec: &[Rec], groups: &[Vec<usize>], norms: &[usize]) -> Vec<(String, String)> {
    let mut out = vec![[String::new(), String::new()]; groups.len()];
    for i in 0..groups.len().saturating_sub(1) {
        for (who, other, end) in [(i, i + 1, 1), (i + 1, i, 0)] {
            let gr = &groups[other];
            if gr.len() != 1 || rec[gr[0]].inn == rec[gr[0]].out {
                continue;
            }
            let b = &rec[gr[0]];
            let miss = if other > who { dirs(b.out) } else { neg(dirs(b.inn)) };
            let k = norms[who];
            let axis = (0..3).find(|&j| miss[j] != 0).unwrap();
            if axis != k {
                continue;
            }
            let sign = if k == 0 || k == 2 { 1 } else { -1 };
            out[who][end] = if miss[axis] == sign { "first".into() } else { "mirror".into() };
        }
    }
    out.into_iter().map(|[a, b]| (a, b)).collect()
}

/// Which openings butt plain rather than couple.
pub fn plain_ends(plans: &[PiecePlan]) -> Vec<(bool, bool)> {
    let n = plans.len();
    let mut plain = vec![[false, false]; n];
    for (i, &(_, pi, po)) in plans.iter().enumerate() {
        if pi && i > 0 {
            plain[i - 1][1] = true;
        }
        if po && i < n - 1 {
            plain[i + 1][0] = true;
        }
    }
    plain[0][0] = true;
    plain[n - 1][1] = true;
    plain.into_iter().map(|[a, b]| (a, b)).collect()
}

fn lap_args(laps: &(String, String)) -> Vec<String> {
    let mut a = vec![];
    if !laps.0.is_empty() {
        a.push(format!("--lap_in={}", laps.0));
    }
    if !laps.1.is_empty() {
        a.push(format!("--lap_out={}", laps.1));
    }
    a
}

fn lap_tag(laps: &(String, String)) -> String {
    if laps.0.is_empty() && laps.1.is_empty() {
        return String::new();
    }
    let f = |s: &String| if s.is_empty() { "-".to_string() } else { s.clone() };
    format!("@{}{}", f(&laps.0), f(&laps.1))
}

fn plane_of(rec: &[Rec], idx: &[usize]) -> Vec<usize> {
    let mut used: Vec<usize> = vec![];
    for &i in idx {
        for a in [axis_of(rec[i].inn), axis_of(rec[i].out)] {
            if !used.contains(&a) {
                used.push(a);
            }
        }
    }
    used.sort();
    used
}

/// Which piece carries the tongue at each stranded turn, and how each is rolled.
pub fn assign_laps(rec: &[Rec], groups: &[Vec<usize>], plans: &[PiecePlan]) -> (Vec<usize>, Vec<(String, String)>, Vec<usize>) {
    let n = groups.len();
    let used: Vec<Vec<usize>> = groups.iter().map(|gr| plane_of(rec, gr)).collect();
    let mut norm: Vec<usize> = plans.iter().map(|p| p.0).collect();
    let mut free: Vec<bool> = used.iter().zip(plans).map(|(u, pl)| u.len() < 2 && !(pl.1 || pl.2)).collect();
    let mut laps = vec![[String::new(), String::new()]; n];
    let mut unfilled = vec![];
    for (e, gr) in groups.iter().enumerate() {
        if gr.len() != 1 || rec[gr[0]].inn == rec[gr[0]].out {
            continue;
        }
        let b = &rec[gr[0]];
        let mut want = vec![];
        if e > 0 {
            want.push((e - 1, dirs(b.out), 1));
        }
        if e < n - 1 {
            want.push((e + 1, neg(dirs(b.inn)), 0));
        }
        let mut filled = 0;
        for (p, d, end) in want {
            let gp = &groups[p];
            if gp.len() == 1 && rec[gp[0]].inn != rec[gp[0]].out && (d == neg(dirs(rec[gp[0]].inn)) || d == dirs(rec[gp[0]].out)) {
                continue;
            }
            let axis = (0..3).find(|&j| d[j] != 0).unwrap();
            if axis == norm[p] && free[p] {
                let travel = used[p][0];
                norm[p] = (0..3).find(|&j| j != travel && j != axis).unwrap();
                free[p] = false;
            }
            if axis == norm[p] {
                continue;
            }
            let Some(f) = face2d(flat3(d, norm[p])) else { continue };
            laps[p][end] = f.to_string();
            free[p] = false;
            filled += 1;
        }
        if filled == 0 {
            unfilled.push(e + 1);
        }
    }
    (norm, laps.into_iter().map(|[a, b]| (a, b)).collect(), unfilled)
}

impl Design {
    /// SnakeBox arguments for one piece, in its own 2D frame: (code, args, note)
    #[allow(clippy::too_many_arguments)]
    pub fn piece_spec(&self, rec: &[Rec], idx: &[usize], k: usize, laps: &(String, String), ports: (bool, bool), plains: (bool, bool), flats: &(String, String)) -> Result<(String, Vec<String>, String), String> {
        let pos: Vec<[i64; 3]> = idx.iter().map(|&i| rec[i].pos).collect();
        let (first, last) = (&rec[idx[0]], &rec[idx[idx.len() - 1]]);
        let (port_in, port_out) = ports;
        let flat = |v: [i64; 3]| flat3(v, k);
        let a_in = flat(neg(dirs(first.inn)));
        let a_out = flat(dirs(last.out));
        let span = format!("{}-{}", idx[0] + 1, idx[idx.len() - 1] + 1);
        if (!port_in && face2d(a_in).is_none()) || (!port_out && face2d(a_out).is_none()) {
            return Err(format!("piece at blocks {span} opens on a face plate but was not given a port"));
        }
        let mut extra = vec![];
        if port_in || port_out {
            extra.push(if port_in { "--port_in" } else { "--port_out" }.to_string());
            let want = if port_in { neg(dirs(first.inn)) } else { dirs(last.out) };
            let sign = if k == 0 || k == 2 { 1 } else { -1 };
            let mut unmirrored = [0; 3];
            unmirrored[k] = sign;
            if want != unmirrored {
                extra.push("--port_mirror".into());
            }
        }
        extra.extend(self.piece_widths(rec, idx, k)?);
        if plains.0 {
            extra.push("--plain_in".into());
        }
        if plains.1 {
            extra.push("--plain_out".into());
        }
        if !flats.0.is_empty() {
            extra.push(format!("--flat_in={}", flats.0));
        }
        if !flats.1.is_empty() {
            extra.push(format!("--flat_out={}", flats.1));
        }
        let ptag = format!(
            "{}{}{}{}{}{}",
            if port_in { "~i" } else { "" },
            if port_out { "~o" } else { "" },
            if plains.0 { "~a" } else { "" },
            if plains.1 { "~b" } else { "" },
            if !flats.0.is_empty() { "~f" } else { "" },
            if !flats.1.is_empty() { "~g" } else { "" }
        );
        if idx.len() == 1 {
            if !self.cubic() {
                return Err(format!(
                    "block {} is a one-cell piece and the cell is not a cube (section {}, straight {}). Straights and stranded turns share one file per shape, which only holds for a cube. Lengthen the run, or use a cubic cell.",
                    idx[0] + 1,
                    g(self.block),
                    g(self.straight)
                ));
            }
            if first.inn == first.out {
                let mut args = vec!["--path=".to_string(), format!("--open_faces={},{}", face2d(a_in).unwrap(), face2d(a_out).unwrap())];
                args.extend(lap_args(laps));
                args.extend(extra);
                return Ok((format!("S1{ptag}{}", lap_tag(laps)), args, "straight".into()));
            }
            let cross = a_in.0 * a_out.1 - a_in.1 * a_out.0;
            let faces = if cross < 0 { ('N', 'E') } else { ('E', 'N') };
            let q = (0..4).find(|&r| face2d(turn2d(a_in, r)) == Some(faces.0) && face2d(turn2d(a_out, r)) == Some(faces.1)).unwrap();
            let tl = |l: &String| if l.is_empty() { String::new() } else { face2d(turn2d(vec2d(l.chars().next().unwrap()), q)).unwrap().to_string() };
            let laps = (tl(&laps.0), tl(&laps.1));
            let mut args = vec!["--path=".to_string(), format!("--open_faces={},{}", faces.0, faces.1)];
            args.extend(lap_args(&laps));
            args.extend(extra);
            return Ok((format!("E{}{}{ptag}{}", faces.0, faces.1, lap_tag(&laps)), args, "elbow".into()));
        }
        let cells: Vec<(i64, i64)> = pos.iter().map(|&p| flat(p)).collect();
        let steps: Vec<(i64, i64)> = cells.windows(2).map(|w| (w[1].0 - w[0].0, w[1].1 - w[0].1)).collect();
        let path: String = steps.iter().map(|&d| path2d(d)).collect();
        if !port_in && a_in != (-steps[0].0, -steps[0].1) {
            return Err(format!("piece at blocks {span}: the bore enters its end block from the side, so that block bends. SnakeBox opens an end only on the face opposite its neighbour."));
        }
        if !port_out && a_out != steps[steps.len() - 1] {
            return Err(format!("piece at blocks {span}: the bore leaves its end block sideways, so that block bends."));
        }
        let straight = path.chars().collect::<HashSet<char>>().len() == 1;
        let code = format!("{}{ptag}{}", if straight { format!("S{}", cells.len()) } else { format!("B{path}") }, lap_tag(laps));
        let mut args = vec![format!("--path={path}")];
        args.extend(lap_args(laps));
        args.extend(extra);
        Ok((code, args, if straight { "straight" } else { "bend" }.into()))
    }
}

#[derive(Clone, Debug)]
pub struct Spec {
    pub group: Vec<usize>,
    pub code: String,
    pub args: Vec<String>,
    pub kind: String,
    pub flat: (String, String),
    pub lap: (String, String),
    pub norm: usize,
    pub plan: PiecePlan,
}

pub struct Specs {
    pub rec: Vec<Rec>,
    pub groups: Vec<Vec<usize>>,
    pub plans: Vec<PiecePlan>,
    pub plan: Vec<Spec>,
    pub unfilled: Vec<usize>,
}

impl Design {
    /// Every section's SnakeBox arguments, in assembly order.
    pub fn specs_for(&self, text: &str) -> Result<Specs, String> {
        let (rec, groups, plans) = self.coplanar_pieces(text)?;
        let plains = plain_ends(&plans);
        let (norm, laps, unfilled) = assign_laps(&rec, &groups, &plans);
        let flats = flat_sides(&rec, &groups, &norm);
        let mut out = vec![];
        let mouths = self.mouth_at.clone().unwrap_or_default();
        if self.mouth_at.is_some() {
            for &b in &mouths {
                if !(1 <= b && b <= rec.len() as i64) {
                    return Err(format!("--mouth-at names block {b} and the walk has {}, numbered 1 to {}.", rec.len(), rec.len()));
                }
            }
            if mouths.iter().collect::<HashSet<_>>().len() != mouths.len() {
                return Err(format!("--mouth-at names a block twice: {}.", py::ints_repr(&mouths)));
            }
        }
        let mut sorted = mouths.clone();
        sorted.sort();
        let plate_of: HashMap<i64, &str> = sorted.iter().enumerate().map(|(n, &b)| (b, if n % 2 == 0 { "first" } else { "mirror" })).collect();
        for (i, gr) in groups.iter().enumerate() {
            let (mut code, mut args, note) = self.piece_spec(&rec, gr, norm[i], &laps[i], (plans[i].1, plans[i].2), plains[i], &flats[i])?;
            let mut mine: Vec<(i64, &str)> = mouths.iter().filter(|&&b| gr[0] as i64 <= b - 1 && b - 1 <= gr[gr.len() - 1] as i64).map(|&b| (b - 1 - gr[0] as i64, plate_of[&b])).collect();
            if !mine.is_empty() {
                if gr.len() == 1 {
                    return Err(format!(
                        "--mouth-at puts a mouth in block {}, which is a one-cell piece shared between rotations: every straight or stranded turn of that shape is the same file, so a hole in one is a hole in all of them.",
                        gr[0] + 1
                    ));
                }
                mine.sort();
                args.push(format!("--mouths={}", mine.iter().map(|(c, p)| format!("{c}:{p}")).collect::<Vec<_>>().join(",")));
                code += &format!("~m{}", mine.iter().map(|(c, p)| format!("{c}{}", &p[..1])).collect::<String>());
            }
            out.push(Spec { group: gr.clone(), code, args, kind: note, flat: flats[i].clone(), lap: laps[i].clone(), norm: norm[i], plan: plans[i] });
        }
        Ok(Specs { rec, groups, plans, plan: out, unfilled })
    }
}

/// A walk, or something that holds one.
pub fn walk_text(arg: &str) -> Result<String, String> {
    if !std::path::Path::new(arg).exists() {
        return Ok(arg.to_string());
    }
    let body = std::fs::read_to_string(arg).map_err(|e| e.to_string())?;
    let low = arg.to_lowercase();
    if low.ends_with(".html") || low.ends_with(".htm") {
        let r = lasermade_tools::re::rx(r#"<div\s[^>]*class="walk"[^>]*>([^<]+)</div>"#);
        return match lasermade_tools::re::first(&r, &body) {
            Some(c) => Ok(lasermade_tools::html::unescape(lasermade_tools::re::g(&c, 1)).trim().to_string()),
            None => Err(format!("{arg} is not one of these pages: no walk in it")),
        };
    }
    Ok(body.trim().to_string())
}

// --- reading SnakeBox's drawing back ------------------------------------------

/// svgpath.pts: every point a path visits, in order.
pub fn pts(d: &str) -> Result<Vec<(f64, f64)>, String> {
    let r = lasermade_tools::re::rx(r"[A-Za-z]|[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?");
    let toks: Vec<&str> = r.find_iter(d).map(|m| m.unwrap().as_str()).collect();
    let mut p = vec![];
    let mut i = 0;
    let mut cur: Option<(f64, f64)> = None;
    let head: String = d.chars().take(60).collect();
    let f = |s: &str| s.parse::<f64>().unwrap();
    while i < toks.len() {
        let c = toks[i];
        if c == "M" || c == "L" {
            cur = Some((f(toks[i + 1]), f(toks[i + 2])));
            p.push(cur.unwrap());
            i += 3;
        } else if c == "H" || c == "V" {
            let Some(q) = cur else {
                return Err(format!("path starts with {c}, which has no point to carry forward: {}", py::str_repr(&head)));
            };
            cur = Some(if c == "H" { (f(toks[i + 1]), q.1) } else { (q.0, f(toks[i + 1])) });
            p.push(cur.unwrap());
            i += 2;
        } else if c == "C" {
            cur = Some((f(toks[i + 5]), f(toks[i + 6])));
            p.push(cur.unwrap());
            i += 7;
        } else if c == "Z" || c == "z" {
            i += 1;
        } else if c.chars().next().unwrap().is_alphabetic() {
            return Err(format!("path command {} is one this reader does not handle, and skipping it would drop the points it carries. Path: {}", py::str_repr(c), py::str_repr(&head)));
        } else {
            return Err(format!("number {} with no command in front of it, at token {i} of {}", py::str_repr(c), py::str_repr(&head)));
        }
    }
    Ok(p)
}

/// One part as cut() hands it on.
#[derive(Clone, Debug)]
pub struct Part {
    pub d: String,
    pub pts: Vec<(f64, f64)>,
    pub role: char,
    pub x0: f64,
    pub y0: f64,
    pub w: f64,
    pub h: f64,
    pub holes: Vec<Part>,
    pub name: String,
}

impl Part {
    pub fn of(d: String, pts: Vec<(f64, f64)>, role: char) -> Part {
        let xs = pts.iter().map(|q| q.0);
        let ys = pts.iter().map(|q| q.1);
        let (x0, x1) = (fmin(xs.clone()), fmax(xs));
        let (y0, y1) = (fmin(ys.clone()), fmax(ys));
        Part { d, pts, role, x0, y0, w: x1 - x0, h: y1 - y0, holes: vec![], name: String::new() }
    }
    /// The outline without its closing repeat
    pub fn ring(&self) -> &[(f64, f64)] {
        open_ring(&self.pts)
    }
}

pub fn open_ring(p: &[(f64, f64)]) -> &[(f64, f64)] {
    if p[0] == p[p.len() - 1] {
        &p[..p.len() - 1]
    } else {
        p
    }
}

/// min() over floats, Python's way (the first of equals)
pub fn fmin(it: impl Iterator<Item = f64>) -> f64 {
    it.reduce(|a, b| if b < a { b } else { a }).unwrap()
}
pub fn fmax(it: impl Iterator<Item = f64>) -> f64 {
    it.reduce(|a, b| if b > a { b } else { a }).unwrap()
}

pub fn polygon(q: &[(f64, f64)]) -> Polygon<f64> {
    let q = open_ring(q);
    Polygon::new(q.iter().map(|&(x, y)| geo::coord! {x: x, y: y}).collect::<Vec<_>>().into(), vec![])
}

/// For each closed path, the index of the path it is a hole in, or None.
pub fn hosts(outlines: &[Vec<(f64, f64)>]) -> Vec<Option<usize>> {
    let gs: Vec<Polygon<f64>> = outlines.iter().map(|q| polygon(q)).collect();
    let mut out = vec![];
    for i in 0..gs.len() {
        let inside: Vec<usize> = (0..gs.len()).filter(|&j| j != i && gs[j].contains(&gs[i])).collect();
        out.push(inside.into_iter().reduce(|a, b| if gs[b].unsigned_area() < gs[a].unsigned_area() { b } else { a }));
    }
    out
}

impl Design {
    /// Draw a piece and read back its parts. Err is the ValueError SnakeBox
    /// raised, with the arguments it was given.
    pub fn cut(&self, args: &[String]) -> Result<Vec<Part>, String> {
        let mut all: Vec<String> = args.to_vec();
        all.extend(self.common.iter().cloned());
        if self.flat {
            all.push("--pin_length=0".into());
        }
        let groups = match Snake::from_args(&all).and_then(|s| s.render()) {
            Ok(gs) => gs,
            Err(e) => return Err(format!("{e} ({})", args.join(" "))),
        };
        let mut ps = vec![];
        for (id, paths) in groups {
            let role = if id == "p-0" || id == "p-1" { 'P' } else { 'W' };
            let (mut black, mut marks) = (vec![], vec![]);
            for (d, stroke) in paths {
                let p = pts(&d).map_err(|e| format!("{e} ({})", args.join(" ")))?;
                let rec = Part::of(d, p, role);
                if stroke == "rgb(255,0,255)" {
                    marks.push(rec);
                } else if stroke == "rgb(0,0,0)" {
                    black.push(rec);
                }
            }
            let host = hosts(&black.iter().map(|q| q.pts.clone()).collect::<Vec<_>>());
            for (i, o) in black.iter().enumerate() {
                if host[i].is_some() {
                    continue;
                }
                let mut o = o.clone();
                o.holes = black.iter().enumerate().filter(|(j, _)| host[*j] == Some(i)).map(|(_, q)| q.clone()).collect();
                for m in &marks {
                    o = crate::geom::subtract(&o, m, THICKNESS);
                }
                ps.push(o);
            }
        }
        let (mut np, mut nw) = (0, 0u8);
        for o in ps.iter_mut() {
            if o.role == 'P' {
                np += 1;
                o.name = format!("P{np}");
            } else {
                o.name = ((b'A' + nw) as char).to_string();
                nw += 1;
            }
        }
        Ok(ps)
    }
}

// --- the engraving -------------------------------------------------------------

include!("glyphs.rs");

/// The label, and a tick that says which way up it was engraved.
pub fn glyphs(text: &str, h: f64) -> (Vec<String>, f64) {
    let (w, gap) = (h * 0.62, h * 0.20);
    let mut out = vec![];
    let mut cx = 0.0f64;
    for ch in text.chars() {
        for st in glyph(ch) {
            let p: Vec<String> = st.iter().map(|&(px, ny)| format!("{:.2},{:.2}", cx + px * w, ny * h)).collect();
            out.push(format!("<polyline points=\"{}\" fill=\"none\" stroke=\"#0000ff\" stroke-width=\"0.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>", p.join(" ")));
        }
        cx += w + gap;
    }
    let right = cx - gap;
    let (tg, tl) = (h * 0.16, h * 0.22);
    out.push(format!(
        "<polyline points=\"{:.2},0.00 {:.2},0.00\" fill=\"none\" stroke=\"#0000ff\" stroke-width=\"0.3\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        right + tg,
        right + tg + tl
    ));
    (out, right + tg + tl)
}

/// Even-odd point-in-polygon.
pub fn inside(poly: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (x1, y1) = poly[i];
        let (x2, y2) = poly[(i + 1) % n];
        if (y1 > y) != (y2 > y) && x < x1 + (y - y1) * (x2 - x1) / (y2 - y1) {
            c = !c;
        }
    }
    c
}

/// Distance from (x,y) to the nearest edge of the outline.
fn clearance(poly: &[(f64, f64)], x: f64, y: f64) -> f64 {
    let mut best = f64::INFINITY;
    let n = poly.len();
    for i in 0..n {
        let (x1, y1) = poly[i];
        let (x2, y2) = poly[(i + 1) % n];
        let (dx, dy) = (x2 - x1, y2 - y1);
        let l = dx * dx + dy * dy;
        let t = if l == 0.0 { 0.0 } else { py::max(0.0, py::min(1.0, ((x - x1) * dx + (y - y1) * dy) / l)) };
        let (px, pyy) = (x1 + t * dx, y1 + t * dy);
        let d = py::powf(py::powf(x - px, 2.0) + py::powf(y - pyy, 2.0), 0.5);
        if d < best {
            best = d;
        }
    }
    best
}

fn hole_rings(part: &Part) -> Vec<Vec<(f64, f64)>> {
    part.holes.iter().map(|q| open_ring(&q.pts).to_vec()).collect()
}

/// Is every point of the drawn label on the part, and off its holes?
fn strokes_fit(part: &Part, gs: &[String], ox: f64, oy: f64) -> bool {
    let poly = part.ring();
    let holes = hole_rings(part);
    let r = lasermade_tools::re::rx(r"(-?[\d.]+),(-?[\d.]+)");
    for gl in gs {
        for c in lasermade_tools::re::all(&r, gl) {
            let x = ox + lasermade_tools::re::g(&c, 1).parse::<f64>().unwrap();
            let y = oy + lasermade_tools::re::g(&c, 2).parse::<f64>().unwrap();
            if !inside(poly, x, y) || holes.iter().any(|h| inside(h, x, y)) {
                return false;
            }
        }
    }
    true
}

/// Centre for a gw x gh label that lands on material.
fn label_spot(part: &Part, gw: f64, gh: f64) -> Option<(f64, f64, f64)> {
    let step = 1.5;
    let poly = part.ring();
    let holes = hole_rings(part);
    let (x0, y0, w, h) = (part.x0, part.y0, part.w, part.h);
    let (mut best, mut bx, mut by) = (-1.0f64, x0 + w / 2.0, y0 + h / 2.0);
    let mut y = y0 + step;
    while y < y0 + h {
        let mut x = x0 + step;
        while x < x0 + w {
            if inside(poly, x, y) && !holes.iter().any(|hp| inside(hp, x, y)) {
                let mut c = clearance(poly, x, y);
                for hp in &holes {
                    let v = clearance(hp, x, y);
                    if v < c {
                        c = v;
                    }
                }
                if c > best {
                    (best, bx, by) = (c, x, y);
                }
            }
            x += step;
        }
        y += step;
    }
    let need = py::powf(py::powf(gw / 2.0, 2.0) + py::powf(gh / 2.0, 2.0), 0.5);
    let mut scale = 1.0f64;
    while need * scale > best && scale > 0.45 {
        scale -= 0.05;
    }
    if need * scale > best {
        return None;
    }
    Some((bx, by, scale))
}

impl Design {
    /// The engraving for one part: its section number, and TAG if there is one.
    pub fn part_labels(&self, p: &Part, code: &str, sections: i64) -> String {
        if sections == 1 && self.tag.is_empty() {
            return String::new();
        }
        let gh = 5.0 / 3.0;
        let code = format!("{code}{}", self.tag);
        let (_, gw0) = glyphs(&code, gh);
        let Some((cx, cy, sc)) = label_spot(p, gw0, gh) else { return String::new() };
        let (gs, gw) = glyphs(&code, gh * sc);
        if !strokes_fit(p, &gs, cx - gw / 2.0, cy + gh * sc / 2.0) {
            return String::new();
        }
        format!("<g transform=\"translate({:.2},{:.2})\">{}</g>", cx - p.x0 - gw / 2.0, cy - p.y0 + gh * sc / 2.0, gs.concat())
    }
}

// --- the sheets --------------------------------------------------------------

pub struct Meta {
    pub span: String,
    pub inn: char,
    pub out: char,
    pub kind: String,
    pub plate: String,
    pub total: usize,
    pub raw: String,
    pub design: String,
}

fn esc(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;")
}

/// os.path.join of two parts
pub fn pjoin(a: &str, b: &str) -> String {
    if b.starts_with('/') || a.is_empty() {
        b.to_string()
    } else if a.ends_with('/') {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

impl Design {
    /// The <title> and <desc> a section's sheet carries.
    fn provenance(&self, meta: &Meta, code: &str, n: usize, nsheets: usize, nparts: usize, sw: f64, sh: f64) -> String {
        let bore = self.block - 2.0 * THICKNESS;
        let of = format!(" of {}", meta.total);
        let part = if nsheets == 1 { String::new() } else { format!(", sheet {n} of {nsheets}") };
        let title = format!(
            "{} bore - section {}{of}{part}, {} {}, {nparts} parts on {sw:.0}x{sh:.0}mm",
            meta.design,
            code.parse::<i64>().unwrap(),
            meta.kind,
            &meta.raw[meta.raw.char_indices().nth(1).map(|x| x.0).unwrap_or(meta.raw.len())..]
        );
        let one = meta.total == 1;
        let desc = format!(
            "1 user unit = 1mm. {}mm square bore in {}mm stock, cut for a {}mm sheet at {}mm kerf, so a {}mm block pitch; a block that runs straight is {}mm long and a block that turns is a {}mm cube. Blocks {} of the walk, entering on {} and leaving on {}, a {} block plate laid flat. Two face plates (mirror images) and the side walls; {}. Inner cuts are the port and come before their outline. {}",
            g(bore),
            g(THICKNESS),
            g(self.sheet),
            g(self.kerf),
            g(self.block),
            g(self.straight),
            g(self.block),
            meta.span,
            meta.inn,
            meta.out,
            meta.plate,
            if one { "one section, so nothing is engraved - every part would carry the same number and the plate is the jig" } else { "every part carries the section number only" },
            if one { "black #000000 cuts; there is no engrave stage on this sheet." } else { "black #000000 cuts, blue #0000ff engraves." }
        );
        format!("<title>{}</title>\n<desc>{}</desc>\n", esc(&title), esc(&desc))
    }

    /// Lay the parts out, turning and re-sheeting as the bed demands.
    /// Returns [(path, w, h)], one entry per sheet written.
    pub fn sheet(&self, parts: &[Part], code: &str, path: &str, meta: &Meta) -> Vec<(String, f64, f64)> {
        let (bed, bed_h) = (BED_W, BED_H);
        let (m, gap) = (12.0f64, 10.0f64);
        let laid: Vec<(&Part, bool, (f64, f64))> = parts
            .iter()
            .map(|p| {
                let rot = p.h > bed_h - 2.0 * m && p.w <= bed_h - 2.0 * m;
                (p, rot, if rot { (p.h, p.w) } else { (p.w, p.h) })
            })
            .collect();
        let mut sheets: Vec<Vec<(&Part, bool, f64, f64, f64, f64)>> = vec![];
        let mut cur = vec![];
        let (mut x, mut y, mut rh) = (m, m, 0.0f64);
        for &(p, rot, (w, h)) in &laid {
            if x > m && x + w + m > bed {
                if y + rh + gap + h + m > bed_h {
                    sheets.push(std::mem::take(&mut cur));
                    (x, y, rh) = (m, m, 0.0);
                } else {
                    (x, y, rh) = (m, y + rh + gap, 0.0);
                }
            }
            cur.push((p, rot, w, h, x, y));
            x += w + gap;
            rh = py::max(rh, h);
        }
        if !cur.is_empty() {
            sheets.push(cur);
        }
        let mut out = vec![];
        let ns = sheets.len();
        for (n, placed) in sheets.iter().enumerate() {
            let n = n + 1;
            let sw = fmax(placed.iter().map(|t| t.4 + t.2)) + m;
            let sh = fmax(placed.iter().map(|t| t.5 + t.3)) + m;
            let mut body = vec![];
            for &(p, rot, w, _h, x, y) in placed {
                let inner: String = p.holes.iter().map(|q| format!("<path d=\"{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.2\"/>", q.d)).collect();
                let local = format!("<g transform=\"translate({:.3},{:.3})\">{inner}<path d=\"{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.2\"/></g>", -p.x0, -p.y0, p.d);
                let lbl = self.part_labels(p, code, meta.total as i64);
                let t = if rot { format!("translate({:.3},{y:.3}) rotate(90)", x + w) } else { format!("translate({x:.3},{y:.3})") };
                body.push(format!("<g transform=\"{t}\">{local}{lbl}</g>"));
            }
            let this = if ns == 1 {
                path.to_string()
            } else {
                let (stem, ext) = splitext(path);
                format!("{stem}-sheet{n}{ext}")
            };
            let svg = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{sw:.2}mm\" height=\"{sh:.2}mm\" viewBox=\"0 0 {sw:.2} {sh:.2}\">\n{}{}\n</svg>\n",
                self.provenance(meta, code, n, ns, placed.len(), sw, sh),
                body.join("\n")
            );
            std::fs::write(&this, svg).unwrap_or_else(|e| panic!("{this}: {e}"));
            out.push((this, sw, sh));
        }
        out
    }
}

/// os.path.splitext
pub fn splitext(p: &str) -> (String, String) {
    let base_start = p.rfind('/').map(|i| i + 1).unwrap_or(0);
    let base = &p[base_start..];
    let lead = base.len() - base.trim_start_matches('.').len();
    match base[lead..].rfind('.') {
        Some(i) => {
            let at = base_start + lead + i;
            (p[..at].to_string(), p[at..].to_string())
        }
        None => (p.to_string(), String::new()),
    }
}

// --- names ---------------------------------------------------------------------

fn dull(s: &str) -> bool {
    let r = lasermade_tools::re::rx(r"^(?:bores?([-_][\d.]+mm)?|[\d.]+mm)$");
    lasermade_tools::re::has(&r, s)
}
const FAMILY: [&str; 5] = ["coil", "meander", "spiral", "hilbert", "swept-curve"];

fn basename(p: &str) -> String {
    p.rsplit('/').next().unwrap_or("").to_string()
}
fn dirname(p: &str) -> String {
    match p.rfind('/') {
        Some(0) => "/".into(),
        Some(i) => p[..i].to_string(),
        None => String::new(),
    }
}

/// (folder name, the folder names that say what the design is).
pub fn folder_stack(outdir: &str) -> (String, Vec<String>) {
    let full = lasermade_tools::text::normpath(&lasermade_tools::text::abspath(outdir));
    let name = basename(&full);
    let mut stack = vec![name.clone()];
    let mut at = full.clone();
    let mut cur = name.clone();
    while dull(&cur.to_lowercase()) {
        at = dirname(&at);
        cur = basename(&at);
        if cur.is_empty() {
            break;
        }
        stack.insert(0, cur.clone());
    }
    let up = dirname(&at);
    let parent = basename(&up);
    if FAMILY.contains(&parent.to_lowercase().as_str()) {
        stack.insert(0, parent);
    }
    (name, stack)
}

/// The design's part of a cut file's name.
pub fn design_slug(stack: &[String]) -> String {
    let keep: Vec<&str> = stack.iter().filter(|w| !dull(&w.to_lowercase())).map(|s| s.as_str()).collect();
    keep.join("-").to_lowercase().replace('_', "-")
}

impl Design {
    pub fn bore_tag(&self) -> String {
        format!("bore{}", g(self.block - 2.0 * THICKNESS))
    }
    pub fn cutname(&self, code: &str, total: usize, raw: &str, stack: &[String]) -> String {
        let lead: Vec<String> = [self.bore_tag(), design_slug(stack)].into_iter().filter(|x| !x.is_empty()).collect();
        format!("{}-{:02}of{:02}-{}-cut-files.svg", lead.join("-"), code.parse::<i64>().unwrap(), total, filename(raw))
    }
}

/// The shape half of a file name, surviving the path letters.
pub fn filename(code: &str) -> String {
    let mut base = code.to_string();
    let mut bits = vec![];
    if let Some((b, faces)) = base.clone().split_once('@') {
        base = b.to_string();
        bits.push(format!("lap{}", faces.replace('-', "")));
    }
    for (tag, name) in [("~i", "portin"), ("~o", "portout"), ("~a", "buttin"), ("~b", "buttout"), ("~f", "flatin"), ("~g", "flatout")] {
        if base.contains(tag) {
            base = base.replace(tag, "");
            bits.push(name.to_string());
        }
    }
    let tail = if bits.is_empty() { String::new() } else { format!("-{}", bits.join("-")) };
    if let Some(r) = base.strip_prefix('E') {
        return format!("elbow-{r}{tail}");
    }
    if let Some(r) = base.strip_prefix('S') {
        return format!("straight{r}{tail}");
    }
    format!("bend-{}{tail}", &base[1..])
}

// --- the report ----------------------------------------------------------------

/// What check.main(text, folder, report=False) hands back:
/// (checks, failed, [(name, failures)])
pub type CheckSummary = (usize, usize, Vec<(String, Vec<String>)>);

fn ljust(s: &str, n: usize) -> String {
    format!("{s:<n$}")
}

impl Design {
    /// bore_split.main(text, outdir): the report, and with outdir the sheets,
    /// the page and the gate. Output goes to `o`, as print() would.
    pub fn main(&self, text: &str, outdir: Option<&str>, o: &mut String) -> Result<(), String> {
        let sp = self.specs_for(text)?;
        let rec = &sp.rec;
        let n = rec.len();
        o.push_str(&format!("bore:  {}\n", text.trim()));
        let mm = rec.iter().fold(0.0, |s, r| s + self.extent(r, axis_of(r.out)));
        let note = if self.cubic() { String::new() } else { format!("  (section {}, straight {}, turns cubic)", g(self.block), g(self.straight)) };
        o.push_str(&format!("       {n} blocks ({}mm of centreline){note}\n", g(mm)));
        o.push_str(&format!("       {} pieces to assemble\n\n", sp.plan.len()));
        // (group, number, args, kind, shape code)
        let specs: Vec<(Vec<usize>, String, Vec<String>, String, String)> = sp.plan.iter().enumerate().map(|(i, p)| (p.group.clone(), (i + 1).to_string(), p.args.clone(), p.kind.clone(), p.code.clone())).collect();
        o.push_str("assembly order   (a piece is one flat snake = one SVG)\n");
        o.push_str("  #    blocks   kind       in   out   plate            shape\n");
        let mut toobig = vec![];
        let mut facts: HashMap<String, (String, char, char, String, String)> = HashMap::new();
        for ((gr, code, _args, note, raw), p) in specs.iter().zip(&sp.plan) {
            let k = p.norm;
            let (r0, r1) = (&rec[gr[0]], &rec[gr[gr.len() - 1]]);
            let span = format!("{}-{}", gr[0] + 1, gr[gr.len() - 1] + 1);
            let (bl, mm) = self.plate_size(rec, gr, k);
            let ok = fits_bed(mm);
            let size = format!("{}x{} bl {:.0}x{:.0}", bl.0, bl.1, py::max(mm.0, mm.1), py::min(mm.0, mm.1));
            if !ok {
                toobig.push(code.clone());
            }
            o.push_str(&format!("  {} {} {} {}    {}    {} {raw}{}\n", ljust(code, 4), ljust(&span, 8), ljust(note, 10), r0.inn, r1.out, ljust(&size, 16), if ok { "" } else { "  !! over the bed" }));
            facts.insert(code.clone(), (span, r0.inn, r1.out, note.clone(), format!("{}x{}", bl.0, bl.1)));
        }
        if self.refuse_elbows {
            let bad: Vec<&str> = specs.iter().filter(|s| s.3 == "elbow").map(|s| s.1.as_str()).collect();
            if !bad.is_empty() {
                return Err(format!(
                    "--refuse-elbows: section{} {} of {} {}. Nothing written. Lengthen the term between the turns: a hairpin needs 2 and a coil 3.",
                    if bad.len() > 1 { "s" } else { "" },
                    bad.join(", "),
                    specs.len(),
                    if bad.len() > 1 { "are elbows" } else { "is an elbow" }
                ));
            }
        }
        o.push_str(&format!("\ncut list   ({})\n", if specs.len() == 1 { "one section, nothing engraved" } else { "every part engraved with its section number" }));
        let mut total = 0;
        let mut shapes: Vec<(String, Vec<String>)> = vec![];
        let (fname, stack) = folder_stack(outdir.unwrap_or("."));
        for (_gr, code, args, _note, raw) in &specs {
            let fnm = self.cutname(code, specs.len(), raw, &stack);
            let mut line = format!("  {code:>3}  {fnm:<64}");
            if let Some(outdir) = outdir {
                std::fs::create_dir_all(pjoin(outdir, "cut-files")).unwrap();
                let parts = self.cut(args)?;
                let f = &facts[code];
                let meta = Meta { span: f.0.clone(), inn: f.1, out: f.2, kind: f.3.clone(), plate: f.4.clone(), total: specs.len(), raw: raw.clone(), design: { let s = design_slug(&stack); if s.is_empty() { fname.clone() } else { s } } };
                let sheets = self.sheet(&parts, code, &pjoin(&pjoin(outdir, "cut-files"), &fnm), &meta);
                total += parts.len();
                let sizes: Vec<String> = sheets.iter().map(|(_, w, h)| format!("{w:.0}x{h:.0}")).collect();
                line += &format!("{} parts   {}mm", parts.len(), sizes.join(" + "));
                if sheets.len() > 1 {
                    line += &format!("   ({} sheets)", sheets.len());
                }
                if sheets.iter().any(|t| t.1 > BED_W || t.2 > BED_H) {
                    line += &format!("   !! over the {BED_W:.0} x {BED_H:.0} bed");
                }
            }
            match shapes.iter_mut().find(|(r, _)| r == raw) {
                Some(e) => e.1.push(code.clone()),
                None => shapes.push((raw.clone(), vec![code.clone()])),
            }
            o.push_str(&line);
            o.push('\n');
        }
        if let Some(outdir) = outdir {
            let words = stack.join(" ");
            let mut title: String = words
                .replace('_', " ")
                .replace('-', " ")
                .split_whitespace()
                .map(|w| if w.chars().next().is_some_and(|c| c.is_ascii_digit()) { w.to_string() } else { pytitle(w) })
                .collect::<Vec<_>>()
                .join(" ");
            if !title.contains("Bore") {
                title += " Bore";
            }
            if let Some(t) = &self.title {
                title = t.clone();
            }
            let path = pjoin(outdir, &format!("{fname}.html"));
            std::fs::write(&path, crate::viewer::build(self, text, &title)?).unwrap();
            o.push_str(&format!("\n  {}drag to turn, colour by direction or section\n", ljust(&basename(&path), 44)));
            let (nchecks, bad, fails) = crate::check::summary(self, text, &pjoin(outdir, "cut-files"))?;
            o.push_str(&format!("  {}{nchecks} checks, {bad} failed\n", ljust("checked", 44)));
            for (name, msgs) in fails {
                o.push_str(&format!("    !! {name}\n"));
                for m in msgs.iter().take(3) {
                    o.push_str(&format!("       {m}\n"));
                }
            }
        }
        o.push_str(&format!("\n  {} sections{}\n", specs.len(), if outdir.is_some() { format!(", {total} flat parts") } else { String::new() }));
        for (_, v) in shapes.iter().filter(|(_, v)| v.len() > 1) {
            o.push_str(&format!("  sections {} are the same shape, cut separately so each carries its own number\n", v.join(", ")));
        }
        let rub = touching(self, rec);
        if !rub.is_empty() {
            let pairs: Vec<String> = rub.iter().take(6).map(|(a, b)| format!("{a}-{b}")).collect();
            o.push_str(&format!(
                "\n  ! {} pair(s) of blocks touch without being joined along the bore:\n    {}{}\n    Legal, but the walk folds back on itself there. Check the 3D view.\n",
                rub.len(),
                pairs.join(", "),
                if rub.len() > 6 { " ..." } else { "" }
            ));
        }
        if !toobig.is_empty() {
            o.push_str(&format!(
                "\n  ! piece(s) {} do not fit the {BED_W:.0} x {BED_H:.0}mm bed.\n    A flat piece may span at most {} blocks one way by {} the other. Put a turn out of the plane to break it up.\n",
                toobig.join(", "),
                crate::snakebox::floordiv(BED_W - 2.0 * PIN - 2.0 * self.burn, self.block) as i64,
                crate::snakebox::floordiv(BED_H - 2.0 * PIN - 2.0 * self.burn, self.block) as i64
            ));
        }
        if !sp.unfilled.is_empty() {
            o.push_str(&format!(
                "\n  ! the inside of the bend is left open at piece(s) {}: neither neighbour can\n    carry the tongue on a wall, so it would have to go on a plate.\n",
                sp.unfilled.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", ")
            ));
        }
        for i in 0..specs.len().saturating_sub(1) {
            if specs[i].3 == "elbow" && specs[i + 1].3 == "elbow" {
                o.push_str(&format!(
                    "\n  ! pieces {} and {} are both elbows meeting directly.\n    Only 2 of the 3 tabs engage if their turns are in perpendicular\n    planes. Consider a block of straight between them.\n",
                    i + 1,
                    i + 2
                ));
            }
        }
        Ok(())
    }
}

/// str.title()
pub fn pytitle(w: &str) -> String {
    let mut out = String::new();
    let mut prev_cased = false;
    for c in w.chars() {
        if c.is_alphabetic() {
            if prev_cased {
                out.extend(c.to_lowercase());
            } else {
                out.extend(c.to_uppercase());
            }
            prev_cased = true;
        } else {
            out.push(c);
            prev_cased = false;
        }
    }
    out
}
