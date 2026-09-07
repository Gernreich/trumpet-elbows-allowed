# CLAUDE.md

**One bell and one mouthpiece, both at the 10mm bore.** That is the default the
square-to-round pair take when `--bore` is not given, and it is the channel every tube
under `bore/` is cut to. Another bore is `--bore=N` away; nothing is shipped at one.

Where a note below is *about* a sheet that is no longer here - the legacy mouthpiece
layout, the hand-nested square bell - it is kept, because the reason it was written has
not stopped being true.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The **bell** and the **mouthpiece**, shared by every trumpet in this repository rather
than owned by one of them. Every bore under `bore/` is cut at the 10mm channel, so one
bell and one mouthpiece serve all of them; only the tube between them differs. The
octagonal trumpet, which was on a channel of its own, left this repository on
2026-09-05.

`--bore` on the two square-to-round generators still makes the parts to suit any
channel, and nothing about the pair is tied to 10. See `--bore` is the channel, and
only that below.

**Every mouthpiece and every bell lives here, whichever instrument cuts it.** The four
fold2 sheets moved out to that design's folder on 2026-08-31 and came back on 2026-09-02:
holding them there rested on "nothing else cuts them", which described what had been cut
rather than what fits. A mouthpiece and a bell suit any tube on the same channel, and
the coiled and octagonal trumpets were both on one, so two general parts were
hidden inside one instrument. The rule is the one the coiled trumpet states: neither end
is touched by the way a bore turns, so **only the tube belongs to an instrument**, and a
bore directory holds only bore.

**A bare `bell-round.py` writes all four budgets**, the 17-ring among them. Pass a ring
budget when you want one sheet, and `--out` to name it.

**The README is gone.** Every `README.md` and `index.html` under `trumpet/` was
removed on 2026-09-05, pending one new writeup for the trumpet as a whole once
the renaming and reorganising is finished. Git has them all. Until it exists,
this file is the documentation, and any recipe below that renders or audits a
README is waiting on that writeup rather than describing something present.

It used to say: read `README.md` first, it carries the geometry. This file covers the code.

## The bore is cylindrical

Constant 10 × 10mm section end to end, 16mm outside in 3mm ply. **The only part of a trumpet that flares is the bell.** Do not
describe a bore, a band or a tube as flaring; say curved if it curves.

## `--bore` is the channel, and only that

`bell-round.py --bore=N` and `mouthpiece-round.py --bore=N` set the air channel and derive
the plate as **N + 6** — a 3mm wall each side, exactly as the tube is. Nothing else moves:

- **The ply is still 3mm and a ring still rises 3mm.** That was never scalable and is not
  now. A smaller bore does not buy thinner laminations.
- **The mouthpiece throat stays ø3.66.** A #27 drill is a real trumpet throat, and a
  mouthpiece is sized by the lip at one end and the drill at the other. `--rim` likewise:
  16–17mm is a trumpet rim whatever it is bolted to. The 10mm mouthpiece is therefore a
  full-size mouthpiece on a quarter-size horn, deliberately.
- **`bell.py` and `mouthpiece.py` do not take it.** Only the square-to-round pair, which is
  what anything new is cut from. Do not add it to the other two without being asked; both
  have cut parts numbered against them.

Filenames carry the bore — `bell-round10-153mm-17rings-x3-rim86-cut-files.svg`,
`mouthpiece-bore10-trumpet-parts-cut-files.svg` — because two parts of the same length on different
bores are different parts and only one of them fits your tube. Those two names are what a
bare run writes into this directory, which is where they belong — every mouthpiece
and every bell lives here, and a bore directory holds only bore.

**The mouthpiece reproduces exactly**: `mouthpiece-round.py --rim=17` gives the shipped
sheet byte for byte. The default rim is 16.5 and the shipped one is 17, which is a choice
about the lip rather than about the bore. **The bell reproduces exactly too**:
`bell-round.py 17 --bore=10 --length=152 --mouth=80` gives the shipped sheet byte for
byte. The built height rounds up to the 153mm in its name, and `--mouth=80` is ø86 at the
outer, which is the rim the name carries. A bare run writes all four ring budgets at the
default rim instead.

## `--mouth` is the hole; `--rim` is the square bell's width

