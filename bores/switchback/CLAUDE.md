# CLAUDE.md

**The 25mm folder was deleted on 2026-09-03.** This repository is a 10mm bore now:
six sections on a 16mm block, 352mm of centreline. `sizes.html` went with it - it
existed to hold both sizes in one page. Notes below that named a 25mm sheet now name
the 10mm one; notes that are ABOUT having had two sizes are kept, because that is
where the play table's two measured figures came from.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A laser-cutting build repository, not a software project. The deliverable is **one bore at
two sizes**, six sections apiece, as **SVG cut files**
that someone sends to a laser, plus the pages describing them. The bores are **generated**
by the sibling repository **`../../tools`**
([CLAUDE.md](https://github.com/Gernreich/trumpet/blob/main/tools/CLAUDE.md)); nothing in
this repository is authored by hand except `README.md` and this file.

It holds **one design at two sizes**:

    25mm/  bore/          31mm block
    10mm/  bore/          16mm block

Cut one folder and you have the tube; the two ends come from `../../parts`.
`index.html` is still at the root and still the published page; nothing else is loose there.

**Three moves got here.** On 2026-08-31 the 25mm set came off the root into `25mm/bore/`,
so the sizes read as siblings rather than as a design and an afterthought; then the bores
went down a level into `<size>/bore/` and the mouthpiece and bell joined them, so a folder
was a whole instrument. On 2026-09-02 those two ends left again for `../../parts`.
Regenerating after each move left every bore SVG **byte-identical**.

**Where the parts come from.** The bell and the mouthpiece are generated in
`../../parts`, which is where the two square-to-round generators live:

**All four sheets live in `../../parts`, and none of them here.** They were held here
from 2026-08-31 to 2026-09-02 on the grounds that nothing else cut them — true of what had
been cut, false of what fits. The 25mm mouthpiece and bell suit any 25mm channel, and
`trumpet-coiled` and `trumpet-octagonal` are both exactly that, so keeping them here hid two
general parts inside one instrument. The rule `trumpet-coiled` states is what decides it:
neither end is touched by the way a bore turns, so **only the tube belongs to an
instrument**. The 10mm pair suit no other bore today, which is an accident of the lineup
rather than a principle, so they went too.

**Do not move them back** without deciding what changed about that argument.

Regenerating any of the four still means running a generator in `../../parts` and
moving the result across, because they write into their own directory. Two traps in that:

- **Name the sheet on the way out.** Both generators take the filename — a positional
  argument for `mouthpiece-round.py`, `--out=` for `bell-round.py` — and the names here put
  the bore in every one: `mouthpiece-bore10-trumpet-parts-cut-files.svg`, `bell-round10-153mm-17rings-x3-rim86-cut-files.svg`. The
  generated names say how a sheet was *made* (which script, what length), which is not what
  someone hunting for a part needs to read, and they left the 25mm sheets with no size in
  their names at all. Take the name and there is nothing to rename afterwards.
- **A bare `bell-round.py` writes `bell-round10-153mm-17rings-x3-rim86-cut-files.svg` back into `../../parts/bell`,**
  since it is one of that script's four standard budgets. Give it a budget and an `--out` so
  it writes one named sheet; otherwise move the result here or delete it, and do not leave a
  second copy there.
- **The generators number their own rings.** Every sheet here carries a
  `<g id="ring-numbers">`, and `bell-round.py` and `mouthpiece-round.py` write it themselves
  as the last step, so a regenerate keeps it. **A regenerate also adds an orientation tick**
  beside each number. All four sheets carry it as of 2026-09-02; the parts already glued up
  predate it. Those sheets are in `../../parts` now. `--numbers=no` opts out; a numbering failure
  deletes the sheet rather than leaving an unnumbered one to be cut. This used to be a
  separate command you had to remember, and forgetting it cost a sheet its numbering once.

Sibling repositories — `trumpet-coiled`, `trumpet-octagonal`, `torus-octagonal`,
`knotwork-soundholes`, `living-hinge` and others — follow the same conventions. Shared
documentation tooling lives in **`../../../lasermade-tools`** (its own repository).

## The design is one line

The bore is a walk through a lattice of blocks, and the walk is the whole specification:

```
N N1 W3 U2 E3 N3 D3 W2 U3 N1
```

The first letter is the way in; each term after it turns where you stand and then travels
*n* blocks, so **the bore is 1 + the sum of the numbers** — 22 blocks here. Axes match
Minecraft: `U`/`D` are +Y/−Y, `N` is −Z, `S` is +Z, `E` is +X, `W` is −X.

**The walk is stored in `../../tools/walks/trumpet_switchback.txt`**,
and `regress.py` there names this repository as where its cut files live. Unlike
`../coiled`, which keeps its walk in its page, the file is the record here — the
page carries the same string in its `<div class="walk">` and `bore_split.py` will read
either, but they are only equal because the page was generated from the file.

Never transcribe the walk from memory. Read it out of the file.

## No lead-out, and why that is not a change

A walk may end with a bare letter naming the way you leave. This one does not, and that
costs nothing: a term whose direction matches your heading does not turn, and a bare term
carries no distance, so after `N1` a trailing `N` would only restate a heading the walk
already has. Checked on 2026-08-29 by writing it both ways — **the six SVGs came back
byte-identical**. It would matter if the exit direction differed from the last term: `N1 U`
turns the final block and buys an elbow.

## One block is 31mm, not 25

A block is 25 × 25 × 25mm of sound space wrapped in **3mm of wall**, so its outside is
**31mm**, and coring it out for air does not shrink it. A run of *N* blocks is **31N mm**
along the bore. The 22 blocks are 682mm of centreline at that size.

**The folders are named for the bore, the switch is the block.** `25mm/bore/` is
`--blocksize=31` and `10mm/bore/` is `--blocksize=16`. The two numbers are 6mm apart and
naming them the same thing is the mistake this section exists to stop.

Standard flags, uniform across the set — mixing `burn` changes finger joint fit while every
outside dimension still matches, which no drawing shows. Do not type them: `bore_split.py`
builds them from its own constants, so `--blocksize` moves the plan and the sheet together.

```
--blocksize=31 --thickness=3 --burn=0.1 --pin_width=12 --labels=0
--reference=0 --inner_corners=corner --spacing=0.5
```

## `10mm/bore/` is the same walk, a smaller square

Added 2026-08-31. The **block pitch is the sound square plus two walls**, so a 10mm bore in
the same 3mm stock is a **16mm** block: `--blocksize=16`, and nothing else changes. Same
walk, same six sections, same shapes, same in and out faces, no elbows — 352mm of
centreline instead of 682. The two bores differ in pitch and in nothing else, so a change
to one is a change to both.

**The two folders shared every filename until 2026-09-03.** `01_bend_DL.svg` existed in both
bores, because the shapes genuinely are the same and only the pitch differs — but nothing
stopped you cutting the wrong one, and the only tell was the sheet size. The bore is now in
the name: `bore10-switchback-01of06-bend-DL-buttin-cut-files.svg`, and each sheet
carries a `<title>` and `<desc>` saying its bore, its blocks and its faces, so a file that
has been renamed or moved can still be asked what it is.

`--pin_width` is the one flag that does not simply scale. SnakeBox defaults it to 12mm,
sized for the 25mm square, and 12mm does not fit a 10mm end frame — SnakeBox raises
`pin_width 12.0 is too wide for the 10.0mm end frame` rather than cutting something wrong.
`bore-generator` derives it as **0.48 × the sound square, floored at the finger tooth**
(`2 × thickness`, which does not shrink with the block) — 6mm at the 10mm bore, and the
fraction's 12 at 25,
so the 31mm files did not move: they were regenerated after the change and came back
**byte-identical**.

Regenerating one set does not touch the other — they are separate `--write` targets — but
**omitting `--blocksize=16` fills `10mm/bore/` with full-size parts under the small set's
names**. Both are entries in `regress.py`; the 10mm one carries a fourth field, the pitch.

## No elbows — the rule that shapes the walk

An **elbow** is a single block stranded as its own piece because the turn in it had no
straight block to fold into. Its opening frame has **three sides, not four**, so both
neighbouring sections need flattened plates butt-glued to it, plus tongues, plus an
unfilled void inside the corner. That gluing is the difficulty of the whole build.
**This design has none.**

Whether a turn costs an elbow is decided over **every window of three consecutive terms**
(outer *A*, middle *m*, outer *C*), not once per walk. Consecutive terms are always on
different axes, so there are three cases:

| *A* and *C* | case | *m* must be |
| --- | --- | ---: |
| same axis, same direction | step | >= 1 |
| same axis, opposite direction | hairpin | >= 2 |
| different axes | coil | >= 3 |

`../../tools/CLAUDE.md`, `../coiled/CLAUDE.md` and that repository's
`README.md` all stated only the coil case until 2026-08-29, lumping the other two together
as a fold that "costs nothing at any spacing" — right for steps, wrong for hairpins, and
this walk has two hairpins sitting exactly on the limit. All three now carry the table
above. Treat `bore_split.py` as the authority, not any document and not a reimplementation
of the rule.

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

**Every window sits exactly on its minimum.** Verified 2026-08-29 by decrementing each
interior term in turn: all seven produce elbows, the two hairpins two apiece and the rest
one. There is no slack anywhere in this walk — it cannot be tightened by a single block
without paying for it, and any proposal to shorten it must add length elsewhere.

## The ends are as short as a section can be

Sections 1 and 6 have a single straight block either side of their turn, which is the least
a section can hold. That was deliberate — the mouth and bell runs were cut back to leave
room for the mouthpiece and the bell to seat — but it means **neither end will absorb
another block of trimming**. If a socket needs to seat *into* a section rather than butt
against it, the walk has to grow, not shrink.

## Six sections, six shapes — and why that changed

Sections 1 and 4 were both `BDL` and sections 3 and 6 both `BRD` until 2026-08-31, when the
bore's two outer ends were made plain. Section 1 is now `BDL~a` and section 6 `BRD~b`,
`_buttin` and `_buttout` in the filenames, so all six are distinct.

**The outer ends carry no coupling because there is nothing to couple to.** The mouthpiece
meets one and the bell the other, and both present a flat plate that glues onto the end
face — the mouthpiece's station one, the bell's ring 0. A tab standing 3mm proud holds that
plate off the face and leaves the joint resting on the tab. Reported from the bench.
`plain_ends()` in `bore_split.py` had the mechanism already but only ever fired for ports;
it now always marks the first piece's entry and the last piece's exit.

**That rename orphans files, and the gate counts them.** `01_bend_DL.svg` and
`06_bend_RD.svg` are no longer written by anything and had to be deleted by hand — the
generator does not remove what it stops writing. Until they were, the gate reported **200
checks instead of 194**: `check_sheets` globs the folder and adds three checks per sheet it
finds, so two stale files bought six checks on parts nothing was cutting. A rising check
count after a rename is a warning, not reassurance. Check both bore folders for orphans
after any regenerate.

This design was in `regress.py` because it was the only one checking that a repeated shape
still gets its own number. It no longer has a repeated shape, so **that coverage has moved
off this design** — if you need it back, some other walk has to carry it.

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
- Regenerating rewrites **every** SVG in the folder you point it at — and only that
  folder, so the two sizes have to be rebuilt one command each. A changed section length
  renames a file: `06_bend_RDD.svg` became `06_bend_RD.svg` when the last term went from
  `N2` to `N1`. Since the names carry `NNofTT`, a walk that gains or loses a *section*
  now renames **all** of them at once — six orphans, not one. The old files are not
  deleted for you, in either folder. Check both for orphans after a regenerate.

## The gate does not run under the system python

`bore_split.py --write` calls the gate itself, but `check.py` imports **shapely**, which is
not in `/usr/bin/python3`. Run as a script it therefore writes all six files and then dies
with `ModuleNotFoundError` — **the files are written and ungated**. shapely lives in the
Boxes.py virtualenv, so run the gate from there:

```sh
W="$(cat walks/trumpet_switchback.txt)"
D=.
~/boxes/venv/bin/python check.py "$W" --files $D/25mm/bore
~/boxes/venv/bin/python check.py "$W" --blocksize=16 --files $D/10mm/bore
```

`--files` only looks at the sheets as the machine sees them — bed fit, overlaps, engraving
on material — and never at the pitch, so it passes on either folder at either
`--blocksize`. What the switch decides is the *geometry* half of the gate, which is recut
in-process. Pass the wrong one and 194 checks still say pass, having checked a design you
are not cutting.

`regress.py` passes `sys.executable` down to `check.py`, so it must be started with the
same interpreter or every design fails on the import.

`regress.py` also only passes `--files` **when the folder exists**. Both `DESIGNS` entries
name a subfolder now, not the repository root, so a rename of either one — as well as of
this repository — leaves the gate printing `pass` while checking the geometry alone and
never touching these SVGs.

## Commands

```sh
G=../../../lasermade-tools
S=../../tools
```

**Test a walk without writing anything** — always do this before proposing a change:

```sh
cd $S && python3 bore_split.py --no-write --refuse-elbows "N N1 W3 U2 E3 N3 D3 W2 U3 N1"
```

**Regenerate the cut files** (rewrites everything — ask first). The bores:

```sh
cd $S
W="$(cat walks/trumpet_switchback.txt)"
D=.
python3 bore_split.py --refuse-elbows "$W" --write $D/25mm/bore
python3 bore_split.py --blocksize=16 --refuse-elbows "$W" --write $D/10mm/bore
```

The mouthpiece and the bell are **not generated here** — they come from
`../../parts` and are copied in. Those generators write into their own directory, so
copy the result across afterwards. They engrave the ring numbers themselves, so these two
commands are the whole job:

```sh
cd ../../parts/mouthpiece
python3 mouthpiece-round.py --bore=10 --rim=17 --layout=trumpet mouthpiece-bore10-trumpet-parts-cut-files.svg
cd ../bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80 \
    --out=bell-round10-153mm-17rings-x3-rim86-cut-files.svg
```

**Always pass `--refuse-elbows` here.** This is a build repository, and the standard for a
build is zero elbows — not the `FEWEST_ELBOWS` default, which only minimises them. The
switch raises before anything is written and names the sections at fault, so a walk that
costs one cannot reach this folder by accident.

**Checks:**

```sh
python3 $G/svg-stroke-check.py --dir . --quiet   # stroke declared twice, disagreeing
cd $S && ~/boxes/venv/bin/python regress.py      # every design in the library
```

The gate reports **194 checks, 0 failed** on each of the two bores, and `regress.py` covers
25 designs. It does not look at the bell or the mouthpiece at all — those are checked by
`bell-round.py` and `mouthpiece-round.py` themselves, before they write, in
`../../parts`. Nothing here should be cut from a file that has not passed one or the
other.

**After editing `README.md`** — regenerate the page, then audit:

```sh
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
```

**Read the audit output before pushing.** It ends with a pass/fail tally. `.doc-audit-ignore`
lists `bore_split.py` and `regress.py`, which the prose names but which live in
`../../tools`; the audit also insists every tracked file is named somewhere, which is
why the section table carries a file column.

## Publishing

GitHub Pages deploys from `main` via Actions — `.github/workflows/pages.yml` is the sibling
repositories' workflow, with a per-sha concurrency group rather than one for the whole
site. A single shared group means a run that wedges holds the lock and every later push
queues behind it; keyed on the sha, a stuck run can only block a re-run of its own commit.

`index.html` is generated and committed, not built on the server, so **a stale `index.html`
publishes stale content**. Pages was switched to Actions with
`gh api -X POST repos/Gernreich/trumpet/pages -f build_type=workflow`
on 2026-08-29; without that the deploy has nowhere to publish to.

**Match the deploy to your SHA**, not to "the most recent run":

```sh
SHA=$(git rev-parse HEAD)
gh run list -L5 --json status,conclusion,headSha \
  -q ".[] | select(.headSha==\"$SHA\") | .status+\" \"+(.conclusion//\"-\")"
```

## The name, twice changed

It was **twin switchback** until 2026-08-29, when it was moved out of the shared library at
`~/LaserMadeMusic/test` and given a repository — and **trumpet-final-youtube-candidate**
until 2026-09-01, when that name was dropped because it claimed a decision that had not been
made. Which bore the video ends up using kept changing, and a repository name is a bad place
to record a preference.

**trumpet-switchback** describes the design instead: the bore folds back on itself twice,
which is exactly the two hairpins the elbow rule counts. That is a fact about the walk, so
it cannot go stale. It also matches `../coiled` and `../octagonal`, which
are named the same way.

Renamed together each time: the folder, the walk file, the `regress.py` entries, the GitHub
repository, the Pages URL and the artifact. If you find either old name anywhere, it is a
leftover.
