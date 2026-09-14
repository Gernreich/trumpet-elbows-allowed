# CLAUDE.md


**There is no README in this folder.** The reading page for the coil that was
built out of this family is `../../../../../../../../three-turn/`, and the
repository's writeup is at the root. This file is the note that sits beside the
cut files.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Bores whose straight blocks are longer than their turns, and a fork of the toolchain that
cuts them. Straight blocks are 30mm long, turning blocks are 16mm cubes, and the airway is
10 x 10mm the whole way.

Four designs, each in its own folder with its own walk in `tools/walks/`. They are **one
coil truncated at its `N` spacers**, not four separate walks.

**A turn is four terms and a group is three, so they only agree at term 12.** That is the
one thing to get straight before touching any of this:

    term    1  2  3  4  5  6  7  8  9 10 11 12
    letter  W  U  E  D  W  U  E  D  W  U  E  D
            <- turn 1 ->  <- turn 2 ->  <- turn 3 ->
    N after       N        N        N        N
           [group 1] [group 2] [group 3] [group 4]

12 terms is four groups, each of exactly 11 blocks, and only the fourth closes the
cross-section because the first three stop mid-circuit. The reachable lengths are therefore
multiples of one group, and nothing between them is available without moving the spacer,
which makes a different coil rather than a shorter one.

**The folders are named for measured rotation, and one group is three-quarters of a
turn.** Each group is three lateral legs, each leg is one side of the circuit, so a group
sweeps 270° and the four come to 270°, 540°, 810° and 1080° — 0.75, 1.5, 2.25 and 3.00
turns. Measured by sweeping the angle of the lateral position about the circuit's own
centre, and the longest agrees three ways: its legs are `W U E D` three times over, and
its first and last blocks sit on the same cross-section point, which only a whole number
of turns can do.

**`spiral_metrics.js` reads a quarter turn low on a walk that closes.** It sums the turn between
CONSECUTIVE PAIRS of lateral legs: twelve legs give eleven quarter turns, 990°, and it
reads as 2.75. That is the tangent's rotation from the first leg to the last, not the
winding, and the two differ by exactly one quarter turn whenever the walk closes its
circuit. The obvious fix, wrapping the sum from
the last leg back to the first, is right for a walk that closes and wrong for one that
does not: it takes `../fold2` from 450° to 360° against a measured 540°. Do not take a
turn count from that tool without checking it closes.

- **`coil-10x10x30-0.75t/`** — 11 blocks, 274mm, 3 sections. The shortest the walk goes.
  Uncut.
- **`coil-10x10x30-1.5t/`** — 22 blocks, 548mm. **All six parts are cut**; treat its files
  as describing wood. It is `../fold2`'s walk exactly — one file,
  `tools/walks/coil_fold2.txt`, named by both.
- **`coil-10x10x30-2.25t/`** — 33 blocks, 822mm, 9 sections. Uncut.
- **`coil-10x10x30-3t/`** — 44 blocks, 1096mm, and the longest; it is the one that was
  BUILT, and it lives in `../../../../../../built/coil/fold2-long-straight-3t`.
  **Coils are named for the block and the measured turns**: a 10 x 10mm airway on a 30mm
  straight, three times round. That is
  what differs between two coils before anything else does; the circuit size and the pitch
  live in the walk, not the name. Its page title comes from `--title`, not the folder — a
  folder has to sort and survive a URL, a title has to read. `WUED` repeated with an `N` spacer every three terms,
  which walks a square circuit in cross-section while stepping north. The first design laid
  out for this lattice. Nothing cut yet.


**One folder here is pinned rather than reproduced.** `coil-10x10x30-1.5t/` holds
six sheets drawn to a 0.1mm kerf where the generator assumes 0.15, because those
files describe wood that exists and a record of what was cut is worth more than a
file nobody is cutting.

**So do not cut from it as it stands.** A slot drawn for a narrower kerf than the
machine takes comes out about 0.05mm wide, and the joints are slack to match.
Regenerate the folder first -- the switches are in `tools/regress.py`, which is
what drives every other folder -- and know that you are then overwriting the
record.