`--rim` feeds the profile, and the area law then opens the section out by `2/√π` where it
is a circle, so **`--rim=80` delivers a ø90.3 mouth**. `--mouth=80` inverts that in closed
form and gives 80. The outer diameter is 6mm larger again — the rim ring's 3mm lap each
side — and the `section` line prints both, so a wall floor biting at the rim would show up
rather than pass as the number that was asked for.

The README's **"Rim diameter" column is the outer**, and always has been. `bell-round.py`
reports `rim = 2*rings[-1]["oh"]`, which is the outer too. Neither is the hole.

## A bell file is cut more than once

Each of the four bell cut files draws every ring **once**. The `Build` column in the README
says how many 3mm laminations a ring is, and that is how many times the sheet goes through
the machine: the 10-ring bell is 7 ply, so 7 passes and 70 pieces. Cut once it yields a
30mm bell instead of 210mm. Only the 67-ring file is a single pass. Counting shapes in the
SVG answers a different question than "how many pieces".

## Ring 0 is a flange, and the throat is the bore's channel

The bore ends in a **square annulus of ply 3mm wide** — 10mm inside, 16mm out. Ring 0 has to
cover all of it. It is a sharp 22mm square with a 10mm square hole, standing 3mm proud of the
plate all round: the one ring whose outer is not the next station offset, and **wider than
the several rings above it**. A run prints it — `flange  ø10 aperture in a ø22 square`.

This was wrong until 2026-08-26. The throat was taken from the bore's *outside* rather than
its channel, so ring 0 sat entirely outside the end face and did not overlap it **at all**;
the only contact was the tube's outer wall against a thin lip, and the airway stepped out by
a full wall thickness per side. It was reported as gaps at the joint, which is exactly what it
was. The mouthpiece had it right all along — its station one is a bore-sized square hole in a
plate-sized square — and the bell now does the same.

**The lap is 3mm, not 1.5.** 1.5mm is the width of the glue land, and it left nothing for
kerf or for a ring set down slightly off centre; joints opened up along the bell. `--lap`
moves it.

**Consequences that bite tooling:**

- **Ring sizes are no longer monotonic**, because the flange is wider than the rings above
  it. Anything recovering assembly order by sorting on the OUTER diameter is now wrong.
  `verify_bell.py` sorts on the **aperture** instead — the airway only ever opens, so that
  is assembly order on any sheet however it was nested. `number_rings.py` refuses and wants
  `--order=document`.
- **The minimum-wall floor no longer binds.** `wall = gain + LAP >= 3mm` on its own, so the
  old `max(gain, MINWALL - LAP)` floor is inactive and the profile follows the Bessel curve
  exactly instead of being inflated by it. That is why all four bells now reach the same
  129.0mm rim where the 67-ring used to run out to 145.7.

## Two bell families

`bell.py` makes four bells that are **square end to end**. `bell-round.py` makes four whose
section **morphs from the bore's square to a round rim** — every ring a rounded square,
corner radius 0 at station one and equal to the half-width at the rim.

They differ in what is held to the Bessel profile. The square bells follow the half-width;
the round ones follow the **area**, because a circle inscribed in a square has 21% less of
it. That is why the round rim is ø144.8mm where the square one is 129.0mm square, and why
the two rim numbers are not meant to match.

`bell-round.py` checks its own sheets — the 1.5mm seat, the 2mm wall and the bore never
stepping inward — and writes nothing if a check fails. It has to: `verify_bell.py` reads an
arc as proof it is looking at the mouthpiece and skips these files by design.

**The 2mm wall floor is directional.** Rounding a corner pulls the diagonal in, so a
station rounder than the one below reaches less far into its corners than its extra
half-width suggests. A wall of 2.6mm across a flat can be 1.97mm through a corner. Both
floors are solved in closed form in `stations()`; do not reduce it to one.

## Three viewers, and only one of them turns

`bell-view.py` and `mouthpiece-view.py` each draw ONE fixed isometric SVG. That
is the right thing for a page and no use for looking at an object.
`part-view.py` draws the same geometry as a solid you can drag, in the family
of the bore viewers in `bore-ribbon`, and writes `<name>-turn.html` beside each
cut file.

