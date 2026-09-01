# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The **bell** and the **mouthpiece**, shared by every trumpet in these repositories rather
than owned by one of them. `../trumpet-coiled` and `../trumpet-octagonal` both build on the
same 25 × 25mm channel, so both take the same bell and the same mouthpiece; only the tube
between them differs.

Since 2026-08-31 there is a **second channel**: `../trumpet-switchback` is cut
at 10mm as well as 25, and `--bore` on the two square-to-round generators makes the parts
to suit. See `--bore` is the channel, and only that below.

**The switchback's sheets are not kept here, only the generators that make them.** That
repository holds a whole instrument per size, and the four sheets it cuts moved out on
2026-08-31 — `10mm/` got the bell and mouthpiece made with `--bore=10`, and `25mm/` got
`bell-round-17rings.svg` and `mouthpiece-trumpet-parts.svg`. Nothing else cuts any of the
four. This repository keeps the generators and the sheets that really are shared.

**A bare `bell-round.py` writes the 17-ring sheet back into `bell/`.** It is one of the four
standard budgets, so the loop still produces it; it just does not belong here any more. Pass
a ring budget, or delete what lands rather than committing a second copy that nothing gates.
The `--layout=trumpet` mouthpiece is safe by comparison: it is only written when its output
path is named.

Read `README.md` first — it carries the geometry. This file covers working on the code.

## The bore is cylindrical

Constant 25 × 25mm section end to end, 31mm outside in 3mm ply — or 10 × 10mm inside 16 on
the small candidate. **The only part of a trumpet that flares is the bell.** Do not
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

Filenames carry a non-default bore — `bell-round-152mm-bore10-17rings.svg`,
`mouthpiece-round-bore10-parts.svg` — because two parts of the same length on different
bores are different parts and only one of them fits your tube. Those two names are what a
`--bore=10` run writes into this directory; move them to `../trumpet-switchback` rather
than committing them here. Both generators were checked after the change by regenerating
at the default and diffing: **byte-identical**.

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

The bore ends in a **square annulus of ply 3mm wide** — 25mm inside, 31mm out. Ring 0 has to
cover all of it. It is a sharp 37mm square with a 25mm square hole: the one ring whose outer
is not the next station offset, and **wider than the several rings above it**.

