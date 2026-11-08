//! `hilbert [ORDER] [--split] [--scale N]`: write a Hilbert cube as a walk in
//! the bore notation. hilbert.py until 2026-11-08.
//!
//! A Hilbert curve visits every cell of a 2^n cube exactly once, in unit steps,
//! which makes it the densest bore the notation can describe. It is also the
//! most expensive to cut: a space-filling curve turns almost every block, and an
//! order-2 cube comes out as 63 sections for 64 blocks.
use lasermade_tools::py;
use trumpet::bore::{touching, walk, Design};

/// Skilling's transform, the inverse of the Hilbert index.
fn transpose_to_axes(mut x: [i64; 3], bits: i64) -> [i64; 3] {
    let n = 2i64 << (bits - 1);
    let mut t = x[2] >> 1;
    for i in (1..=2).rev() {
        x[i] ^= x[i - 1];
    }
    x[0] ^= t;
    let mut q = 2;
    while q != n {
        let p = q - 1;
        for i in (0..=2).rev() {
            if x[i] & q != 0 {
                x[0] ^= p;
            } else {
                t = (x[0] ^ x[i]) & p;
                x[0] ^= t;
                x[i] ^= t;
            }
        }
        q <<= 1;
    }
    x
}

fn name(v: [i64; 3]) -> char {
    match v {
        [1, 0, 0] => 'E',
        [-1, 0, 0] => 'W',
        [0, 1, 0] => 'U',
        [0, -1, 0] => 'D',
        [0, 0, 1] => 'S',
        _ => 'N',
    }
}

/// The cells of a 3D Hilbert curve of this order, in order.
fn cube(order: i64) -> Result<Vec<[i64; 3]>, String> {
    if order < 1 {
        return Err("negative shift count".into());
    }
    let mut pts = vec![];
    for h in 0..(1i64 << (3 * order)) {
        let mut x = [0i64; 3];
        for i in 0..3 * order {
            let b = (h >> (3 * order - 1 - i)) & 1;
            x[(i % 3) as usize] |= b << (order - 1 - i / 3);
        }
        pts.push(transpose_to_axes(x, order));
    }
    let n = 1 << order;
    let set: std::collections::HashSet<[i64; 3]> = pts.iter().copied().collect();
    if set.len() != pts.len() {
        return Err("the curve visits a cell twice".into());
    }
    if pts.iter().flatten().any(|&c| !(0 <= c && c < n)) {
        return Err(format!("the curve leaves the {n}x{n}x{n} cube"));
    }
    for w in pts.windows(2) {
        if (0..3).map(|k| (w[1][k] - w[0][k]).abs()).sum::<i64>() != 1 {
            return Err(format!("the curve jumps from {} to {}", trumpet::bore::tuple3(w[0]), trumpet::bore::tuple3(w[1])));
        }
    }
    Ok(pts)
}

/// Cells to notation: run-length encode the steps.
fn to_walk(pts: &[[i64; 3]], scale: i64) -> String {
    let steps: Vec<char> = pts.windows(2).map(|w| name([w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]])).flat_map(|d| (0..scale).map(move |_| d)).collect();
    let mut runs: Vec<(char, i64)> = vec![];
    let (mut cur, mut n) = (steps[0], 0);
    for &s in &steps {
        if s == cur {
            n += 1;
        } else {
            runs.push((cur, n));
            (cur, n) = (s, 1);
        }
    }
    runs.push((cur, n));
    runs.iter().map(|(d, k)| format!("{d}{k}")).collect::<Vec<_>>().join(" ")
}

fn run(order: i64, split: bool, scale: i64, o: &mut String) -> Result<(), String> {
    let d = Design::default();
    let pts = cube(order)?;
    let text = to_walk(&pts, scale);
    let rec = walk(&text)?;
    let n = ((1 << order) - 1) * scale + 1;
    let vol = py::powf(n as f64 * d.block, 3.0) / 1e6;
    o.push_str(&format!("{text}\n"));
    o.push_str(&format!("\n  order {order} at scale {scale}: {} blocks, {} mm of centreline in a {} mm cube\n", rec.len(), py::g(rec.len() as f64 * d.block), py::g(n as f64 * d.block)));
    o.push_str(&format!("  {:.0} mm of bore per litre, {} pairs of blocks touching\n", rec.len() as f64 * d.block / vol, touching(&d, &rec).len()));
    if split {
        d.main(&text, None, o)?;
    }
    Ok(())
}

fn main() {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    let mut scale = 1;
    if let Some(i) = argv.iter().position(|a| a == "--scale") {
        let v = argv.get(i + 1).cloned().unwrap_or_else(|| trumpet::bore::sys_exit("IndexError: list index out of range"));
        scale = trumpet::bore::pyint(&v).unwrap_or_else(|| trumpet::bore::sys_exit(&format!("ValueError: invalid literal for int() with base 10: {}", py::str_repr(&v))));
        argv.drain(i..i + 2);
    }
    let split = argv.iter().any(|a| a == "--split");
    let a: Vec<&String> = argv.iter().filter(|x| *x != "--split").collect();
    let mut o = String::new();
    let r = match a.first() {
        None => run(2, split, scale, &mut o),
        Some(x) => match trumpet::bore::pyint(x) {
            Some(k) => run(k, split, scale, &mut o),
            None => Err(format!("invalid literal for int() with base 10: {}", py::str_repr(x))),
        },
    };
    print!("{o}");
    if let Err(e) = r {
        println!("error: {e}");
        std::process::exit(1);
    }
}
