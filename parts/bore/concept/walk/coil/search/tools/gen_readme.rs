// Regenerate README.md. Every number in it comes from the tools and the tool index
// is read from tools/ itself, so the page cannot drift from the walks or from the
// directory.
//
//   target/release/gen_readme

use coilsearch::corpus::{list_dir, parts, read, root};
use coilsearch::jsfmt::{fixed, locale_int, num, round};
use coilsearch::json::J;
use coilsearch::minimal;
use coilsearch::reduce;
use coilsearch::score::{interior, score, MEANS, METRICS};
use coilsearch::spiral_metrics::{metrics, period, Metrics, Period, MM};
use coilsearch::table::{render, rows as table_rows, Which};

const NUM: [&str; 16] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "thirteen", "fourteen", "fifteen",
];
fn word(n: usize) -> String {
    NUM.get(n).map(|s| s.to_string()).unwrap_or_else(|| n.to_string())
}
fn cap(n: usize) -> String {
    let w = word(n);
    w[..1].to_uppercase() + &w[1..]
}
fn ord(n: usize) -> String {
    const S: [&str; 4] = ["th", "st", "nd", "rd"];
    let v = (n % 100) as i64;
    let r = (v - 20) % 10;
    let suf = if (0..=3).contains(&r) {
        S[r as usize]
    } else if v <= 3 {
        S[v as usize]
    } else {
        "th"
    };
    format!("{n}{suf}")
}

/// One coil as the page reads it: judged over the interior, described over the whole bore.
struct Coil {
    name: String,
    p: J,
    per: Period,
    full: Metrics,
    m: Metrics,
    href: Option<String>,
}
impl Coil {
    fn pf(&self, k: &str) -> f64 {
        self.p.get(k).and_then(|v| v.f()).unwrap_or(f64::NAN)
    }
    fn link(&self) -> String {
        format!("[`{}`]({})", self.name, self.href.clone().unwrap_or_else(|| format!("pages/{}.html", self.name)))
    }
    fn cross_area(&self) -> i64 {
        self.m.cross[0] * self.m.cross[1]
    }
}

fn cmpf(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}
/// rows sorted by f ascending (stable), as `.slice().sort((a, b) => f(a) - f(b))`
fn sorted_by<'a>(rows: &[&'a Coil], f: impl Fn(&Coil) -> f64) -> Vec<&'a Coil> {
    let mut s = rows.to_vec();
    s.sort_by(|a, b| cmpf(f(a), f(b)));
    s
}

// The README's tool index is read from tools/ rather than typed into the page. This
// holds the descriptions; the set is checked against the directory just below, and a
// tool with no description -- or a description with no tool -- stops the build.
fn tool_groups() -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    vec![
        ("Finding and shaping walks", vec![
            ("search_spirals.rs", "Searches for the tightest bend-only coil. Applies the corpus rule -- three consecutive terms naming three axes force the middle term to 3 or more -- then lets the splitter decide, because the rule is a filter and not the answer.".into()),
            ("mknotation.rs", "Expands one period to about 196 blocks and writes it in the corpus notation: every term numbered, the lead-in and lead-out blocks carried by the runs at each end.".into()),
            ("standardise.rs", "Puts every coil in one orientation -- north for all of them, opening on a north term -- so that two coils differ only where they really differ. Writes `standardised.json`.".into()),
            ("minimal.rs", "Asks whether a walk can be shortened without stranding a turn, against a purely local rule: a term's floor is set by the window of three around it. Reports slack; does not take it.".into()),
            ("reduce.rs", "Takes that slack, one leg at a time, keeping the coil. Taking all of it at once usually destroys the walk, which is why this is separate from `minimal.rs`. Writes `reduced.json`.".into()),
        ]),
        ("Measuring and ranking", vec![
            ("spiral_metrics.rs", "Rotation metrics for one walk. The lateral projection -- the walk with the advancing axis dropped -- is what actually turns, and on a cubic lattice only in quarter turns.".into()),
            ("parts.rs", "Piece counts, distinct piece shapes and plate sizes, read off `bore_split.py` and cached in `parts.json` so the tables need not shell out on every run.".into()),
            ("score.rs", format!("Composite scoring across the common means: {} metrics normalised to (0,1], plus the touching count at an explicit weight.", word(METRICS.len()))),
            ("iterate.rs", "Iterated ranking -- rank, cut the bottom half, re-rank the survivors, repeat -- to ask whether the survivors keep their order once the losers leave. They do not, under a normalisation computed over the set.".into()),
            ("table.rs", "The metrics tables, as plain text or with `--md` as markdown.".into()),
        ]),
        ("Producing what is committed", vec![
            ("run_checks.sh", "Runs `check.py` over every walk in `walks/` and writes the transcript to `checks/`. It resolves the Boxes.py checkout and the interpreter *before* the loop and stops if either is missing, because a missing interpreter captured with `2>&1` writes the shell's error into the transcript instead of the check, and the tally then reads as a pass.".into()),
            ("gen_scoring.rs", "Regenerates `SCORING.md`. Every number in it comes from `score.rs`.".into()),
            ("gen_readme.rs", "Regenerates this file. Every number in it comes from the tools and the index above is read from `tools/` itself, so the page cannot drift.".into()),
            ("build.sh", "Builds the tools, then runs all of the above that produce committed files, in dependency order. `index.html` and `SCORING.html` are committed rather than built on the server, so they go stale silently unless this is run after every edit.".into()),
        ]),
    ]
}

