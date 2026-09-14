# CLAUDE.md

**One bell and one mouthpiece, both at the 10mm bore.** That is the default the
square-to-round pair take when `--bore` is not given, and it is the channel every tube
under `bore/` is cut to. Another bore is `--bore=N` away; nothing is shipped at one.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The **bell** and the **mouthpiece**, shared by every trumpet in this repository rather
than owned by one of them. Every bore under `bore/` is cut at the 10mm channel, so one
bell and one mouthpiece serve all of them; only the tube between them differs.

`--bore` on the two square-to-round generators makes the parts to suit any
channel, and nothing about the pair is tied to 10. See `--bore` is the channel, and
only that below.

**Every mouthpiece and every bell lives here, whichever instrument cuts it.** A
mouthpiece and a bell suit any tube on the same channel, so holding a set beside one bore
would hide two general parts inside one instrument — "nothing else cuts them" describes
what has been cut rather than what fits. The rule is the one the coiled trumpet states:
neither end is touched by the way a bore turns, so **only the tube belongs to an
instrument**, and a bore directory holds only bore.

**A bare `bell-round.py` writes all four budgets**, the 17-ring among them. Pass a ring
budget when you want one sheet, and `--out` to name it.

**The writeup is `README.md` at the repository root**, and the two ends have a page of
their own at `../ends/`. Those carry the geometry; this file covers the code.

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

**The mouthpiece reproduces exactly**: a bare `mouthpiece-round.py` gives the shipped
sheet byte for byte. `RIM` is 17 because the shipped sheet is, and a bare run has to
rebuild what is shipped. **The rim is not in the filename**, so moving `RIM` makes a bare
run silently replace a mouthpiece that has been cut with one of a different lip. The bell
has no such hazard — its rim IS in its name, so a different rim writes a different file. **The bell reproduces exactly too**:
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

The README's **"Rim diameter" column is the outer**. `bell-round.py`
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

**The throat is the bore's channel, not its outside.** Take it from the outside and ring 0
sits entirely outside the end face without overlapping it **at all**: the only contact is
the tube's outer wall against a thin lip, the airway steps out by a full wall thickness per
side, and it shows on the bench as gaps at the joint. The mouthpiece has the same shape —
its station one is a bore-sized square hole in a plate-sized square.

**Nothing here compensates for the kerf, and nothing here says so.** `bell.py`,
`bell-round.py`, `mouthpiece.py` and `mouthpiece-round.py` contain not one
mention of it between them, while the bore generators next door subtract half a
kerf from every hole and add it to every part. For a laminated stack that is
defensible -- the rings glue face to face and the lap below absorbs an outer
profile a kerf small -- but it is a consequence to count rather than a decision
anyone made.

**Apertures come out one kerf oversize**, because a hole opens as the laser goes
round it. At the 0.15mm kerf the bore generators use:

    the throat    drawn  3.66 -> cut  3.81mm   +8.4% in area
    the bore      drawn 10.00 -> cut 10.15mm   +3.0%
    the lip       drawn 17.00 -> cut 17.15mm   +1.8%

The throat is the smallest hole in the instrument and the one that voices it, so
that is where a fixed 0.15mm hurts most. Whether it wants compensating is a
voicing question, not a drawing one, which is why this is written down rather
than changed.

**Stack heights are nominal.** They are ring count times WALL, and WALL is 3.0: 30 rings
reports 90.0mm and 24 rings 72.0mm. A glued stack measures whatever the ply under it
actually is, so caliper a stack rather than reading its height off a ring count — a
reported height that is exactly `count x 3.0` is a count and not a measurement.

**The lap is 3mm, not 1.5.** 1.5mm is the width of the glue land, and it leaves nothing for
kerf or for a ring set down slightly off centre, so joints open up along the bell. `--lap`
moves it.

**Consequences that bite tooling:**

- **Ring sizes are not monotonic**, because the flange is wider than the rings above
  it. Anything recovering assembly order by sorting on the OUTER diameter is wrong.
  `verify_bell.py` sorts on the **aperture** instead — the airway only ever opens, so that
  is assembly order on any sheet however it was nested. `number_rings.py` refuses and wants
  `--order=document`.
- **The minimum-wall floor does not bind.** `wall = gain + LAP >= 3mm` on its own, so the
  `max(gain, MINWALL - LAP)` floor is inactive and the profile follows the Bessel curve
  exactly rather than being inflated by it. That is why all four bells reach the same
  129.0mm rim.

## The adapter is what makes a ported bore take a bell

`bell-adapter.py` is a third part and not a third bell. A ported bore does not open at
its mouth; it opens through the **7 x 14mm slot** in the cheek that `ribbon_bore.py`
cuts, and a bell's throat is the bore's own 10mm square. The adapter is the stack of
rings that turns one into the other, and it is why the bell needs no change at all:
its last ring is a **collar, 10mm square in a 16mm square face**, which is exactly the
end face the bore presents, so `bell-round.py`'s ring 0 glues onto it as it was always
going to glue onto the bore.