This was wrong until 2026-08-26. The throat was 15.5 (ø31 — the bore's *outside*), so ring 0
spanned radius 15.5 to 17.8 and did not overlap the ø25–ø31 end face **at all**; the only
contact was the tube's outer wall against a 2.3mm lip, and the airway stepped 25 → 31 with a
3mm shoulder per side. It was reported as gaps at the joint, which is exactly what it was.
The mouthpiece had it right all along — its station one is a 25mm square hole in a 31mm
plate — and the bell now does the same.

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
- **`bell.py` with no argument rewrites all four sheets.** Pass a ring budget to regenerate
  one — `python3 bell.py 20` writes only the 17-ring — or the numbering on the other three
  is silently discarded.
- **`bell-round.py` and `mouthpiece-round.py` number their own rings**, calling
  `number_rings.py --order=document` as the last step of writing the sheet, and they say so
  in the run report. `--numbers=no` writes a bare sheet. If the numbering fails the sheet is
  **deleted**, because a sheet left on disk gets cut and an unnumbered one is rings nobody
  can order.

  This was not always so: numbering used to be a separate command you had to remember, and
  a regenerate silently threw the engraving away — `mouthpiece-round.py` with no arguments
  cost `mouthpiece-round-parts.svg` its numbering on 2026-08-31, restored from git. Changing
  the default was checked by regenerating all four sheets the switchback trumpet is cut from
  and diffing against the cut files: **byte-identical**, numbering included.

  **`bell.py`, `mouthpiece.py` and `mouthpiece-cup.py` still write bare** and need
  `number_rings.py` afterwards. `mouthpiece-cup.py` is an extension sheet that continues an
  existing stack, so it needs `--start=N` as well; numbering it from 0 would put a second
  ring 0 in one mouthpiece.
- **These scripts have no `--help`**; a bare run to see the options IS a run. Read the
  docstring, or write to a scratch path.
- Verify a hand-edited bell with `verify_bell.py` rather than diffing path data — once
  paths are converted to Bézier curves, a byte diff says nothing.

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
`mouthpiece-round-parts.svg`. `bell-view.py` reads corner radii and would draw it, but calls
whatever it is given a bell.

## Numbering a sheet

`number_rings.py` engraves each ring's hex index, smallest = 0, in `#0000ff` in its own
`<g id="ring-numbers">` written before the black that frees the part. Re-running replaces
that group rather than adding a second one, and it refuses to write if any cut path's `d`
changed.

**Number in ASSEMBLY order, not size order.** On a bell the two agree, because it
telescopes. On the mouthpiece they do not: the airway narrows 25 -> 5 then opens 3.66 ->
10.06, so the two runs share diameters and sorting by size interleaves them. The tool now
refuses to guess when the sizes are not monotonic in the file — pass `--order=document`,
which is assembly order for anything these generators write. `b` and `d` are lower case
because seven-segment upper-case B is 8 and upper-case D is 0.

**A ring is two concentric outlines — and so is an engraved 0, 4, 6, 8 or 9.** That is why
`bell-section.py` reads 25 rings in the hand-labelled 17-ring sheet, and `number_rings.py`
would have inherited the same miscount. Subpath counting cannot separate them; **colour
can**, and here it already does — blue is engraving, never a part. A sheet that already
carries blue is refused outright rather than given a second set of labels.

**It reads only what the generators write** — `M`, `H`, `V`, `A`, `Z`. `bell-trumpet-17rings.svg`
came back from Inkscape in relative commands and quadratic curves (`h l m q v z`), which this
refuses by name rather than half-parsing into a plausible wrong answer.

**Digits are seven-segment strokes, not `<text>`**, and each is grown to the largest size
that still lands on material — measured against the real curves, not assumed from the wall.
That matters because the wall is not the room available: the rim ring's aperture is nearly a
circle, so there is no flat to sit a digit on, and a radius-by-angle lookup that samples only
the ends of a straight run reads the bottom of a square as its corner, 21.9mm instead of
15.5mm. Sample `H` and `V`, not just their endpoints.

## The mouthpiece has two layouts, and the default is the wrong one

Both are 30 rings and 90mm. `--layout=trumpet` puts 75mm into the backbore and keeps a 12mm
cup, which is how a real mouthpiece is proportioned. `--layout=asbuilt` — the default — has
a 27mm backbore and 51mm of near-cylindrical entrance on the LIP side of the throat, close
to inverted. **It is the default only because a mouthpiece exists to it and its rings are
numbered for it.** Cut `trumpet` for anything new; do not change which is default without
asking, and check `mouthpiece-round-parts.svg` still comes out byte-identical if you touch
the profile code.

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

**`mouthpiece.py` names `cup` and `backbore` backwards.** Its `cup = [25.0 … 5.0]` is the
end that meets the bore, which is anatomically the backbore. Every number is correct and
the README describes the profile correctly; only the variable names and the `<desc>` string
the script emits are reversed. Fixing it means regenerating a part that has been cut.

## A smaller bell is a shorter profile, not a scaled one

The throat is ø31 because the bore is, and a ring rises 3mm because the ply does. Neither is
ours to scale. `--length`, `--rim` and `--gamma` on both generators move the profile instead;
`--rim` is the bore's diameter **at** the rim, before the wall, so it reads a wall smaller
than the "Rim diameter" the README tabulates.

**A non-default profile puts its length in the filename** — `bell-trumpet-99mm-11rings.svg`.
Without that, a 100mm bell landing on 17 rings would overwrite `bell-trumpet-17rings.svg`,
which is hand-nested and not reproducible from the script. Do not "tidy" `STEM` away.

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
cd bell && python3 bell-section.py bell-trumpet-17rings.svg
cd bell && python3 bell-view.py bell-round-14rings.svg
cd bell && python3 verify_bell.py bell-trumpet-17rings.svg
cd bell && python3 number_rings.py bell-round-99mm-11rings.svg   # engrave 0..A
cd bell && python3 number_rings.py ../mouthpiece/mouthpiece-round-parts.svg --order=document
cd mouthpiece && python3 mouthpiece-round.py    # square on the bore, round by the throat
cd mouthpiece && python3 mouthpiece-cup.py      # the bowl that stacks on its end
cd mouthpiece && python3 ../bell/number_rings.py mouthpiece-cup-parts.svg --start=26  # not kept
cd mouthpiece && python3 mouthpiece.py          # the previous 23-ring design
cd mouthpiece && python3 mouthpiece-view.py     # draws the previous design ONLY
```

`bell-section.py` counts a path with two subpaths as a ring, so **outline** digits — a
hand-drawn 0, 4, 6, 8 or 9 has a counter — register as rings and it read 25 in the old
hand-labelled 17-ring sheet. `number_rings.py` draws single-stroke seven-segment digits
with no counters, so a sheet it numbers still counts correctly.

**After editing either document** — regenerate the page, then audit both:

```sh
G=../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
python3 $G/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.**