```sh
python3 part-view.py bell/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
python3 part-view.py mouthpiece/mouthpiece-bore10-trumpet-parts-cut-files.svg
```

It reads the ring sizes with **`bell-view.py`'s own `sections()`**, executed out
of that file rather than copied, so the two cannot come to disagree about what
is on a sheet.

Two things it has to do that the isometrics do not:

- **`prof()` returns exactly n points whatever the corner radius.** `outline()`
  gives four points for a square and `4*(per+1)` for anything rounded, and a
  stack that mixes the two cannot be zipped into quads. It resamples by arc
  length, which also stops a circle bunching its points at corners it does not
  have.
- **Every ring draws its full top annulus.** Working out which part the ring
  above covers is unnecessary: the painter's order overdraws it.

## The view generators broke when the apertures moved colour

On 2026-09-03 the apertures were split into their own orange group so they cut
before the outline that frees the part. **Both view generators found rings by
looking for one path holding two subpaths**, so after the split they found none
at all - `bell-view.py` on all ten bells, `mouthpiece-view.py` on all three
mouthpieces. Nothing noticed, because nothing ran them afterwards.

`mouthpiece-view.py` was broken twice over: it also opened
`mouthpiece-parts-cut-files.svg` by name, and that file was renamed the same
day to carry its bore and layout.

Both now read the two groups and **pair ring i's aperture with ring i's
outline, in file order.** That is what the generators write - both groups come
off one list - and it is assembly order.

**Do not pair by size, and do not pair by containment.** Sorting each list and
zipping works only while a profile grows monotonically; a mouthpiece narrows to
its throat and opens again, so it drew the cup as a stack of alternating
bulges. Containment is no better: an aperture also contains any ring nested in
its spare space. Both were tried here and both drew a plausible object that was
not the one on the sheet - which is the warning the mouthpiece viewer's own
docstring already carried, about sorting by diameter.

`mouthpiece-view.py` takes `--src=`; a bare argument is still the OUTPUT, as it
always was. Making the source positional wrote a display SVG straight over a
cut file.

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**; black frees the part; violet
`#8000ff` means skip.

**No sheet here is nested today.** Every one cuts in a single black stage with its numbers
in blue. The **black → red ramp** `ramp_bell.py` applies belongs to a *nested* sheet, where
a ring sitting inside another's aperture must be cut first or it is freed with the waste;
one stage per ring by size says so in the only channel an importer always reads. The
hand-nested 17-ring sheet that needed it was deleted 2026-08-25 and regenerated flat.
`verify_bell.py` still checks that a ramp, if present, rises with ring size.

## Cut files belong to the author

These have been cut. Treat every SVG as concurrently modified in Inkscape:

- **Stage by name.** Never `git add -A` or `git add .`.
- **`bell.py` with no argument rewrites all four sheets.** Pass a ring budget to
  regenerate one — `python3 bell.py 20` writes only the 17-ring. The other three used to
  lose their numbering to a full run; they no longer do, but rewriting four sheets to
  change one is still four files to review instead of one.
- **A mouthpiece sheet is named by its bore AND its layout**, and says both in its
  `<title>`. Two mouthpieces exist — `legacy` and `trumpet` — with the same ring
  count, bore, throat and rim, and until 2026-09-02 they carried character-for-character
  identical titles while their profiles were nothing alike: 9 backbore rings plus 17
  entrance rings against 26 backbore and none. Neither the filename nor the file itself
  could tell you which part you had. Naming only the non-default parameter is what caused
  it, so both now go in every name.
- **A ring is two cuts in two colours.** Orange `#ff8000` takes the aperture, black
  `#000000` the outline, and the orange group is written first. One path holding both, as
  these sheets had until 2026-09-02, lets a per-colour job free the part before its hole is
  in. Anything that reads a ring back -- `number_rings.py`, `bell-section.py` -- pairs the
  two by their shared centre, and still handles the one-path form for hand-edited files.