That is enforced rather than hoped for. The six sheets are pinned by hash in
`tools/as-built.sha256`, and `tools/repro.py` checks them against those hashes
instead of redrawing them -- redrawing them *should* differ, which is why an
ordinary reproduction gate cannot protect them. Overwrite one and the gate fails
with the line *this file describes wood that was cut*. Re-pin with
`repro.py --update`, which is the deliberate act this paragraph asks for; nothing
else in the tree would notice, because every invariant still passes on a
regenerated sheet.

None of them is an instrument on its own: this repository is all bore, and nothing here
closes either end. The 10mm mouthpiece and bell in `../../../../../../../mouthpiece` and
`../../../../../../../bell` fit any of
them — a coil's mouth is a 10mm square in a 16mm face, which is what both seat onto, and
neither cares how long the bore behind it is. They are **named, not copied**: two copies of
a cut file drift.

## One toolchain, not a fork

There is one generator for every bore here,
[`../../../../../../../../tools`](../../../../../../../../tools), and every design in the
tree is gated against it.

**Do not introduce a copy.** The generator is a superset of a cubic-lattice one —
cell indices are real millimetre boxes, so a straight block can run longer than a
turn, and a cubic lattice is the case where every box is the same size. All 24 uniform
designs pass on it with check counts identical line for line.

`bore_split.py`'s tightened page-title rule and `viewer.py`'s real-box rendering are
what everything uses; there is no second version for a fix to fail to flow back to.

Its generator installs into Boxes.py as **`SnakeBoxVar`**, beside `SnakeBox` rather
than over it, so both are available at once — which is the reason installing anything
into `~/Software/boxes` deserves care: it is a shared checkout.

**Boxes.py is Florian Festi's**, GPL-3.0-or-later, at
<https://github.com/florianfesti/boxes>. It is an external dependency: a
checkout of it, not a copy in this repository.


**The play figure is measured here, not conventional.** 0.025 per side, from four goes
in ply. A measured fit beats a conventional one, which is the order to keep.

**The render is the weak point, and only partly gated.** `check.py` never looks at
the page, and a render can be wrong while every geometry check passes: the geometry, the
plates and the voxel model can all be right and the picture still show a uniform 10mm
lattice. `regress.py`'s `check_page` asserts that the page beside a design was handed that design's walk, block count and
centreline, so a stale page fails. It still does not judge the picture. **Look at the
render after changing the geometry.**

**Check a shipped file still reproduces after touching Boxes.py.** The output folder has
to be named the way the real one is, because the design's name is in the
file's name and in its `<title>` — write to `/tmp/f` and you get `bore10-f-...`, which
will not compare against anything.


```sh
cd ../../../../../../../../tools
W="$(cat walks/coil_fold2.txt)"
# the temp path has to mirror the real folder stack: the sheet names are built
# from it, so a different tree writes differently named files and cmp has nothing
# to compare.
rm -rf /tmp/f && mkdir -p /tmp/f/coil/fold2
~/Software/boxes/venv/bin/python bore_split.py --blocksize=16 --refuse-elbows "$W" \
    --write /tmp/f/coil/fold2/bore
B=bore10-coil-fold2-02of06-bend-LUUR-cut-files.svg
cmp /tmp/f/coil/fold2/bore/cut-files/$B \
    ../parts/bore/concept/walk/no-elbows/coil/no-contact/fold2/bore/cut-files/$B
```

## A turn is a cube because it has to be

A turning block has two openings on two different faces, both of which must sit square in
the same frame. Stretch it along one axis and one opening comes out longer than the other.
So `--straight` lengthens straight blocks only, and `extent()` is the single place that
decides: long along the axis a block runs straight on, the section everywhere else.

## The lattice is not a grid any more, and that is the whole cost

The same column index wants 30mm in one part of the bore and 16mm in another, so **no single
number takes a lattice index to a millimetre**. Positions are carried in mm by
`block_boxes()`, each block butting its entry face onto the last block's exit face. For a
cubic cell that reproduces `index x blocksize` exactly — there is a test for it, and it is
the first thing to re-run if geometry looks wrong.

Everything downstream of that had to stop counting cells:

