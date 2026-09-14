# CLAUDE.md

**Nothing here is cut at two pitches.** Every design is on the 16mm block - 10mm of
air inside 3mm walls - so `regress.py` names one `coil fold2` design rather than two,
and `sizes.py` writes no page that is kept. Notes below about a design held at two
sizes describe machinery that still works and currently has nothing to show.

**A design naming a folder that is not there fails.** Without that it falls through to
a geometry-only run and still says pass, on a design whose sheets are not where it
says they are. `check.py`'s empty-folder guard cannot catch it, because that fires
on a folder which exists and is empty.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The bore toolchain: a walk through a lattice of blocks goes in, per-piece laser
cut files come out, and every one of them is checked before you cut. Unlike its
sibling repositories this **is** a software project — it ships no cut files of
its own, only the thing that makes them.

It produces every bore under **`../parts/bore`** - the one that has been built and
every one that is only a candidate. It depends on nothing in `octomino-snakes`, which
enumerated the 369 octominoes and is archived and private: a live instrument should not
need a frozen repository to rebuild its parts.

**The writeup is `README.md` at the repository root**, with a page each for the bores
beside it. It carries the geometry — why an elbow's opening frame has three sides, what a
lap closes, how the notation splits into pieces. This file covers only how to work on the
code.

## Where things are

The scripts are flat at the repository root, one level below it. That is deliberate: it
puts them at the same depth `octomino-snakes/generator/` had, so every relative path came
across unchanged.

    ../parts/bore/concept   the design library - candidates, not instruments
    ../parts/bore/built     the bores that have actually been cut
    ../parts/bell           the two ends, shared by every bore
    ../parts/mouthpiece

`bore_split.py` writes to `../parts/bore/concept` when `--write` is given no path.

## The dependency graph

    svgpath      <- bore_split, check
    bore_split   <- bore_render, check, viewer, mcwalk, nest, hilbert, piece_render
    bore_render  <- viewer, mcwalk
    assemble     <- check
    snakeboxvar  is a Boxes.py generator, driven by subprocess, not imported

`snakeboxvar.py` is not standalone — it installs into a Boxes.py checkout, which
supplies the finger joints, burn compensation and SVG writer. `bore_split.py`
shells out to it. Point `SNAKEBOX_BOXES` at the checkout and `SNAKEBOX_PY` at
its venv python.

**Boxes.py is Florian Festi's**, GPL-3.0-or-later, at
<https://github.com/florianfesti/boxes>. It is an external dependency: a
checkout of it, not a copy in this repository.


**There is one generator, and it is `SnakeBoxVar`.** The simpler `snakebox.py` — no
variable cell widths, no ports, no laps, no plain ends — is not part of this toolchain:
`bore_split.py` names only `SnakeBoxVar` to `scripts/boxes`, `check.py` imports the Var,
and `piece_render.py` uses it. A copy may still be sitting in your Boxes checkout,
untracked and unused; the drift check does not watch it.

## The switches must reach the gate

`--ports`, `--flat` and `--fewest-pieces` are module globals. Run as a script this file
is `__main__`, and `check.py`'s `import bore_split` loads it a second time with its own
globals — so a switch set on `__main__` would reach the writer and not the gate, and
`--ports --write` would write ported cut files, gate the *unported* design, and report a
clean run on parts nothing had looked at. The `__main__` block delegates to the imported
module so there is one set of globals. If another switch is
added, set it on `B`, not on `globals()`.

**The bore's outer ends are plain.** `plain_ends()` always marks the first piece's entry
and the last piece's exit, not just the openings that meet a port. Nothing couples there —
the mouthpiece and the bell each glue a flat plate onto the end face, and a proud tab holds
it off. It gives an end section its own shape where it would otherwise share one with an
inner section (`BDL` -> `BDL~a`, `01_bend_DL.svg` -> `01_bend_DL_buttin.svg`), and a
rename like that **orphans the old file**, which nothing deletes for you. `check_sheets`
then globs the folder and gates the orphan too, at three checks a sheet, so a folder with
stale files in it reports more checks than its design has. **A check count that rises
after a rename is a warning.**

