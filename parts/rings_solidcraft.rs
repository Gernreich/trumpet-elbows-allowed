//! A stack of rings as a SolidCraft design (2026-11-08), shared by the bell
//! and the mouthpiece: each ring a rounded square of half-width h and corner
//! radius c round a rounded-square aperture, one body per ply, stacked up Z.
//! The generators give each ring's four numbers as expressions over their
//! parameters (read-only parameters themselves, plain millimetre numbers).
//!
//! A profile is a sharp square, a circle or a rounded square as it is at the
//! defaults: SolidCraft's rounded rectangle needs a radius strictly between
//! none and half its side, so the two ends of that range are drawn as what
//! they are, and stay that while the numbers move.
#![allow(dead_code)]
use lasermade_tools::solidcraft::Script;

/// A ring's aperture (h, c) and outside (h, c): expression names, and values.
pub struct Ring {
    pub e: [String; 4],
    pub v: [f64; 4],
}

fn profile(s: &mut Script, h: &str, c: &str, hv: f64, cv: f64) {
    let mm = |x: String| format!("({x})*1 mm");
    if cv < 1e-9 {
        s.cmd("sketch.rectangle.two_point", format!("{{\"p0\":[\"{}\",\"{}\"],\"p1\":[\"{}\",\"{}\"]}}", mm(format!("-{h}")), mm(format!("-{h}")), mm(h.into()), mm(h.into())));
    } else if (cv - hv).abs() < 1e-6 {
        s.cmd("sketch.circle.center", format!("{{\"center\":[\"0 mm\",\"0 mm\"],\"diameter\":\"{}\"}}", mm(format!("2*{h}"))));
    } else {
        s.cmd(
            "sketch.rectangle.rounded",
            format!("{{\"p0\":[\"{}\",\"{}\"],\"p1\":[\"{}\",\"{}\"],\"radius\":\"{}\"}}", mm(format!("-{h}")), mm(format!("-{h}")), mm(h.into()), mm(h.into()), mm(c.into())),
        );
    }
}

/// Every ring, `plies` bodies of `ply` each, the first ring's underside at Z = 0.
pub fn stack(s: &mut Script, rings: &[Ring], plies: usize, ply: &str) {
    for (k, r) in rings.iter().enumerate() {
        for j in 0..plies {
            let name = if plies == 1 { format!("ring {}", k + 1) } else { format!("ring {} ply {}", k + 1, j + 1) };
            let sk = s.sketch("XY", Some(&format!("{}*{ply}", k * plies + j)), &format!("{name} sketch"));
            profile(s, &r.e[2], &r.e[3], r.v[2], r.v[3]);
            profile(s, &r.e[0], &r.e[1], r.v[0], r.v[1]);
            s.finish();
            s.extrude(&sk, ply, false, "new", &[], &name);
        }
    }
    s.measure();
}

/// Each body's volume, as the stack should build it.
pub fn volumes(rings: &[Ring], plies: usize, ply: f64) -> Vec<f64> {
    let a = |h: f64, c: f64| 4.0 * h * h - (4.0 - std::f64::consts::PI) * c * c;
    rings.iter().flat_map(|r| std::iter::repeat((a(r.v[2], r.v[3]) - a(r.v[0], r.v[1])) * ply).take(plies)).collect()
}
