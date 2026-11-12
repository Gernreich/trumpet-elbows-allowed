//! Every design we have built or tried, with the switches it is cut with:
//! the table regress and repro both run over (regress.py's DESIGNS until
//! 2026-11-08). Passing the wrong switches gates a design nobody is cutting --
//! --files never looks at the pitch, so it will not notice.
//!
//! Two lattices, one gate. UNIFORM designs sit on a cubic block; STRETCHED ones
//! run their straights longer than their turns. They were gated by two forked
//! copies of this toolchain until 2026-09-05; this runs both.

/// (name, walk or walks/NAME.txt, folder of cut files beside tools/, switches)
pub type Entry = (&'static str, &'static str, Option<&'static str>, &'static [&'static str]);

pub const UNIFORM: &[Entry] = &[
    ("first trumpet", "N10 U2 W2 S7 U2 E4 N9 W2 D2 N4", Some("../parts/bore/concept/walk/meander/first"), &[]),
    ("spiral trumpet", "U2 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U4", Some("../parts/bore/concept/walk/spiral/expanding"), &[]),
    ("minimal coil", "U2 N1 E1 S1 U1 W1 U1 N1 E1 S1 U1 W1 U1 N1 E1 S1 U1 W1 U2", Some("../parts/bore/concept/walk/coil/minimal"), &[]),
    ("square rise 1", "W1 S2 E2 U1 W1 S2 E2 U1 W1 S2 E2 U1 W1 S2 E2 U1 W1 S2 E2 U1 W1 S2 E2 U4", Some("../parts/bore/concept/walk/coil/square-rise1"), &[]),
    ("telescope", "U2 N1 W1 S2 E2 N3 U1 W2 S3 E3 N4 U1 W4 S5 E5 N6 U1 W6 S7 E7 N8 U1 W8 S9 E9 N10 U2", Some("../parts/bore/concept/walk/spiral/telescope"), &[]),
    ("helix, rise 2", "N4 U2 E4 U2 S4 U2 W4 U2 N4", None, &[]),
    ("helix, rise 1", "N4 U1 E4 U1 S4 U1 W4 U1 N4", None, &[]),
    ("helix, side 6", "N6 U2 E6 U2 S6 U2 W6 U2 N6 U2 E6", None, &[]),
    ("test bore", "U1 E3 S3 U1", None, &[]),
    ("test bore, flat ends", "U1 E3 S3 U1", None, &["--flat"]),
    ("one bend", "D3 E4", None, &[]),
    ("4 corners, flat", "N2 U1 N1 U1 N2", None, &[]),
    ("4 corners, solid", "N2 U1 E1 S1 E4", None, &[]),
    ("5 corners, solid", "N2 U1 E1 S1 W1 S3", None, &[]),
    ("6 corners, solid", "N2 E1 U1 E1 U1 E1 U3", None, &[]),
    ("tightest coil", "U3 N1 E1 S1 U1 W1 U1 N1 E1 S1 U1 W1 U1 N1 E1 S1 U1 W1 U1", None, &[]),
    ("touching coil", "N2 E1 S2 U1 W1 U1 N2 E1 S2 U1 W1 U1 N2 E1 S2 U1 W1 U1", None, &[]),
    ("hilbert cube 1", "S1 U1 N1 E1 S1 D1 N1", None, &[]),
    ("hilbert cube 2", "walks/hilbert_cube.txt", None, &[]),
    ("corner to corner", "walks/corner_to_corner.txt", None, &[]),
    ("double spiral", "N4 W4 S3 E3 N2 W2 U2 E2 N3 W3 S4 E4", None, &[]),
    ("metre spring", "walks/metre_spring.txt", None, &[]),
    ("coil fold2", "walks/coil_fold2.txt", Some("../parts/bore/concept/walk/coil/fold2/bore"), &["--blocksize=16"]),
    ("tight coil", "walks/tight_coil.txt", Some("../parts/bore/concept/walk/coil/tight/bore"), &[]),
    ("ziggurat", "walks/ziggurat.txt", Some("../parts/bore/concept/walk/spiral/ziggurat/bore"), &[]),
    ("hilbert open", "walks/hilbert_open.txt", Some("../parts/bore/concept/walk/hilbert/open"), &[]),
    ("greek spiral", "walks/greek_spiral.txt", Some("../parts/bore/concept/walk/meander/greek-key/bore"), &["--bore=10"]),
    ("coil 2x2 134", "walks/coil-2x2-134.txt", Some("../parts/bore/concept/walk/coil/2x2-134"), &[]),
    ("coil 3x3 54", "walks/coil-3x3-54.txt", Some("../parts/bore/concept/walk/coil/3x3-54"), &[]),
    ("coil 4x4 50", "walks/coil-4x4-50.txt", Some("../parts/bore/concept/walk/coil/4x4-50"), &[]),
    ("coil 5x5 50", "walks/coil-5x5-50.txt", Some("../parts/bore/concept/walk/coil/5x5-50"), &[]),
    ("three-side coil", "walks/coil-three-side.txt", Some("../parts/bore/concept/walk/coil/three-side"), &[]),
    ("closed loop 52, flat, mouth past a plain cap", "walks/closed_loop_52.txt", None, &["--bore=10", "--flat", "--mouth-at=31,34"]),
];

pub const STRETCHED: &[Entry] = &[
    ("coil 10x10x30 0.75t", "walks/coil-0.75t.txt", Some("../parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-0.75t"), &["--bore=10", "--straight=30"]),
    ("coil 10x10x30 1.5t", "walks/coil_fold2.txt", Some("../parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-1.5t"), &["--bore=10", "--straight=30"]),
    ("coil 10x10x30 2.25t", "walks/coil-2.25t.txt", Some("../parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-2.25t"), &["--bore=10", "--straight=30"]),
    ("coil 10x10x30 3t", "walks/coil-3t.txt", Some("../built/coil-fold2-long-straight-3t"), &["--bore=10", "--straight=30"]),
];

pub fn designs() -> Vec<Entry> {
    UNIFORM.iter().chain(STRETCHED).copied().collect()
}

/// tools/, where the walks and the folders are named from
pub fn here() -> String {
    format!("{}/tools", crate::common::repo())
}

/// A design is a walk, or the name of a file holding one.
pub fn walk_of(spec: &str) -> String {
    if spec.ends_with(".txt") {
        let p = format!("{}/{spec}", here());
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")).trim().to_string()
    } else {
        spec.to_string()
    }
}

/// A sibling binary of the one running: target/release/NAME.
pub fn sibling(name: &str) -> String {
    let me = std::env::current_exe().expect("current_exe");
    me.with_file_name(name).to_string_lossy().into_owned()
}

/// Run each job on a pool as wide as the machine; results in input order.
pub fn pool<T: Send, R: Send>(jobs: Vec<T>, f: impl Fn(T) -> R + Sync) -> Vec<R> {
    let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let jobs: std::sync::Mutex<Vec<(usize, T)>> = std::sync::Mutex::new(jobs.into_iter().enumerate().rev().collect());
    let out: std::sync::Mutex<Vec<(usize, R)>> = std::sync::Mutex::new(vec![]);
    std::thread::scope(|s| {
        for _ in 0..n {
            s.spawn(|| loop {
                let Some((i, j)) = jobs.lock().unwrap().pop() else { break };
                let r = f(j);
                out.lock().unwrap().push((i, r));
            });
        }
    });
    let mut out = out.into_inner().unwrap();
    out.sort_by_key(|x| x.0);
    out.into_iter().map(|x| x.1).collect()
}