**The tab is floored at the finger tooth, and the notch carries the play.** Both are
failures the gate cannot see. A `--pin_width` that scales with the block while the finger
teeth do not (Boxes.py sizes those at `2 x thickness`) puts a 4.8mm seam tab against 6mm
teeth at the 10mm bore — the narrowest feature on the sheet, where it should be the
strongest. A `--pin_play` of 0 draws the notch exactly the tab's width, and section 1 will
not enter section 2. `pin_width()` floors at the tooth and caps at `MIN_SHOULDER`, and
`pin_play()` returns 0.025 per side out of `PLAY_BY_BORE`. Both are in
`COMMON`, so `check.py` sees the same geometry.

The gate did not catch either. Its floor is `MIN_FEATURE`, 1.5mm, which 4.8 clears
comfortably, and nothing compares a feature against the other features beside it or checks
that a joint has any clearance at all. **A passing gate means no check failed, not that the
part is buildable.**

**`--play` overrides the table, and it is for measuring, not for cutting.** `PLAY_BY_BORE`
is a lookup of what has actually been cut — one row, 0.025 per side at the 10mm bore — and
a bore that is not in it gets 0.025 too, the safe direction, saying on stderr that it is
guessing. The comment above the table sets out the coupon that would settle whether the
requirement is absolute or a fraction of the tab;
`--play` is what cuts it, at values nobody has measured, without editing the table to say
they have. Whatever fits, add the row and stop passing the flag.

It is the same hazard as `--blocksize` and it bit the same way: `COMMON` is built once at
import, so `set_play()` has to rebuild it. Until it did, the three coupons came out
byte-identical — which reads as *play makes no difference* and is really *the flag is not
connected*. The tell was that the difference should have been visible and exactly the play:
it is, now, at every notch edge of section 1, and section 2 does not move at all, because
the play comes off the notch and never off the tab.

`--blocksize` is the same hazard with a quieter failure. It is two numbers, not one:
`BLOCK`, the pitch the plan is laid out on, and `--blocksize` in `COMMON`, the pitch
SnakeBox cuts to. Both come from `set_blocksize()`, and `COMMON` is built by `_common()`
from the constants rather than typed out, so there is one copy of that list and not a
second place to forget. Use `bore_split.COMMON`.

A design folder is usually just `bore/`, and may sit under a size folder as `<size>/bore/`
— so a page title climbs past any ancestor that only names a size or says "bore" until it
reaches one that names the design. `coil/fold2/bore` reads "Coil Fold2 Bore" rather
than "Bore", and would still read that under a `10mm/` level.

`check.py --files` never looks at the pitch: it checks the written sheets for bed fit,
overlaps and engraving on material, all of which a folder cut at one pitch passes when
the gate is told another.
The pitch decides the *geometry* half of the gate, which is recut in-process. Gate a folder
at the wrong blocksize and it reports its full 195 checks and 0 failed on a design nobody cut.

## One viewer, one bore or several

`viewer.build_many([(label, walk), ...], title)` is the only page builder. `build()` is it
with a single item, and a single item hides the selector, so a per-bore page is what it
always was. `sizes.py` passes two and writes one page holding a design at both the block
sizes it is cut at.

A gallery is **more sets in the same viewer**, not a second viewer: two templates drift,
and the drawing code is the part most easily got wrong.

Anything derived from `D` has to be rebuilt when the set changes — `occAll` and `centre`
both are, and a stale `occAll` hides faces the new set has no neighbour for. The switch
**keeps** the camera, which is the point of a switch: you are comparing two shapes, and you
cannot compare them if the view jumps every time you swap.

**The scale is locked to the biggest set, and the cells are drawn in millimetres.** Two
things have to be true before the comparison means anything:

- `data_for` emits **lattice** positions, because occupancy and adjacency need a lattice.
  If the pitch reaches the caption and nothing else, two different pitches draw as the
  same picture — literally the same, the two `cells` lists comparing equal. The data
  carries `u`, the millimetres per step, and `rotC` scales by it.
- A set that fits itself to the canvas normalises away exactly what a size control exists
  to show. `draw()` takes its **scale** from the reference set and its **position** from
  the set on screen, so the small one is small and still centred.

