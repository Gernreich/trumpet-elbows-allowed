# CLAUDE.md


**There is no README in this folder.** The reading page for this bore is
`../../../../../../switchback/`, and the repository's writeup is at its root.
This file is the note that sits beside the cut files.

**This is a 10mm bore**: six sections on a 16mm block, 352mm of centreline. There is
one pitch here, so there is no `sizes.html`.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A laser-cutting build repository, not a software project. The deliverable is **one bore**,
six sections, as **SVG cut files**
that someone sends to a laser, plus the pages describing them. The bores are **generated**
by **`../../../../../../tools`**
([CLAUDE.md](https://github.com/Gernreich/trumpet/blob/main/tools/CLAUDE.md)); nothing in
this folder is authored by hand except this file.

One design, one pitch:

    bore/          10mm of air, a 16mm block

`bore/` sits directly under the design, the same shape as every sibling under
`walk/coil/`. The machinery for a second pitch beside it — `--blocksize`, `sizes.py`,
`folder_stack` climbing past a size folder — is there and unused. Cut this folder and you
have the tube; the two ends come from `../../../../../../..`.

**Where the parts come from.** The bell and the mouthpiece are generated in
`../../../../../../..`, which is where the two square-to-round generators live:

**All four sheets live in `../../../../../../..`, and none of them here.** A mouthpiece and
a bell suit any tube on the same channel, so holding them beside one bore would hide two
general parts inside one instrument. What decides it: neither end is touched by the way a
bore turns, so **only the tube belongs to an instrument**. **Every bore in the repository
is on the 10mm channel** — all 96 lattice sheets and every swept-curve one — so one pair
serves all of them, which is the whole reason they live in `../../../../../../..`.

**Do not move them back** without deciding what changed about that argument.

Regenerating any of the four means running a generator in `../../../../../../..` and
writing it where it belongs. Three things to know:

- **A generated sheet lands in the generator's own `cut-files/`**, not
  loose beside the script, and a bare run there rebuilds the shipped sheet byte for byte.
  So there is nothing to name and nothing to move: `mouthpiece-round.py` and
  `bell-round.py 17 --bore=10 --length=152 --mouth=80` write
  `mouthpiece-bore10-trumpet-parts-cut-files.svg` and
  `bell-round10-153mm-17rings-x3-rim86-cut-files.svg` where the readers already look.
- **A bare `bell-round.py` writes four budgets, not one.** The shipped sheet is among them
  and comes back identical; the other three are scratch in the same directory. Pass a ring
  budget to write one.
- **The generators number their own rings.** Every sheet there carries a
  `<g id="ring-numbers">`, and `bell-round.py` and `mouthpiece-round.py` write it themselves
  as the last step, so a regenerate keeps it, and writes an orientation tick beside each
  number with it. `--numbers=no` opts out; a numbering failure
  deletes the sheet rather than leaving an unnumbered one to be cut.

Sibling repositories — `knotwork-soundholes`, `living-hinge` and others — follow
the same conventions. Shared
documentation tooling lives in **`../../../../../../../lasermade-tools`** (its own repository).

## The design is one line

The bore is a walk through a lattice of blocks, and the walk is the whole specification:

```
N1 W3 U2 E3 N3 D3 W2 U3 N1
```

The first term is the way in and how far you go before anything turns; each term after it
turns where you stand and then travels *n* blocks, so **the bore is 1 + the sum of the
numbers** — 22 blocks here. Axes match
Minecraft: `U`/`D` are +Y/−Y, `N` is −Z, `S` is +Z, `E` is +X, `W` is −X.

**The walk is stored in `../../../../../../tools/walks/coil_fold2.txt`**,
and `regress.py` there names this repository as where its cut files live. Unlike
`../square-rise3`, which keeps its walk in its page, the file is the record here — the
page carries the same string in its `<div class="walk">` and `bore_split.py` will read
either, but they are only equal because the page was generated from the file.

Never transcribe the walk from memory. Read it out of the file.

## No lead-out to write, and why that is not a change

A walk used to be allowed to end with a bare letter naming the way you leave. This one
never did, and the notation no longer has one: you leave facing the last term. That costs
nothing here — a bare letter carrying your own heading only restated it, and **the six
SVGs come out byte-identical** written either way. It would have mattered if the exit
direction differed from the last term: `N1 U` turned the final block and stranded it,
which is a thing that can no longer be asked for.

## One block is 16mm, not 10

A block is 10 × 10 × 10mm of sound space wrapped in **3mm of wall**, so its outside is
**16mm**, and coring it out for air does not shrink it. A run of *N* blocks is **16N mm**
along the bore. The 22 blocks are 352mm of centreline at that size.

**The bore is not the block, and this design is gated on the block.** 10mm of air is
`--blocksize=16`. The two numbers are 6mm apart and naming them the same thing is the
mistake this section exists to stop: the bore is the air, the block is the air plus two
walls, and a folder named for one gated at the other cuts a tube nobody asked for.

Standard flags, uniform across the set — mixing `burn` changes finger joint fit while every
outside dimension still matches, which no drawing shows. Do not type them: `bore_split.py`
builds them from its own constants, so `--blocksize` moves the plan and the sheet together.

```
--blocksize=16 --thickness=3 --burn=0.075 --labels=0 --reference=0
--inner_corners=corner --spacing=0.5
```

`--thickness` is the SHEET and `--burn` is HALF the kerf, because that is what
Boxes.py means by it. Both come from `bore_split._common()` and change with
`--sheet=` and `--kerf=`; print them rather than trusting this block.

**Boxes.py is Florian Festi's**, GPL-3.0-or-later, at
<https://github.com/florianfesti/boxes>. It is an external dependency: a
checkout of it, not a copy in this repository.


`--pin_width=12` does not belong in that list: a 12mm tab does not fit a 10mm end
frame, which is the whole of the section below.

## The walk and the square are separate: the walk is here, the square is a flag

The **block pitch is the sound square plus two walls**, so a 10mm bore in 3mm stock is a
**16mm** block: `--blocksize=16`, and nothing else changes. Six sections, every turn a bend,
352mm of centreline. Cut the same walk at another pitch and you get the same shapes, the
same in and out faces, and different millimetres — pitch is the only thing a size
carries, so a change to the walk is a change to every size of it.

**A sheet named only for its shape cannot say which pitch it is.** `01_bend_DL.svg` names
a shape two pitches would share, and the only tell would be the sheet size. The bore is in
the name instead — `bore10-coil-fold2-01of06-bend-DL-buttin-cut-files.svg` — and each
sheet carries a `<title>` and `<desc>` saying its bore, its blocks and its faces, so a file
that has been renamed or moved can still be asked what it is.

`--pin_width` is the one flag that does not simply scale. SnakeBox defaults it to 12mm and
12mm does not fit a 10mm end frame — SnakeBox raises
`pin_width 12.0 is too wide for the 10.0mm end frame` rather than cutting something wrong.
`bore_split.py` derives it instead, as **0.48 × the sound square, floored at the finger
tooth** (`2 × thickness`, which does not shrink with the block) — so 6mm here, the floor
rather than the fraction.

**Omitting `--blocksize=16` fills `bore/` with parts at the stock pitch under this
set's names**, and nothing in the sheet's own name would say so. This design is an entry in
`regress.py` carrying a fourth field, the pitch, which is what keeps `--blocksize` honest.

## Every turn a bend — the rule that shapes the walk

A turn with no straight block to fold into is **stranded**: a single block left as its own
piece. Its opening frame has **three sides, not four**, so both neighbouring sections need
flattened plates butt-glued to it, plus tongues, plus an unfilled void inside the corner.
That gluing is the difficulty of the whole build. **This design strands nothing**, and
since 2026-09-15 `bore_split.py` refuses to write a walk that would.

Whether a turn can fold is decided over **every window of three consecutive terms**
(outer *A*, middle *m*, outer *C*), not once per walk. Consecutive terms are always on
different axes, so there are three cases:

| *A* and *C* | case | *m* must be |
| --- | --- | ---: |
| same axis, same direction | step | >= 1 |
| same axis, opposite direction | hairpin | >= 2 |
| different axes | coil | >= 3 |

`../../../../../../tools/CLAUDE.md` and the repository's
`README.md` both carry that table. Stating only the coil case lumps the other two together
as a fold that "costs nothing at any spacing" — right for steps, wrong for hairpins, and
this walk has two hairpins sitting exactly on the limit. Treat `bore_split.py` as the
authority, not any document and not a reimplementation of the rule.

Checking the current walk:

```
N1 W3 U2     3 axes         coil,    m = 3   OK
W3 U2 E3     W/E opposed    hairpin, m = 2   OK
U2 E3 N3     3 axes         coil,    m = 3   OK
E3 N3 D3     3 axes         coil,    m = 3   OK
N3 D3 W2     3 axes         coil,    m = 3   OK
D3 W2 U3     D/U opposed    hairpin, m = 2   OK
W2 U3 N1     3 axes         coil,    m = 3   OK
```

**Every window sits exactly on its minimum.** Decrement any interior term and it strands
turns — all seven do, the two hairpins two apiece and the rest one, and the generator
refuses each one rather than writing it. There is no slack
anywhere in this walk — it cannot be tightened by a single block
without paying for it, and any proposal to shorten it must add length elsewhere.

## The ends are as short as a section can be

Sections 1 and 6 have a single straight block either side of their turn, which is the least
a section can hold. That is deliberate — the mouth and bell runs are short so the
mouthpiece and the bell have room to seat — and it means **neither end will absorb
another block of trimming**. If a socket needs to seat *into* a section rather than butt
against it, the walk has to grow, not shrink.

## Six sections, six shapes

The bore's two outer ends are plain, so section 1 is `BDL~a` and section 6 `BRD~b` —
`buttin` and `buttout` in the filenames — and all six sections are distinct. Without
those plain ends, 1 and 4 would both be `BDL` and 3 and 6 both `BRD`.

**The outer ends carry no coupling because there is nothing to couple to.** The mouthpiece
meets one and the bell the other, and both present a flat plate that glues onto the end
face — the mouthpiece's station one, the bell's ring 0. A tab standing 3mm proud holds that
plate off the face and leaves the joint resting on the tab, which is what the bench
reports. `plain_ends()` in `bore_split.py` marks the first piece's entry and the last
piece's exit.

**A rename orphans files, and the gate counts them.** The generator does not remove what
it stops writing, and `check_sheets` globs the folder and adds three checks per sheet it
finds — so an orphan left behind buys checks on a part nothing is cutting. A rising check
count after a rename is a warning, not reassurance. Check the bore folder for orphans
after any regenerate.

This design has no repeated shape, so it is not the walk that checks a repeated shape
still gets its own number. Some other walk has to carry that.

## Colour is the cut order

Shared across all these repositories: **blue engraves, then green → orange → cyan →
black**; black is always the cut that frees the part; **violet `#8000ff` means skip**.

These nets use two stages — `#0000ff` engraves the section number on every part, then
`#000000` cuts.

## Cut files belong to the author

The author edits SVGs in Inkscape **during a session**. Treat every cut file as
concurrently modified:

- **Stage by name.** Never `git add -A` or `git add .` — it will sweep up an in-progress
  Inkscape save.
- **Never regenerate a cut file** the author has hand-edited (nesting, numbering, curve
  conversion) without asking.
- Regenerating rewrites **every** SVG in the folder you point it at, and only that
  folder. A changed section length renames a file — a last term of `N2` rather than `N1`
  writes `06_bend_RDD.svg` where the shipped sheet is `06_bend_RD.svg`. Since the names
  carry `NNofTT`, a walk that gains or loses a *section* renames **all** of them at once —
  six orphans, not one. The old files are not deleted for you. Check for orphans after a
  regenerate.

## The gate does not run under the system python

`bore_split.py --write` calls the gate itself, but `check.py` imports **shapely**, which is
not in `/usr/bin/python3`. Run as a script it therefore writes all six files and then dies
with `ModuleNotFoundError` — **the files are written and ungated**. shapely lives in the
Boxes.py virtualenv, so run the gate from there:

```sh
W="$(cat walks/coil_fold2.txt)"
D=.
~/Software/boxes/venv/bin/python check.py "$W" --blocksize=16 --files $D/bore/cut-files
```

`--files` only looks at the sheets as the machine sees them — bed fit, overlaps, engraving
on material — and never at the pitch, so it passes at any `--blocksize`. What the switch decides is the *geometry* half of the gate, which is recut
in-process. Pass the wrong one and 195 checks still say pass, having checked a design you
are not cutting.

`regress.py` passes `sys.executable` down to `check.py`, so it must be started with the
same interpreter or every design fails on the import.

`regress.py` also only passes `--files` **when the folder exists**. Its `DESIGNS` entries
name a subfolder rather than the repository root, so a rename of one — as well as of
this repository — leaves the gate printing `pass` while checking the geometry alone and
never touching these SVGs.

## Commands

```sh
G=../../../../../../../lasermade-tools
S=../../../../../../tools
```

**Test a walk without writing anything** — always do this before proposing a change:

```sh
cd $S && python3 bore_split.py --no-write "N1 W3 U2 E3 N3 D3 W2 U3 N1"
```

**Regenerate the cut files** (rewrites everything — ask first). The bores:

```sh
cd $S
W="$(cat walks/coil_fold2.txt)"
D=.
~/Software/boxes/venv/bin/python bore_split.py --blocksize=16 "$W" --write $D/bore
```

The mouthpiece and the bell are **not generated here** — they live in
`../../../../../../..` and are cut from there, not copied in. Each generator writes into its
own `cut-files/`, and engraves the ring numbers itself, so these two commands are the
whole job:

```sh
cd ../../../../../mouthpiece && python3 mouthpiece-round.py
cd ../bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80
```

Both rebuild the shipped sheet byte for byte from a clean tree; the bell's line also
writes nothing else, where a bare `bell-round.py` would add three more budgets.

**The refusal is on and there is no way off it.** `FOLD_TURNS` only biases the split
toward folding; `REFUSE_STRANDED` rejects a stranded turn outright, raises before anything
is written, names the sections at fault and exits 1 — so a walk that strands one cannot
reach this folder by accident. Note it is the command line that refuses: `check.py` does
not consult it, so a green `regress.py` says nothing about whether a walk can be written.

**Checks:**

```sh
python3 $G/svg-stroke-check.py --dir . --quiet   # stroke declared twice, disagreeing
cd $S && ~/Software/boxes/venv/bin/python regress.py      # every design in the library
```

The gate reports **196 checks, 0 failed** on this bore, and `regress.py` covers
24 designs. A check count that moves is worth chasing to the reason, which is the
whole argument of the section below. It does not look at the bell or the mouthpiece at all — those are checked by
`bell-round.py` and `mouthpiece-round.py` themselves, before they write, in
`../../../../../../..`. Nothing here should be cut from a file that has not passed one or the
other.

**After editing this bore's page** — regenerate it, then audit:

```sh
cd ../../../../../../switchback
python3 ../../lasermade-tools/md2html.py README.md index.html
python3 ../../lasermade-tools/doc-audit.py README.md --html index.html
```

**Read the audit output before pushing.** It ends with a pass/fail tally. `.doc-audit-ignore`
lists `bore_split.py` and `regress.py`, which the prose names but which live in
`../../../../../../tools`; the audit also insists every tracked file is named somewhere, which is
why the section table carries a file column.

## Publishing

GitHub Pages deploys from `main` via Actions — `.github/workflows/pages.yml` is the sibling
repositories' workflow, with a per-sha concurrency group rather than one for the whole
site. A single shared group means a run that wedges holds the lock and every later push
queues behind it; keyed on the sha, a stuck run can only block a re-run of its own commit.

`index.html` is generated and committed, not built on the server, so **a stale `index.html`
publishes stale content**. Pages has to be set to build from a workflow —
`gh api -X POST repos/Gernreich/trumpet/pages -f build_type=workflow` — or the
deploy has nowhere to publish to.

**Match the deploy to your SHA**, not to "the most recent run":

```sh
SHA=$(git rev-parse HEAD)
gh run list -L5 --json status,conclusion,headSha \
  -q ".[] | select(.headSha==\"$SHA\") | .status+\" \"+(.conclusion//\"-\")"
```

## Why this is filed as a coil and not a meander

Seen from the side the walk folds back on itself twice, which is what "fold2" describes
and which reads as a meander. Measured, it is a coil:
`spiral_metrics.js` gives it a coil axis (z, north–south), a handedness (right), and 450°
— 1.25 turns — of rotation about that axis. A real meander scores 0° and no handedness;
`../../meander/greek-key` does exactly that. The walk advances monotonically along one axis
while circulating in the other two, and that is a coil however it looks in projection.

The name is carried in five places at once: the folder, the walk file
(`tools/walks/coil_fold2.txt`), the `regress.py` entry and its label, the cut-file names —
which are built from the folder stack, so they read `bore10-coil-fold2-…` — and the page
title. They move together or not at all.