- `SnakeBoxVar.span()` sums the widths of the columns a boundary run crosses, instead of
  steps times one pitch. `bore_split.piece_widths()` supplies them, **rebased on the piece's
  first cell**, because SnakeBox lays its own cells out from (0,0) following `--path`.
- `assemble.build()` takes real boxes and decides face contact geometrically, so a face a
  neighbour only partly covers is walled over the rest of itself.
- `plate_span_mm()` sums column widths for the bed-fit test. `blocksize x blocks` would
  under-report it by 13mm a side here.

**The grid still has to hold within a piece.** `piece_widths()` raises, naming the column,
when a straight and a turn share one — a plate is drawn on a rectilinear grid even though
the bore as a whole is not on one. This walk happens to satisfy it; most will not.

## The 1mm spurs are known, benign, and not worth fixing

Five of the six plates carry **four 1.00mm zero-width spurs** each: the cut path runs 1mm
past a corner and retraces the same line. They are left there deliberately.

- **They cut into waste, not into the part.** Every spur tip tests outside the plate
  outline, so it is a slit in the offcut, not a nick in the piece. No dimension moves.
- **They are not from the stretched lattice.** The 10mm bore in
  `../fold2` has the identical 20.
  They appear once the end frame drops below 12mm: zero at block 18 and above, four at 17
  and 16.
- **The cause is below this code.** The polygon `plateBorders()` hands over is clean — no
  negative or malformed lengths — so it is Boxes.py's edge drawing at a small frame.
  Narrowing the coupling does not help: at a 4.0mm notch there are still four, and at 4.6
  and 5.3 there are eight.

Section 1 is the only one with none.

**Do not "fix" this.** Chasing it means changing `~/Software/boxes`, a shared checkout
every design here depends on, to remove a scorch mark in the waste.

## Do not trust a passing gate

195 checks and 0 failed is not the evidence: a passing gate means no check failed, not
that the part is buildable. The number that means something:

    voxelised bore volume   55520 mm3
    the same from the walk  55520 mm3     14 straights, 8 turns, corner voids

Two unrelated routes to one figure, exact against a 0.5% tolerance, with the passage coming
back as one region and no leak. If you change the geometry, re-derive that by hand and
compare — do not read the tally.

## Commands

One command per coil, and **`--title` is part of the command, not a flourish**: leave it out
and the page is retitled "Coil 10x10x30 1.5t Bore" off the folder name. Regenerating without
it is a silent change to a shipped page.

```sh
cd tools
# the 1.5t's walk is named for its design, because ../fold2 is cut from the same file
for t in 0.75:coil-0.75t:"¾ Turn" 1.5:coil_fold2:"1½ Turns" \
         2.25:coil-2.25t:"2¼ Turns" 3:coil-3t:"3 Turns"; do
  n=${t%%:*}; rest=${t#*:}; w=${rest%%:*}; lab=${rest#*:}
  W="$(cat walks/$w.txt)"
  ~/Software/boxes/venv/bin/python bore_split.py --bore=10 --straight=30 --refuse-elbows \
      --title="10x10x30 Coil, $lab" "$W" \
      --write ../parts/bore/concept/walk/no-elbows/coil/no-contact/fold2-long-straight/coil-10x10x30-${n}t
done
python3 coils.py                    # coils.html here, all four in one viewer
```

`--write` runs the gate itself. To run it alone against a folder:

```sh
~/Software/boxes/venv/bin/python check.py "$(cat walks/coil_fold2.txt)" \
    --bore=10 --straight=30 \
    --files ../parts/bore/concept/walk/no-elbows/coil/no-contact/fold2-long-straight/coil-10x10x30-1.5t/cut-files
```

**Tune the fit with `PLAY_BY_BORE`, which `pin_play()` reads. Do not move the tab.** Two standing decisions, both the
author's:

- **One tab size and one notch size across the whole bore.** Every joint is the same joint.
  A fix that leaves one seam different from the rest is not a fix.
- **Adjust by narrowing the notch, never by widening the tab.** The tab is the finger-tooth
  width and the load-bearing half; a notch is a hole. `--notch` exists and would move the
  tab to suit — this design does not use it.