Together those make the smaller pitch draw smaller in exact proportion, and the ¾ coil at
just under half the 3-turn. Change one without the other and the page silently goes back to lying.

**`rot()` scales millimetres, so anything that is a direction must not go through it.**
`shade()` takes a face normal and its dot product with the light as a number in `[-1, 1]`.
Rotate that normal with `rot()` like a position and it is multiplied by the block size,
driving the dot product to ±16 and clamping every face to pure white or pure black. Use
`rotC(p, centre.c, 1)` for a direction. `regress.py` cannot see it — it gates geometry, and
a page whose every face is white is geometrically perfect. **Screenshot the page after any
change to the drawing code**; nothing that reads the file will catch this class.

## bore_split.py's guards

Twenty-one of them, each given inputs it should refuse. **Nearly all hold.** A
stray character, a walk that reverses instead of turning, a walk too short to
have a direction, a walk that revisits a cell, a run of zero length, a one-cell
piece that is not a cube, a notch narrower than its own play, a notch that
leaves no ply beside it, and `--refuse-elbows` against a walk with an elbow --
every one refuses, with a message naming the block or section at fault.

**A guard that does not fire has not been tested until you know your input
reached it.** Two of these are easy to probe wrongly: the notch guards sit behind
a code path a walk without a notched joint never reaches, and `--refuse-elbows`
needs a walk that actually strands a turn.

**`--bore` and `--blocksize` are two spellings of one number** -- `set_bore()`
calls `set_blocksize(bore + 2t)` -- so the pair is refused unless the two agree.
Applied in order unconditionally, `--bore` would silently overwrite
`--blocksize`: `--bore=30 --blocksize=16` would cut 49mm blocks and report them
as though asked for.

**Not guarded:** a run of absurd length (`N9999`) builds until it hangs
rather than refusing. Nothing guards the total block count.

## check.py, against artefacts it should reject

The method: hand the gate something wrong and see whether it says so.

**Effective.** *no two parts overlap* and *engraving on material* both fire when
one part is dragged on top of another. *bore volume matches the walk* is sound
by construction, and the comment above it says why: the expected volume is
computed here, from the walk, and must never be refactored to call
`bore_split.extent()`, because a check that shares its source with the thing it
checks is comparing something to itself.

**The folder is compared with the walk.** *the sheets are this walk's sections*
reads `-NNofMM-` out of every filename, and requires MM to be the section count
this walk splits into and NN to run 1..MM with none missing. Every other check
passes on a folder missing a sheet, and on a folder holding another coil's
sheets entirely; `seen > 0` catches only the empty one. In a repository whose
whole claim is that the cut file IS the design, that the sheets in front of you
belong to the bore you asked for is the thing the gate most needs to say.

**Untested this way**, and worth the same treatment before being trusted: the
checkers in `lasermade-tools` — `doc-audit.py` and `flat-part-check.py`
especially, since both are used to clear work for publication.

## The ply is 3.0 and the kerf 0.15

`SHEET` is 3.0 and `KERF` 0.15, matching `ribbon_bore.py` next door. `THICKNESS`
is 3.0 as well and is what dimensions the lattice; `SHEET` reaches only the slot
Boxes cuts for a sheet to pass through. They are two numbers that happen to
agree, so moving `SHEET` moves no airway.

**`KERF` here is the full width and `BURN` is the radius Boxes wants.** The
paragraph above the constant says so twice, in both directions, because the two
are easy to swap.

**An `old/` inside `cut-files/` is invisible to `repro.py`**, which lists `.svg`
and ignores directories, so a superseded sheet archived beside a design never
disturbs the gate. Every `old/` is gitignored: they sit on the working disk, a
fresh clone does not carry them, and no gate can see them.

**`coil-10x10x30-1.5t` is the only folder here whose sheets describe an object
rather than an intention.** Six pins in `as-built.sha256`; `repro.py` redraws the
other 76 designs and holds these 6 frozen. Redrawing it SHOULD differ, and that
difference failing the gate is the entire point of the mechanism. Pinning a
redrawn sheet would claim it records wood when it does not.

## A page turns back into cut files

