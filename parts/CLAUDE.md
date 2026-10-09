# CLAUDE.md

Rules for working on the bell and mouthpiece generators. The reasoning, history and
measurements behind each one are in `NOTES.md`, under the heading given in brackets.
Read that section before changing the code it names.

## What lives here

- **One bell and one mouthpiece, both at the 10mm bore**, shared by every bore under
  `bore/`. Every mouthpiece and every bell lives here, whichever instrument cuts it. A
  bore directory holds only bore. [What this is]
- The writeup is `README.md` at the repository root; the ends have their own page at
  `../ends/`.
- **The bore is cylindrical. Only the bell flares.** Never describe a bore, band or
  tube as flaring; say curved. [The bore is cylindrical]
- **Only two sheets in `cut-files/` are shipped**:
  `bell-round10-153mm-17rings-x3-rim86-cut-files.svg` and
  `mouthpiece-bore10-trumpet-parts-cut-files.svg`. Anything else there is scratch from
  a survey run: check with `git status`, sweep with
  `git clean -f bell/cut-files mouthpiece/cut-files`. [Commands]

## Generator rules

- `--bore=N` on `bell-round.py` and `mouthpiece-round.py` sets the channel only; the
  plate is N + 6. Ply stays 3mm, throat stays ø3.66, rim stays trumpet-sized.
  **Do not add `--bore` to `bell.py` or `mouthpiece.py`** without being asked.
  [`--bore` is the channel, and only that]
- Filenames carry the bore, length and ring count, and the same in `<title>`. Never
  "tidy" a name shorter. [A smaller bell is a shorter profile]
- **Both shipped sheets reproduce byte for byte**: bare `mouthpiece-round.py`, and
  `bell-round.py 17 --bore=10 --length=152 --mouth=80`. The mouthpiece's rim is **not**
  in its filename, so changing `RIM` silently replaces a cut part.
- A bare `bell-round.py` (or `bell.py`) writes **all four** budgets. Pass a ring budget
  for one sheet, and `--out` to name it.
- `--mouth` is the hole; `--rim` is the square bell's width (`--rim=80` gives ø90.3).
  The README's "Rim diameter" column is the **outer**. [`--mouth` is the hole]
- Each bell file draws every ring **once**; the README's `Build` column is how many
  passes. Counting shapes in the SVG is not counting pieces. [A bell file is cut more than once]
- Ring 0 is a 22mm square flange around the 10mm channel, wider than the rings above
  it. **The throat is the bore's channel, not its outside.** [Ring 0 is a flange]
- Nothing here compensates for kerf: apertures cut one kerf oversize. That is a voicing
  question, recorded rather than changed. Stack heights are nominal (count × 3.0):
  caliper, don't trust them. The lap is 3mm, not 1.5.
- **Ring sizes are not monotonic.** Never recover assembly order by sorting on outer
  diameter; sort on aperture, or use file order.
- `bell-adapter.py` turns the 7 x 14 port into the bell's 10mm square. The area is the
  schedule (smoothstep, both ends tangent); a half-width moves at most 1mm a ring, so
  three rings is the floor. [The adapter is what makes a ported bore take a bell]
- `bell.py` bells are square; `bell-round.py` bells morph square-to-round and follow
  **area**. The 2mm wall floor is directional: keep both floors in `stations()`.
  [Two bell families]
- `mouthpiece-round.py` uses 2.5mm taper steps because 4mm leaves no corner seat.
  Roundness is solved per station, not scheduled. [The mouthpiece's taper is 2.5mm]
- `--layout=trumpet` is the default and the one to cut. `legacy` is a record of a built
  part. `--layout=asbuilt` refuses on purpose. The wall is per-ring, not a constant.
  [The mouthpiece has two layouts]
- **`mouthpiece-round.py` and `mouthpiece-cup.py` must stay identical ring for ring.**
  Change one, check the other. [Two routes to the same mouthpiece]
- `mouthpiece.py` names `cup` and `backbore` backwards. **Do not fix it**: that would
  regenerate a cut part. [Known wrong, deliberately not fixed]
- A ring past the 201mm profile is still a full `step` tall. `bell.py` reports the
  **steepest** angle. [The rim ring is not always the steepest]

## Numbering and reading rings

- Every sheet generator calls `number_rings.py --order=document` itself as its last
  step; a numbering failure **deletes** the sheet. `--numbers=no` writes a bare sheet.
- **`--order=document` is required** on both shipped sheets; the refusal without it is
  correct. Number in assembly order, not size order. Hex, upper case. [Numbering a sheet]
- `number_rings.py` reads only `M H V A Z`, refuses a sheet that already has blue, and
  draws polyline digits. Sample `H` and `V` along their length, not just endpoints.
- `mouthpiece-cup.py` continues a stack: `--start` defaults to 23 and is **required**
  with `--onto`. Never number a cup from 0.
- Each label carries an orientation tick (`--mark=no` omits it). The shipped sheets
  lack it **on purpose**; do not regenerate them to add it.
- A ring is two cuts: orange aperture group first, black outline. Readers pair ring i's
  aperture with ring i's outline **in file order**: never by size, never by
  containment. [How the view generators find rings]
- `verify_bell.py` reads **square** bells only and pairs by concentricity. Use it, not a
  byte diff, on a hand-edited bell.
- `bell-section.py` counts outline digits (0 4 6 8 9) as rings on hand-labelled sheets.

## Viewers

- `bell-view.py` and `mouthpiece-view.py` draw fixed isometrics; `part-view.py` draws a
  turnable solid and writes `<name>-turn.html` beside the part, outside `cut-files/`.
  It executes `bell-view.py`'s `sections()` rather than copying it.
  [Three viewers, and only one of them turns]
- `mouthpiece-view.py` draws the **old** mouthpiece only; don't point it at the trumpet
  sheet. It takes `--src=`; a bare argument is the **output**.

## Cut files belong to the author

These have been cut. Treat every SVG as open in Inkscape right now.

- **Stage by name.** Never `git add -A` or `git add .`.
- Regenerate one bell by budget (`python3 bell.py 20`), not all four.
- A mouthpiece sheet is named by bore **and** layout.
- These scripts have **no `--help`**: a bare run is a run. Read the docstring, or
  write to a scratch path.
- Cut order by colour: **blue engraves, then green → orange → cyan → black**; violet
  `#8000ff` means skip. No sheet here is nested. There is no ramp tool: recolour a nest
  in the tool that made it. [Colour is the cut order]

## Commands

Run each from its own directory. Generated sheets go into `cut-files/`; display
drawings go beside the part. Survey lines add scratch sheets: delete them.

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

After editing either writeup (the root README, or `../ends/`), regenerate its page
and audit it; there is no README in this folder:

```sh
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ..          # the root README
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ../ends     # the ends page
python3 ../../lasermade-tools/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.**