- **Every label carries an orientation mark**, a short tick on the baseline right of the
  last character. A ring is a circle, so nothing about the part says which way up it was
  engraved — and turned over, `3` and `E` swap, and so do `6` and `9`. Find the tick and
  every character identifies itself. `--mark=no` leaves it off.

  **The shipped sheets do not have it, and that is deliberate.** Adding it renumbers
  nothing — ring 6 still reads 6 — but it does change every file, and those files describe
  rings already cut and glued. The mark starts with the next sheet that goes on the bed. A
  ring without one is exactly as readable as it has always been, so a mixed stack is never
  worse than what you have.

  It costs one step of the fitting search on the tightest sheet in the repository, the
  30-ring mouthpiece: smallest character 1.88mm becomes 1.78mm. The bells are limited by
  their wall rather than by label width and do not move at all.
- **Every sheet generator numbers its own rings** — `bell.py`, `bell-round.py`,
  `mouthpiece.py`, `mouthpiece-round.py` and `mouthpiece-cup.py` all call
  `number_rings.py --order=document` as the last step of writing a sheet, and say so in the
  run report. `--numbers=no` writes a bare sheet. If the numbering fails the sheet is
  **deleted**, because a sheet left on disk gets cut and an unnumbered one is rings nobody
  can order.

  This was not always so: numbering used to be a separate command you had to remember, and
  a regenerate silently threw the engraving away — `mouthpiece-round.py` with no arguments
  cost `mouthpiece-bore10-trumpet-parts-cut-files.svg` its numbering on 2026-08-31, restored from git. Changing
  the default was checked by regenerating all four sheets the fold2 trumpet is cut from
  and diffing against the cut files: **byte-identical**, numbering included.

  **`mouthpiece-cup.py` continues a stack rather than starting one.** Its rings are glued
  on top of a mouthpiece already numbered from 0, so it numbers from `--start`, which
  defaults to **23** — `mouthpiece.py` writes rings 0 to 22 and ends at the ø10.06 the cup
  stacks onto. Pass `--onto` and that no longer holds, so `--start` stops being optional and
  the script refuses rather than guessing. Numbering a cup from 0 would put a second ring 0
  in one mouthpiece, which is the confusion the numbers exist to prevent.

  `mouthpiece.py` had no option parsing at all — `sys.argv[1]` was the output path — so it
  would have written a file called `--numbers=no`. Options are separated from the path now.
- **These scripts have no `--help`**; a bare run to see the options IS a run. Read the
  docstring, or write to a scratch path.
- Verify a hand-edited bell with `verify_bell.py` rather than diffing path data — once
  paths are converted to Bézier curves, a byte diff says nothing. **It reads the SQUARE
  bells only.** The square-to-round ones are drawn with arcs and lap by whatever their
  3mm wall leaves rather than by a fixed amount, so it skips them and says so — a checker
  that answers about a file it does not understand teaches you to ignore it.
- **`verify_bell.py` pairs an aperture with the outline concentric with it**, so it reads
  both the one-path form and the two-group form the generators write now, and survives a
  nest that reorders the paths. It was blind to the two-group form from 2026-09-03 until
  2026-09-06: it found no rings, called every generated sheet "a section drawing or a
  sheet of something else", and skipped it — passing silently on sheets nobody had
  checked. An outline that finds no aperture is now reported as what it is, a solid disc.

## The mouthpiece's taper is 2.5mm because it has to be

`mouthpiece-round.py` starts square on the bore and is round by the throat. It could not be
built on `mouthpiece.py`'s 4mm backbore steps, and this is the reason:

A ring's outer is its aperture offset by `WALL`, so the seat is `WALL` minus how far two
apertures differ in that direction. Across the flats that is half the aperture step — a 4mm
step leaves 3.00 − 2.00 = 1.00mm. **Through the corners of a sharp square the same step
costs step × √2 = 2.83mm, leaving 0.17mm**, and giving a corner any radius pulls the
diagonal in further and takes it negative. At 2.5mm the corner seat is 1.23mm. The cone is
identical, sampled finer: 9 backbore rings instead of 6, 78mm instead of 69mm.

**Roundness is not scheduled**, unlike the bell's `--morph`. Each station takes the largest
corner radius that still leaves `MINSEAT` in both directions, in closed form, so the part
rounds as fast as its geometry allows. The generator refuses to write rather than emit a
part that cannot stack — `--taper=4.0` exits with the reason.