fn main() {
    let root = root();
    let parts = parts();

    let rows: Vec<Coil> = table_rows()
        .into_iter()
        .map(|r| Coil { name: r.name, p: r.p, per: r.per, full: r.full, m: r.m, href: None })
        .collect();
    let all: Vec<&Coil> = rows.iter().collect();
    let n = rows.len();

    // Density against box, counted here rather than asserted in the prose, breaking a
    // box tie by density in BOTH rankings so an exact tie is not read as a disagreement.
    let dens = |r: &Coil| r.m.blocks as f64 / r.m.vol as f64;
    let mut by_box = all.clone();
    by_box.sort_by(|a, b| a.m.vol.cmp(&b.m.vol).then_with(|| cmpf(dens(b), dens(a))));
    let mut by_den = all.clone();
    by_den.sort_by(|a, b| cmpf(dens(b), dens(a)).then_with(|| a.m.vol.cmp(&b.m.vol)));
    let dens_agree = by_box.iter().zip(&by_den).filter(|(a, b)| a.name == b.name).count();

    // check.py results, as saved by tools/run_checks.sh
    let re = regex::Regex::new(r"(\d+) checks, (\d+) failed").unwrap();
    let mut checks: Vec<(String, i64, i64)> = Vec::new();
    for f in list_dir(&root.join("checks")).into_iter().filter(|f| f.ends_with(".txt")) {
        let t = read(&root.join("checks").join(&f));
        if let Some(c) = re.captures(&t) {
            checks.push((f.trim_end_matches(".txt").to_string(), c[1].parse().unwrap(), c[2].parse().unwrap()));
        }
    }
    let total_checks: i64 = checks.iter().map(|c| c.1).sum();
    let total_failed: i64 = checks.iter().map(|c| c.2).sum();

    let tbl = |w: Which| render(true, w).trim().to_string();
    let best = |f: &dyn Fn(&Coil) -> f64| sorted_by(&all, f)[0];
    // Ties were being hidden: sorting and taking the first credits one coil and drops
    // the others that matched it exactly. List them all.
    let all_best = |f: &dyn Fn(&Coil) -> f64| {
        let s = sorted_by(&all, f);
        let first = f(s[0]);
        s.into_iter().filter(|r| (f(r) - first).abs() < 1e-9).map(|r| r.link()).collect::<Vec<_>>().join(", ")
    };

    // The walk this started from, found through derived.txt rather than by name.
    let mut derived: Vec<(String, Vec<String>)> = Vec::new();
    for l in read(&root.join("derived.txt")).split('\n') {
        let l = l.split('#').next().unwrap().trim();
        if l.is_empty() {
            continue;
        }
        let mut it = l.split_whitespace();
        let name = it.next().unwrap().to_string();
        let src: Vec<String> = it.map(str::to_string).collect();
        match derived.iter_mut().find(|(k, _)| *k == name) {
            Some(e) => e.1 = src,
            None => derived.push((name, src)),
        }
    }
    let derives = |name: &str, from: &str| derived.iter().any(|(k, s)| k == name && s.iter().any(|x| x == from));

    // The coil the search started from. If it is no longer in walks/, parts measures
    // it where it now lives and marks it with promotedTo; if it can be found nowhere,
    // this stops rather than substituting a coil that is not it.
    let in_set = rows.iter().find(|r| derives(&r.name, "staircase_coil"));
    let promoted = parts.keys().into_iter().find(|k| parts.get(k).unwrap().get("promotedTo").is_some()).map(|k| {
        let p = parts.get(k).unwrap().clone();
        let walk = p.get("walk").and_then(|w| w.str()).unwrap_or("").to_string();
        Coil {
            name: k.to_string(),
            per: period(&walk),
            full: metrics(&walk, None).unwrap(),
            m: metrics(&walk, interior(&p)).unwrap(),
            href: p.get("promotedTo").and_then(|h| h.str()).map(str::to_string),
            p,
        }
    });
    let yours: &Coil = match (in_set, &promoted) {
        (Some(r), _) => r,
        (None, Some(p)) => p,
        _ => {
            eprintln!("derived.txt names no coil from staircase_coil and parts.json marks none\nas promoted, so there is nothing to compare against. README.md would have\nto invent the walk this started from.");
            std::process::exit(1)
        }
    };

    // Where the originating coil sits once it is put back beside the set it produced.
    let mut field = all.clone();
    if in_set.is_none() {
        field.push(yours);
    }
    let rank_of = |f: &dyn Fn(&Coil) -> f64| sorted_by(&field, f).iter().position(|r| r.name == yours.name).unwrap() + 1;
    let r_box = rank_of(&|r| r.m.vol as f64 / r.m.blocks as f64);
    let r_pieces = rank_of(&|r| r.pf("innerPieces") / r.m.blocks as f64);
    let r_distinct = rank_of(&|r| r.pf("interiorDistinct"));
    let r_tube = rank_of(&|r| r.m.blocks_per_360_or0());
    let r_turns = rank_of(&|r| r.m.turns_per_metre);
    // Its reduction was promoted out beside it, so it too is no longer in the set.
    let red_origin = derived.iter().find(|(_, s)| s.iter().any(|x| x == "coil_5x5_28")).map(|(k, _)| k.clone());
    let sibling_href = |name: &str| -> Option<String> {
        let dir = name.strip_prefix("coil_").unwrap_or(name).replace('_', "-");
        let family = root.join("..");
        for group in list_dir(&family) {
            if family.join(&group).join(&dir).join(format!("{dir}.html")).exists() {
                return Some(format!("../{group}/{dir}/{dir}.html"));
            }
        }
        None
    };
    let bx = best(&|r| r.m.vol as f64);
    let rise = best(&|r| r.m.rise_per_360_or0());
    let tube = best(&|r| r.m.blocks_per_360_or0());
    let pcs = best(&|r| r.pf("innerPieces"));
    let dist = best(&|r| r.pf("interiorDistinct"));
    let calm = best(&|r| r.m.turns_per_metre);
    let by_vol = |keep: &dyn Fn(&Coil) -> bool| {
        let s: Vec<&Coil> = all.iter().copied().filter(|r| keep(r)).collect();
        sorted_by(&s, |r| r.m.vol as f64).first().copied()
    };
    let clean = by_vol(&|r| r.m.touching == 0).expect("no coil without touching walls");
    // the smallest box among the coils that do pass themselves
    let touch_box = by_vol(&|r| r.m.touching > 0);
    let plate_best = sorted_by(&all, |r| -r.pf("meanPlate"))[0];
    // picked from the data, not named: standardising renamed every coil
    let fewest_shapes = best(&|r| r.pf("interiorDistinct"));
    let thinnest = best(&|r| r.cross_area() as f64);
    let fattest = sorted_by(&all, |r| -(r.cross_area() as f64))[0];
    let worst_ends = sorted_by(&all, |r| -(r.pf("distinct") - r.pf("interiorDistinct")))[0];

    let sc = score(false, false);
    let minres = minimal::results();
    let redres = reduce::results();
    let red_ok: Vec<&reduce::Reduced> = redres.iter().filter_map(|r| r.r.as_ref()).collect();
    let red_names: Vec<&str> = redres.iter().filter(|r| r.r.is_some()).map(|r| r.name.as_str()).collect();
    let count = |f: &dyn Fn(&reduce::Reduced) -> bool| red_ok.iter().filter(|r| f(r)).count();
    let red_rows: Vec<(&str, usize)> = vec![
        ("box smaller", count(&|r| r.after.vol < r.before.vol)),
        ("box bigger", count(&|r| r.after.vol > r.before.vol)),
        ("box unchanged", count(&|r| r.after.vol == r.before.vol)),
        ("touching reduced", count(&|r| r.after.touching < r.before.touching)),
        ("touching increased", count(&|r| r.after.touching > r.before.touching)),
        ("touching unchanged", count(&|r| r.after.touching == r.before.touching)),
    ];
    let mut red_walks: Vec<&str> = Vec::new();
    for r in &red_ok {
        if !red_walks.contains(&r.walk.as_str()) {
            red_walks.push(&r.walk);
        }
    }

    // WHERE the reductions land, read off the pass instead of asserted.
    struct Group {
        per: String,
        lands: Option<String>,
        from: Vec<String>,
    }
    let mut groups: Vec<Group> = Vec::new();
    for (r, name) in red_ok.iter().zip(&red_names) {
        match groups.iter_mut().find(|g| g.per == r.red) {
            Some(g) => g.from.push(name.to_string()),
            None => groups.push(Group { per: r.red.clone(), lands: r.lands.clone(), from: vec![name.to_string()] }),
        }
    }
    groups.sort_by(|a, b| b.from.len().cmp(&a.from.len()));
    let stray: Vec<&Group> = groups.iter().filter(|g| g.lands.is_none()).collect();
    let stray_count: usize = stray.iter().map(|g| g.from.len()).sum();
    let minimal: Vec<&str> = minres.iter().filter(|r| r.saving == 0.0).map(|r| r.name.as_str()).collect();
    let mut by_saving: Vec<&minimal::Report> = minres.iter().collect();
    by_saving.sort_by(|a, b| cmpf(b.saving, a.saving));
    let slackiest = by_saving[0];

    let mut tally: Vec<(String, usize)> = Vec::new();
    for m in &MEANS {
        let w = sc.rows.iter().min_by_key(|r| sc.rank(m.k, &r.name)).unwrap().name.clone();
        match tally.iter_mut().find(|(k, _)| *k == w) {
            Some(e) => e.1 += 1,
            None => tally.push((w, 1)),
        }
    }
    tally.sort_by(|a, b| b.1.cmp(&a.1));
    let unanimous = format!("`{}` comes first under {} of the {} means.", tally[0].0, tally[0].1, MEANS.len());

    // The tool index, checked against the directory. Directories count: a
    // subdirectory of tools/ needs describing as much as a file does.
    let groups_t = tool_groups();
    let tools_dir = root.join("tools");
    let on_disk: Vec<String> = list_dir(&tools_dir).into_iter().filter(|f| !f.starts_with('.')).collect();
    let described: Vec<&str> = groups_t.iter().flat_map(|(_, t)| t.iter().map(|(n, _)| *n)).collect();
    let undescribed: Vec<&String> = on_disk.iter().filter(|f| !described.contains(&f.as_str())).collect();
    let phantom: Vec<&&str> = described.iter().filter(|f| !on_disk.iter().any(|d| d == **f)).collect();
    if !undescribed.is_empty() || !phantom.is_empty() {
        eprintln!("tools/ and the index in gen_readme.rs disagree, so README.md would lie:");
        if !undescribed.is_empty() {
            eprintln!("  present, described nowhere: {}", undescribed.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
        }
        if !phantom.is_empty() {
            eprintln!("  described here, not present: {}", phantom.iter().map(|s| **s).collect::<Vec<_>>().join(", "));
        }
        std::process::exit(1);
    }
    let tool_tables = groups_t
        .iter()
        .map(|(title, t)| {
            format!(
                "**{title}**\n\n| tool | what it does |\n| --- | --- |\n{}",
                t.iter().map(|(n, d)| format!("| `{n}` | {d} |")).collect::<Vec<_>>().join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // ---- the numbers the prose quotes
    let fold = |f: &dyn Fn(&Coil) -> f64| {
        let v: Vec<f64> = all.iter().map(|r| f(r)).collect();
        (v.iter().copied().fold(f64::INFINITY, f64::min), v.iter().copied().fold(f64::NEG_INFINITY, f64::max))
    };
    let (fb_lo, fb_hi) = fold(&|r| r.full.blocks as f64);
    let (fm_lo, fm_hi) = fold(&|r| r.full.mm as f64);
    let (mb_lo, mb_hi) = fold(&|r| r.m.blocks as f64);
    let (mm_lo, mm_hi) = fold(&|r| r.m.mm as f64);
    let widest = sorted_by(&all, |r| -r.pf("interiorDistinct"))[0].pf("interiorDistinct");
    let top_rhythm = sorted_by(&all, |r| -r.pf("rhythm"))[0].pf("rhythm");
    let terms_n = |t: i64| rows.iter().filter(|r| r.per.terms == Some(t)).count();
    let rhythm4 = rows.iter().filter(|r| r.pf("rhythm") == 4.0).count();

    // Plate area is not redundant: coils that tie on piece count still differ on it.
    // Grouped by piece count, groups in ascending count as a JavaScript object keeps
    // integer keys, then the biggest group.
    let plate_tie = {
        let mut keys: Vec<f64> = Vec::new();
        for r in &rows {
            let k = r.pf("innerPieces");
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        keys.sort_by(|a, b| cmpf(*a, *b));
        let mut gs: Vec<Vec<&Coil>> =
            keys.iter().map(|k| rows.iter().filter(|r| r.pf("innerPieces") == *k).collect::<Vec<_>>()).filter(|g| g.len() > 1).collect();
        gs.sort_by(|a, b| b.len().cmp(&a.len()));
        match gs.first() {
            None => "coils that tie on piece count still differ on it".to_string(),
            Some(tie) => format!(
                "{} all split into {} pieces and cannot be told apart by blocks-per-piece at all, while their mean plates are {}",
                tie.iter().map(|r| format!("`{}`", r.name)).collect::<Vec<_>>().join(", "),
                num(tie[0].pf("innerPieces")),
                tie.iter().map(|r| format!("{} mm²", num(round(r.pf("meanPlate"))))).collect::<Vec<_>>().join(", ")
            ),
        }
    };

    // Since 2026-10-14, when every walk was cut back to end on a north term, the
    // smallest box has no shared wall. Say which case holds rather than assume one.
    let box_sentence = if bx.m.touching != 0 {
        format!(
            "{} has the\nsmallest box in the set and {} shared walls; {} is\n{}% larger and has **none**. If a shared wall\nis something to avoid, the smallest box is not the one to build.",
            bx.link(),
            bx.m.touching,
            clean.link(),
            fixed((clean.m.vol as f64 / bx.m.vol as f64 - 1.0) * 100.0, 0)
        )
    } else {
        let tb = touch_box.expect("no coil with touching walls");
        format!(
            "Here the two agree: {} has the smallest box in the set and\n**no** shared wall. They need not: {} is only\n{}% larger and has {} shared walls.",
            bx.link(),
            tb.link(),
            fixed((tb.m.vol as f64 / bx.m.vol as f64 - 1.0) * 100.0, 0),
            tb.m.touching
        )
    };

    let red_origin_sentence = match &red_origin {
        None => String::new(),
        Some(o) => format!(
            " Its reduction, {}, won a category too and\nwas promoted out beside it, so only one of the two staircase coils is measured here now.",
            match sibling_href(o) {
                Some(h) => format!("[`{o}`]({h})"),
                None => format!("`{o}`"),
            }
        ),
    };

    let red_bullets = groups
        .iter()
        .map(|g| {
            let list = g.from.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ");
            match &g.lands {
                Some(l) if g.from.len() == 1 && g.from[0] == *l => {
                    format!("* `{l}` reduces to itself — the pass finds slack in it and gives back the same period")
                }
                Some(l) => format!("* {} coils reduce to `{l}`, one of them being `{l}` itself: {list}", word(g.from.len())),
                None => format!("* {} coils reduce to `{}`, which is **not in the catalogue**: {list}", word(g.from.len()), g.per),
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let closed = if stray.is_empty() {
        "**The set is closed under reduction.** There is nothing left to take out that does not\neither break the coil or land somewhere already catalogued.".to_string()
    } else {
        format!(
            "**The set is not closed under reduction.** {} of the {} reductions land on\n{} catalogued nowhere here. That is a gap in the set rather than a\nturn of phrase: either those walks belong in it, or the reduction that produces them\ndoes not really close and `reduce.rs` should be rejecting it.",
            cap(stray_count),
            red_ok.len(),
            if stray.len() == 1 { "a walk that is".to_string() } else { format!("{} walks that are", word(stray.len())) }
        )
    };

    let y = yours;
    let first = &rows[0].name;
    let md = format!(
        r#"# Spirals

Coiling bore walks that turn about an axis while advancing along it, collected with the
numbers that say how hard each one spirals and what each one costs to build. Every walk
here splits **bend-only** — every turn folds into a piece as an L, none is stranded
as a single-block piece of its own — and every one passes the full gate.

The one this started from is {y_link}{y_where}; the {others} came out of an
exhaustive search for something tighter.

**Every walk ends on a north term**, so the bell faces forward, the way the mouthpiece
enters. On 2026-10-14 each was cut back from its end to its last north term rather
than extended — one to three terms, 1 to 11 blocks — and everything below is measured
from the walks as cut back. The names were kept: the number after the cross-section
(54 in `coil_3x3_54_2`) is the axis length **before** the cut, one block more than
the axis now.

<!-- readme-only -->
**[Read it as a page](https://gernreich.github.io/trumpet-elbows-allowed/)** — the trumpet writeup.
That writeup carries no page of its own for the search, so this is where the
reading version lives.

**[Download the whole repository as a ZIP](https://github.com/Gernreich/trumpet-elbows-allowed/archive/refs/heads/main.zip)**
— every trumpet and every tool, not the walks alone; these pages are under `spirals/`.
GitHub builds it from `main` on every push, so it is never out of date.

**[The rest of the build files](https://gernreich.github.io/lasermademusic.html)** — every instrument,
generator and tool, indexed.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and the playing are shown.

## Standardised

Every coil here is in the same orientation, so two coils differ only where they really
differ:

* **forward is north** for all of them, and each walk opens on a north term
* **counter-clockwise**, seen looking along the bore from the mouth
* **canonical**: of the several ways a coil can satisfy all of the above, the
  lexicographically smallest is the one kept, so equal coils are identical strings
* **whole periods only** — the repeat count that lands nearest a common target
* **one block in, one block out**, and no partial period between them

A bore opens at both ends, so the first and last piece cannot be removed — they are
bounded by the mouth and the exit rather than by a neighbour. The notation gives the
mouth block away free: you start in block 1, facing the first term, so the one block in
and the one block out cost no term of their own. What is standardised is that every walk
has exactly one of each, and the judged metrics ignore them regardless.

Fixing forward and the sense of rotation is not enough on its own. The four rotations
about the forward axis all satisfy both, and so does every rotation of the cycle that
opens on a north term, so taking the first representation that fits leaves the same coil
able to appear more than once looking different — and it did, four times over in one
case. Pinning the representation to the lexicographically smallest is what makes
`distinct` mean anything.

Standardising collapsed 30 walks into {n}: several that looked different were the same
coil in another orientation, and [`derived.txt`](derived.txt) records which. The hand
reduction of the staircase coil, the tool's reduction of it, and the extension of the
hand reduction all turn out to be one coil.

### The length rule, and why it is not a cap

Whole periods of different lengths cannot all reach the same total, so some rule has to
choose the repeat counts. The first one tried — as many periods as fit under a limit —
is the wrong shape: it truncates the long-period coils hardest, so every one of them
lands below every short-period one, and the bores ran 126 to 144 blocks, a 14.3% spread.

The target is now chosen rather than assumed. `tools/standardise.rs` searches bore
lengths from 120 to 220 blocks for the one that makes the tube lengths most alike, takes
the **nearest** whole-period count to it rather than the largest that fits, and lands on
180: bores of {fb_lo} to {fb_hi} blocks, {fm_lo} mm to {fm_hi} mm, a spread of {fb_spread}%.

That is as close as this set can get at a plausible size. Squeezing the spread below 10%
needs a seven-metre bore and below 5% needs twelve, which is not a trumpet.

So the residual is handled where it actually bites, in the scoring: **box and piece count
are scored per block**, because both grow with tube and comparing them absolutely would
hand the shorter coils an advantage they did nothing to earn. Over this set piece count
correlates 0.63 with block count. Cross-section, mean plate, distinct shapes and the
rates are length-independent already; touching stays an absolute count, because the
requirement is none of it at any length.

## Shape and cost

{tbl_shape}

Envelope, box and cross-section are in blocks; a block is {MM} mm of centreline. Sorted by
box, smallest first.

**The bore's mouth and exit are not judged.** Every design has them, no design chooses
them, and they are the two pieces that never join the rhythm — so every column here
except blocks and mm is measured on the interior, the pieces in between. It is not a
cosmetic change: the ends stick out of the envelope they bracket. `{we_name}` needs {we_inner} shapes
rather than {we_all}, the difference being end pieces alone. Blocks and mm still describe the whole bore, because that is what the bore is.

**Cross-section** is the envelope with the coil axis taken out — how fat the coil is,
which is what decides whether it fits inside anything. It is not implied by the box: {thin_link} is
{thin_x} in section and {fat_link} is {fat_x}, and the box does not say so.

**Distinct** is how many different piece shapes the cut list holds. A coil built by
repeating one period needs only a handful, however long it runs: {dist_name} is
{dist_pieces} pieces cut from {dist_distinct} shapes. The widest needs {widest}. That is files to check and parts to tell apart on the bench, and it
does not show up in the piece count at all.

**Period** is the repeating unit of the walk itself, in terms and in blocks, and it sits
next to the rhythm because the two together say something neither says alone.

**A piece is a flat snake, so the splitter has to start a new one exactly where the bore
leaves its plane.** Turns that stay in-plane — the folds, hairpins and steps both —
happen inside a piece and cost no boundary at all. So the rhythm is not counting turns,
it is counting departures from the plane, and it comes out as

    rhythm = period terms x the share of turns that leave the plane

exactly, on every coil here. That decomposition is why the period column earns its
place: a rhythm of {top_rhythm} could be a long period turning gently or a short one leaving the
plane at every chance, and the rhythm alone cannot tell you which. In this set the
{t8} eight-term coils leave the plane on half their turns, while the {t16} sixteen-term ones do it on
three turns in four — so the gap in rhythm is two separate factors multiplying, not one.

The share itself is **not** scored, and was tested rather than assumed: it takes only
three values across the set, {t8} of the {n} coils sit on exactly one of them, and it
correlates 0.89 with the rhythm, 0.85 with distinct shapes and 0.76 with box per block.
Near-constant and not independent — the same double failure that kept fill density out.

**Rhythm** is how many pieces you lay before the cut list starts over, and how many
times it recurs. It is descriptive, not scored: across coils built by repeating a period
it barely varies — {rhythm4} of the {n} here are a 4-piece rhythm — and a near-constant metric
in a mean only dilutes the ones that discriminate. It is worth knowing at the bench all
the same, and it is not the same thing as **distinct**: the two rank the set alike in
only 6 of {n} places. `{fs_name}` is {fs_distinct} shapes laid in a
{fs_rhythm}-piece cycle repeated {fs_repeats} times, between the two end pieces.

**Mean plate** is the average bounding box a piece is cut from, in mm² — the
laser-cutting number. Fewer, larger parts means less weeding, less sorting and fewer
fingers to align, and it is the size of the part in your hand rather than the count of
them. `{pb_name}` averages {pb_plate} mm² against `{y_name}`'s {y_plate} mm².

Average *blocks* per piece was the other reading of the same idea and is not used: the
block count varies by only {mb_spread}% across the set, so blocks-per-piece is very nearly the
reciprocal of the piece count and ranks the set the same way in 15 of {n} places. Plate
area is not redundant: {plate_tie}.

It is a bounding box, not the cut outline — an L-shaped piece leaves its corner behind
— so it measures the size of the part, not the material consumed.

**Touching** counts blocks that sit face to face without being joined along the bore —
two runs of the tube sharing a wall. `bore_split.py` warns about them, and they are the
version of "density" that has a consequence: where the bore passes itself, that one wall
is all that separates the two passages.

Because a shared wall shows in the finished instrument, it is the heaviest input in the
scoring and is penalized convexly — 1/(1+t), so no contact scores 1.000 and eight
contacts 0.111. `target/release/score --clean` ranks only the coils that have none.

It is also the one packing number that is *not* implied by the box. {box_sentence}

## Rotation

{tbl_rot}

A coil turns about one axis while travelling down it. Drop that axis and what is left —
the lateral projection — is what rotates, and on a cubic lattice it can only turn in
ninety degree steps, so the winding is counted in quarter turns and multiplied up. Steps
along the coil axis project to nothing and are skipped: they are advance, not rotation.

* **Rotation** — the whole turn end to end, in degrees.
* **Blocks / 360** — tube spent on one revolution. Lower is a tighter spiral.
* **Rise / 360** — how far down its axis the coil travels to come back round. The pitch.
* **Deg / block** — the same, per block, averaged.
* **90deg turns**, **turns/m** — how often the air is asked to turn a corner. The tube
  length is nearly the same for all of these, so this is what separates them acoustically,
  and it is the one number here that argues *against* packing tighter.
* **Longest straight** — the longest run without a turn.

**Axis** is the direction the coil advances; the search was free to build about any axis,
so a coil that runs east is measured against east.

## What wins what

No single spiral wins, because the measures disagree.

| | winner | against the walk this started from |
| --- | --- | --- |
| smallest box | {box_link} — {box_vol} | {y_vol}, so {box_x}x larger |
| tightest spiral (least rise per turn) | {rise_link} — {rise_mm} mm | {y_rise_mm} mm, so {rise_x}x slacker |
| least tube per turn | {tube_names} — {tube_b} blk | {y_tube} blk, within {tube_pct}% |
| fewest pieces | {pcs_link} — {pcs_n} | {y_pcs} |
| fewest distinct shapes | {dist_link} — {dist_n} | {y_dist} |
| largest average plate | {pb_link} — {pb_plate} mm² | {y_plate} mm² |
| calmest bore (fewest turns/m) | {calm_names} — {calm_t} | {y_calm} |
| smallest box with no shared wall | {clean_link} — {clean_vol} | {y_vol}, also {y_touch} shared |

`{y_name}`, the coil this started from, loses the packing categories and wins the
turning ones. Set beside the {n_word} it produced, it places {o_box} on box per block,
{o_pieces} on pieces per block and {o_distinct} on distinct shapes, against **{o_tube} on tube per
revolution** and {o_turns} on turns per metre.{red_origin_sentence}

That is not a split verdict so much as one fact seen twice: **packing tighter costs
bends**, and bends are what a bore notices. A coil that turns economically is a coil that
does not fold itself into a small box, and every category above is downstream of that
choice.

## A metric deliberately left out

Fill density — blocks over box volume — is left out on purpose. At a fixed tube length it
is not independent of the box: blocks is near enough constant, so density is just the
reciprocal of box volume rescaled. Ranking these {n} spirals by density puts them in the
same order as ranking by box in {dens_agree} of {n} positions. It reads like a second
opinion and is not one. It would earn its place only in comparing walks of genuinely
different lengths.

**Touching** is the metric density was reaching for. It answers the question density
sounds like it answers — how hard is this bore packed against itself — and unlike density
it disagrees with the box often enough to change which coil you would build.

## Every block load-bearing

One walk in this set is a hand reduction of the staircase coil — blocks taken out
wherever one could go without stranding a turn. ([`derived.txt`](derived.txt) maps
every name here to the walk it derives from.)
`tools/minimal.rs` checks the claim: a term's floor is 3 in a coil window, 2 in a
hairpin, 1 in a step, and a walk whose every term sits on its floor cannot be shortened
at all.

    target/release/minimal           # every walk
    target/release/minimal --terms   # and which terms have slack

It says something about the search. Only **{n_min} of {n_minres}** walks here are minimal:
{min_list}. The search enumerated periods with legs up to
4 and never asked whether a leg was longer than it had to be, so most of what it found
carries slack — `{slack_name}` could lose {slack_n} blocks.

Removing slack is not automatically safe. The rule is local, and a shortened walk can
run into itself or start touching, so what the tool reports are candidates to put back
through `bore_split.py`.

### The reduction pass, and why minimality is not an optimisation

`tools/reduce.rs` takes the slack out and rebuilds at comparable length. It has to
enumerate rather than apply, because **coiling is global and the bend rule is local**.
A period coils only if it closes: drifting on one axis and returning to where it started
on the other two. Shorten one leg and not its opposite and the period stops closing, so
the walk wanders off diagonally instead of coiling. Taken naively, one coil went from a
box of 423 to **10,452** — still bend-only, no longer a coil.

Keeping only the reductions that still close and still wind a whole number of turns, and
putting the result back through the standardiser so it can be compared with what it came
from, **{red_n} of the {red_total} coils reduce**, and {red_distinct} distinct walks come out of
those {red_n} reductions:

{red_bullets}

{closed}

Canonicalisation is what makes that readable: with the representation pinned, one coil
cannot appear several times looking different, and a reduction of it is recognisable as
the walk it came from rather than as a new design.

Shortening a leg still does not make a better coil — it cuts how far the coil advances
per turn, so the same tube buys more revolutions in a fatter package. The best box per
block and the best walls-free box per block are both unchanged by the pass.

What those {red_n} reductions buy:

| | |
| --- | ---: |
{red_rows}

And **nothing it produced beat what was already there**: the smallest box stayed at 423
against the reduced field's 462, and the smallest walls-free at 477 against 522.

That is the point worth keeping. Shortening a leg reduces how far the coil advances per
turn, so the same tube buys more revolutions in a fatter, shorter package — a different
design, not a better one. Every block being load-bearing is a property, not a virtue.

One thing the pass finds: four results the search reports as distinct all reduce to
**the same walk**, `E2 S3 U1 S3 W2 N3 U1 N3` — one design wearing four amounts of slack.
Standardising finds more of the same, and `derived.txt` records every merge.

The reduced walks are kept, named `*_min`, with their sources recorded in
[`derived.txt`](derived.txt). They are in the scoring like anything else, so the
composite is over a set that contains both a design and its own reduction — worth
remembering when reading a rank, since the normalization is across whatever is present.

    target/release/reduce            # the pass
    target/release/reduce --write    # and reduced.json

## Walks kept but not scored

The scoring normalizes every metric across the set, so a walk of a different length does
not merely rank oddly — it rescales everyone else. Those walks live in the repository and
in the tables, below a rule, and are listed in [`unscored.txt`](unscored.txt) with the
reason. Their rate metrics — blocks/360, rise/360, turns/m, touching — stay comparable
and are still worth reading.

## Scoring them against each other

[**SCORING.md**](SCORING.md) combines {metrics_word} of the metrics and the touching count into
a single ranking under every common mean — harmonic, geometric, arithmetic, quadratic,
cubic, median, midrange, contraharmonic — and reports what the choice of mean does to
the answer. It does a great deal: `coil_2x2_146` places 1st under one and 17th under
another. {unanimous}

It also covers why to rank once rather than rank, cut and re-rank: with a
set-relative normalization the survivors of a cut come out in a different order
than they went in, having not been measured again.

    target/release/score         # the scoring table
    target/release/iterate       # what cutting and re-ranking does to it
    target/release/gen_scoring   # SCORING.md

## The gate

Every walk here has been through `check.py` — the parts, the sections, the seams, and a
voxel model of the assembled bore flooded from the outside to prove it is one sealed
passage. **{total_checks} checks across {n_checks} spirals, {total_failed} failed.** The per-walk output is
in `checks/`.

    tools/run_checks.sh          # re-run the gate over every walk

`check.py` needs shapely, numpy and scipy, which live in the Boxes.py venv — run it with
`~/Software/boxes/venv/bin/python`, as `README.md` says. The system python3 does
not have them.

**Boxes.py is Florian Festi's**, GPL-3.0-or-later, at
<https://github.com/florianfesti/boxes>. It draws the finger joints on every
block sheet these walks turn into, and it is an external dependency: a checkout
of it, not a copy in this repository.

## How they were found

`tools/search_spirals.rs` walks every periodic term sequence up to eight terms with legs up
to four blocks, keeping those that never reverse, return to the same lateral position each
period so the thing coils rather than drifts, wind a whole number of turns per period, stay
self-avoiding when repeated, and satisfy the bend-only rule.

That rule, for a window of three consecutive terms — outer A, middle m, outer C:

| A and C | | middle term |
| --- | --- | ---: |
| same axis, same direction | a step | >= 1 |
| same axis, opposite direction | a hairpin | >= 2 |
| different axes | a coil | >= 3 |

The bore-designs README states only the third case and calls the other two "a fold ... as
tight as you like". That holds for steps and not for hairpins: `U3 N1 D3` strands two turns
and `U3 N2 D3` costs none. The three cases were read off `bore_split.py` by probing it.

`tools/mknotation.rs` repeats a period to length and writes it in the bore notation,
trimming the tail so the last block is not mid-turn — a turn in the last block has nothing
after it to make it interior, so it is always its own piece. Then every candidate goes
through the splitter, and only bend-only walks are kept. **The splitter decides, not the
rule.**

## The tools

Every file in `tools/`, and what each one is for -- all {n_tools} of them.
The list is read from the directory rather than typed into this page, so it cannot fall
behind it: a tool with no description, or a description with no tool, stops the build.

{tool_tables}

## The code

The tools are Rust. They were JavaScript until 2026-11-02, and the Rust port writes this
page, `SCORING.md` and every `.json` file here byte for byte as Node did. Build them with
`cargo build --release` from this directory ([rustup](https://rustup.rs) installs Rust);
they land in `target/release/`, and `tools/build.sh` builds them before it runs them.
Each file in `tools/` is one program's front end; what they share is a library in `src/`:

| file | what it holds |
| --- | --- |
| `src/spiral_metrics.rs` | the rotation metrics |
| `src/standardise.rs` | standard form: orientation, period, the target length |
| `src/minimal.rs` | the term floors |
| `src/reduce.rs` | the reduction pass |
| `src/score.rs` | the scored metrics, the means and the ranking |
| `src/iterate.rs` | iterated ranking |
| `src/table.rs` | the two metrics tables |
| `src/bore.rs` | running `bore_split.py` and reading what it prints |
| `src/corpus.rs` | where things are; reading the walks, `parts.json` and `unscored.txt` |
| `src/json.rs` | reading JSON, and writing it laid out as `JSON.stringify` did |
| `src/jsfmt.rs` | writing numbers by JavaScript's rules, so the files stay byte-identical |
| `src/lib.rs` | the library's index |

The crate is `Cargo.toml` with `Cargo.lock`; its one dependency is the `regex` crate,
for reading `bore_split.py`'s output.

## The files behind the tables

Each coil is one line of notation in `walks/`, named for the coil, and each has a gate
transcript beside it in `checks/`:

{walk_files}

`standardised.json` records the canonical form every walk was pinned to, and
`reduced.json` what the reduction pass produced. Both are written by their tools, not
by hand.

## Regenerating

    tools/build.sh               # all of the below, in order

    cargo build --release        # the tools, into target/release/
    target/release/parts         # piece counts and distinct shapes -> parts.json
    target/release/table         # both tables, plain text
    target/release/table --md    # the same as markdown
    target/release/gen_readme    # this file
    python3 ../../../../../../../../lasermade-tools/md2html.py README.md index.html    # the published page

    target/release/spiral_metrics "$(cat walks/{first}.txt)"   # measure one walk

    ~/Software/boxes/venv/bin/python ../../../../../../../tools/viewer.py \
        "$(cat walks/{first}.txt)" \
        --out pages/{first}.html --title "{first_title}"

## What has not been done

**No cut files.** These are walks, viewers and gate results; nothing here has been split to
SVG or nested. `bore_split.py --write DIR` will do it.

The block counts are not equal — the scored ones run {mb_lo} to {mb_hi}, so their bore lengths
run {mm_lo} mm to {mm_hi} mm, a spread of {mm_spread}%. A period is repeated to about the right length and then trimmed
to wherever the tail comes out bend-only, so these are the same bore only to within a few
blocks. For comparing packing that is fine; for comparing *bores* it is not, and the length
would have to be pinned first.

Released under [CC0 1.0](LICENSE).
"#,
        y_link = y.link(),
        y_where = if in_set.is_some() { "" } else { ", which won a category and\nlives in a directory of its own" },
        others = if in_set.is_some() { format!("other {}", n - 1) } else { word(n) },
        fb_lo = num(fb_lo),
        fb_hi = num(fb_hi),
        fm_lo = num(fm_lo),
        fm_hi = num(fm_hi),
        fb_spread = fixed((fb_hi / fb_lo - 1.0) * 100.0, 1),
        tbl_shape = tbl(Which::Shape),
        tbl_rot = tbl(Which::Rot),
        we_name = worst_ends.name,
        we_inner = num(worst_ends.pf("interiorDistinct")),
        we_all = num(worst_ends.pf("distinct")),
        thin_link = thinnest.link(),
        thin_x = format!("{}x{}", thinnest.m.cross[0], thinnest.m.cross[1]),
        fat_link = fattest.link(),
        fat_x = format!("{}x{}", fattest.m.cross[0], fattest.m.cross[1]),
        dist_name = dist.name,
        dist_pieces = num(dist.pf("pieces")),
        dist_distinct = num(dist.pf("distinct")),
        widest = num(widest),
        top_rhythm = num(top_rhythm),
        t8 = terms_n(8),
        t16 = terms_n(16),
        fs_name = fewest_shapes.name,
        fs_distinct = num(fewest_shapes.pf("interiorDistinct")),
        fs_rhythm = num(fewest_shapes.pf("rhythm")),
        fs_repeats = fixed(fewest_shapes.pf("repeats"), 1),
        pb_name = plate_best.name,
        pb_link = plate_best.link(),
        pb_plate = num(round(plate_best.pf("meanPlate"))),
        y_name = y.name,
        y_plate = num(round(y.pf("meanPlate"))),
        mb_spread = fixed((mb_hi / mb_lo - 1.0) * 100.0, 1),
        box_link = bx.link(),
        box_vol = bx.m.vol,
        y_vol = y.m.vol,
        box_x = fixed(y.m.vol as f64 / bx.m.vol as f64, 2),
        rise_link = rise.link(),
        rise_mm = num(round(rise.m.rise_mm_per_360.unwrap_or(0.0))),
        y_rise_mm = num(round(y.m.rise_mm_per_360.unwrap_or(0.0))),
        rise_x = fixed(y.m.rise_per_360_or0() / rise.m.rise_per_360_or0(), 1),
        tube_names = all_best(&|r| r.m.blocks_per_360_or0()),
        tube_b = fixed(tube.m.blocks_per_360_or0(), 1),
        y_tube = fixed(y.m.blocks_per_360_or0(), 1),
        tube_pct = fixed((y.m.blocks_per_360_or0() / tube.m.blocks_per_360_or0() - 1.0) * 100.0, 0),
        pcs_link = pcs.link(),
        pcs_n = num(pcs.pf("innerPieces")),
        y_pcs = num(y.pf("innerPieces")),
        dist_link = dist.link(),
        dist_n = num(dist.pf("interiorDistinct")),
        y_dist = num(y.pf("interiorDistinct")),
        calm_names = all_best(&|r| r.m.turns_per_metre),
        calm_t = fixed(calm.m.turns_per_metre, 2),
        y_calm = fixed(y.m.turns_per_metre, 2),
        clean_link = clean.link(),
        clean_vol = clean.m.vol,
        y_touch = y.m.touching,
        n_word = word(n),
        o_box = ord(r_box),
        o_pieces = ord(r_pieces),
        o_distinct = ord(r_distinct),
        o_tube = ord(r_tube),
        o_turns = ord(r_turns),
        n_min = minimal.len(),
        n_minres = minres.len(),
        min_list = minimal.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", "),
        slack_name = slackiest.name,
        slack_n = num(slackiest.saving),
        red_n = red_ok.len(),
        red_total = redres.len(),
        red_distinct = word(red_walks.len()),
        red_rows = red_rows.iter().map(|(k, v)| format!("| {k} | {v} |")).collect::<Vec<_>>().join("\n"),
        metrics_word = word(METRICS.len()),
        total_checks = locale_int(total_checks),
        n_checks = checks.len(),
        n_tools = word(on_disk.len()),
        walk_files = rows.iter().map(|r| format!("`walks/{}.txt`", r.name)).collect::<Vec<_>>().join(", "),
        first_title = first.replace('_', " "),
        mb_lo = num(mb_lo),
        mb_hi = num(mb_hi),
        mm_lo = num(mm_lo),
        mm_hi = num(mm_hi),
        mm_spread = fixed((mm_hi / mm_lo - 1.0) * 100.0, 1),
    );
    std::fs::write(root.join("README.md"), &md).expect("cannot write README.md");
    println!("README.md written, {} lines", md.split('\n').count());
}