**Why the port is not square** is argued next door beside `PORT_ACROSS`, and it is the
whole reason this part exists. The tab slots run up both cheek walls the length of the
bore, so a port's clearance is measured **across** the run and nothing else; length
**along** the run is free. 7 x 14 clears by 1.66mm where a bore-square 10 x 10 clears
by 0.16mm, and buys the area back in the direction that costs nothing. The price is an
opening the wrong shape for a horn, and it is paid here rather than in the bore.

**Area is the schedule and the aspect ratio is what moves** — 98mm2 to 100mm2 while the
two half-widths run 2:1 down to 1:1. Interpolating the half-widths straight is the
obvious thing and it **dips the area 4% mid-transition**, because the long axis loses
faster than the short axis gains.

**Both ends are tangent.** The schedule is a smoothstep on the area and the aspect
alike, so the section leaves the port and reaches the throat with zero rate of change.
That is what "smooth" has to mean for a part bolted to two others; a linear schedule
meets the port at full tilt and creases the first joint.

The section stays a **sharp rectangle** the whole way. Both ends are sharp, so unlike
`bell-round.py` there is nothing to round and nothing to round back.

**How fast the section may move is set by the lap**, not by taste: a half-width may
shrink by at most `LAP - MINWALL` = 1mm in one ring or that side's wall goes under the
minimum. The long axis has 2mm to lose, so **three rings is the hard floor** and the
default is six plus the collar. The generator refuses a smaller `--rings` and says so.

At the default it is 7 rings of one ply, 21mm tall, with a 13.14 x 19.75mm footprint on
a cheek whose narrow band is 15.85mm — 1.35mm of cheek each side of it.

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
of the bore viewers in `bore/concept/swept-curve`, and writes `<name>-turn.html` beside the
PART — climbing out of `cut-files/`, because a turn page is not a cut file.

```sh
python3 part-view.py bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
python3 part-view.py mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
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

## How the view generators find rings

The apertures sit in their own orange group so they cut before the outline that
frees the part. A reader that looks for **one path holding two subpaths** finds
no rings at all on these sheets, and nothing in any gate runs the viewers, so it
would find none silently.

Both viewers read the two groups and **pair ring i's aperture with ring i's
outline, in file order.** That is what the generators write - both groups come
off one list - and it is assembly order.

**Do not pair by size, and do not pair by containment.** Sorting each list and
zipping works only while a profile grows monotonically; a mouthpiece narrows to
its throat and opens again, so it drew the cup as a stack of alternating
bulges. Containment is no better: an aperture also contains any ring nested in
its spare space. Either draws a plausible object that is not the one on the
sheet - which is the warning the mouthpiece viewer's own docstring carries,
about sorting by diameter.

`mouthpiece-view.py` takes `--src=`; a bare argument is the OUTPUT. Making the
source positional would write a display SVG straight over a cut file.

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**; black frees the part; violet
`#8000ff` means skip.

**No sheet here is nested today.** Every one cuts in a single black stage with its numbers
in blue. A **black → red ramp** belongs to a *nested* sheet, where a ring sitting inside
another's aperture must be cut first or it is freed with the waste; one stage per ring by
size says so in the only channel an importer always reads.

**There is no tool here that applies one.** `ramp_bell.py` rewrote a `stroke:` property
inside a style attribute, and the sheets these generators write carry a single stroke
ATTRIBUTE on the enclosing `<g>` and nothing on the paths, so it matches nothing on every
sheet in the repository. Recolour a nest in the tool that made it. `verify_bell.py` checks
that a ramp, if present, rises with ring size, which is the half of this worth keeping.

## Cut files belong to the author

These have been cut. Treat every SVG as concurrently modified in Inkscape:

- **Stage by name.** Never `git add -A` or `git add .`.
- **`bell.py` with no argument rewrites all four sheets.** Pass a ring budget to
  regenerate one — `python3 bell.py 20` writes only the 17-ring. Rewriting four sheets to
  change one is four files to review instead of one.
- **A mouthpiece sheet is named by its bore AND its layout**, and says both in its
  `<title>`. Two mouthpieces exist — `legacy` and `trumpet` — with the same ring
  count, bore, throat and rim, and profiles nothing alike: 9 backbore rings plus 17
  entrance rings against 26 backbore and none. Name only the non-default parameter and
  neither the filename nor the file itself can tell you which part you have, so both go in
  every name.
- **A ring is two cuts in two colours.** Orange `#ff8000` takes the aperture, black
  `#000000` the outline, and the orange group is written first. One path holding both lets
  a per-colour job free the part before its hole is in. Anything that reads a ring back -- `number_rings.py`, `bell-section.py` -- pairs the
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

  Numbering as a separate command you have to remember is how a regenerate silently throws
  the engraving away, which is why it is not one.

  **`mouthpiece-cup.py` continues a stack rather than starting one.** Its rings are glued
  on top of a mouthpiece already numbered from 0, so it numbers from `--start`, which
  defaults to **23** — `mouthpiece.py` writes rings 0 to 22 and ends at the ø10.06 the cup
  stacks onto. Pass `--onto` and that no longer holds, so `--start` stops being optional and
  the script refuses rather than guessing. Numbering a cup from 0 would put a second ring 0
  in one mouthpiece, which is the confusion the numbers exist to prevent.

  `mouthpiece.py` separates options from the path. Without that, `sys.argv[1]` is the
  output path and `--numbers=no` writes a file of that name.