`mouthpiece-view.py` draws the OLD mouthpiece and only it: every ring a circle apart from a
hardcoded square plate with a round bore, and "23 rings" in its label. Do not point it at
`mouthpiece-bore10-trumpet-parts-cut-files.svg`. `bell-view.py` reads corner radii and would draw it, but calls
whatever it is given a bell.

## Numbering a sheet

`number_rings.py` engraves each ring's hex index, smallest = 0, in `#0000ff` in its own
`<g id="ring-numbers">` written before the black that frees the part. Re-running replaces
that group rather than adding a second one, and it refuses to write if any cut path's `d`
changed.

**Number in ASSEMBLY order, not size order.** On a bell the two agree, because it
telescopes. On the mouthpiece they do not: the airway narrows from the bore to 5 then
opens 3.66 -> 10.06, so the two runs share diameters and sorting by size interleaves them. The tool now
refuses to guess when the sizes are not monotonic in the file — pass `--order=document`,
which is assembly order for anything these generators write. `b` and `d` are lower case
in the seven-segment table this replaced, because there upper-case B was 8 and D was 0. The polyline glyphs share no shape, so hex is upper case throughout.

**A ring is two concentric outlines — and so is an engraved 0, 4, 6, 8 or 9.** That is why
`bell-section.py` reads 25 rings in the hand-labelled 17-ring sheet, and `number_rings.py`
would have inherited the same miscount. Subpath counting cannot separate them; **colour
can**, and here it already does — blue is engraving, never a part. A sheet that already
carries blue is refused outright rather than given a second set of labels.

**It reads only what the generators write** — `M`, `H`, `V`, `A`, `Z`. `bell-round10-153mm-17rings-x3-rim86-cut-files.svg`
came back from Inkscape in relative commands and quadratic curves (`h l m q v z`), which this
refuses by name rather than half-parsing into a plausible wrong answer.

**Digits are polyline outlines, not `<text>`** — the same sixteen `bore_split.py` puts on a bore section — and each is grown to the largest size
that still lands on material — measured against the real curves, not assumed from the wall.
That matters because the wall is not the room available: the rim ring's aperture is nearly a
circle, so there is no flat to sit a digit on, and a radius-by-angle lookup that samples only
the ends of a straight run reads the bottom of a square as its corner, 21.9mm instead of
15.5mm. Sample `H` and `V`, not just their endpoints.

## The mouthpiece has two layouts, and only one of them is worth cutting

Both are 30 rings and 90mm. `--layout=trumpet`, the default, puts 75mm into the backbore
and keeps a 12mm cup, which is how a real mouthpiece is proportioned. `--layout=legacy` has
a 27mm backbore and 51mm of near-cylindrical entrance on the LIP side of the throat, close
to inverted — a 48mm-deep cup by any honest reading.

**`legacy` is a record of a part, not a design to cut.** A mouthpiece was built to it before
the trumpet layout existed and its rings are numbered for it, which is the only reason its
sheet is kept. It was the default, and called `asbuilt`, until 2026-09-03: that name said
when the profile was made rather than whether to cut it, and being the default meant a bare
run produced the profile nobody wants. `--layout=asbuilt` now exits saying so rather than
quietly doing something else.

Check `mouthpiece-bore10-trumpet-parts-cut-files.svg` still comes out byte-identical if you
touch the profile code.

**The wall is per-ring, not a constant.** The seat above a ring is the wall of whichever
ring is narrow at that joint, less half the aperture step, so a step that big needs a wall
to match: a trumpet cup runs ø3.66 to ø11.25 in one ring and wants 4.80mm. `Ws` is computed
from the steps and the roundness bound reads the wall on the NARROW side of each joint —
taking the smaller of the two refuses a cup the walls can actually hold.

## Two routes to the same mouthpiece

`mouthpiece-round.py` writes the whole part — backbore, entrance, bowl — as 30 rings, 90mm.
`mouthpiece-cup.py` writes the bowl alone, four rings that stack on a mouthpiece glued
before the bowl existed, numbered from 26 with `--start=26`.

**The two routes must stay identical ring for ring**, and are: the unified sheet's first 26
apertures match the old 26-ring part and its last 4 match the extension, so both number
`0`..`19` then `1A`..`1d`. Both use the same ellipse arc for the bowl. Change one and check
the other, or a part half-built by one route cannot be finished by the other.

