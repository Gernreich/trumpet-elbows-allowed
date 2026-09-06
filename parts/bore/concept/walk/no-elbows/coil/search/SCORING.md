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
on all 10. What changes with *p* is not the size of the score but how much a single
bad metric is allowed to sink it.

## The table

| spiral | touch | harmonic | # | geometric | # | arithmetic | # | quadratic | # | cubic | # | median | # | midrange | # | contraharmonic | # |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| coil_3x3_54_2 | 0 | 0.1007 | 1 | 0.5310 | 1 | 0.7978 | 1 | 0.8661 | 1 | 0.8945 | 1 | 1.0000 | 1 | 0.5050 | 1 | 0.9402 | 1 |
| coil_3x3_59 | 0 | 0.0582 | 6 | 0.3478 | 2 | 0.6548 | 2 | 0.7623 | 2 | 0.8163 | 2 | 0.8632 | 2 | 0.5050 | 2 | 0.8874 | 5 |
| coil_3x9_18 | 13 | 0.0722 | 2 | 0.2183 | 3 | 0.4559 | 6 | 0.6065 | 6 | 0.6887 | 6 | 0.3400 | 5 | 0.5050 | 6 | 0.8068 | 6 |
| coil_4x8_18 | 13 | 0.0707 | 3 | 0.2005 | 4 | 0.4211 | 7 | 0.5712 | 7 | 0.6545 | 7 | 0.1567 | 7 | 0.5050 | 8 | 0.7749 | 7 |
| coil_3x8_20 | 15 | 0.0680 | 4 | 0.1940 | 5 | 0.4080 | 8 | 0.5562 | 8 | 0.6469 | 8 | 0.4246 | 4 | 0.5050 | 5 | 0.7582 | 9 |
| coil_5x7_18 | 13 | 0.0652 | 5 | 0.1789 | 6 | 0.3994 | 9 | 0.5550 | 9 | 0.6419 | 9 | 0.0714 | 9 | 0.5050 | 10 | 0.7712 | 8 |
| coil_4x9_18 | 13 | 0.0470 | 7 | 0.1739 | 7 | 0.4687 | 5 | 0.6527 | 5 | 0.7447 | 5 | 0.0714 | 8 | 0.5050 | 9 | 0.9090 | 4 |
| coil_3x4_68 | 0 | 0.0306 | 9 | 0.1681 | 8 | 0.5278 | 3 | 0.6961 | 3 | 0.7744 | 3 | 0.6903 | 3 | 0.5050 | 3 | 0.9181 | 3 |
| coil_3x4_79 | 0 | 0.0251 | 10 | 0.1280 | 9 | 0.4826 | 4 | 0.6707 | 4 | 0.7603 | 4 | 0.1729 | 6 | 0.5050 | 4 | 0.9322 | 2 |
| coil_4x7_20 | 15 | 0.0445 | 8 | 0.1272 | 10 | 0.2983 | 10 | 0.4385 | 10 | 0.5320 | 10 | 0.0625 | 10 | 0.5050 | 7 | 0.6447 | 10 |

**The median does not survive the weighting.** It is an order statistic, and weight is
applied by repetition, so touching occupies 5 of the 13 values and can simply *be*
the median. 1 coils tie at exactly 1.0000. Read the median column knowing that;
the power means do not have this problem.

## Choosing a mean is choosing how much a weak spot counts

That choice is worth more than any metric in it. coil_3x4_79 ranks **2** under one mean
and **10** under another — a swing of 8 places in a field of 10. coil_3x4_68 swings 6.

| spiral | worst single input | harmonic | contraharmonic |
| --- | --- | ---: | ---: |
| `coil_3x4_79` | box/block = 0.010 | #10 | #2 |
| `coil_3x4_68` | pieces/block = 0.010 | #9 | #3 |
| `coil_3x8_20` | distinct = 0.010 | #4 | #9 |

`coil_3x4_79` is the clearest case: one input on the floor and another at the ceiling.
The mean that punishes weak spots reads it as disqualified; the mean that rewards strong
ones reads it as the best thing here. Both are arithmetically correct — they are
answering different questions. `coil_3x4_68` is the same shape of argument, its weak spot
being pieces/block.

Harmonic and contraharmonic agree on **1** of 10 placings — they are as opposed as
two means of the same numbers can be. Harmonic and geometric agree on 2, which is
why the geometric mean is the usual choice when no weak spot should be forgiven but
outright disqualification is too strong.

## What survives

`coil_3x3_54_2` comes first under **8 of the 8 means**, and the reason is visible in the
table above: its worst input is 0.010, where every other contender has something
at 0.01. It does not win by being outstanding anywhere. It wins by having nothing to
punish, which is the one way to be robust to the choice of mean.

If a weak spot is genuinely fatal — a shared wall that will leak, a coil too fat for
the body — use the harmonic mean, or filter and then rank. If the design is allowed one
bad number in exchange for a very good one, use quadratic or cubic. The arithmetic mean
is the choice that declines to say.

## Ranking once, not repeatedly

A tempting variant is to rank, cut the bottom half, and re-rank the survivors. Do not.

* **harmonic, min-max (what SCORING.md uses)** — survivors reordered 5/5, 2/3, 2/2 over the rounds (9 moves in total)
* **geometric, min-max** — survivors reordered 2/5, 0/3, 0/2 over the rounds (2 moves in total)
* **geometric, pure ratio-to-best** — survivors reordered 0/5, 0/3, 0/2 over the rounds (0 moves in total)

The first two reorder coils that did not change, purely because other coils left the
set. `coil_4x8_18` places 3rd of 10 and 1st of the surviving 9; `coil_3x3_54_2`
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
because a composite is a compromise. `coil_3x3_54_2` has the narrowest cross-section in
the set and does not survive round 0 of 0 of the 3 runs.

    node tools/iterate.js        # the numbers above
