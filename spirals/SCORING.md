# Scoring

Seven metrics, one touching count, and every common mean, so the ranking can be
read against the thing that produced it. Regenerate with `node tools/gen_scoring.js`.

## What is scored

* **box/block** — less is better
* **cross area** — less is better
* **pieces/block** — less is better
* **distinct** — less is better
* **rise/360** — less is better
* **turns/m** — less is better
* **longest str** — more is better
* **mean plate** — more is better
* **touching** — carried the same way as the rest, at weight 5

**Nothing is judged on the bore's mouth and exit.** Every design has those two pieces
and no design chooses them, so every metric here is measured over the interior — the
pieces in between — and the piece and shape counts are counts of those. Blocks and mm,
which only describe, still cover the whole bore.

The metrics are normalized to (0,1] with 1 the best in the set, oriented so bigger is
better, and floored at 0.01 so that one worst-in-set value cannot zero a product.

**Touching is treated differently, on purpose.** A wall the bore shares with itself is
visible in the finished instrument, so it is the heaviest input here at weight 5,
and it is penalized convexly — **1/(1+t)** — rather than fading linearly. The step from
no contact to any contact is far larger than any step after it: 0 contacts scores 1.000,
8 contacts 0.111, 20 contacts 0.048. Nothing but a clean coil can score 1.

That form is also absolute where a linear fade against the set maximum is not. Dropping
the worst coil would move everyone else's touching term under a linear fade; under
1/(1+t) it moves nothing.

Override the weight with `SPIRAL_TOUCH_WEIGHT=8 node tools/score.js`.

### A heavy weight is a preference, not a guarantee

Weighting touching at 5 does not stop a coil with contact from beating a clean one.
Under the harmonic mean the first coil *with* touching places **3rd**, above **6** coils
that have none — because each of those has some other metric sitting on the 0.01 floor,
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
on all 17. What changes with *p* is not the size of the score but how much a single
bad metric is allowed to sink it.

## The table

| spiral | touch | harmonic | # | geometric | # | arithmetic | # | quadratic | # | cubic | # | median | # | midrange | # | contraharmonic | # |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| coil_3x3_54 | 0 | 0.8514 | 1 | 0.8665 | 1 | 0.8797 | 1 | 0.8911 | 1 | 0.9009 | 1 | 0.9260 | 2 | 0.7757 | 1 | 0.9026 | 3 |
| coil_3x3_54_2 | 0 | 0.8514 | 2 | 0.8665 | 2 | 0.8797 | 2 | 0.8911 | 2 | 0.9009 | 2 | 0.9260 | 3 | 0.7757 | 2 | 0.9026 | 4 |
| coil_3x3_59 | 0 | 0.1124 | 9 | 0.5652 | 3 | 0.7528 | 3 | 0.8032 | 3 | 0.8322 | 4 | 0.8625 | 4 | 0.5050 | 10 | 0.8571 | 8 |
| coil_3x4_68 | 0 | 0.1058 | 11 | 0.4640 | 4 | 0.6701 | 4 | 0.7502 | 6 | 0.7962 | 6 | 0.7571 | 7 | 0.5050 | 11 | 0.8399 | 9 |
| coil_3x4_79 | 0 | 0.1060 | 10 | 0.4476 | 5 | 0.6395 | 7 | 0.7235 | 7 | 0.7770 | 8 | 0.6700 | 8 | 0.5050 | 12 | 0.8187 | 10 |
| coil_3x3_51 | 7 | 0.2575 | 3 | 0.3840 | 6 | 0.5322 | 9 | 0.6319 | 12 | 0.6914 | 13 | 0.6645 | 12 | 0.5625 | 3 | 0.7503 | 15 |
| coil_4x4_50 | 0 | 0.0584 | 15 | 0.3439 | 7 | 0.6514 | 5 | 0.7622 | 5 | 0.8166 | 5 | 0.8058 | 5 | 0.5050 | 14 | 0.8920 | 5 |
| coil_3x9_18 | 13 | 0.1602 | 4 | 0.3055 | 8 | 0.5137 | 12 | 0.6412 | 11 | 0.7103 | 11 | 0.6678 | 11 | 0.5357 | 4 | 0.8002 | 11 |
| coil_4x9_18 | 13 | 0.1509 | 7 | 0.2920 | 9 | 0.5293 | 10 | 0.6793 | 10 | 0.7563 | 10 | 0.6700 | 9 | 0.5357 | 6 | 0.8717 | 7 |
| coil_4x8_18 | 13 | 0.1566 | 5 | 0.2907 | 10 | 0.4918 | 13 | 0.6208 | 13 | 0.6915 | 12 | 0.6509 | 13 | 0.5357 | 5 | 0.7838 | 12 |
| coil_5x7_18 | 13 | 0.1518 | 6 | 0.2778 | 11 | 0.4780 | 14 | 0.6107 | 14 | 0.6837 | 14 | 0.5885 | 14 | 0.5357 | 7 | 0.7802 | 13 |
| coil_3x8_20 | 15 | 0.1415 | 8 | 0.2770 | 12 | 0.4749 | 15 | 0.5990 | 15 | 0.6711 | 15 | 0.4943 | 15 | 0.5313 | 8 | 0.7556 | 14 |
| coil_2x2_134 | 0 | 0.0410 | 16 | 0.2726 | 13 | 0.6423 | 6 | 0.7766 | 4 | 0.8381 | 3 | 1.0000 | 1 | 0.5050 | 9 | 0.9391 | 1 |
| coil_5x8_18 | 13 | 0.0731 | 12 | 0.2409 | 14 | 0.5212 | 11 | 0.6804 | 9 | 0.7599 | 9 | 0.6700 | 10 | 0.5050 | 17 | 0.8884 | 6 |
| coil_4x7_20 | 15 | 0.0677 | 13 | 0.1862 | 15 | 0.3772 | 16 | 0.5070 | 16 | 0.5851 | 16 | 0.3400 | 16 | 0.5050 | 15 | 0.6814 | 17 |
| coil_3x7_22 | 17 | 0.0636 | 14 | 0.1660 | 16 | 0.3481 | 17 | 0.4882 | 17 | 0.5800 | 17 | 0.2414 | 17 | 0.5050 | 13 | 0.6848 | 16 |
| coil_5x5_50 | 0 | 0.0255 | 17 | 0.1542 | 17 | 0.5460 | 8 | 0.7071 | 8 | 0.7792 | 7 | 0.8058 | 6 | 0.5050 | 16 | 0.9158 | 2 |