`walk_text()` reads the walk out of `<div class="walk">` and allows attributes on
it, because every page this repository writes emits `<div class="walk" id="walk">`.
That is the route `flat-drop/CLAUDE.md` documents — *the walk is stored in the
page … the cut files regenerate from it and nothing else*.

**Two designs have no other route.** `flat-drop` and `square-rise3` keep their
walk **only** in a page: `regress.py`'s `DESIGNS` carries a walk as a string for
every other design, and these two are also the ones with `cut-files/` that no
`DESIGNS` entry claims, so `repro.py` does not look at them either. A break in
the page route is invisible everywhere except on those two.

## Never regenerate what you cannot check

`regress.py` runs the full gate over every design in `walks/` and the design
library. It is the only reason any of this stays honest, it takes about four
minutes, and **it must pass before anything is pushed**:

```sh
python3 regress.py       # all 27 designs, ~7800 checks
```

A change that alters cut geometry and still passes has not been proved right —
it has been proved not obviously wrong. Say which it is.

## Standing decisions

**Fewest elbows at any cost** unless told otherwise. An elbow's opening frame has
three sides rather than four, so both neighbours need flattened plates butt-glued
to it, plus tongues, plus an unfilled void in the corner. Flat-to-flat gluing is
the difficulty of the whole build. `FEWEST_ELBOWS` is the switch and
`--fewest-pieces` turns it off.

**For a build, fewest is not the standard — none is.** `--refuse-elbows` raises
before a single file is written, naming the sections at fault, so a walk that
would cost one cannot be cut by accident:

    error: --refuse-elbows: section 2 of 3 is an elbow. Nothing written.

It is off by default because much of the library exists to exercise elbows —
17 of the 27 designs in `regress.py` contain them, the Hilbert curves 22 and 24 —
so turning it on globally would refuse the corpus. Use it on anything headed for
a build repository.

The rest are there for the opposite reason: `hilbert open` (190 blocks,
27 pieces), `wide telescope`, `metre spring`, `4 corners, flat` and the trumpet
candidate all split with **no** elbows, so a
change that started stranding turns would break them and leave the elbow-heavy
designs looking fine.

**What a turn costs is set by the window of three consecutive terms around it**,
outer A, middle m, outer C — checked over every window, not once per walk.
Consecutive terms are always on different axes, which leaves three cases:

| A and C | case | m must be |
| --- | --- | ---: |
| same axis, same direction | step | >= 1 |
| same axis, opposite direction | hairpin | >= 2 |
| different axes | coil | >= 3 |

Steps are free; hairpins are not — the distinction is the one most easily lost.
Probed with `--no-write`:

    N N3 U1 N3 N   step      0 elbows      N N3 U1 E3 E   coil   2 elbows
    N N3 U1 S3 S   hairpin   2 elbows      N N3 U2 E3 E   coil   1 elbow
    N N3 U2 S3 S   hairpin   0 elbows      N N3 U3 E3 E   coil   0 elbows

`bore_split.py` is the authority on this, not this file and not a
reimplementation of the rule.

**A walk that revisits a cell is refused.** The cells are the air path, so a cell
filled twice is a junction with two ways out and no box section has an opening in
four sides. **Trust the build and suspect the transcription**: the design is laid
out in Minecraft floating in open space, where a section running into an earlier
one is plainly visible, so a refusal usually means the walk was written down
wrong — and a wrong direction before a wrong length. `mcwalk.py` draws a refused
walk under permissive rules and lights up every cell entered more than once,
which the ordinary viewer cannot do.

## Colour is the cut order

Shared across these repositories: **blue engraves, then green → orange → cyan →
black**; black frees the part; **violet `#8000ff` means skip**. Bore nets use two
stages — blue engraves the section number, black cuts.

## Cut files belong to the author

Output lands in other people's repositories. `--write` rewrites **every** file in
the directory it is given, so ask before pointing it at one that has cut files in
it, and never at a directory holding hand-nested or hand-edited work.

## Publishing

Pages deploys from `main` through `.github/workflows/pages.yml`, keyed per commit.
`index.html` is `README.md` rendered by `md2html.py` and committed, not built on the
server, so a stale `index.html` publishes stale content — regenerate it after editing the
README, and read the audit before pushing:

```sh
G=../../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
```