Sized on the bench, over four goes:

| clearance | outcome |
| ---: | --- |
| 0.0mm | would not go together at all |
| 0.3mm | went together, perceptible rock |
| 0.1mm | very slightly loose |
| **0.05mm** | current: tab 6.0, notch 6.05 — **confirmed on the bench**, parts 1 and 2 fit themselves and each other |

A cut notch cannot be narrowed, so a clearance change made once parts exist has to widen
the tab instead — which leaves two tab sizes in one bore unless the cut parts are recut.

**Before changing this again, ask what is already cut.** A regenerate that moves a joint
leaves parts in wood that no file describes — and if recutting is on the table, say so,
because it decides whether the tab may return to its natural width.

`--bore` is the airway and `--blocksize` is the outside; they differ by two wall
thicknesses and confusing them is the mistake this switch exists to stop. `check.py` needs
**the same two switches** as the writer or it gates a design nobody cut — `--files` looks
only at the sheets as the machine sees them and never at the geometry.

`bore_split.py --write` calls the gate itself, but `check.py` imports **shapely**, which is
not in `/usr/bin/python3`. Run as a script it writes all six files and then dies on the
import, files written and ungated. Use the Boxes.py venv.

## One viewer, one or several coils

`viewer.build_many([(label, walk), ...], title)` is the only page builder. `build()` is it
with a single item, and a single item hides the selector, so a per-coil page is what it
always was. `tools/coils.py` passes four and writes `coils.html`.

That is deliberate: a gallery is **more sets in the same viewer**, not a second viewer. Two
templates would drift, and the drawing code is the part that has been wrong before.

**The scale is locked to the longest coil, not fitted to whichever is shown.** Fitting each
one to the canvas drew four coils the same size, which is the one thing a page comparing
lengths must not do. `draw()` takes its **scale** from the reference set and its
**position** from the set on screen, so the ¾ draws at 0.485 of the 3-turn and stays
centred. That ratio is bounding box, not centreline: the centrelines are 274mm and 1096mm,
a factor of four, but a coil folds, so its box grows more slowly than its length.

The switch **keeps** the camera. Nothing can leave the frame, and holding the angle is
the point: you are comparing lengths, which you cannot do if the view jumps each time you
swap. Reset is still a button.

## `regress.py` gates these designs, and what it cannot see

```sh
cd ../../../../../../../../tools && ~/Software/boxes/venv/bin/python regress.py        # every design
cd ../../../../../../../../tools && ~/Software/boxes/venv/bin/python regress.py coil   # the coils
```

Every folder here has a row in that corpus, carrying the switches it is cut with —
`--bore=10 --straight=30` — because `--files` never looks at the pitch and will happily
gate a design nobody is cutting.

**It bites.** Adding 1mm to `SnakeBoxVar.span()` fails these designs and names the sections.

**It has one blind spot, and it is the interesting one.** `extent()` decides how long a
straight block is, and `check.py` computes the expected bore volume from that same function.
Break `extent` and the measured volume and the expected volume move together, so the check
that looks strongest here cannot see it: adding 1mm to every straight block still passes
all 393 checks on the 3t. What the volume check proves is that the voxel model agrees with the formula,
not that either is right. **A change to `extent()` has to be checked by hand.**

`coil-10x10x30-1.5t/` describes cut parts. A rebuild that changes it needs asking about
first.

**After editing this family's page** — regenerate it, then audit, and check the strokes
here:

```sh
G=../../../../../../../../../lasermade-tools
python3 $G/svg-stroke-check.py --dir . --quiet
cd ../../../../../../../../three-turn
python3 ../../lasermade-tools/md2html.py README.md index.html
python3 ../../lasermade-tools/doc-audit.py README.md --html index.html
```

## Cut files belong to the author

The author edits SVGs in Inkscape during a session. **Stage by name** — never `git add -A`.
A changed section length renames a file and the generator does not delete what it stops
writing, so check the design folder for orphans after a regenerate.

## Colour is the cut order

**Blue engraves, then green -> orange -> cyan -> black**; black frees the part, violet
`#8000ff` means skip.
