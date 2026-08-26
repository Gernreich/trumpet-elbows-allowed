# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The **bell** and the **mouthpiece**, shared by every trumpet in these repositories rather
than owned by one of them. `../trumpet-coiled` and `../trumpet-octagonal` both build on the
same 25 × 25mm channel, so both take the same bell and the same mouthpiece; only the tube
between them differs.

Read `README.md` first — it carries the geometry. This file covers working on the code.

## The bore is cylindrical

Constant 25 × 25mm section end to end, 31mm outside in 3mm ply. **The only part of a
trumpet that flares is the bell.** Do not describe a bore, a band or a tube as flaring;
say curved if it curves.

## A bell file is cut more than once

Each of the four bell cut files draws every ring **once**. The `Build` column in the README
says how many 3mm laminations a ring is, and that is how many times the sheet goes through
the machine: the 10-ring bell is 7 ply, so 7 passes and 70 pieces. Cut once it yields a
30mm bell instead of 210mm. Only the 67-ring file is a single pass. Counting shapes in the
SVG answers a different question than "how many pieces".

## Two bell families

`bell.py` makes four bells that are **square end to end**. `bell-round.py` makes four whose
section **morphs from the bore's square to a round rim** — every ring a rounded square,
corner radius 0 at station one and equal to the half-width at the rim.

They differ in what is held to the Bessel profile. The square bells follow the half-width;
the round ones follow the **area**, because a circle inscribed in a square has 21% less of
it. That is why the round rim is ø141.8mm where the square one is 126.0mm square, and why
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

## mouthpiece-round.py still makes a cup-less mouthpiece

Its cup run ends at ø10.06 opening 0.40mm a ring — a 3.8 degree half-angle, a tube. A
trumpet rim is 16 to 17mm inside. `mouthpiece-cup.py` adds the missing bowl as four rings
that stack on an already-glued part, numbered from 26 (`--start=26`) so they continue the
count rather than repeating it.

**Anyone cutting a fresh mouthpiece needs both sheets**, which is a trap. Folding the bowl
into `mouthpiece-round.py` would renumber every ring in a part that has been cut, so it has
not been done — ask before changing it.

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
cd bell && python3 bell-section.py bell-trumpet-17rings.svg
cd bell && python3 bell-view.py bell-round-17rings.svg
cd bell && python3 verify_bell.py bell-trumpet-17rings.svg
cd bell && python3 number_rings.py bell-round-99mm-11rings.svg   # engrave 0..A
cd bell && python3 number_rings.py ../mouthpiece/mouthpiece-round-parts.svg --order=document
cd mouthpiece && python3 mouthpiece-round.py    # square on the bore, round by the throat
cd mouthpiece && python3 mouthpiece-cup.py      # the bowl that stacks on its end
cd mouthpiece && python3 ../bell/number_rings.py mouthpiece-cup-parts.svg --start=26
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
