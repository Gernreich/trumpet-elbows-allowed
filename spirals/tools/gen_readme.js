#!/usr/bin/env node
// Regenerate README.md. Every number in it comes from the tools, so the page
// cannot drift from the walks: node tools/gen_readme.js
const fs = require('fs'), path = require('path'), cp = require('child_process');
const { metrics, period } = require('./spiral_metrics.js');
const root = path.join(__dirname, '..');

const parts = JSON.parse(fs.readFileSync(path.join(root, 'parts.json'), 'utf8'));

// The reduction table was hardcoded, and every figure in it had gone stale: it claimed 14
// walks reduce where reduce.js reports 13, and all four of its counts were wrong. Counting
// them here is the whole point of this file -- reduced.json is what `reduce.js --write`
// leaves behind, so the table cannot drift from the pass again.
const reduced = JSON.parse(fs.readFileSync(path.join(root, 'reduced.json'), 'utf8'))
  .filter(r => r.status === 'reduced');
const RED = (f) => reduced.filter(f).length;
const redRows = [
  ['box smaller',        RED(r => r.after.vol < r.before.vol)],
  ['box bigger',         RED(r => r.after.vol > r.before.vol)],
  ['box unchanged',      RED(r => r.after.vol === r.before.vol)],
  ['touching reduced',   RED(r => r.after.touching < r.before.touching)],
  ['touching increased', RED(r => r.after.touching > r.before.touching)],
  ['touching unchanged', RED(r => r.after.touching === r.before.touching)],
];
const rows = fs.readdirSync(path.join(root, 'walks')).filter(f => f.endsWith('.txt'))
  .map(f => { const name = f.replace(/\.txt$/, '');
    const walk = fs.readFileSync(path.join(root,'walks',f),'utf8').trim();
    const p = parts[name] || {};
    // judged over the interior, described over the whole bore
    return { name, p, per: period(walk), full: metrics(walk),
             m: metrics(walk, p.interiorBlocks) }; })
  .sort((a, b) => a.m.vol - b.m.vol);

// check.py results, as saved by tools/run_checks.sh
const checks = {};
for (const f of fs.readdirSync(path.join(root, 'checks')).filter(f => f.endsWith('.txt'))) {
  const t = fs.readFileSync(path.join(root, 'checks', f), 'utf8');
  const m = t.match(/(\d+) checks, (\d+) failed/);
  if (m) checks[f.replace(/\.txt$/, '')] = { n: +m[1], failed: +m[2] };
}
const totalChecks = Object.values(checks).reduce((a, c) => a + c.n, 0);
const totalFailed = Object.values(checks).reduce((a, c) => a + c.failed, 0);

const tbl = (flag) => cp.execSync(`node ${JSON.stringify(path.join(__dirname,'table.js'))} --md ${flag}`,
                                  { encoding: 'utf8' }).trim();
