# greek-spiral

A trumpet whose bore is a **flat meander** — the Greek key, wound all the way in
and brought back out beside itself. **This directory is the bore**: one
laser-cut section that assembles into a single sealed passage, at a **10mm bore
on a 16mm block**.

The **mouthpiece** and the **bell** live in
**[`parts/`](../../../../..)**, because neither
is touched by the way the bore turns. Only the tube belongs to an instrument.

    N N10 W9 S6 E5 N2 W3 N2 E5 S6 W9 N10

Sixty-eight blocks, **1088mm of centreline**, inside a footprint 12 blocks wide
by 13 tall. Both mouths face **north**, out of the two top corners, 8 blocks
apart:

```
#...........
#...........
#.##########
#.#........#
#.#.######.#
#.#.#....#.#
#.#.####.#.#
#.#....#.#.#
#.######.#.#
#........#.#
##########.#
...........#
...........#
```

**The walk never leaves the plane.** That is the whole point of this one: a
bore that stays flat splits into **one** section rather than several, so there
are no section seams at all, and — because a turn only becomes an elbow when it
is stranded as its own single block — **no elbows either**. Nothing here needs
a neighbour's plate flattened and butt-glued to it, which is the difficulty of
the whole build.

<!-- readme-only -->
**[Read the writeup](https://gernreich.github.io/trumpet/parts/bore/concept/walk/no-elbows/greek-spiral/)** — the same
text as this page, set for reading, with a table of contents.

**[The repository](https://github.com/Gernreich/trumpet/tree/main/parts/bore/concept/walk/no-elbows/greek-spiral)** — the bore and
the page it regenerates from.

**[Download the whole repository as a ZIP](https://github.com/Gernreich/trumpet/archive/refs/heads/main.zip)**
— every trumpet, not this one alone; the section here is under
`bores/greek-spiral/`. GitHub builds it from `main` on every push, so it is
never out of date.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and the playing are shown.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
generator and tool, indexed.

## Nothing here has been cut yet

**This design is generated and gated, not built.** It passes all 85 checks and
no part of it has been on the machine. The gate proves that no check failed —
not that the part is buildable — and this repository has no bench result behind
it the way [trumpet-switchback](https://github.com/Gernreich/trumpet/tree/main/parts/bore/concept/walk/no-elbows/switchback)
does. Cut the two sheets, and if anything wants changing, change it here.

The one number carried over from a build rather than derived is the seam
clearance, and at a single section **it never comes into play**: there is no
section-to-section joint on this bore.

## What is in each folder

    bore/                    two cut sheets and the page they belong to

A block is the sound square plus a wall each side, so **the bore is the block
less 6mm** in 3mm stock — 10mm of air on a 16mm pitch.

| | |
| --- | --- |
| bore | 10mm square |
| block | 16mm outside |
| blocks | 68 |
| centreline | 1088mm |
| sections | 1 |
| elbows | 0 |

### The cut files — `bore/`

One section, but **24 flat parts**, and they do not fit on one sheet. They are
written as two:

| # | file | blocks | in | out | plate | parts | sheet |
|---|---|---|---|---|---|---|---|
| 1 | [`bore/bore10-greek-spiral-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet1.svg`](bore/bore10-greek-spiral-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet1.svg) | 1-68 | N | N | 12×13 | 13 | 592×285mm |
| 2 | [`bore/bore10-greek-spiral-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet2.svg`](bore/bore10-greek-spiral-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet2.svg) | 1-68 | N | N | 12×13 | 11 | 540×93mm |

Sheet 1 is **592×285mm against a 600×308mm bed** — within 8mm of the width. It
fits, and it is the largest sheet in any of these repositories; check the bed
origin before running it rather than after.

**Both sheets belong to the same one section.** There is no assembly order to
get wrong and no numbered sequence to keep straight, because there is only ever
one number. Cut both, then glue up the single snake.

**Both ends are plain.** Every seam inside a bore is normally a tab entering a
notch, but this section has nothing beyond it at either end — what meets it is
the mouthpiece at one end and the bell at the other, and both present a flat
plate that glues straight onto the end face. A tab standing 3mm proud of that
face would hold the plate off it. So both mouths are plain edges, which is what
`~a` and `~b` mark in the shape and `-buttin-buttout-` in the filename.

That is the only bore in this lineup where *every* end is plain, because it is
the only one that is a single section.

## The mouthpiece and the bell

**Neither is here.** Both live in
**[trumpet-parts](https://github.com/Gernreich/trumpet/tree/main/parts)** with the rest of
the shared parts. These are the ones to cut with this bore:

| | |
| --- | --- |
| mouthpiece | [`mouthpiece-bore10-trumpet-parts-cut-files.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/mouthpiece/mouthpiece-bore10-trumpet-parts-cut-files.svg) |
| bell | [`bell-round10-153mm-17rings-x3-rim86-cut-files.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/bell/bell-round10-153mm-17rings-x3-rim86-cut-files.svg) |

They are the same 10mm pair that
[trumpet-switchback](https://github.com/Gernreich/trumpet/tree/main/parts/bore/concept/walk/no-elbows/switchback) uses, and
they fit this bore for the same reason they fit that one: neither end cares how
the tube gets from the mouthpiece to the bell.

**The bell sheet draws every ring once and is cut more than once.** A ring is a
stack of 3mm laminations, three of them here, so the sheet goes through the
machine that many times and you glue the copies up into one ring. Cut it once
and you get a **51mm bell** instead of 153, which is the single most expensive
mistake in this build. The bore is not like this: its two sheets are cut once
each.

The mouthpiece is 30 rings of one lamination, so its sheet is cut once.

**Every ring carries its own number, engraved in blue before the cut**, hex, one
character, 0 at the bore end, with an orientation tick right of it — a ring is a
circle and has no top, so turned over a `3` reads as an `E` and a `6` as a `9`.

## Both mouths point the same way

The bore enters and leaves heading **north**, from the two top corners of the
footprint, 8 blocks — **128mm** — apart. The mouthpiece and the bell therefore
stand side by side at the same end of a flat panel, pointing the same way,
rather than at opposite ends of a tube.

That is a playing decision as much as a geometric one, and it has not been
played. It is what makes the shape read as a meander from the front; whether it
is comfortable to hold is a question for the first one cut.

## The page

**[`bore/bore.html`](bore/bore.html)** is the bore, drawn and turnable. Drag to
turn, colour by direction or by section, and a slider follows the bore from the
mouth. It is self-contained apart from its fonts, so it opens from a checkout as
readily as from the published page.

The section colouring has only one colour to use here. Direction is the useful
one on this design.

## Where it comes from

Generated by
**[bore-generator](https://github.com/Gernreich/trumpet/tree/main/tools)**. Its
[CLAUDE.md](https://github.com/Gernreich/trumpet/blob/main/tools/CLAUDE.md)
carries the conventions — the elbow rule, which switches reach the gate, and why
a regenerated folder is not a checked one. To rebuild and check:

```sh
cd ../../../../../../tools
W="N N10 W9 S6 E5 N2 W3 N2 E5 S6 W9 N10"
~/boxes/venv/bin/python bore_split.py --blocksize=16 --refuse-elbows "$W" --write ./bore
~/boxes/venv/bin/python check.py --blocksize=16 "$W" --files ./bore
```

**Run both under the Boxes.py venv python, not the system one.** `check.py`
imports shapely, which the system `python3` does not have — and `bore_split.py`
writes every file *before* it gates them, so a system-python run leaves a folder
of finished-looking cut files and a traceback where the gate should be.

`--refuse-elbows` is not optional here: it stops before writing anything if the
walk would cost an elbow. Neither is `--blocksize=16` — leave it off and the
folder quietly fills with 25mm parts under the same names, which no drawing
shows.

**The walk is not yet in the generator's corpus.** `regress.py` gates every
design in `walks/` on every change to the toolchain; this one is passed on the
command line instead, so a change that broke it would not be caught. Adding
`walks/greek_spiral.txt` and a row in `regress.py` is what makes that stop being
true.

## Colour is the cut order

Shared across these repositories: **blue engraves, then green → orange → cyan →
black**; black frees the part; **violet `#8000ff` means skip**. These bore nets
use two stages — blue engraves the section number, black cuts.

## Licence

CC0 1.0 Universal. Do what you like with it.