**The median does not survive the weighting.** It is an order statistic, and weight is
applied by repetition, so touching occupies 5 of the 13 values and can simply *be*
the median. 1 coils tie at exactly 1.0000. Read the median column knowing that;
the power means do not have this problem.

## Choosing a mean is choosing how much a weak spot counts

That choice is worth more than any metric in it. coil_2x2_134 ranks **1** under one mean
and **16** under another — a swing of 15 places in a field of 17. coil_5x5_50 swings 15.

| spiral | worst single input | harmonic | contraharmonic |
| --- | --- | ---: | ---: |
| `coil_2x2_134` | rise/360 = 0.010 | #16 | #1 |
| `coil_5x5_50` | box/block = 0.010 | #17 | #2 |
| `coil_3x3_51` | touching = 0.125 | #3 | #15 |

`coil_2x2_134` is the clearest case: one input on the floor and another at the ceiling.
The mean that punishes weak spots reads it as disqualified; the mean that rewards strong
ones reads it as the best thing here. Both are arithmetically correct — they are
answering different questions. `coil_5x5_50` is the same shape of argument, its weak spot
being box/block.

Harmonic and contraharmonic agree on **2** of 17 placings — they are as opposed as
two means of the same numbers can be. Harmonic and geometric agree on 3, which is
why the geometric mean is the usual choice when no weak spot should be forgiven but
outright disqualification is too strong.

## What survives

`coil_3x3_54` comes first under **6 of the 8 means**, and the reason is visible in the
table above: its worst input is 0.551, where every other contender has something
at 0.01. It does not win by being outstanding anywhere. It wins by having nothing to
punish, which is the one way to be robust to the choice of mean.

If a weak spot is genuinely fatal — a shared wall that will leak, a coil too fat for
the body — use the harmonic mean, or filter and then rank. If the design is allowed one
bad number in exchange for a very good one, use quadratic or cubic. The arithmetic mean
is the choice that declines to say.

## Ranking once, not repeatedly

A tempting variant is to rank, cut the bottom half, and re-rank the survivors. Do not.

* **harmonic, min-max (what SCORING.md uses)** — survivors reordered 7/9, 5/5, 3/3, 0/2 over the rounds (15 moves in total)
* **geometric, min-max** — survivors reordered 6/9, 3/5, 0/3, 0/2 over the rounds (9 moves in total)
* **geometric, pure ratio-to-best** — survivors reordered 0/9, 0/5, 0/3, 0/2 over the rounds (0 moves in total)

The first two reorder coils that did not change, purely because other coils left the
set. `coil_3x3_51` places 3rd of 17 and 1st of the surviving 9; `coil_3x3_54`
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
because a composite is a compromise. `coil_2x2_134` has the narrowest cross-section in
the set and does not survive round 0 of 2 of the 3 runs.

    node tools/iterate.js        # the numbers above
