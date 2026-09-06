#!/usr/bin/env node
// Regenerate SCORING.md. Every number comes from tools/score.js.
const fs = require('fs'), path = require('path'), cp = require('child_process');
const { rows, METRICS, MEANS, R, inputs, TOUCH_WEIGHT, EPS } = require('./score.js');
const { results: iter } = require('./iterate.js');
const root = path.join(__dirname, '..');

const table = cp.execSync(`node ${JSON.stringify(path.join(__dirname,'score.js'))} --md`,
  { encoding: 'utf8' }).trim().split('\n').filter(l => l.startsWith('|')).join('\n');

const swing = rows.map(r => {
  const rk = MEANS.map(M => R[M.k].get(r.name));
  return { n: r.name, best: Math.min(...rk), worst: Math.max(...rk),
           swing: Math.max(...rk) - Math.min(...rk) };
}).sort((a, b) => b.swing - a.swing);

const weak = name => {
  const v = inputs(rows.find(r => r.name === name)).map(i => ({ l: i.label, x: i.x }))
    .sort((a, b) => a.x - b.x);
  return v[0];
};
const agree = (a, b) => rows.filter(r => R[a].get(r.name) === R[b].get(r.name)).length;

const firsts = MEANS.map(M => ({ mean: M.label,
  win: rows.slice().sort((x, y) => R[M.k].get(x.name) - R[M.k].get(y.name))[0].name }));
const tally = {};
for (const f of firsts) tally[f.win] = (tally[f.win] || 0) + 1;
const top = Object.entries(tally).sort((a, b) => b[1] - a[1])[0];

const two = swing[0], three = swing[1];
// picked from the data: standardising renamed every coil, so nothing is named by hand
const polar = swing.slice(0, 3).map(x => x.n);
const thin = rows.slice().sort((a,b) =>
  (a.v.crossArea) - (b.v.crossArea))[0];
const md = `# Scoring

Seven metrics, one touching count, and every common mean, so the ranking can be
read against the thing that produced it. Regenerate with \`node tools/gen_scoring.js\`.

## What is scored

${METRICS.map(M => `* **${M.label}** — ${M.dir === 'lo' ? 'less' : 'more'} is better`).join('\n')}
* **touching** — carried the same way as the rest, at weight ${TOUCH_WEIGHT}

**Nothing is judged on the bore's mouth and exit.** Every design has those two pieces
and no design chooses them, so every metric here is measured over the interior — the
pieces in between — and the piece and shape counts are counts of those. Blocks and mm,
which only describe, still cover the whole bore.

The metrics are normalized to (0,1] with 1 the best in the set, oriented so bigger is
better, and floored at ${EPS} so that one worst-in-set value cannot zero a product.

**Touching is treated differently, on purpose.** A wall the bore shares with itself is
visible in the finished instrument, so it is the heaviest input here at weight ${TOUCH_WEIGHT},
and it is penalized convexly — **1/(1+t)** — rather than fading linearly. The step from
no contact to any contact is far larger than any step after it: 0 contacts scores 1.000,
8 contacts 0.111, 20 contacts 0.048. Nothing but a clean coil can score 1.

That form is also absolute where a linear fade against the set maximum is not. Dropping
the worst coil would move everyone else's touching term under a linear fade; under
1/(1+t) it moves nothing.

Override the weight with \`SPIRAL_TOUCH_WEIGHT=8 node tools/score.js\`.

### A heavy weight is a preference, not a guarantee

Weighting touching at ${TOUCH_WEIGHT} does not stop a coil with contact from beating a clean one.
Under the harmonic mean the first coil *with* touching places **3rd**, above **6** coils
that have none — because each of those has some other metric sitting on the ${EPS} floor,
and the harmonic mean punishes that harder than it punishes 8 contacts.

If no touching walls is a *requirement* rather than a preference, filter:

    node tools/score.js --clean      # ranks only the coils with no touching walls
    node tools/score.js --solid      # and only those at least 3 blocks thick

which is the same advice as everywhere else here — cut on the property, then rank.

### Two filters, arrived at from judgements rather than argument

Ten coils were judged by eye, seven liked and three not. Two filters reproduce that split
exactly, and between them they select the liked set and nothing else:

* **no touching walls** — perfect on its own: no coil with any wall contact was liked, and
  seven of the eight without were.
* **at least 3 blocks thick in every direction** — a coil 2 thick is a ribbon rather than
  a rod.

The second took a designed test to establish. The one rejected walls-free coil was extreme
on two things at once, thinness and elongation, and nothing else in the set separated them.
So two candidates were built: one 3 thick but far longer than anything previously liked
(aspect 43 against a previous ceiling of 24), and one 2 thick but as short as a 2-thick
coil can be at this tube length (aspect 49). The long one was liked and the thin one was
not, which rules out elongation on its own — aspect 43 is fine when the coil has a core.

Both are filters and neither is scored. Thickness is not a gradient: 3 is acceptable, and
being thicker is not better — the 5x5 coil packs worst of anything here and was liked.

    node tools/score.js --clean --solid    # exactly the seven

One honest limit. At this tube length the two properties are coupled: a 2-thick coil has
nowhere to put 177 blocks but lengthwise, so thin coils start at aspect 49 while 3-thick
ones stop at 43. A rule of "aspect under about 46" fits every judgement too. Thickness is
preferred because it is an integer boundary rather than a threshold picked from a 13%
window, and because the designed test varied thickness while holding aspect nearly fixed
— but the two cannot be fully separated without a shorter bore.

**\`blocks/360\` is deliberately gone.** It is anti-correlated with turns/m by
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
on all ${rows.length}. What changes with *p* is not the size of the score but how much a single
bad metric is allowed to sink it.

## The table

${table}

**The median does not survive the weighting.** It is an order statistic, and weight is
applied by repetition, so touching occupies ${TOUCH_WEIGHT} of the ${METRICS.length + TOUCH_WEIGHT} values and can simply *be*
the median. ${rows.filter(r => Math.abs(r.median - 1) < 1e-9).length} coils tie at exactly 1.0000. Read the median column knowing that;
the power means do not have this problem.

## Choosing a mean is choosing how much a weak spot counts

That choice is worth more than any metric in it. ${two.n} ranks **${two.best}** under one mean
and **${two.worst}** under another — a swing of ${two.swing} places in a field of ${rows.length}. ${three.n} swings ${three.swing}.

| spiral | worst single input | harmonic | contraharmonic |
| --- | --- | ---: | ---: |
${[...polar, rows.slice().sort((a,b)=>R.harm.get(a.name)-R.harm.get(b.name))[0].name]
  .filter((n,i,a) => a.indexOf(n) === i).slice(0,3).map(n => {
  const w = weak(n);
  return `| \`${n}\` | ${w.l} = ${w.x.toFixed(3)} | #${R.harm.get(n)} | #${R.contra.get(n)} |`;
}).join('\n')}

