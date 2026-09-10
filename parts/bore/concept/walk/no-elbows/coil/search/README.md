# Spirals

Coiling bore walks that turn about an axis while advancing along it, collected with the
numbers that say how hard each one spirals and what each one costs to build. Every walk
here splits with **no elbows** — every turn folds into a piece as an L, none is stranded
as a single-block piece of its own — and every one passes the full gate.

The one this started from is [`coil_3x8_20`](pages/coil_3x8_20.html); the other 9 came out of an
exhaustive search for something tighter.

<!-- readme-only -->
**[Read it as a page](https://gernreich.github.io/trumpet/)** — the trumpet writeup.
Every per-design page in that repository was removed on 2026-09-05, so this is
where the reading version lives.

**[Download the whole repository as a ZIP](https://github.com/Gernreich/trumpet/archive/refs/heads/main.zip)**
— every trumpet and every tool, not the walks alone; these pages are under `spirals/`.
GitHub builds it from `main` on every push, so it is never out of date.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
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
bounded by the mouth and the exit rather than by a neighbour, and the notation says as
much: the first term is only the way you came in. What is standardised is that every
walk has exactly one of each, and the judged metrics ignore them regardless.

Fixing forward and the sense of rotation is not enough on its own. The four rotations
about the forward axis all satisfy both, and so does every rotation of the cycle that
opens on a north term, so taking the first representation that fits leaves the same coil
able to appear more than once looking different — and it did, four times over in one
case. Pinning the representation to the lexicographically smallest is what makes
`distinct` mean anything.

Standardising collapsed 30 walks into 10: several that looked different were the same
coil in another orientation, and [`derived.txt`](derived.txt) records which. The hand
reduction of the staircase coil, the tool's reduction of it, and the extension of the
hand reduction all turn out to be one coil.

### The length rule, and why it is not a cap

Whole periods of different lengths cannot all reach the same total, so some rule has to
choose the repeat counts. The first one tried — as many periods as fit under a limit —
is the wrong shape: it truncates the long-period coils hardest, so every one of them
lands below every short-period one, and the bores ran 126 to 144 blocks, a 14.3% spread.

The target is now chosen rather than assumed. `tools/standardise.js` searches bore
lengths from 120 to 220 blocks for the one that makes the tube lengths most alike, takes
the **nearest** whole-period count to it rather than the largest that fits, and lands on
180: bores of 177 to 193 blocks, 2832mm to 3088mm, a spread of 9.0%.

That is as close as this set can get at a plausible size. Squeezing the spread below 10%
needs a seven-metre bore and below 5% needs twelve, which is not a trumpet.

So the residual is handled where it actually bites, in the scoring: **box and piece count
are scored per block**, because both grow with tube and comparing them absolutely would
hand the shorter coils an advantage they did nothing to earn. Over this set piece count
correlates 0.63 with block count. Cross-section, mean plate, distinct shapes and the
rates are length-independent already; touching stays an absolute count, because the
requirement is none of it at any length.

## Shape and cost

| spiral | blocks | mm | envelope | box | cross-section | along axis | pieces | distinct | period | rhythm | mean plate mm2 | touching |
| --- | ---: | ---: | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [`coil_3x8_20`](pages/coil_3x8_20.html) | 181 | 2896 | 3 x 8 x 18 | 432 | 3 x 8 blk / 48 x 128mm | 18 / 288mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 2193 | 15 |
| [`coil_3x9_18`](pages/coil_3x9_18.html) | 177 | 2832 | 3 x 9 x 16 | 432 | 3 x 9 blk / 48 x 144mm | 16 / 256mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 2473 | 13 |
| [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) | 177 | 2832 | 3 x 3 x 49 | 441 | 3 x 3 blk / 48 x 48mm | 49 / 784mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 2601 | 0 |
| [`coil_3x3_59`](pages/coil_3x3_59.html) | 181 | 2896 | 3 x 3 x 54 | 486 | 3 x 3 blk / 48 x 48mm | 54 / 864mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 2193 | 0 |
| [`coil_4x7_20`](pages/coil_4x7_20.html) | 181 | 2896 | 4 x 7 x 18 | 504 | 4 x 7 blk / 64 x 112mm | 18 / 288mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 2065 | 15 |
| [`coil_4x8_18`](pages/coil_4x8_18.html) | 177 | 2832 | 4 x 8 x 16 | 512 | 4 x 8 blk / 64 x 128mm | 16 / 256mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 2446 | 13 |
| [`coil_5x7_18`](pages/coil_5x7_18.html) | 177 | 2832 | 5 x 7 x 16 | 560 | 5 x 7 blk / 80 x 112mm | 16 / 256mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 2345 | 13 |
| [`coil_4x9_18`](pages/coil_4x9_18.html) | 193 | 3088 | 4 x 9 x 16 | 576 | 4 x 9 blk / 64 x 144mm | 16 / 256mm | 30 | 4 | 8 tm / 24 blk | 4 x7.5 | 2881 | 13 |
| [`coil_3x4_68`](pages/coil_3x4_68.html) | 177 | 2832 | 4 x 3 x 63 | 756 | 3 x 4 blk / 48 x 64mm | 63 / 1008mm | 41 | 4 | 6 tm / 16 blk | 4 x10.3 | 1697 | 0 |
| [`coil_3x4_79`](pages/coil_3x4_79.html) | 188 | 3008 | 4 x 3 x 73 | 876 | 3 x 4 blk / 48 x 64mm | 73 / 1168mm | 43 | 4 | 8 tm / 17 blk | 4 x10.8 | 1878 | 0 |

Envelope, box and cross-section are in blocks; a block is 16mm of centreline. Sorted by
box, smallest first.

**The bore's mouth and exit are not judged.** Every design has them, no design chooses
them, and they are the two pieces that never join the rhythm — so every column here
except blocks and mm is measured on the interior, the pieces in between. It is not a
cosmetic change: the ends stick out of the envelope they bracket. `coil_3x8_20` needs 4 shapes
rather than 6, the difference being end pieces alone. Blocks and mm still describe the whole bore, because that is what the bore is.

**Cross-section** is the envelope with the coil axis taken out — how fat the coil is,
which is what decides whether it fits inside anything. It is not implied by the box: [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) is
3x3 in section and [`coil_4x9_18`](pages/coil_4x9_18.html) is 4x9, and the box does not say so.

**Distinct** is how many different piece shapes the cut list holds. A coil built by
repeating one period needs only a handful, however long it runs: coil_3x8_20 is
36 pieces cut from 6 shapes. The widest needs 4. That is files to check and parts to tell apart on the bench, and it
does not show up in the piece count at all.

**Period** is the repeating unit of the walk itself, in terms and in blocks, and it sits
next to the rhythm because the two together say something neither says alone.

**A piece is a flat snake, so the splitter has to start a new one exactly where the bore
leaves its plane.** Turns that stay in-plane — the folds, hairpins and steps both —
happen inside a piece and cost no boundary at all. So the rhythm is not counting turns,
it is counting departures from the plane, and it comes out as

    rhythm = period terms x the share of turns that leave the plane

exactly, on every coil here. That decomposition is why the period column earns its
place: a rhythm of 4 could be a long period turning gently or a short one leaving the
plane at every chance, and the rhythm alone cannot tell you which. In this set the
9 eight-term coils leave the plane on half their turns, while the 0 sixteen-term ones do it on
three turns in four — so the gap in rhythm is two separate factors multiplying, not one.

The share itself is **not** scored, and was tested rather than assumed: it takes only
three values across the set, 9 of the 10 coils sit on exactly one of them, and it
correlates 0.89 with the rhythm, 0.85 with distinct shapes and 0.76 with box per block.
Near-constant and not independent — the same double failure that kept fill density out.

**Rhythm** is how many pieces you lay before the cut list starts over, and how many
times it recurs. It is descriptive, not scored: across coils built by repeating a period
it barely varies — 10 of the 10 here are a 4-piece rhythm — and a near-constant metric
in a mean only dilutes the ones that discriminate. It is worth knowing at the bench all
the same, and it is not the same thing as **distinct**: the two rank the set alike in
only 6 of 10 places. `coil_3x8_20` is 4 shapes laid in a
4-piece cycle repeated 8.5 times, between the two end pieces.

**Mean plate** is the average bounding box a piece is cut from, in mm2 — the
laser-cutting number. Fewer, larger parts means less weeding, less sorting and fewer
fingers to align, and it is the size of the part in your hand rather than the count of
them. `coil_4x9_18` averages 2,881 mm2 against `coil_3x8_20`'s 2,193.

Average *blocks* per piece was the other reading of the same idea and is not used: the
block count varies by only 11.0% across the set, so blocks-per-piece is very nearly the
reciprocal of the piece count and ranks the set the same way in 15 of 10 places. Plate
area is not redundant: `coil_3x9_18`, `coil_3x3_54_2`, `coil_4x8_18`, `coil_5x7_18`, `coil_4x9_18` all split into 30 pieces and cannot be told apart by blocks-per-piece at all, while their mean plates are 2,473, 2,601, 2,446, 2,345, 2,881 mm2.

It is a bounding box, not the cut outline — an L-shaped piece leaves its corner behind
— so it measures the size of the part, not the material consumed.

**Touching** counts blocks that sit face to face without being joined along the bore —
two runs of the tube sharing a wall. `bore_split.py` warns about them, and they are the
version of "density" that has a consequence: where the bore passes itself, that one wall
is all that separates the two passages.

Because a shared wall shows in the finished instrument, it is the heaviest input in the
scoring and is penalized convexly — 1/(1+t), so no contact scores 1.000 and eight
contacts 0.111. `node tools/score.js --clean` ranks only the coils that have none.

It is also the one packing number that is *not* implied by the box. [`coil_3x8_20`](pages/coil_3x8_20.html) has the
smallest box in the set and 15 shared walls; [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) is
2% larger and has **none**. If a shared wall
is something to avoid, the smallest box is not the one to build.

## Rotation

| spiral | axis | rotation | turns | blocks/360 | rise/360 blk / mm | deg/block | 90deg turns | turns/m | longest straight |
| --- | :-: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [`coil_3x8_20`](pages/coil_3x8_20.html) | N | 3060 | 8.50 | 20.0 | 2.00 / 32 | 18.0 | 68 | 25.0 | 4 blk / 64mm |
| [`coil_3x9_18`](pages/coil_3x9_18.html) | N | 2700 | 7.50 | 22.0 | 2.00 / 32 | 16.4 | 60 | 22.7 | 4 blk / 64mm |
| [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) | N | 2610 | 7.25 | 22.8 | 6.62 / 106 | 15.8 | 60 | 22.7 | 4 blk / 64mm |
| [`coil_3x3_59`](pages/coil_3x3_59.html) | N | 2970 | 8.25 | 20.6 | 6.42 / 103 | 17.5 | 68 | 25.0 | 3 blk / 48mm |
| [`coil_4x7_20`](pages/coil_4x7_20.html) | N | 3060 | 8.50 | 20.0 | 2.00 / 32 | 18.0 | 68 | 25.0 | 3 blk / 48mm |
| [`coil_4x8_18`](pages/coil_4x8_18.html) | N | 2700 | 7.50 | 21.9 | 2.00 / 32 | 16.5 | 60 | 22.9 | 4 blk / 64mm |
| [`coil_5x7_18`](pages/coil_5x7_18.html) | N | 2700 | 7.50 | 22.0 | 2.00 / 32 | 16.4 | 60 | 22.7 | 4 blk / 64mm |
| [`coil_4x9_18`](pages/coil_4x9_18.html) | N | 2700 | 7.50 | 24.0 | 2.00 / 32 | 15.0 | 60 | 20.8 | 4 blk / 64mm |
| [`coil_3x4_68`](pages/coil_3x4_68.html) | N | 3600 | 10.00 | 16.3 | 6.20 / 99 | 22.1 | 61 | 23.4 | 3 blk / 48mm |
| [`coil_3x4_79`](pages/coil_3x4_79.html) | N | 3780 | 10.50 | 17.2 | 6.86 / 110 | 20.9 | 84 | 29.0 | 3 blk / 48mm |

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
| smallest box | [`coil_3x8_20`](pages/coil_3x8_20.html) — 432 | 432, so 1.00x larger |
| tightest spiral (least rise per turn) | [`coil_3x8_20`](pages/coil_3x8_20.html) — 32mm | 32mm, so 1.0x slacker |
| least tube per turn | [`coil_3x4_68`](pages/coil_3x4_68.html) — 16.3 blk | 20.0 blk, within 23% |
| fewest pieces | [`coil_3x9_18`](pages/coil_3x9_18.html) — 30 | 34 |
| fewest distinct shapes | [`coil_3x8_20`](pages/coil_3x8_20.html) — 4 | 4 |
| largest average plate | [`coil_4x9_18`](pages/coil_4x9_18.html) — 2,881 mm2 | 2,193 mm2 |
| calmest bore (fewest turns/m) | [`coil_4x9_18`](pages/coil_4x9_18.html) — 20.83 | 25.00 |
| smallest box with no shared wall | [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) — 441 | 432, also 15 shared |

The two staircase coils — `coil_3x8_20` as submitted and `its reduction` reduced — lose the
packing categories outright and win the turning ones. The reduction spends **less tube per
revolution than anything else here**, and the original is tied for the fewest turns per
metre. They are last and second-to-last on box per block, on pieces per block and on distinct shapes.

That is not a split verdict so much as one fact seen twice: **packing tighter costs
bends**, and bends are what a bore notices. A coil that turns economically is a coil that
does not fold itself into a small box, and every category above is downstream of that
choice.

## A metric deliberately left out

Fill density — blocks over box volume — was tried and dropped. At a fixed tube length it
is not independent of the box: blocks is near enough constant, so density is just the
reciprocal of box volume rescaled. Ranking these 10 spirals by density puts them in the
same order as ranking by box in 16 of 10 positions, and the two that swap are a tie at
477 broken by a one-block difference in length. It reads like a second opinion and is not
one. It would earn its place only in comparing walks of genuinely different lengths.

**Touching** is the metric density was reaching for. It answers the question density
sounds like it answers — how hard is this bore packed against itself — and unlike density
it disagrees with the box often enough to change which coil you would build.

## Every block load-bearing

The staircase coil was reduced by hand — blocks removed wherever one could go without
introducing an elbow — and that reduction is in this set. (The names below predate
standardising, which renamed every coil; [`derived.txt`](derived.txt) maps them.)
`tools/minimal.js` checks the claim: a term's floor is 3 in a coil window, 2 in a
hairpin, 1 in a step, and a walk whose every term sits on its floor cannot be shortened
at all.

    node tools/minimal.js            # every walk
    node tools/minimal.js --terms    # and which terms have slack

Before it was reduced, the staircase coil had 16 terms with slack, every one a hairpin
sitting at 3 where 2 would do, and the reduction left **none**. It was exhaustive: the
tool later reproduced it exactly, and standardising then showed the two to be one coil.

It also says something about the search. Only **1 of 10** walks here are minimal:
`coil_3x4_79`. The search enumerated periods with legs up to
4 and never asked whether a leg was longer than it had to be, so most of what it found
carries slack — `coil_4x9_18` could lose 48 blocks. Two of the 1 minimal walks are
the two that were reduced by hand.

Removing slack is not automatically safe. The rule is local, and a shortened walk can
run into itself or start touching, so what the tool reports are candidates to put back
through `bore_split.py`.

### The reduction pass, and why minimality is not an optimisation

`tools/reduce.js` takes the slack out and rebuilds at comparable length. It has to
enumerate rather than apply, because **coiling is global and the elbow rule is local**.
A period coils only if it closes: drifting on one axis and returning to where it started
on the other two. Shorten one leg and not its opposite and the period stops closing, so
the walk wanders off diagonally instead of coiling. Taken naively, one coil went from a
box of 423 to **10,452** — still elbow-free, no longer a coil.

Keeping only the reductions that still close and still wind a whole number of turns, and
putting the result back through the standardiser so it can be compared with what it came
from, **9 of the 10 coils reduce — and every one lands on a coil already here.**
3 distinct walks come out of 9 reductions, all of them already in the set: seven
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

What those 9 reductions buy:

| | |
| --- | ---: |
| box smaller | 4 |
| box bigger | 5 |
| box unchanged | 0 |
| touching reduced | 0 |
| touching increased | 6 |
| touching unchanged | 3 |

And **nothing it produced beat what was already there**: the smallest box stayed at 423
against the reduced field's 462, and the smallest walls-free at 477 against 522.

That is the point worth keeping. Shortening a leg reduces how far the coil advances per
turn, so the same tube buys more revolutions in a fatter, shorter package — a different
design, not a better one. Every block being load-bearing is a property, not a virtue.

One thing the pass did find: four results the search reported as distinct all reduce to
**the same walk**, `E2 S3 U1 S3 W2 N3 U1 N3` — one design wearing four amounts of slack.
Standardising later found more of the same, and `derived.txt` records every merge.

The reduced walks are kept, named `*_min`, with their sources recorded in
[`derived.txt`](derived.txt). They are in the scoring like anything else, so the
composite is now over a set that contains both a design and its own reduction — worth
remembering when reading a rank, since the normalization is across whatever is present.

    node tools/reduce.js             # the pass
    node tools/reduce.js --write     # and reduced.json

## Walks kept but not scored

The scoring normalizes every metric across the set, so a walk of a different length does
not merely rank oddly — it rescales everyone else. Those walks live in the repository and
in the tables, below a rule, and are listed in [`unscored.txt`](unscored.txt) with the
reason. Their rate metrics — blocks/360, rise/360, turns/m, touching — stay comparable
and are still worth reading.

## Scoring them against each other

[**SCORING.md**](SCORING.md) combines eight of the metrics and the touching count into
a single ranking under every common mean — harmonic, geometric, arithmetic, quadratic,
cubic, median, midrange, contraharmonic — and reports what the choice of mean does to
the answer. It does a great deal: `coil_2x2_146` places 1st under one and 17th under
another. `coil_3x3_54_2` comes first under 8 of the 8 means.

It also covers why to rank once rather than rank, cut and re-rank: with a
set-relative normalization the survivors of a cut come out in a different order
than they went in, having not been measured again.

    node tools/score.js          # the scoring table
    node tools/iterate.js        # what cutting and re-ranking does to it
    node tools/gen_scoring.js    # SCORING.md

## The gate

Every walk here has been through `check.py` — the parts, the sections, the seams, and a
voxel model of the assembled bore flooded from the outside to prove it is one sealed
passage. **11,872 checks across 10 spirals, 0 failed.** The per-walk output is
in `checks/`.

    node tools/run_checks.sh     # re-run the gate over every walk

`check.py` needs shapely, numpy and scipy, which live in the Boxes.py venv — run it with
`~/Software/boxes/venv/bin/python`, as `README.md` says. The system python3 does
not have them.

## How they were found

`tools/search_spirals.js` walks every periodic term sequence up to eight terms with legs up
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
tight as you like". That holds for steps and not for hairpins: `U3 N1 D3` costs two elbows
and `U3 N2 D3` costs none. The three cases were read off `bore_split.py` by probing it.

`tools/mknotation.js` repeats a period to length and writes it in the bore notation,
trimming the tail so the last block is not mid-turn — a turn in the last block has nothing
after it to make it interior, so it is always its own piece. Then every candidate goes
through the splitter, and only zero-elbow walks are kept. **The splitter decides, not the
rule.**

## The tools

Every file in `tools/`, and what each one is for -- all fourteen of them.
The list is read from the directory rather than typed into this page, so it cannot fall
behind it: a tool with no description, or a description with no tool, stops the build.
There was no such list until 2026-09-10, and none of these was described anywhere, here
or elsewhere. This file named some of them in passing but never said what they were, and
a passing mention is all the orphan check asks for.

**Finding and shaping walks**

| tool | what it does |
| --- | --- |
| `search_spirals.js` | Searches for the tightest elbow-free coil. Applies the corpus rule -- three consecutive terms naming three axes force the middle term to 3 or more -- then lets the splitter decide, because the rule is a filter and not the answer. |
| `mknotation.js` | Expands one period to about 196 blocks and writes it in the corpus notation: bare lead-in term, numbered middle terms, bare lead-out term. |
| `standardise.js` | Puts every coil in one orientation -- north for all of them, opening on a north term -- so that two coils differ only where they really differ. Writes `standardised.json`. |
| `minimal.js` | Asks whether a walk can be shortened without introducing an elbow, against a purely local rule: a term's floor is set by the window of three around it. Reports slack; does not take it. |
| `reduce.js` | Takes that slack, one leg at a time, keeping the coil. Taking all of it at once usually destroys the walk, which is why this is separate from `minimal.js`. Writes `reduced.json`. |

**Measuring and ranking**

| tool | what it does |
| --- | --- |
| `spiral_metrics.js` | Rotation metrics for one walk. The lateral projection -- the walk with the advancing axis dropped -- is what actually turns, and on a cubic lattice only in quarter turns. |
| `parts.js` | Piece counts, distinct piece shapes and plate sizes, read off `bore_split.py` and cached in `parts.json` so the tables need not shell out on every run. |
| `score.js` | Composite scoring across the common means: eight metrics normalised to (0,1], plus the touching count at an explicit weight. |
| `iterate.js` | Iterated ranking -- rank, cut the bottom half, re-rank the survivors, repeat -- to ask whether the survivors keep their order once the losers leave. They do not, under a normalisation computed over the set. |
| `table.js` | The metrics tables, as plain text or with `--md` as markdown. |

**Producing what is committed**

| tool | what it does |
| --- | --- |
| `run_checks.sh` | Runs `check.py` over every walk in `walks/` and writes the transcript to `checks/`. It resolves the Boxes.py checkout and the interpreter *before* the loop and stops if either is missing, because a missing interpreter captured with `2>&1` writes the shell's error into the transcript instead of the check, and the tally then reads as a pass. |
| `gen_scoring.js` | Regenerates `SCORING.md`. Every number in it comes from `score.js`. |
| `gen_readme.js` | Regenerates this file. Every number in it comes from the tools and the index above is read from `tools/` itself, so the page cannot drift. |
| `build.sh` | All of the above that produce committed files, in dependency order. `index.html` and `SCORING.html` are committed rather than built on the server, so they go stale silently unless this is run after every edit. |

## The files behind the tables

Each coil is one line of notation in `walks/`, named for the coil, and each has a gate
transcript beside it in `checks/`:

`walks/coil_3x8_20.txt`, `walks/coil_3x9_18.txt`, `walks/coil_3x3_54_2.txt`, `walks/coil_3x3_59.txt`, `walks/coil_4x7_20.txt`, `walks/coil_4x8_18.txt`, `walks/coil_5x7_18.txt`, `walks/coil_4x9_18.txt`, `walks/coil_3x4_68.txt`, `walks/coil_3x4_79.txt`

`standardised.json` records the canonical form every walk was pinned to, and
`reduced.json` what the reduction pass produced. Both are written by their tools, not
by hand.

## Regenerating

    tools/build.sh               # all of the below, in order

    node tools/parts.js          # piece counts and distinct shapes -> parts.json
    node tools/table.js          # both tables, plain text
    node tools/table.js --md     # the same as markdown
    node tools/gen_readme.js     # this file
    python3 ../../../../../../../../lasermade-tools/md2html.py README.md index.html    # the published page

    node tools/spiral_metrics.js "$(cat walks/coil_3x8_20.txt)"   # measure one walk

    ~/Software/boxes/venv/bin/python ../../../../../../../tools/viewer.py \
        "$(cat walks/coil_3x8_20.txt)" \
        --out pages/coil_3x8_20.html --title "coil 3x8 20"

## What has not been done

**No cut files.** These are walks, viewers and gate results; nothing here has been split to
SVG or nested. `bore_split.py --write DIR` will do it.

The block counts are not equal — the scored ones run 163 to 181, so their bore lengths
run 2608mm to 2896mm, a spread of 11.0%. A period is repeated to about the right length and then trimmed
to wherever the tail comes out elbow-free, so these are the same bore only to within a few
blocks. For comparing packing that is fine; for comparing *bores* it is not, and the length
would have to be pinned first.

Released under [CC0 1.0](LICENSE).
