# The ziggurat

A trumpet bore wound as a square spiral that widens as it climbs. **63 blocks,
1008mm of centreline, 13 sections, three turns, nothing touching, 4 elbows.**
Each turn is one block wider on every side than the one before, and all three
stand on the same centre, so the bore steps outward along its axis like a
stepped pyramid seen from below — a cone in blocks.

<!-- readme-only -->
**[Read this page](https://gernreich.github.io/trumpet-elbows-allowed/ziggurat/)**

**[Turn it →](../parts/bore/concept/walk/spiral/ziggurat/bore/bore.html)**
The viewer is a page of its own, not a frame in this one: drag to rotate, and
the slider reveals the bore a block at a time in the order you would glue it.

![The ziggurat from four sides and from above: a square tube spiralling outward in three widening square turns as it steps along its axis, each of its 13 sections a different colour and numbered from the mouthpiece, IN near the centre and OUT at the outer corner](../parts/bore/concept/walk/spiral/ziggurat/bore/ziggurat-pieces.svg)

## The walk is the whole design

```
N1 W2 U2 E3 N2 D3 W4 U4 N2 E5 D5 W6 N2 U6 E7 D7 N1
```

The first term is the way you face at the mouth and how far you go before
anything turns; every term after it turns where you stand and then travels that
many blocks. So **the bore is 1 + the sum of the numbers** — 62 + 1 = 63. Axes
are Minecraft's: `U`/`D` are +Y/−Y, `N` is −Z, `S` is +Z, `E` is +X, `W` is −X.

It goes round `W U E D` and every other side is one block longer than the side
before it: 2, 2, 3, 3, 4, 4, up to 7. That is a square spiral, and it is what
keeps it centred. Lengthening all four sides of a loop at once would grow it
from one corner; lengthening them a pair at a time grows it one block outward
on each side every turn. Its three turns measure 4 × 4, 6 × 6 and 8 × 8 blocks,
all on the same centre, and the mouthpiece starts beside that centre.

Between every three sides it steps two blocks north, so it climbs 2⅔ blocks —
42.7mm — a turn, the same rate as the [tight coil](../tight-coil/); the whole
walk measures 8 blocks of axis in three turns. It is right-handed, the same way
round as the tight coil and the three-turn trumpet. The bell end leaves from the
outermost corner, three blocks east and three down from where the mouthpiece
enters.

The walk is kept in `../tools/walks/ziggurat.txt`, and `regress.py` beside it
names this folder as where its cut files land.

## Four elbows, and the walk that has none

A turn folds into a bend when the leg before it is long enough. Between two
legs on different axes that takes 3, and four legs here are 2: the first `W2`,
between `N1` and `U2`, and the three `N2` steps. Each strands one turn as an
**elbow**, a single block that is a piece of its own — sections 2, 5, 8 and 11.
Every other turn folds: once the sides reach 3, the spiral cuts as long bends
of up to fourteen blocks.

**Three of the elbows are left open on the inside of the turn** — sections 5,
8 and 11. An elbow's two openings share an edge, so the inside corner of the
turn has no wall of its own; a neighbour normally closes it with a tongue, a
wall run one ply past the joint. At these three neither neighbour can put the
tongue on a wall, so the generator leaves the corner open rather than guessing.
It is not a leak — the neighbours' walls and the elbow's plates seal it from
outside — but it is a 3 × 3mm notch along the inside of the turn, and a joint
with less glue on it.

Raise those four legs to 3 and there are no elbows at all:

```
N1 W3 U2 E3 N3 D3 W4 U4 N3 E5 D5 W6 N3 U6 E7 D7 N1
```

That walk is 67 blocks, 1072mm, and touches itself nowhere either. It climbs
4 blocks a turn instead of 2⅔. Its turns are the same 4, 6 and 8 blocks
across, but the longer first `W` sets the whole spiral one block further west of
the mouthpiece. It has been run through `bore_split.py` and not cut.

## The numbers

| | |
| --- | --- |
| blocks | 63 |
| centreline | 1008mm |
| sections | 13 — 1 straight, 4 elbows, 8 bends |
| parts | 76, over 13 sheets |
| bounding box | 128 × 128 × 144mm — 8 × 8 × 9 blocks |
| turns | three, 4 × 4, 6 × 6 and 8 × 8 blocks |
| airway | 10mm square, constant |
| block pitch | 16mm — 10mm of air in 3mm walls |
| rise | 42.7mm a turn — 2⅔ blocks |
| elbows | 4; three of them open on the inside of the turn |
| contact | none |
| legs | east 15, down 15, west 12, up 12, north 9 |
| with the ends | 1251mm — 90mm mouthpiece, 1008mm bore, 153mm bell |

## The 13 sections

Numbered from the mouthpiece; assemble in order. A section is one flat snake,
so it is one SVG, and every part is engraved with its section number because
the sections only go together one way.

| # | blocks | kind | in → out | plate | shape | parts | sheet |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 | straight | N → N | 1×1 | `S1~a@-W` | 4 | 122 × 40mm |
| 2 | 2 | elbow | N → W | 1×1 | `EEN` | 4 | 128 × 40mm |
| 3 | 3–7 | bend | W → E | 2×3 | `BLUUR~f` | 8 | 340 × 72mm |
| 4 | 8–10 | bend | E → N | 2×2 | `BRD~g` | 6 | 253 × 56mm |
| 5 | 11 | elbow | N → D | 1×1 | `EEN` | 4 | 128 × 40mm |
| 6 | 12–19 | bend | D → U | 5×3 | `BDDLLLLU~f` | 8 | 529 × 75mm |
| 7 | 20–23 | bend | U → N | 3×2 | `BRRD~g` | 6 | 317 × 56mm |
| 8 | 24 | elbow | N → E | 1×1 | `ENE` | 4 | 122 × 43mm |
| 9 | 25–35 | bend | E → W | 5×6 | `BRRRRDDDDDL~f` | 8 | 599 × 146mm |
| 10 | 36–41 | bend | W → N | 5×2 | `BLLLLD~g` | 6 | 445 × 56mm |
| 11 | 42 | elbow | N → U | 1×1 | `ENE` | 4 | 122 × 43mm |
| 12 | 43–56 | bend | U → D | 8×6 | `BUUUUURRRRRRRD~f` | 8 | 570 × 149mm |
| 13 | 57–63 | bend | D → N | 6×2 | `BLLLLLD~b` | 6 | 509 × 56mm |

`~a` and `~b` are the plain ends, where the mouthpiece and the bell land; the
file names say `buttin` and `buttout`. `@` names a wall that runs on as a
tongue into the elbow beside it, and `~f` and `~g` mark a piece whose plate is
flattened where it meets an elbow.

Sections 2 and 5 are the same shape, and so are 8 and 11; each is cut
separately so that it carries its own number.

**Section 9 is 599.05mm wide**, 0.95mm inside a 600mm bed. It passes the gate's
`sheet fits the bed` check, which allows anything up to 600.0, but there is
nothing to spare: place it square to the bed and check it before cutting.

## The cut files

In `../parts/bore/concept/walk/spiral/ziggurat/bore/cut-files/`:

```
bore10-spiral-ziggurat-01of13-straight1-lapW-buttin-cut-files.svg
bore10-spiral-ziggurat-02of13-elbow-EN-cut-files.svg
bore10-spiral-ziggurat-03of13-bend-LUUR-flatin-cut-files.svg
bore10-spiral-ziggurat-04of13-bend-RD-flatout-cut-files.svg
bore10-spiral-ziggurat-05of13-elbow-EN-cut-files.svg
bore10-spiral-ziggurat-06of13-bend-DDLLLLU-flatin-cut-files.svg
bore10-spiral-ziggurat-07of13-bend-RRD-flatout-cut-files.svg
bore10-spiral-ziggurat-08of13-elbow-NE-cut-files.svg
bore10-spiral-ziggurat-09of13-bend-RRRRDDDDDL-flatin-cut-files.svg
bore10-spiral-ziggurat-10of13-bend-LLLLD-flatout-cut-files.svg
bore10-spiral-ziggurat-11of13-elbow-NE-cut-files.svg
bore10-spiral-ziggurat-12of13-bend-UUUUURRRRRRRD-flatin-cut-files.svg
bore10-spiral-ziggurat-13of13-bend-LLLLLD-buttout-cut-files.svg
```

Every file is millimetre-true at 1 user unit = 1mm. **Blue engraves, then black
cuts** — blue writes the section number on every part, black frees it.

## Rebuild it

The generator lives in `tools/` at the repository root, and both commands run
from there. Report only, writing nothing:

```
python3 tools/bore_split.py tools/walks/ziggurat.txt --no-write
```

Writing rewrites every sheet in the folder and gates them, so it runs under the
venv python that has the gate's dependencies:

```
cd tools && ~/Software/boxes/venv/bin/python bore_split.py walks/ziggurat.txt \
    --write ../parts/bore/concept/walk/spiral/ziggurat/bore
```

## The two ends

Only the tube belongs to an instrument. Neither the mouthpiece nor the bell is
touched by the way a bore turns, and every bore here is on the same 10mm
channel, so one of each serves all of them —
**[the bell and the mouthpiece](../ends/)**. The walk opens and closes on a
single block, `N1`: the mouthpiece and the bell are long enough to carry the
instrument clear of the spiral without a longer lead.

## More, and licence

**[The tight coil](../tight-coil/)** — the same rise a turn, but a coil that
stays three blocks across rather than a spiral that widens.

**[The trumpet writeup](https://gernreich.github.io/trumpet-elbows-allowed/)** — the idea, the notation, the gate, and the whole
library.

**[The rest of the build files](https://gernreich.github.io/)** — every
instrument, each with its own writeup.

Released under [CC0 1.0](../LICENSE).