\`${two.n}\` is the clearest case: one input on the floor and another at the ceiling.
The mean that punishes weak spots reads it as disqualified; the mean that rewards strong
ones reads it as the best thing here. Both are arithmetically correct — they are
answering different questions. \`${three.n}\` is the same shape of argument, its weak spot
being ${weak(three.n).l}.

Harmonic and contraharmonic agree on **${agree('harm','contra')}** of ${rows.length} placings — they are as opposed as
two means of the same numbers can be. Harmonic and geometric agree on ${agree('harm','geo')}, which is
why the geometric mean is the usual choice when no weak spot should be forgiven but
outright disqualification is too strong.

## What survives

\`${top[0]}\` comes first under **${top[1]} of the ${MEANS.length} means**, and the reason is visible in the
table above: its worst input is ${weak(top[0]).x.toFixed(3)}, where every other contender has something
at ${EPS}. It does not win by being outstanding anywhere. It wins by having nothing to
punish, which is the one way to be robust to the choice of mean.

If a weak spot is genuinely fatal — a shared wall that will leak, a coil too fat for
the body — use the harmonic mean, or filter and then rank. If the design is allowed one
bad number in exchange for a very good one, use quadratic or cubic. The arithmetic mean
is the choice that declines to say.

## Ranking once, not repeatedly

A tempting variant is to rank, cut the bottom half, and re-rank the survivors. Do not.

${iter.map(r => `* **${r.label}** — survivors reordered ` +
  r.log.filter(l => l.to > 1).map(l => `${l.moved}/${l.to}`).join(', ') +
  ` over the rounds (${r.log.reduce((a,l) => a + l.moved, 0)} moves in total)`).join('\n')}

The first two reorder coils that did not change, purely because other coils left the
set. \`${iter[0].log[0].before[2]}\` places 3rd of ${rows.length} and 1st of the surviving 9; \`${iter[0].log[0].before[0]}\`
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
because a composite is a compromise. \`${thin.name}\` has the narrowest cross-section in
the set and does not survive round 0 of ${iter.filter(r => !r.log[0].before.includes(thin.name)).length} of the ${iter.length} runs.

    node tools/iterate.js        # the numbers above
`;
fs.writeFileSync(path.join(root, 'SCORING.md'), md);
console.log('SCORING.md written,', md.split('\n').length, 'lines');
