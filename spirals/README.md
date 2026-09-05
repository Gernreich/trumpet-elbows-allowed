# Spirals

Coiling bore walks that turn about an axis while advancing along it, collected with the
numbers that say how hard each one spirals and what each one costs to build. Every walk
here splits with **no elbows** — every turn folds into a piece as an L, none is stranded
as a single-block piece of its own — and every one passes the full gate.

The one this started from is [`coil_4x4_50`](pages/coil_4x4_50.html); the other 16 came out of an
exhaustive search for something tighter.

<!-- readme-only -->
**[Read it as a page](https://gernreich.github.io/trumpet/spirals/)** — the same text set for
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
`distinct` mean anything.

Standardising collapsed 30 walks into 17: several that looked different were the same
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
180: bores of 177 to 193 blocks, 5487mm to 5983mm, a spread of 9.0%.

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
| [`coil_3x3_51`](pages/coil_3x3_51.html) | 190 | 5890 | 3 x 3 x 46 | 414 | 3 x 3 blk / 93 x 93 mm | 46 / 1426 mm | 34 | 4 | 8 tm / 21 blk | 4 x8.5 | 8428 | 7 |
| [`coil_3x7_22`](pages/coil_3x7_22.html) | 181 | 5611 | 3 x 7 x 20 | 420 | 3 x 7 blk / 93 x 217 mm | 20 / 620 mm | 38 | 4 | 8 tm / 18 blk | 4 x9.5 | 6240 | 17 |
| [`coil_3x8_20`](pages/coil_3x8_20.html) | 181 | 5611 | 3 x 8 x 18 | 432 | 3 x 8 blk / 93 x 248 mm | 18 / 558 mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 7728 | 15 |
| [`coil_3x9_18`](pages/coil_3x9_18.html) | 177 | 5487 | 3 x 9 x 16 | 432 | 3 x 9 blk / 93 x 279 mm | 16 / 496 mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 8736 | 13 |
| [`coil_3x3_54`](pages/coil_3x3_54.html) | 177 | 5487 | 3 x 3 x 49 | 441 | 3 x 3 blk / 93 x 93 mm | 49 / 1519 mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 9216 | 0 |
| [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) | 177 | 5487 | 3 x 3 x 49 | 441 | 3 x 3 blk / 93 x 93 mm | 49 / 1519 mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 9216 | 0 |
| [`coil_3x3_59`](pages/coil_3x3_59.html) | 181 | 5611 | 3 x 3 x 54 | 486 | 3 x 3 blk / 93 x 93 mm | 54 / 1674 mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 7728 | 0 |
| [`coil_2x2_134`](pages/coil_2x2_134.html) | 177 | 5487 | 2 x 2 x 126 | 504 | 2 x 2 blk / 62 x 62 mm | 126 / 3906 mm | 42 | 2 | 8 tm / 16 blk | 4 x10.5 | 6240 | 0 |
| [`coil_4x7_20`](pages/coil_4x7_20.html) | 181 | 5611 | 4 x 7 x 18 | 504 | 4 x 7 blk / 124 x 217 mm | 18 / 558 mm | 34 | 4 | 8 tm / 20 blk | 4 x8.5 | 7248 | 15 |
| [`coil_4x8_18`](pages/coil_4x8_18.html) | 177 | 5487 | 4 x 8 x 16 | 512 | 4 x 8 blk / 124 x 248 mm | 16 / 496 mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 8636 | 13 |
| [`coil_5x7_18`](pages/coil_5x7_18.html) | 177 | 5487 | 5 x 7 x 16 | 560 | 5 x 7 blk / 155 x 217 mm | 16 / 496 mm | 30 | 4 | 8 tm / 22 blk | 4 x7.5 | 8255 | 13 |
| [`coil_4x9_18`](pages/coil_4x9_18.html) | 193 | 5983 | 4 x 9 x 16 | 576 | 4 x 9 blk / 124 x 279 mm | 16 / 496 mm | 30 | 4 | 8 tm / 24 blk | 4 x7.5 | 10224 | 13 |
| [`coil_5x8_18`](pages/coil_5x8_18.html) | 193 | 5983 | 5 x 8 x 16 | 640 | 5 x 8 blk / 155 x 248 mm | 16 / 496 mm | 30 | 4 | 8 tm / 24 blk | 4 x7.5 | 10704 | 13 |
| [`coil_4x4_50`](pages/coil_4x4_50.html) | 193 | 5983 | 4 x 4 x 46 | 736 | 4 x 4 blk / 124 x 124 mm | 46 / 1426 mm | 45 | 8 | 16 tm / 48 blk | 12 x3.8 | 5568 | 0 |
| [`coil_3x4_68`](pages/coil_3x4_68.html) | 177 | 5487 | 4 x 3 x 63 | 756 | 3 x 4 blk / 93 x 124 mm | 63 / 1953 mm | 41 | 4 | 6 tm / 16 blk | 4 x10.3 | 5934 | 0 |
| [`coil_3x4_79`](pages/coil_3x4_79.html) | 188 | 5828 | 4 x 3 x 73 | 876 | 3 x 4 blk / 93 x 124 mm | 73 / 2263 mm | 43 | 4 | 8 tm / 17 blk | 4 x10.8 | 6593 | 0 |
| [`coil_5x5_50`](pages/coil_5x5_50.html) | 177 | 5487 | 5 x 5 x 46 | 1150 | 5 x 5 blk / 155 x 155 mm | 46 / 1426 mm | 45 | 8 | 16 tm / 44 blk | 12 x3.8 | 4897 | 0 |

Envelope, box and cross-section are in blocks; a block is 31mm of centreline. Sorted by
box, smallest first.

**The bore's mouth and exit are not judged.** Every design has them, no design chooses
them, and they are the two pieces that never join the rhythm — so every column here
except blocks and mm is measured on the interior, the pieces in between. It is not a
cosmetic change: the ends stick out of the envelope they bracket. `coil_3x3_51` needs 4 shapes
rather than 6, the difference being end pieces alone. Blocks and mm still describe the whole bore, because that is what the bore is.

**Cross-section** is the envelope with the coil axis taken out — how fat the coil is,
which is what decides whether it fits inside anything. It is not implied by the box: [`coil_2x2_134`](pages/coil_2x2_134.html) is
2x2 in section and [`coil_5x8_18`](pages/coil_5x8_18.html) is 5x8, and the box does not say so.

**Distinct** is how many different piece shapes the cut list holds. A coil built by
repeating one period needs only a handful, however long it runs: coil_2x2_134 is
44 pieces cut from 4 shapes. The widest needs 8. That is files to check and parts to tell apart on the bench, and it
does not show up in the piece count at all.

**Period** is the repeating unit of the walk itself, in terms and in blocks, and it sits
next to the rhythm because the two together say something neither says alone.

**A piece is a flat snake, so the splitter has to start a new one exactly where the bore
leaves its plane.** Turns that stay in-plane — the folds, hairpins and steps both —
happen inside a piece and cost no boundary at all. So the rhythm is not counting turns,
it is counting departures from the plane, and it comes out as

    rhythm = period terms x the share of turns that leave the plane

exactly, on every coil here. That decomposition is why the period column earns its
place: a rhythm of 12 could be a long period turning gently or a short one leaving the
plane at every chance, and the rhythm alone cannot tell you which. In this set the
14 eight-term coils leave the plane on half their turns, while the 2 sixteen-term ones do it on
three turns in four — so the gap in rhythm is two separate factors multiplying, not one.

The share itself is **not** scored, and was tested rather than assumed: it takes only
three values across the set, 14 of the 17 coils sit on exactly one of them, and it
correlates 0.89 with the rhythm, 0.85 with distinct shapes and 0.76 with box per block.
Near-constant and not independent — the same double failure that kept fill density out.

**Rhythm** is how many pieces you lay before the cut list starts over, and how many
times it recurs. It is descriptive, not scored: across coils built by repeating a period
it barely varies — 15 of the 17 here are a 4-piece rhythm — and a near-constant metric
in a mean only dilutes the ones that discriminate. It is worth knowing at the bench all
the same, and it is not the same thing as **distinct**: the two rank the set alike in
only 6 of 17 places. `coil_2x2_134` is 2 shapes laid in a
4-piece cycle repeated 10.5 times, between the two end pieces.

**Mean plate** is the average bounding box a piece is cut from, in mm2 — the
laser-cutting number. Fewer, larger parts means less weeding, less sorting and fewer
fingers to align, and it is the size of the part in your hand rather than the count of
them. `coil_5x8_18` averages 10,704 mm2 against `coil_4x4_50`'s 5,568.

Average *blocks* per piece was the other reading of the same idea and is not used: the
block count varies by only 11.0% across the set, so blocks-per-piece is very nearly the
reciprocal of the piece count and ranks the set the same way in 15 of 17 places. Plate
area is not redundant: `coil_3x9_18`, `coil_3x3_54`, `coil_3x3_54_2`, `coil_4x8_18`, `coil_5x7_18`, `coil_4x9_18`, `coil_5x8_18` all split into 30 pieces and cannot be told apart by blocks-per-piece at all, while their mean plates are 8,736, 9,216, 9,216, 8,636, 8,255, 10,224, 10,704 mm2.

It is a bounding box, not the cut outline — an L-shaped piece leaves its corner behind
— so it measures the size of the part, not the material consumed.

**Touching** counts blocks that sit face to face without being joined along the bore —
two runs of the tube sharing a wall. `bore_split.py` warns about them, and they are the
version of "density" that has a consequence: where the bore passes itself, that one wall
is all that separates the two passages.

Because a shared wall shows in the finished instrument, it is the heaviest input in the
scoring and is penalized convexly — 1/(1+t), so no contact scores 1.000 and eight
contacts 0.111. `node tools/score.js --clean` ranks only the coils that have none.

It is also the one packing number that is *not* implied by the box. [`coil_3x3_51`](pages/coil_3x3_51.html) has the
smallest box in the set and 7 shared walls; [`coil_3x3_54`](pages/coil_3x3_54.html) is
7% larger and has **none**. If a shared wall
is something to avoid, the smallest box is not the one to build.

## Rotation

| spiral | axis | rotation | turns | blocks/360 | rise/360 blk / mm | deg/block | 90deg turns | turns/m | longest straight |
| --- | :-: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [`coil_3x3_51`](pages/coil_3x3_51.html) | N | 2970 | 8.25 | 21.6 | 5.45 / 169 | 16.7 | 68 | 12.3 | 4 blk / 124 mm |
| [`coil_3x7_22`](pages/coil_3x7_22.html) | N | 3420 | 9.50 | 18.0 | 2.00 / 62 | 20.0 | 76 | 14.3 | 3 blk / 93 mm |
| [`coil_3x8_20`](pages/coil_3x8_20.html) | N | 3060 | 8.50 | 20.0 | 2.00 / 62 | 18.0 | 68 | 12.9 | 4 blk / 124 mm |
| [`coil_3x9_18`](pages/coil_3x9_18.html) | N | 2700 | 7.50 | 22.0 | 2.00 / 62 | 16.4 | 60 | 11.7 | 4 blk / 124 mm |
| [`coil_3x3_54`](pages/coil_3x3_54.html) | N | 2610 | 7.25 | 22.8 | 6.62 / 205 | 15.8 | 60 | 11.7 | 4 blk / 124 mm |
| [`coil_3x3_54_2`](pages/coil_3x3_54_2.html) | N | 2610 | 7.25 | 22.8 | 6.62 / 205 | 15.8 | 60 | 11.7 | 4 blk / 124 mm |
| [`coil_3x3_59`](pages/coil_3x3_59.html) | N | 2970 | 8.25 | 20.6 | 6.42 / 199 | 17.5 | 68 | 12.9 | 3 blk / 93 mm |
| [`coil_2x2_134`](pages/coil_2x2_134.html) | N | 3690 | 10.25 | 16.4 | 12.20 / 378 | 22.0 | 84 | 16.1 | 3 blk / 93 mm |
| [`coil_4x7_20`](pages/coil_4x7_20.html) | N | 3060 | 8.50 | 20.0 | 2.00 / 62 | 18.0 | 68 | 12.9 | 3 blk / 93 mm |
| [`coil_4x8_18`](pages/coil_4x8_18.html) | N | 2700 | 7.50 | 21.9 | 2.00 / 62 | 16.5 | 60 | 11.8 | 4 blk / 124 mm |
| [`coil_5x7_18`](pages/coil_5x7_18.html) | N | 2700 | 7.50 | 22.0 | 2.00 / 62 | 16.4 | 60 | 11.7 | 4 blk / 124 mm |
| [`coil_4x9_18`](pages/coil_4x9_18.html) | N | 2700 | 7.50 | 24.0 | 2.00 / 62 | 15.0 | 60 | 10.8 | 4 blk / 124 mm |
| [`coil_5x8_18`](pages/coil_5x8_18.html) | N | 2700 | 7.50 | 24.0 | 2.00 / 62 | 15.0 | 60 | 10.8 | 4 blk / 124 mm |
| [`coil_4x4_50`](pages/coil_4x4_50.html) | N | 4050 | 11.25 | 16.0 | 4.00 / 124 | 22.5 | 60 | 10.8 | 3 blk / 93 mm |
| [`coil_3x4_68`](pages/coil_3x4_68.html) | N | 3600 | 10.00 | 16.3 | 6.20 / 192 | 22.1 | 61 | 12.1 | 3 blk / 93 mm |
| [`coil_3x4_79`](pages/coil_3x4_79.html) | N | 3780 | 10.50 | 17.2 | 6.86 / 213 | 20.9 | 84 | 15.0 | 3 blk / 93 mm |
| [`coil_5x5_50`](pages/coil_5x5_50.html) | N | 4050 | 11.25 | 14.7 | 4.00 / 124 | 24.5 | 60 | 11.7 | 3 blk / 93 mm |

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
| smallest box | [`coil_3x3_51`](pages/coil_3x3_51.html) — 414 | 736, so 1.78x larger |
| tightest spiral (least rise per turn) | [`coil_3x7_22`](pages/coil_3x7_22.html) — 62mm | 124mm, so 2.0x slacker |
| least tube per turn | [`coil_5x5_50`](pages/coil_5x5_50.html) — 14.7 blk | 16.0 blk, within 9% |
| fewest pieces | [`coil_3x9_18`](pages/coil_3x9_18.html) — 30 | 45 |
| fewest distinct shapes | [`coil_2x2_134`](pages/coil_2x2_134.html) — 2 | 8 |
| largest average plate | [`coil_5x8_18`](pages/coil_5x8_18.html) — 10,704 mm2 | 5,568 mm2 |
| calmest bore (fewest turns/m) | [`coil_4x9_18`](pages/coil_4x9_18.html), [`coil_5x8_18`](pages/coil_5x8_18.html), [`coil_4x4_50`](pages/coil_4x4_50.html) — 10.75 | 10.75 |
| smallest box with no shared wall | [`coil_3x3_54`](pages/coil_3x3_54.html) — 441 | 736, also 0 shared |

The two staircase coils — `coil_4x4_50` as submitted and `coil_5x5_50` reduced — lose the
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
reciprocal of box volume rescaled. Ranking these 17 spirals by density puts them in the
same order as ranking by box in 16 of 17 positions, and the two that swap are a tie at
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

It also says something about the search. Only **4 of 17** walks here are minimal:
`coil_2x2_134`, `coil_3x4_79`, `coil_3x7_22`, `coil_5x5_50`. The search enumerated periods with legs up to
4 and never asked whether a leg was longer than it had to be, so most of what it found
carries slack — `coil_4x9_18` could lose 48 blocks. Two of the 4 minimal walks are
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
from, **13 of the 17 coils reduce — and every one lands on a coil already here.**
4 distinct walks come out of 13 reductions, all of them already in the set: seven
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

What those 13 reductions buy:

| | |
| --- | ---: |
| box smaller | 7 |
| box bigger | 4 |
| box unchanged | 2 |
| touching reduced | 1 |
| touching increased | 7 |
| touching unchanged | 5 |

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

[**SCORING.md**](SCORING.md) combines seven of the metrics and the touching count into
a single ranking under every common mean — harmonic, geometric, arithmetic, quadratic,
cubic, median, midrange, contraharmonic — and reports what the choice of mean does to
the answer. It does a great deal: `coil_2x2_146` places 1st under one and 17th under
another. `coil_3x3_54` comes first under 6 of the 8 means.

It also covers why to rank once rather than rank, cut and re-rank: with a
set-relative normalization the survivors of a cut come out in a different order
than they went in, having not been measured again.

    node tools/score.js          # the scoring table
    node tools/iterate.js        # what cutting and re-ranking does to it
    node tools/gen_scoring.js    # SCORING.md

## The gate

Every walk here has been through `check.py` — the parts, the sections, the seams, and a
voxel model of the assembled bore flooded from the outside to prove it is one sealed
passage. **20,894 checks across 17 spirals, 0 failed.** The per-walk output is
in `checks/`.

    node tools/run_checks.sh     # re-run the gate over every walk

`check.py` needs shapely, numpy and scipy, which live in the Boxes.py venv — run it with
`~/boxes/venv/bin/python`, as `../tools/README.md` says. The system python3 does
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

## The files behind the tables

Each coil is one line of notation in `walks/`, named for the coil, and each has a gate
transcript beside it in `checks/`:

`walks/coil_3x3_51.txt`, `walks/coil_3x7_22.txt`, `walks/coil_3x8_20.txt`, `walks/coil_3x9_18.txt`, `walks/coil_3x3_54.txt`, `walks/coil_3x3_54_2.txt`, `walks/coil_3x3_59.txt`, `walks/coil_2x2_134.txt`, `walks/coil_4x7_20.txt`, `walks/coil_4x8_18.txt`, `walks/coil_5x7_18.txt`, `walks/coil_4x9_18.txt`, `walks/coil_5x8_18.txt`, `walks/coil_4x4_50.txt`, `walks/coil_3x4_68.txt`, `walks/coil_3x4_79.txt`, `walks/coil_5x5_50.txt`

`standardised.json` records the canonical form every walk was pinned to, and
`reduced.json` what the reduction pass produced. Both are written by their tools, not
by hand.

## Regenerating

    tools/build.sh               # all of the below, in order

    node tools/parts.js          # piece counts and distinct shapes -> parts.json
    node tools/table.js          # both tables, plain text
    node tools/table.js --md     # the same as markdown
    node tools/gen_readme.js     # this file
    python3 ../lasermade-tools/md2html.py README.md index.html    # the published page

    node tools/spiral_metrics.js "$(cat walks/coil_3x3_51.txt)"   # measure one walk

    cd ../tools         # rebuild a viewer page
    ~/boxes/venv/bin/python viewer.py "$(cat ./walks/coil_3x3_51.txt)" \
        --out ./pages/coil_3x3_51.html --title "coil 3x3 51"

## What has not been done

**No cut files.** These are walks, viewers and gate results; nothing here has been split to
SVG or nested. `bore_split.py --write DIR` will do it.

The block counts are not equal — the scored ones run 163 to 181, so their bore lengths
run 5053mm to 5611mm, a spread of 11.0%. A period is repeated to about the right length and then trimmed
to wherever the tail comes out elbow-free, so these are the same bore only to within a few
blocks. For comparing packing that is fine; for comparing *bores* it is not, and the length
would have to be pinned first.

Released under [CC0 1.0](LICENSE).