## Known wrong, deliberately not fixed

**`mouthpiece.py` names `cup` and `backbore` backwards.** Its `cup` — the run from the
bore down to the neck — is the end that meets the bore, which is anatomically the backbore.
Every number is correct; only the variable names and the `<desc>` string the script emits
are reversed. Fixing it means regenerating a part that has been cut.

## A smaller bell is a shorter profile, not a scaled one

The throat is the bore's own channel, and a ring rises 3mm because the ply does. Neither is
ours to scale. `--length`, `--rim` and `--gamma` on both generators move the profile instead;
`--rim` is the bore's diameter **at** the rim, before the wall, so it reads a wall smaller
than the "Rim diameter" the README tabulates.

**Every filename states its length, and `bell-round`'s its bore as well** — the length
used to appear only when it was not 201mm, so a 100mm bell landing on 17 rings would
overwrite `bell-round10-153mm-17rings-x3-rim86-cut-files.svg`, which is hand-nested and not reproducible from
the script. It also left `bell-round-67rings.svg` saying nothing about the tube it fits or
how long it was, while `bell-round-99mm-11rings.svg` said one of the two. Both go in every
name now, and in the `<title>`. Do not "tidy" `STEM` back.

Short bells make the overshoot matter: 99mm divides evenly by 3, 9 and 33mm of rise, 100mm
divides by none of them. Both generators report it and name a length that would have worked.

## The rim ring is not always the steepest

The profile is defined over 201mm, but a whole number of rings at each rise usually
overshoots it — the 14-ring bell is 14 × 15 = 210mm. **A ring past 201mm is still a full
`step` tall** and simply has less curve left to draw, so it flares less than the ring below.
The 14-ring's steepest ring is 36.8° and its rim ring 24.6°.

`bell.py` therefore reports the **steepest** angle, with the rim ring in brackets where the
two differ, and the README's Angle column is the steepest. Dividing by anything other than
`step` would be wrong: the ring really is `plies` laminations of 3mm ply, whatever the curve
did over its height.

## Commands

Generators read hardcoded relative filenames — run each from its own directory:

```sh
cd bell && python3 bell.py            # all four square bells
cd bell && python3 bell.py 20         # one, at most 20 rings
cd bell && python3 bell-round.py      # all four square-to-round bells
cd bell && python3 bell-round.py 67 --morph=flare --law=width
cd bell && python3 bell-round.py --length=99 --rim=80   # a half-size bell
cd bell && python3 bell-round.py --bore=10 --length=152 --mouth=80   # the 10mm bell
cd mouthpiece && python3 mouthpiece-round.py --bore=10 --rim=17 --layout=trumpet
cd bell && python3 bell-section.py bell-round10-153mm-17rings-x3-rim86-cut-files.svg
cd bell && python3 bell-view.py bell-round10-153mm-17rings-x3-rim86-cut-files.svg
cd bell && python3 bell.py 20 && python3 verify_bell.py \
      bell-square10-204mm-17rings-x4-rim129-cut-files.svg   # SQUARE bells only
cd bell && python3 number_rings.py bell-round10-153mm-17rings-x3-rim86-cut-files.svg   # engrave 0..A
cd bell && python3 number_rings.py ../mouthpiece/mouthpiece-bore10-trumpet-parts-cut-files.svg --order=document
cd mouthpiece && python3 mouthpiece-round.py    # square on the bore, round by the throat
cd mouthpiece && python3 mouthpiece-cup.py      # the bowl that stacks on its end
cd mouthpiece && python3 ../bell/number_rings.py mouthpiece-cup-parts-cut-files.svg --start=26  # not kept
cd mouthpiece && python3 mouthpiece.py          # the previous 23-ring design
cd mouthpiece && python3 mouthpiece-view.py     # draws the previous design ONLY
```

`bell-section.py` counts a path with two subpaths as a ring, so **outline** digits — a
hand-drawn 0, 4, 6, 8 or 9 has a counter — register as rings and it read 25 in the old
hand-labelled 17-ring sheet. `number_rings.py` draws single-stroke polyline digits
with no counters, so a sheet it numbers still counts correctly.

**After editing either document** — regenerate the page, then audit both:

```sh
G=../../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
python3 $G/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.**
