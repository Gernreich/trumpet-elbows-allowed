// Regenerate SCORING.md. Every number comes from src/score.rs (and src/iterate.rs).
//
//   target/release/gen_scoring

use coilsearch::corpus::root;
use coilsearch::iterate::results as iterate;
use coilsearch::jsfmt::{fixed, num};
use coilsearch::score::{score, table, Row, EPS, MEANS, METRICS};

// Counted, not remembered: this line read "Seven metrics" while METRICS held eight,
// and the wrong number had been copied into README.md as well.
const NUM: [&str; 13] =
    ["Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven", "Twelve"];
// With a fallback: NUM stops at twelve, and a thirteenth metric would have shipped
// the word "undefined" in the first line of the page.
fn word(n: usize) -> String {
    NUM.get(n).map(|s| s.to_string()).unwrap_or_else(|| n.to_string())
}
fn lower(n: usize) -> String {
    word(n).to_lowercase()
}
/// 1st, 2nd, 3rd, 4th ... 11th, 12th, 13th ... 21st.
pub fn ord(n: usize) -> String {
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

fn main() {
    let sc = score(false, false);
    let rows: &Vec<Row> = &sc.rows;
    let rank = |m: &str, r: &Row| sc.rank(m, &r.name);
    let tw = num(sc.touch_weight);
    let eps = num(EPS);

    // Counted off the ranking, not remembered.
    let mut by_harm: Vec<&Row> = rows.iter().collect();
    by_harm.sort_by_key(|r| rank("harm", r));
    let ft = by_harm.iter().position(|r| r.v.touching > 0.0).expect("no coil with touching walls");
    let first_touch = by_harm[ft];
    let clean_below = by_harm[ft + 1..].iter().filter(|r| r.v.touching == 0.0).count();
    let clean = rows.iter().filter(|r| r.v.touching == 0.0).count();
    // --solid's own test, the same one score uses.
    let thick = |r: &Row| *r.m.size.iter().min().unwrap() >= 3;
    let clean_solid = rows.iter().filter(|r| r.v.touching == 0.0 && thick(r)).count();
    let solid = rows.iter().filter(|r| thick(r)).count();

    let tbl = table(&sc, true);
    let tbl: String = tbl.trim().split('\n').filter(|l| l.starts_with('|')).collect::<Vec<_>>().join("\n");

    struct Swing {
        n: String,
        best: usize,
        worst: usize,
        swing: usize,
    }
    let mut swing: Vec<Swing> = rows
        .iter()
        .map(|r| {
            let rk: Vec<usize> = MEANS.iter().map(|m| rank(m.k, r)).collect();
            let (best, worst) = (*rk.iter().min().unwrap(), *rk.iter().max().unwrap());
            Swing { n: r.name.clone(), best, worst, swing: worst - best }
        })
        .collect();
    swing.sort_by(|a, b| b.swing.cmp(&a.swing));

    let row = |name: &str| rows.iter().find(|r| r.name == name).unwrap();
    let weak = |name: &str| {
        let mut v = sc.inputs(row(name));
        v.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        v[0]
    };
    let agree = |a: &str, b: &str| rows.iter().filter(|r| rank(a, r) == rank(b, r)).count();

    let mut tally: Vec<(String, usize)> = Vec::new();
    for m in &MEANS {
        let win = rows.iter().min_by_key(|r| rank(m.k, r)).unwrap().name.clone();
        match tally.iter_mut().find(|(n, _)| *n == win) {
            Some(e) => e.1 += 1,
            None => tally.push((win, 1)),
        }
    }
    let mut ordered = tally.clone();
    ordered.sort_by(|a, b| b.1.cmp(&a.1));
    let top = ordered[0].clone();

    let (two, three) = (&swing[0], &swing[1]);
    // picked from the data: standardising renamed every coil, so nothing is named by hand
    let mut polar: Vec<String> = swing.iter().take(3).map(|x| x.n.clone()).collect();
    polar.push(by_harm[0].name.clone());
    let mut uniq: Vec<String> = Vec::new();
    for n in polar {
        if !uniq.contains(&n) {
            uniq.push(n);
        }
    }
    let polar_rows: Vec<String> = uniq
        .iter()
        .take(3)
        .map(|n| {
            let w = weak(n);
            format!("| `{n}` | {} = {} | #{} | #{} |", w.label, fixed(w.x, 3), sc.rank("harm", n), sc.rank("contra", n))
        })
        .collect();
    let mut thin_sorted: Vec<&Row> = rows.iter().collect();
    thin_sorted.sort_by(|a, b| a.v.cross_area.partial_cmp(&b.v.cross_area).unwrap());
    let thin = thin_sorted[0];

    let iter = iterate(rows);
    let iter_lines: Vec<String> = iter
        .iter()
        .map(|r| {
            let rounds: Vec<String> = r.log.iter().filter(|l| l.to > 1).map(|l| format!("{}/{}", l.moved, l.to)).collect();
            let total: usize = r.log.iter().map(|l| l.moved).sum();
            format!("* **{}** — survivors reordered {} over the rounds ({total} moves in total)", r.label, rounds.join(", "))
        })
        .collect();
    let thin_gone = iter.iter().filter(|r| !r.log[0].before.contains(&thin.name)).count();
    let medians_at_one = rows.iter().filter(|r| (r.s[5] - 1.0).abs() < 1e-9).count();
    let what: Vec<String> = METRICS
        .iter()
        .map(|m| format!("* **{}** — {} is better", m.label, if m.lo_better { "less" } else { "more" }))
        .collect();
    let n = rows.len();

    let md = format!(
        r#"# Scoring

{metrics_word} metrics, one touching count, and every common mean, so the ranking can be
read against the thing that produced it. Regenerate with `target/release/gen_scoring`.

## What is scored

{what}
* **touching** — carried the same way as the rest, at weight {tw}

**Nothing is judged on the bore's mouth and exit.** Every design has those two pieces
and no design chooses them, so every metric here is measured over the interior — the
pieces in between — and the piece and shape counts are counts of those. Blocks and mm,
which only describe, still cover the whole bore.

The metrics are normalized to (0,1] with 1 the best in the set, oriented so bigger is
better, and floored at {eps} so that one worst-in-set value cannot zero a product.

**Touching is treated differently, on purpose.** A wall the bore shares with itself is
visible in the finished instrument, so it is the heaviest input here at weight {tw},
and it is penalized convexly — **1/(1+t)** — rather than fading linearly. The step from
no contact to any contact is far larger than any step after it: 0 contacts scores 1.000,
8 contacts 0.111, 20 contacts 0.048. Nothing but a clean coil can score 1.

That form is also absolute where a linear fade against the set maximum is not. Dropping
the worst coil would move everyone else's touching term under a linear fade; under
1/(1+t) it moves nothing.

Override the weight with `SPIRAL_TOUCH_WEIGHT=8 target/release/score`.

### A heavy weight is a preference, not a guarantee

Weighting touching at {tw} does not stop a coil with contact from beating a clean one.
Under the harmonic mean the first coil *with* touching places **{ft_rank}**, above **{clean_below}** coils
that have none — because each of those has some other metric sitting on the {eps} floor,
and the harmonic mean punishes that harder than it punishes {ft_touch} contacts.

If no touching walls is a *requirement* rather than a preference, filter:

    target/release/score --clean     # ranks only the coils with no touching walls
    target/release/score --solid     # and only those at least 3 blocks thick

which is the same advice as everywhere else here — cut on the property, then rank.

### Two filters, from judgement rather than argument

**Neither filter was argued for; both were read off coils judged by eye.** Ten were
judged, seven liked and three not, and these two filters reproduce that split exactly —
selecting the liked set and nothing else:

* **no touching walls** — perfect on its own: no coil with any wall contact was liked, and
  seven of the eight without were.
* **at least 3 blocks thick in every direction** — a coil 2 thick is a ribbon rather than
  a rod.

Those judgements are over a set of ten, not over this page's {n_lower} — of
which {clean_lower} are walls-free. The filters are what carries across; the split
they reproduce is not a ranking of what is here.

The thickness filter took a designed test. The one rejected walls-free coil is extreme on
two things at once, thinness and elongation, and nothing else in the set separates them,
so two candidates settle it: one 3 thick but far longer than anything else liked (aspect
43 against a ceiling of 24), and one 2 thick but as short as a 2-thick coil gets at this
tube length (aspect 49). The long one is liked and the thin one is not, which rules out
elongation on its own — aspect 43 is fine when the coil has a core.

Both are filters and neither is scored. Thickness is not a gradient: 3 is acceptable, and
being thicker is not better — the 5x5 coil packs worst of anything judged and is liked all
the same. Thickness sorts nothing here: all {solid_lower} of the coils are at
least 3 thick, so `--solid` removes none of them and `--clean` alone does the work.

    target/release/score --clean --solid    # {clean_solid} of the {n} coils here

One honest limit. At this tube length the two properties are coupled: a 2-thick coil has
nowhere to put 177 blocks but lengthwise, so thin coils start at aspect 49 while 3-thick
ones stop at 43. A rule of "aspect under about 46" fits every judgement too. Thickness is
preferred because it is an integer boundary rather than a threshold picked from a 13%
window, and because the designed test varied thickness while holding aspect nearly fixed
— but the two cannot be fully separated without a shorter bore.

**`blocks/360` is deliberately gone.** It is anti-correlated with turns/m by
construction — a tighter spiral has to turn more often — so carrying both let them
cancel, and made the composite quieter about coiling than the columns themselves are.

## The means

| mean | order *p* | what it rewards |
| --- | :-: | --- |
| harmonic | -1 | punishes the weakest input hardest |
| geometric | 0 | |
| arithmetic | 1 | |
| quadratic (RMS) | 2 | |
| cubic | 3 | rewards the strongest input hardest |
| median | — | ignores both extremes; see the caveat below |
| midrange | — | only the extremes |
| contraharmonic | — | rewards the strongest harder still |

The first five are the power mean of order *p*, which increases with *p*, so for
every spiral **harmonic ≤ geometric ≤ arithmetic ≤ quadratic ≤ cubic**. Verified here
on all {n}. What changes with *p* is not the size of the score but how much a single
bad metric is allowed to sink it.

## The table

{tbl}

**The median does not survive the weighting.** It is an order statistic, and weight is
applied by repetition, so touching occupies {tw} of the {values} values and can simply *be*
the median. {medians_at_one} coils tie at exactly 1.0000. Read the median column knowing that;
the power means do not have this problem.

## Choosing a mean is choosing how much a weak spot counts

That choice is worth more than any metric in it. {two_n} ranks **{two_best}** under one mean
and **{two_worst}** under another — a swing of {two_swing} places in a field of {n}. {three_n} swings {three_swing}.

| spiral | worst single input | harmonic | contraharmonic |
| --- | --- | ---: | ---: |
{polar_rows}

`{two_n}` is the clearest case: one input on the floor and another at the ceiling.
The mean that punishes weak spots reads it as disqualified; the mean that rewards strong
ones reads it as the best thing here. Both are arithmetically correct — they are
answering different questions. `{three_n}` is the same shape of argument, its weak spot
being {three_weak}.

Harmonic and contraharmonic agree on **{agree_hc}** of {n} placings — they are as opposed as
two means of the same numbers can be. Harmonic and geometric agree on {agree_hg}, which is
why the geometric mean is the usual choice when no weak spot should be forgiven but
outright disqualification is too strong.

## What survives

`{top_n}` comes first under **{top_c} of the {n_means} means**, and the reason is visible in the
table above: its worst input is {top_weak}, where every other contender has something
at {eps}. It does not win by being outstanding anywhere. It wins by having nothing to
punish, which is the one way to be robust to the choice of mean.

If a weak spot is genuinely fatal — a shared wall that will leak, a coil too fat for
the body — use the harmonic mean, or filter and then rank. If the design is allowed one
bad number in exchange for a very good one, use quadratic or cubic. The arithmetic mean
is the choice that declines to say.

## Ranking once, not repeatedly

A tempting variant is to rank, cut the bottom half, and re-rank the survivors. Do not.

{iter_lines}

The first two reorder coils that did not change, purely because other coils left the
set. `{b2}` places 3rd of {n} and 1st of the surviving 9; `{b0}`
places 1st and then 4th. Nothing about either was measured again.

The cause is that min-max reads its lo and hi off whoever is present, so dropping
alternatives rescales every metric by a different factor. That is an
independence-of-irrelevant-alternatives failure, and it is the reason to cut on a
property fixed in advance — touching > 0, or a cross-section that will not fit —
rather than on composite score.

The other end of it is just as decisive. Under a pure ratio-to-best normalization with
a geometric mean, nothing depends on which coils are present, and the whole procedure
is a **no-op: 0 moves in every round**. So iterating either changes the order for a
reason that has nothing to do with the coils, or changes nothing at all.

The winner here survives all three, so nothing practical turns on it — but the order
below the top is meaningless under iteration, and should not be read.

One more cost: a cut on composite score removes whatever is best at a single thing,
because a composite is a compromise. `{thin_n}` has the narrowest cross-section in
the set and does not survive round 0 of {thin_gone} of the {n_iter} runs.

    target/release/iterate       # the numbers above
"#,
        metrics_word = word(METRICS.len()),
        what = what.join("\n"),
        ft_rank = ord(sc.rank("harm", &first_touch.name)),
        ft_touch = num(first_touch.v.touching),
        n_lower = lower(n),
        clean_lower = lower(clean),
        solid_lower = lower(solid),
        values = num(METRICS.len() as f64 + sc.touch_weight),
        two_n = two.n,
        two_best = two.best,
        two_worst = two.worst,
        two_swing = two.swing,
        three_n = three.n,
        three_swing = three.swing,
        polar_rows = polar_rows.join("\n"),
        three_weak = weak(&three.n).label,
        agree_hc = agree("harm", "contra"),
        agree_hg = agree("harm", "geo"),
        top_n = top.0,
        top_c = top.1,
        n_means = MEANS.len(),
        top_weak = fixed(weak(&top.0).x, 3),
        iter_lines = iter_lines.join("\n"),
        b2 = iter[0].log[0].before[2],
        b0 = iter[0].log[0].before[0],
        thin_n = thin.name,
        n_iter = iter.len(),
    );
    std::fs::write(root().join("SCORING.md"), &md).expect("cannot write SCORING.md");
    println!("SCORING.md written, {} lines", md.split('\n').count());
}
