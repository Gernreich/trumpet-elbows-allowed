# The greek spiral

A trumpet bore drawn as a flat meander — the Greek key, wound in and brought
back out beside itself. **68 blocks, 1088mm of centreline, one section, no
elbows and no contact.** It is the only lattice bore in this repository that
cuts as a single section — every other one splits into six, eight,
twenty-seven — because it is planar: there is nothing to fold, and no
section-to-section joint to glue square.

<!-- readme-only -->
**[Read this page](https://gernreich.github.io/trumpet/greek-spiral/)**

**[Turn it →](../parts/bore/concept/walk/no-elbows/meander/no-contact/greek-key/bore/bore.html)**
The viewer is a page of its own, not a frame in this one: drag to rotate, and
the slider reveals the bore a block at a time. Turn it edge-on and the whole
design is one block thick.

## The walk is the whole design

```
N N10 W9 S6 E5 N2 W3 N2 E5 S6 W9 N10
```

The first letter is the way you face at the mouth; every term after it turns
where you stand and then travels that many blocks. So **the bore is 1 + the sum
of the numbers** — 67 + 1 = 68. Axes are Minecraft's: `U`/`D` are +Y/−Y, `N` is
−Z, `S` is +Z, `E` is +X, `W` is −X. There is no `U` or `D` in it at all, which
is the whole point: the walk never leaves its plane.

Read it out of `../tools/walks/greek_spiral.txt` rather than from here.

**It is a palindrome about its middle.** `N10 W9 S6 E5 N2` winds in, `W3` crosses
the centre, and `N2 E5 S6 W9 N10` comes back out — the same five legs in reverse.
That is what puts both openings on the outside of the figure rather than
stranding one of them at the eye, which is the failure a plain spiral has.

## The numbers

| | |
| --- | --- |
| blocks | 68 |
| centreline | 1088mm |
| sections | 1 |
| parts | 24, over 2 sheets |
| bounding box | 192 × 16 × 208mm — 12 × 1 × 13 blocks |
| airway | 10mm square, constant |
| block pitch | 16mm — 10mm of air in 3mm walls |
| elbows | none |
| contact | none |
| legs | north 25, west 21, south 12, east 10 |

**A block is 16mm, not 10.** Ten millimetres of sound space wrapped in 3mm of
wall is 16mm outside. A run of *N* blocks is 16*N* mm along the bore, which is
where 1088 comes from.

## One section is the whole design

Every other lattice bore here splits into several sections that couple
tab-into-notch. This one is planar, so it splits into one, and that changes
three things:

- **There is no assembly order.** Two sheets, one section. The `01of01` in the
  filename is not a sequence.
- **The seam clearance is inert.** The 0.025mm per side that the 10mm bore gets
  at a section joint has nothing to apply to here, because there is no
  section-to-section joint. Do not cite the switchback's measured play as
  evidence about this bore.
- **Both ends are plain**, not just one. On a one-piece bore the first piece's
  entry and the last piece's exit are the same piece, so it carries both marks
  and the filename says `-buttin-buttout-`.

A change that makes this design split into more than one section has changed the
walk, not the toolchain. Check the walk first.

**Nothing is engraved.** The other bores write a section number on every part in
blue before black cuts. Here the number would read `1` on all twenty-four parts
and answer a question nobody can ask, so black is the only colour on the sheet.
Marking each wall with its own length was tried and reverted: it named the stick,
but the plate carries no matching mark, and the one that would complete it
could not be derived. The plate is the jig: a wall of length *L* fits only the
run of length *L*.

## Two sheets, and one of them is the biggest in the repository

| sheet | parts | size |
| --- | --- | --- |
| 1 of 2 | 13 | 592.0 × 284.4mm |
| 2 of 2 | 11 | 539.8 × 92.5mm |

Sheet 1 is **1684cm², the largest sheet here by area**, with 8.0mm to spare on a
600mm bed. It is not the widest — `telescope-wide` section 9 and the volute's
narrow panels both reach 598.9mm, and width is what runs out first. The gate's
`sheet fits the bed` check passes everything up to 600.0, and the nester may
split differently rather than failing, so compare the reported sheet sizes after
any change and do not assume two sheets stays two.

## The cut files

In `../parts/bore/concept/walk/no-elbows/meander/no-contact/greek-key/bore/cut-files/`,
both named for the same single section:

```
bore10-meander-greek-key-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet1.svg
bore10-meander-greek-key-01of01-bend-DDDDDDDDDDLLLLLLLLLUUUUUURRRRRDDLLLDDRRRRRUUUUUULLLLLLLLLDDDDDDDDDD-buttin-buttout-cut-files-sheet2.svg
```

The long middle is the flattened shape of the whole snake — sixty-eight letters
of `D`, `L`, `U` and `R` saying which way the plate turns at each block, with
`~a~b` collapsed into the `-buttin-buttout-` that says both ends are plain.
Every file is millimetre-true at 1 user unit = 1mm.

## Rebuild it

The generator lives in `../tools`. Report only, writing nothing:

```
python3 tools/bore_split.py "N N10 W9 S6 E5 N2 W3 N2 E5 S6 W9 N10" --bore=10 --no-write --refuse-elbows
```

Writing rewrites both sheets, and must run under the venv python: `check.py`
imports shapely, and `bore_split.py` writes every file *before* it gates them,
so a system-python `--write` leaves a folder of finished-looking cut files and a
traceback where the gate should be.

```
cd tools && ~/Software/boxes/venv/bin/python bore_split.py \
    walks/greek_spiral.txt --bore=10 \
    --write ../parts/bore/concept/walk/no-elbows/meander/no-contact/greek-key/bore
```

This walk is in the regression corpus — `regress.py` names it *greek spiral* and
gates it at `--bore=10` on every toolchain change, so a change that broke it
would be caught.

## The two ends

Only the tube belongs to an instrument. Neither the mouthpiece nor the bell is
touched by the way a bore turns, and every bore here is on the same 10mm
channel, so one of each serves all of them —
**[the bell and the mouthpiece](../ends/)**.

## More, and licence

**[The three-turn trumpet](../three-turn/)** — the bore that exists as an
object, glued up and blown, rather than a candidate.

**[The trumpet writeup](../)** — the idea, the notation, the gate, and the whole
library.

**[The rest of the build files](https://gernreich.github.io/)** — every
instrument, each with its own writeup.

Released under [CC0 1.0](../LICENSE).