- **These scripts have no `--help`**; a bare run to see the options IS a run. Read the
  docstring, or write to a scratch path.
- Verify a hand-edited bell with `verify_bell.py` rather than diffing path data — once
  paths are converted to Bézier curves, a byte diff says nothing. **It reads the SQUARE
  bells only.** The square-to-round ones are drawn with arcs and lap by whatever their
  3mm wall leaves rather than by a fixed amount, so it skips them and says so — a checker
  that answers about a file it does not understand teaches you to ignore it.
- **`verify_bell.py` pairs an aperture with the outline concentric with it**, so it reads
  both the one-path form and the two-group form the generators write now, and survives a
  nest that reorders the paths. An outline that finds no aperture is reported as what it
  is, a solid disc — a reader blind to the two-group form instead calls every generated
  sheet "a section drawing or a sheet of something else" and skips it, passing silently on
  sheets nobody has checked.

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
sheet is kept. It is not the default and it is not called `asbuilt`: that name says when a
profile was made rather than whether to cut it, and as a default it would make a bare run
produce the profile nobody wants. `--layout=asbuilt` exits saying so rather than quietly
doing something else.

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

**Every filename states its length, and `bell-round`'s its bore as well.** A length that
appeared only when it was not 201mm would let a 100mm bell landing on 17 rings overwrite
`bell-round10-153mm-17rings-x3-rim86-cut-files.svg`, and would leave
`bell-round-67rings.svg` saying nothing about the tube it fits or how long it is, where
`bell-round-99mm-11rings.svg` says one of the two. Both go in every name, and in the
`<title>`. Do not "tidy" that name back: it is built inline at
`bell-round.py:337` and there is no stem variable to shorten, which is the point --
the length and the ring count have to survive being read off a sheet.

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

Run each from its own directory: several read a sibling script by relative path.

**A generated sheet goes into `cut-files/`; a display drawing goes beside the part.**
`in_cut_files()` in each generator places the first, `display_out()` in each reader places
the second, climbing back out of `cut-files/` — so a section drawing never lands among the
cut files and never gets sent to a laser. Name a path yourself and it is used exactly as
given, wherever it points.

All nineteen lines below run in order from a clean tree and exit 0. **No line
modifies a tracked sheet**: every shipped
sheet it rewrites comes back byte-identical, which is the property worth having — a command
block nobody can run is a command block nobody can trust.

Keep the block complete the way `.repro` is kept complete: ask which generators in
`bell/` are absent from the list that claims to run them all. A generator missing from it
is the gap both mechanisms have.

Six of them do **add** sheets, fifteen in all: the four lines that survey a range (a bare
`bell.py` and a bare `bell-round.py` write four budgets each, `--morph=flare` and
`--rim=80` write their own) and the two at the end that generate designs this repository
does not ship. Those fifteen are scratch. Delete them, or they read as shipped parts.

```sh
cd bell && python3 bell.py            # all four square bells, into cut-files/
cd bell && python3 bell.py 20         # one, at most 20 rings
cd bell && python3 bell-round.py      # all four square-to-round bells, at the default rim
cd bell && python3 bell-round.py 67 --morph=flare --law=width
cd bell && python3 bell-round.py --length=99 --rim=80          # a half-size bell
cd bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80   # THE SHIPPED BELL
cd bell && python3 bell-adapter.py                             # THE SHIPPED ADAPTER
cd mouthpiece && python3 mouthpiece-round.py                   # THE SHIPPED MOUTHPIECE
cd bell && python3 bell-section.py cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
cd bell && python3 bell-view.py cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
cd bell && python3 bell-section.py ../mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
cd parts && python3 part-view.py bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
cd parts && python3 part-view.py mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
cd mouthpiece && python3 mouthpiece-view.py     # its default source is the shipped sheet
cd bell && python3 bell.py 20 && python3 verify_bell.py \
      cut-files/bell-square10-204mm-17rings-x4-rim129-cut-files.svg   # SQUARE bells only
cd bell && python3 number_rings.py --order=document \
      cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg     # engrave 0..A
cd bell && python3 number_rings.py --order=document \
      ../mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg
cd mouthpiece && python3 mouthpiece-cup.py      # the bowl that stacks on its end; not kept
cd mouthpiece && python3 mouthpiece.py          # the previous 23-ring design; not kept
```

**`--order=document` is not optional on either sheet**, and the refusal without it is
correct rather than a bug: a bell's flange ring is wider than the rings just above it and a
mouthpiece doubles back through the same diameters, so size order is not assembly order on
either. `number_rings.py` says so and stops rather than engraving a sequence nobody can
build to.

**Only two sheets in `cut-files/` are shipped** —
`bell-round10-153mm-17rings-x3-rim86-cut-files.svg` and
`mouthpiece-bore10-trumpet-parts-cut-files.svg`. Anything else there came from a survey
line and is scratch: `git status` is the check, and `git clean -f bell/cut-files
mouthpiece/cut-files` is the sweep.

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