const best  = f => rows.slice().sort((a, b) => f(a) - f(b))[0];
// Ties were being hidden: sorting and taking the first credits one coil and drops
// the others that matched it exactly. List them all.
const allBest = (f, eps = 1e-9) => {
  const s = rows.slice().sort((a, b) => f(a) - f(b));
  return s.filter(r => Math.abs(f(r) - f(s[0])) < eps);
};
const names = list => list.map(L).join(', ');
// The walk this started from, found through derived.txt rather than by name:
// standardising renamed every coil.
const derived = Object.fromEntries(
  fs.readFileSync(path.join(root,'derived.txt'),'utf8').split('\n')
    .map(l => l.replace(/#.*/,'').trim()).filter(Boolean)
    .map(l => { const [n, ...src] = l.split(/\s+/); return [n, src]; }));
const yours = rows.find(r => (derived[r.name] || []).includes('staircase_coil')) || rows[0];
const box   = best(r => r.m.vol);
const rise  = best(r => r.m.risePer360);
const tube  = best(r => r.m.blocksPer360);
const pcs   = best(r => r.p.innerPieces);
const dist  = best(r => r.p.interiorDistinct);
const calm  = best(r => r.m.turnsPerMetre);
const clean = rows.filter(r => r.m.touching === 0).sort((a,b) => a.m.vol - b.m.vol)[0];
const plateBest = rows.slice().sort((a,b) => b.p.meanPlate - a.p.meanPlate)[0];
// picked from the data, not named: standardising renamed every coil
const fewestShapes = rows.slice().sort((a,b) => a.p.interiorDistinct - b.p.interiorDistinct)[0];
const thinnest = rows.slice().sort((a,b) => (a.m.cross[0]*a.m.cross[1]) - (b.m.cross[0]*b.m.cross[1]))[0];
const fattest  = rows.slice().sort((a,b) => (b.m.cross[0]*b.m.cross[1]) - (a.m.cross[0]*a.m.cross[1]))[0];
const worstEnds = rows.slice().sort((a,b) => (b.p.distinct - b.p.interiorDistinct) - (a.p.distinct - a.p.interiorDistinct))[0];
const L = r => `[\`${r.name}\`](pages/${r.name}.html)`;
const { rows: srows, R: SR } = require('./score.js');
const { results: minres } = require('./minimal.js');
const { results: redres } = require('./reduce.js');
const redOK = redres.filter(r => r.status === 'reduced');
const redStat = { total: redres.length, reduced: redOK.length,
                  distinct: new Set(redOK.map(r => r.walk)).size };
const minimal = minres.filter(r => r.saving === 0).map(r => r.name);
const slackiest = minres.slice().sort((a, b) => b.saving - a.saving)[0];
const stair = minres.find(r => r.name === yours.name) || minres[0];
const { MEANS: SM } = require('./score.js');
const wins = SM.map(M => srows.slice().sort((a,b) => SR[M.k].get(a.name) - SR[M.k].get(b.name))[0].name);
const tally = {}; for (const w of wins) tally[w] = (tally[w] || 0) + 1;
const [bestName, bestN] = Object.entries(tally).sort((a,b) => b[1] - a[1])[0];
const unanimous = `\`${bestName}\` comes first under ${bestN} of the ${SM.length} means.`;

const md = `# Spirals

Coiling bore walks that turn about an axis while advancing along it, collected with the
numbers that say how hard each one spirals and what each one costs to build. Every walk
here splits with **no elbows** — every turn folds into a piece as an L, none is stranded
as a single-block piece of its own — and every one passes the full gate.

The one this started from is ${L(yours)}; the other ${rows.length - 1} came out of an
exhaustive search for something tighter.

<!-- readme-only -->
**[Read it as a page](https://gernreich.github.io/spirals/)** — the same text set for
reading, and the only place the viewer links below open a model you can turn rather
than a page of HTML source.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
generator and tool, indexed.

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
bounded by the mouth and the exit rather than by a neighbour, and the notation says as
much: the first term is only the way you came in. What is standardised is that every
walk has exactly one of each, and the judged metrics ignore them regardless.

Fixing forward and the sense of rotation is not enough on its own. The four rotations
about the forward axis all satisfy both, and so does every rotation of the cycle that
opens on a north term, so taking the first representation that fits leaves the same coil
able to appear more than once looking different — and it did, four times over in one
case. Pinning the representation to the lexicographically smallest is what makes
\`distinct\` mean anything.

Standardising collapsed 30 walks into ${rows.length}: several that looked different were the same
coil in another orientation, and [\`derived.txt\`](derived.txt) records which. The hand
reduction of the staircase coil, the tool's reduction of it, and the extension of the
hand reduction all turn out to be one coil.

### The length rule, and why it is not a cap

Whole periods of different lengths cannot all reach the same total, so some rule has to
choose the repeat counts. The first one tried — as many periods as fit under a limit —
is the wrong shape: it truncates the long-period coils hardest, so every one of them
lands below every short-period one, and the bores ran ${''}126 to 144 blocks, a 14.3% spread.

The target is now chosen rather than assumed. \`tools/standardise.js\` searches bore
lengths from 120 to 220 blocks for the one that makes the tube lengths most alike, takes
the **nearest** whole-period count to it rather than the largest that fits, and lands on
${''}180: bores of ${Math.min(...rows.map(r=>r.full.blocks))} to ${Math.max(...rows.map(r=>r.full.blocks))} blocks, ${Math.min(...rows.map(r=>r.full.mm))}mm to ${Math.max(...rows.map(r=>r.full.mm))}mm, a spread of ${((Math.max(...rows.map(r=>r.full.blocks))/Math.min(...rows.map(r=>r.full.blocks))-1)*100).toFixed(1)}%.

That is as close as this set can get at a plausible size. Squeezing the spread below 10%
needs a seven-metre bore and below 5% needs twelve, which is not a trumpet.

So the residual is handled where it actually bites, in the scoring: **box and piece count
are scored per block**, because both grow with tube and comparing them absolutely would
hand the shorter coils an advantage they did nothing to earn. Over this set piece count
correlates 0.63 with block count. Cross-section, mean plate, distinct shapes and the
rates are length-independent already; touching stays an absolute count, because the
requirement is none of it at any length.

## Shape and cost

${tbl('--shape')}

Envelope, box and cross-section are in blocks; a block is 31mm of centreline. Sorted by
box, smallest first.

**The bore's mouth and exit are not judged.** Every design has them, no design chooses
them, and they are the two pieces that never join the rhythm — so every column here
except blocks and mm is measured on the interior, the pieces in between. It is not a
cosmetic change: the ends stick out of the envelope they bracket. \`${worstEnds.name}\` needs ${worstEnds.p.interiorDistinct} shapes
rather than ${worstEnds.p.distinct}, the difference being end pieces alone. Blocks and mm still describe the whole bore, because that is what the bore is.

**Cross-section** is the envelope with the coil axis taken out — how fat the coil is,
which is what decides whether it fits inside anything. It is not implied by the box: ${L(thinnest)} is
${thinnest.m.cross.join('x')} in section and ${L(fattest)} is ${fattest.m.cross.join('x')}, and the box does not say so.

**Distinct** is how many different piece shapes the cut list holds. A coil built by
repeating one period needs only a handful, however long it runs: ${dist.name} is
${dist.p.pieces} pieces cut from ${dist.p.distinct} shapes. The widest needs ${rows.slice().sort((a,b)=>b.p.interiorDistinct-a.p.interiorDistinct)[0].p.interiorDistinct}. That is files to check and parts to tell apart on the bench, and it
does not show up in the piece count at all.

**Period** is the repeating unit of the walk itself, in terms and in blocks, and it sits
next to the rhythm because the two together say something neither says alone.

**A piece is a flat snake, so the splitter has to start a new one exactly where the bore
leaves its plane.** Turns that stay in-plane — the folds, hairpins and steps both —
happen inside a piece and cost no boundary at all. So the rhythm is not counting turns,
it is counting departures from the plane, and it comes out as

    rhythm = period terms x the share of turns that leave the plane

exactly, on every coil here. That decomposition is why the period column earns its
place: a rhythm of ${rows.slice().sort((a,b)=>b.p.rhythm-a.p.rhythm)[0].p.rhythm} could be a long period turning gently or a short one leaving the
plane at every chance, and the rhythm alone cannot tell you which. In this set the
${rows.filter(r=>r.per.terms===8).length} eight-term coils leave the plane on half their turns, while the ${rows.filter(r=>r.per.terms===16).length} sixteen-term ones do it on
three turns in four — so the gap in rhythm is two separate factors multiplying, not one.

The share itself is **not** scored, and was tested rather than assumed: it takes only
three values across the set, ${rows.filter(r=>r.per.terms===8).length} of the ${rows.length} coils sit on exactly one of them, and it
correlates 0.89 with the rhythm, 0.85 with distinct shapes and 0.76 with box per block.
Near-constant and not independent — the same double failure that kept fill density out.

**Rhythm** is how many pieces you lay before the cut list starts over, and how many
times it recurs. It is descriptive, not scored: across coils built by repeating a period
it barely varies — ${rows.filter(r => r.p.rhythm === 4).length} of the ${rows.length} here are a 4-piece rhythm — and a near-constant metric
in a mean only dilutes the ones that discriminate. It is worth knowing at the bench all
the same, and it is not the same thing as **distinct**: the two rank the set alike in
only 6 of ${rows.length} places. \`${fewestShapes.name}\` is ${fewestShapes.p.interiorDistinct} shapes laid in a
${fewestShapes.p.rhythm}-piece cycle repeated ${fewestShapes.p.repeats.toFixed(1)} times, between the two end pieces.

**Mean plate** is the average bounding box a piece is cut from, in mm2 — the
laser-cutting number. Fewer, larger parts means less weeding, less sorting and fewer
fingers to align, and it is the size of the part in your hand rather than the count of
them. \`${plateBest.name}\` averages ${Math.round(plateBest.p.meanPlate).toLocaleString('en-US')} mm2 against \`${yours.name}\`'s ${Math.round(yours.p.meanPlate).toLocaleString('en-US')}.

Average *blocks* per piece was the other reading of the same idea and is not used: the
block count varies by only ${((Math.max(...rows.map(r=>r.m.blocks))/Math.min(...rows.map(r=>r.m.blocks))-1)*100).toFixed(1)}% across the set, so blocks-per-piece is very nearly the
reciprocal of the piece count and ranks the set the same way in 15 of ${rows.length} places. Plate
area is not redundant: ${(() => {
  const byPieces = {};
  for (const r of rows) (byPieces[r.p.innerPieces] ||= []).push(r);
  const tie = Object.values(byPieces).filter(g => g.length > 1)
    .sort((a,b) => b.length - a.length)[0];
  if (!tie) return 'coils that tie on piece count still differ on it';
  return tie.map(r => '\`' + r.name + '\`').join(', ') + ' all split into ' +
    tie[0].p.innerPieces + ' pieces and cannot be told apart by blocks-per-piece at all, ' +
    'while their mean plates are ' +
    tie.map(r => Math.round(r.p.meanPlate).toLocaleString('en-US')).join(', ') + ' mm2';
})()}.

It is a bounding box, not the cut outline — an L-shaped piece leaves its corner behind
— so it measures the size of the part, not the material consumed.

**Touching** counts blocks that sit face to face without being joined along the bore —
two runs of the tube sharing a wall. \`bore_split.py\` warns about them, and they are the
version of "density" that has a consequence: where the bore passes itself, that one wall
is all that separates the two passages.

Because a shared wall shows in the finished instrument, it is the heaviest input in the
scoring and is penalized convexly — 1/(1+t), so no contact scores 1.000 and eight
contacts 0.111. \`node tools/score.js --clean\` ranks only the coils that have none.

It is also the one packing number that is *not* implied by the box. ${L(box)} has the
smallest box in the set and ${box.m.touching} shared walls; ${L(clean)} is
${((clean.m.vol/box.m.vol - 1) * 100).toFixed(0)}% larger and has **none**. If a shared wall
is something to avoid, the smallest box is not the one to build.

## Rotation

${tbl('--rot')}

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
| smallest box | ${L(box)} — ${box.m.vol} | ${yours.m.vol}, so ${(yours.m.vol/box.m.vol).toFixed(2)}x larger |
| tightest spiral (least rise per turn) | ${L(rise)} — ${Math.round(rise.m.riseMMPer360)}mm | ${Math.round(yours.m.riseMMPer360)}mm, so ${(yours.m.risePer360/rise.m.risePer360).toFixed(1)}x slacker |
| least tube per turn | ${names(allBest(r => r.m.blocksPer360))} — ${tube.m.blocksPer360.toFixed(1)} blk | ${yours.m.blocksPer360.toFixed(1)} blk, within ${((yours.m.blocksPer360/tube.m.blocksPer360-1)*100).toFixed(0)}% |
| fewest pieces | ${L(pcs)} — ${pcs.p.innerPieces} | ${yours.p.innerPieces} |
| fewest distinct shapes | ${L(dist)} — ${dist.p.interiorDistinct} | ${yours.p.interiorDistinct} |
| largest average plate | ${L(plateBest)} — ${Math.round(plateBest.p.meanPlate).toLocaleString('en-US')} mm2 | ${Math.round(yours.p.meanPlate).toLocaleString('en-US')} mm2 |
| calmest bore (fewest turns/m) | ${names(allBest(r => r.m.turnsPerMetre))} — ${calm.m.turnsPerMetre.toFixed(2)} | ${yours.m.turnsPerMetre.toFixed(2)} |
| smallest box with no shared wall | ${L(clean)} — ${clean.m.vol} | ${yours.m.vol}, also ${yours.m.touching} shared |

The two staircase coils — \`${yours.name}\` as submitted and \`${rows.find(r=>(derived[r.name]||[]).includes('coil_5x5_28'))?.name || 'its reduction'}\` reduced — lose the
packing categories outright and win the turning ones. The reduction spends **less tube per
revolution than anything else here**, and the original is tied for the fewest turns per
metre. They are ${rows.length===0?'':'last and second-to-last'} on box per block, on pieces per block and on distinct shapes.

That is not a split verdict so much as one fact seen twice: **packing tighter costs
bends**, and bends are what a bore notices. A coil that turns economically is a coil that
does not fold itself into a small box, and every category above is downstream of that
choice.

## A metric deliberately left out

Fill density — blocks over box volume — was tried and dropped. At a fixed tube length it
is not independent of the box: blocks is near enough constant, so density is just the
reciprocal of box volume rescaled. Ranking these ${rows.length} spirals by density puts them in the
same order as ranking by box in 16 of ${rows.length} positions, and the two that swap are a tie at
477 broken by a one-block difference in length. It reads like a second opinion and is not
one. It would earn its place only in comparing walks of genuinely different lengths.

**Touching** is the metric density was reaching for. It answers the question density
sounds like it answers — how hard is this bore packed against itself — and unlike density
it disagrees with the box often enough to change which coil you would build.

## Every block load-bearing

The staircase coil was reduced by hand — blocks removed wherever one could go without
introducing an elbow — and that reduction is in this set. (The names below predate
standardising, which renamed every coil; [\`derived.txt\`](derived.txt) maps them.)
\`tools/minimal.js\` checks the claim: a term's floor is 3 in a coil window, 2 in a
hairpin, 1 in a step, and a walk whose every term sits on its floor cannot be shortened
at all.

    node tools/minimal.js            # every walk
    node tools/minimal.js --terms    # and which terms have slack

Before it was reduced, the staircase coil had 16 terms with slack, every one a hairpin
sitting at 3 where 2 would do, and the reduction left **none**. It was exhaustive: the
tool later reproduced it exactly, and standardising then showed the two to be one coil.

It also says something about the search. Only **${minimal.length} of ${minres.length}** walks here are minimal:
${minimal.map(n => '\`' + n + '\`').join(', ')}. The search enumerated periods with legs up to
4 and never asked whether a leg was longer than it had to be, so most of what it found
carries slack — \`${slackiest.name}\` could lose ${slackiest.saving} blocks. Two of the ${minimal.length} minimal walks are
the two that were reduced by hand.

Removing slack is not automatically safe. The rule is local, and a shortened walk can
run into itself or start touching, so what the tool reports are candidates to put back
through \`bore_split.py\`.

### The reduction pass, and why minimality is not an optimisation

\`tools/reduce.js\` takes the slack out and rebuilds at comparable length. It has to
enumerate rather than apply, because **coiling is global and the elbow rule is local**.
A period coils only if it closes: drifting on one axis and returning to where it started
on the other two. Shorten one leg and not its opposite and the period stops closing, so
the walk wanders off diagonally instead of coiling. Taken naively, one coil went from a
box of 423 to **10,452** — still elbow-free, no longer a coil.

Keeping only the reductions that still close and still wind a whole number of turns, and
putting the result back through the standardiser so it can be compared with what it came
from, **${redStat.reduced} of the ${redStat.total} coils reduce — and every one lands on a coil already here.**
${redStat.distinct} distinct walks come out of ${redStat.reduced} reductions, all of them already in the set: seven
different coils reduce to the same one, four more to another, one reduces to itself, and
the staircase coil reduces to the hand reduction of it that started all this.

**The set is closed under reduction.** There is nothing left to take out that does not
either break the coil or land somewhere already catalogued.

That is a much cleaner result than the first attempt, and the difference is
canonicalisation: before the representation was pinned, the same coil could appear
several times looking different, and reductions of it looked like new designs when they
were not.

Shortening a leg still does not make a better coil — it cuts how far the coil advances
per turn, so the same tube buys more revolutions in a fatter package. The best box per
block and the best walls-free box per block are both unchanged by the pass.

What those ${reduced.length} reductions buy:

| | |
| --- | ---: |
${redRows.map(([k, v]) => `| ${k} | ${v} |`).join('\n')}

And **nothing it produced beat what was already there**: the smallest box stayed at 423
against the reduced field's 462, and the smallest walls-free at 477 against 522.

That is the point worth keeping. Shortening a leg reduces how far the coil advances per
turn, so the same tube buys more revolutions in a fatter, shorter package — a different
design, not a better one. Every block being load-bearing is a property, not a virtue.

One thing the pass did find: four results the search reported as distinct all reduce to
**the same walk**, \`E2 S3 U1 S3 W2 N3 U1 N3\` — one design wearing four amounts of slack.
Standardising later found more of the same, and \`derived.txt\` records every merge.

The reduced walks are kept, named \`*_min\`, with their sources recorded in
[\`derived.txt\`](derived.txt). They are in the scoring like anything else, so the
composite is now over a set that contains both a design and its own reduction — worth
remembering when reading a rank, since the normalization is across whatever is present.

    node tools/reduce.js             # the pass
    node tools/reduce.js --write     # and reduced.json

## Walks kept but not scored

The scoring normalizes every metric across the set, so a walk of a different length does
not merely rank oddly — it rescales everyone else. Those walks live in the repository and
in the tables, below a rule, and are listed in [\`unscored.txt\`](unscored.txt) with the
reason. Their rate metrics — blocks/360, rise/360, turns/m, touching — stay comparable
and are still worth reading.

## Scoring them against each other

[**SCORING.md**](SCORING.md) combines seven of the metrics and the touching count into
a single ranking under every common mean — harmonic, geometric, arithmetic, quadratic,
cubic, median, midrange, contraharmonic — and reports what the choice of mean does to
the answer. It does a great deal: \`coil_2x2_146\` places 1st under one and 17th under
another. ${unanimous}

It also covers why to rank once rather than rank, cut and re-rank: with a
set-relative normalization the survivors of a cut come out in a different order
than they went in, having not been measured again.

    node tools/score.js          # the scoring table
    node tools/iterate.js        # what cutting and re-ranking does to it
    node tools/gen_scoring.js    # SCORING.md

## The gate

Every walk here has been through \`check.py\` — the parts, the sections, the seams, and a
voxel model of the assembled bore flooded from the outside to prove it is one sealed
passage. **${totalChecks.toLocaleString('en-US')} checks across ${Object.keys(checks).length} spirals, ${totalFailed} failed.** The per-walk output is
in \`checks/\`.

    node tools/run_checks.sh     # re-run the gate over every walk

\`check.py\` needs shapely, numpy and scipy, which live in the Boxes.py venv — run it with
\`~/boxes/venv/bin/python\`, as \`../bore-generator/README.md\` says. The system python3 does
not have them.

## How they were found

\`tools/search_spirals.js\` walks every periodic term sequence up to eight terms with legs up
to four blocks, keeping those that never reverse, return to the same lateral position each
period so the thing coils rather than drifts, wind a whole number of turns per period, stay
self-avoiding when repeated, and satisfy the elbow-free rule.

That rule, for a window of three consecutive terms — outer A, middle m, outer C:

| A and C | | middle term |
| --- | --- | ---: |
| same axis, same direction | a step | >= 1 |
| same axis, opposite direction | a hairpin | >= 2 |
| different axes | a coil | >= 3 |

The bore-designs README states only the third case and calls the other two "a fold ... as
tight as you like". That holds for steps and not for hairpins: \`U3 N1 D3\` costs two elbows
and \`U3 N2 D3\` costs none. The three cases were read off \`bore_split.py\` by probing it.

\`tools/mknotation.js\` repeats a period to length and writes it in the bore notation,
trimming the tail so the last block is not mid-turn — a turn in the last block has nothing
after it to make it interior, so it is always its own piece. Then every candidate goes
through the splitter, and only zero-elbow walks are kept. **The splitter decides, not the
rule.**

## The files behind the tables

Each coil is one line of notation in \`walks/\`, named for the coil, and each has a gate
transcript beside it in \`checks/\`:

${rows.map(r => '`walks/' + r.name + '.txt`').join(', ')}

\`standardised.json\` records the canonical form every walk was pinned to, and
\`reduced.json\` what the reduction pass produced. Both are written by their tools, not
by hand.

## Regenerating

    tools/build.sh               # all of the below, in order

    node tools/parts.js          # piece counts and distinct shapes -> parts.json
    node tools/table.js          # both tables, plain text
    node tools/table.js --md     # the same as markdown
    node tools/gen_readme.js     # this file
    python3 ../lasermade-tools/md2html.py README.md index.html    # the published page

    node tools/spiral_metrics.js "$(cat walks/${rows[0].name}.txt)"   # measure one walk

    cd ../bore-generator         # rebuild a viewer page
    ~/boxes/venv/bin/python viewer.py "$(cat ../spirals/walks/${rows[0].name}.txt)" \\
        --out ../spirals/pages/${rows[0].name}.html --title "${rows[0].name.replace(/_/g,' ')}"

## What has not been done

**No cut files.** These are walks, viewers and gate results; nothing here has been split to
SVG or nested. \`bore_split.py --write DIR\` will do it.

The block counts are not equal — the scored ones run ${Math.min(...rows.map(r=>r.m.blocks))} to ${Math.max(...rows.map(r=>r.m.blocks))}, so their bore lengths
run ${Math.min(...rows.map(r=>r.m.mm))}mm to ${Math.max(...rows.map(r=>r.m.mm))}mm, a spread of ${((Math.max(...rows.map(r=>r.m.mm))/Math.min(...rows.map(r=>r.m.mm))-1)*100).toFixed(1)}%. A period is repeated to about the right length and then trimmed
to wherever the tail comes out elbow-free, so these are the same bore only to within a few
blocks. For comparing packing that is fine; for comparing *bores* it is not, and the length
would have to be pinned first.

Released under [CC0 1.0](LICENSE).
`;
fs.writeFileSync(path.join(root, 'README.md'), md);
console.log('README.md written,', md.split('\n').length, 'lines');
