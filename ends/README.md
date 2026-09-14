# The bell and the mouthpiece

Neither end is touched by the way a bore turns, so **only the tube belongs to an
instrument**. Both ends are stacks of laminated 3mm rings, both close onto the
same 10mm square in a 16mm face, and every bore in this repository is on that
channel — so one bell and one mouthpiece serve all of them.

<!-- readme-only -->
**[Read this page](https://gernreich.github.io/trumpet/ends/)**

They were kept inside one instrument's folder from 2026-08-31 to 2026-09-02, on
the grounds that nothing else cut them. That was true of what had been cut and
false of what fits, and it hid two general parts inside one design. They live in
`../parts/` now.

## The bell

**17 rings of 9mm, 153mm tall, a 10mm square throat opening to a ø86mm round
rim** — ø80 of air. The section morphs square to round on the way up while
holding area, so the bore never steps in cross-section even though the shape of
it changes completely.

**[Turn it →](../parts/bell/bell-round10-153mm-17rings-x3-rim86-turn.html)**
The viewer is a page of its own: drag to rotate, and the slider stacks the rings
one at a time, which is the useful part.

![A 17-ring bell seen from above the rim: rings stacked and morphing from square at the throat to round at the rim, widening toward the viewer, the bore visible down the middle](../parts/bell/bell-round10-153mm-17rings-x3-rim86-view.svg)

![Axial section of a 17-ring bell: the bore climbing one staircase and the outside another, wall between them](../parts/bell/bell-round10-153mm-17rings-x3-rim86-section.svg)

| | |
| --- | --- |
| rings | 17, each 3 laminations of 3mm |
| height | 153mm, built on a 152mm profile |
| throat | 10mm square |
| rim | ø86.0mm outside, ø80 of air |
| profile | Bessel horn, gamma 0.7 |
| sheet | 187 × 192mm |
| pieces | 51 — the sheet goes through the machine three times |

**The bell is cut more than once.** The sheet draws every ring once, and the
`x3` in its filename is how many passes it takes. Cut it once and you get a
51mm stub instead of a 153mm bell.

Rings stack; they do not telescope. Each seats on the one below over 3.0mm per
side. They are engraved in hex with `0` at the bore, and the generator writes
the numbering itself as the last step, with an orientation tick beside each
number — a numbering failure deletes the sheet rather than leaving an unnumbered
one to be cut.

```
cd parts/bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80 \
    --out=cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
```

A bare `bell-round.py` writes four ring budgets, not one; the shipped sheet is
among them and comes back identical, the other three are scratch. Pass a budget
to write one.

### The adapter

A swept-curve bore leaves its cheek through a **7 × 14mm port**, not a 10mm
square, so it cannot meet this bell directly. `bell-adapter.py` draws the piece
that carries one into the other: **7 rings of one ply, 21mm**, holding area
across the change on a smoothstep so both ends are tangent. The last ring is a
collar — 10mm square in a 16mm square face, which is the bore's own end face, so
the bell stacks on it unchanged.

```
cd parts/bell && python3 bell-adapter.py \
    --out=cut-files/bell-adapter-port7x14-to-bore10-7rings-x1-cut-files.svg
```

## The mouthpiece

**30 rings of 3mm, 90mm tall.** A 16mm square plate at the instrument and a
ø23mm rim at the lip — ø17 where the lip actually sits — narrowing to a
**ø3.66mm throat** before the backbore. Full size on a quarter-size instrument,
which is the point of it.

**[Turn it →](../parts/mouthpiece/mouthpiece-bore10-trumpet-parts-turn.html)**
Again a page of its own, and again the slider stacks it a ring at a time.

![The mouthpiece assembled from 30 rings, seen from above the cup: the cup narrowing to a throat, then the backbore opening slowly below it](../parts/mouthpiece/mouthpiece-bore10-trumpet-parts-view.svg)

![Axial section of the 30-ring mouthpiece: the bore narrowing to the throat and opening again, the outside stepping the other way, wall between them](../parts/mouthpiece/mouthpiece-bore10-trumpet-parts-section.svg)

| | |
| --- | --- |
| rings | 30, each 1 lamination of 3mm |
| height | 90mm |
| layout | 26 backbore rings, 0 entrance rings, 4 bowl rings |
| at the instrument | 10mm square aperture in a 16mm square plate |
| throat | ø3.66mm |
| rim | ø23mm outside, ø17 at the lip |
| wall | 3 to 4.96mm |
| sheet | 111 × 101mm |
| pieces | 30 — one pass |

Station one is a **sharp** 10mm square aperture in a **sharp** 16mm square
plate, matching the bore corner for corner. The corners round away going up, and
the section is a true circle from the 9.026mm station on, so the throat and the
whole cup are round.

```
cd parts/mouthpiece && python3 mouthpiece-round.py \
    cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
```

**Sand and fill the staircase and round the rim over before you put a lip to
it.** Thirty 3mm rings make a 90mm cone out of thirty steps, and the lip is the
one part of this instrument that can feel every one of them.

> The mouthpiece on the instrument that has been built is **24 rings, 72mm** —
> not the 30 this sheet cuts. It is the instrument that is short, not the
> drawing.

## Every sheet is claimed

Both directories carry a `.repro` manifest naming the exact command that draws
each shipped SVG, checked by `repro-svg.py`. Nothing ran these eleven generators
at all until 2026-09-10: the block bores had two byte gates and the ribbon
sheets a third, and the one thing that did touch these directories checked the
preview against the cut file rather than the cut file against the code. So a
sheet on disk could have been one the current code would no longer draw, with
every gate still green. The adapter was found exactly that way — reported
`UNCLAIMED` three days after it was added.

## The bores these fit

**[the coiled trumpet](../coiled/)** · **[the switchback trumpet](../switchback/)**
· **[the greek spiral](../greek-spiral/)** — and every other candidate in
`../parts/bore/`, all of them on the same 10mm channel.

## More, and licence

**[The trumpet writeup](../)** — the idea, the notation, the gate, and the one
bore that exists as an object.

**[The rest of the build files](https://gernreich.github.io/)** — every
instrument, each with its own writeup.

Released under [CC0 1.0](../LICENSE).
