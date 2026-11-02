// Expand a period to ~TARGET blocks and write it in the corpus notation: every term
// numbered. The bare lead-in and lead-out terms this used to write are not part of
// the notation any more -- a walk enters facing its first term and leaves facing
// its last -- so the two blocks they stood for are carried by the runs at each end.
//
//   target/release/mknotation "N1 D4 E3 U4"

const TARGET: usize = 196;

fn steps(period: &str) -> Vec<char> {
    let mut out = Vec::new();
    for t in period.split_whitespace() {
        let d = t.chars().next().unwrap();
        let rest = if t.len() > 1 { &t[1..] } else { "1" };
        let l: usize = rest.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
        for _ in 0..l {
            out.push(d);
        }
    }
    out
}

fn build(period: &str) -> String {
    let u = steps(period);
    let mut s: Vec<char> = Vec::new();
    while s.len() < TARGET - 1 && !u.is_empty() {
        s.push(u[s.len() % u.len()]);
    }
    let mut terms: Vec<(char, usize)> = Vec::new();
    if let Some(&first) = s.first() {
        let (mut cur, mut n) = (first, 1);
        for &c in &s[1..] {
            if c == cur {
                n += 1;
            } else {
                terms.push((cur, n));
                cur = c;
                n = 1;
            }
        }
        terms.push((cur, n));
    }
    // The bore's last block must not be a turn: with nothing after it to make it
    // interior it is always its own piece, i.e. stranded. Trim back to a run the
    // lead-out can sit inside. (A walk with no run of 2 anywhere -- the
    // all-stranded coil -- has no such tail; leave it alone rather than trimming it
    // away to nothing.)
    if terms.iter().any(|t| t.1 >= 2) {
        while terms.len() > 1 && terms[terms.len() - 1].1 < 2 {
            terms.pop();
        }
    }
    // You start IN block 1 facing the first term, so the opening run carries one
    // move fewer than the period asks for and block 1 is the lead-in; the closing
    // run's last block is the lead-out, so it gives up a move too.
    if let Some(t) = terms.first_mut() {
        if t.1 > 1 {
            t.1 -= 1;
        }
    }
    if let Some(t) = terms.last_mut() {
        if t.1 > 1 {
            t.1 -= 1;
        }
    }
    terms.iter().map(|(d, l)| format!("{d}{l}")).collect::<Vec<_>>().join(" ")
}

fn main() {
    let Some(period) = std::env::args().nth(1).filter(|p| !p.is_empty()) else {
        eprintln!("usage: target/release/mknotation \"<period>\"   e.g. \"N1 D4 E3 U4\"");
        eprintln!("  Expands a period to about {TARGET} blocks in the corpus notation.");
        std::process::exit(2);
    };
    println!("{}", build(&period));
}
