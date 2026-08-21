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

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**; black frees the part; violet
`#8000ff` means skip.

`bell-trumpet-17rings.svg` is the exception and its ordering is load-bearing: a **black →
red ramp**, one stage per ring by size, `#000000` on the smallest through `#ff0000` on the
rim. The rings nest five deep, so smallest-first is inner-before-outer for every nested
pair. `ramp_bell.py` applies it; `verify_bell.py` checks the ramp rises with ring size.

## Cut files belong to the author

These have been cut. Treat every SVG as concurrently modified in Inkscape:

- **Stage by name.** Never `git add -A` or `git add .`.
- **`bell-trumpet-17rings.svg` is hand-nested and hand-labelled.** Regenerating it from
  `bell.py` discards that work. Ask first, every time.
- Verify a hand-edited bell with `verify_bell.py` rather than diffing path data — once
  paths are converted to Bézier curves, a byte diff says nothing.

## Known wrong, deliberately not fixed

**`mouthpiece.py` names `cup` and `backbore` backwards.** Its `cup = [25.0 … 5.0]` is the
end that meets the bore, which is anatomically the backbore. Every number is correct and
the README describes the profile correctly; only the variable names and the `<desc>` string
the script emits are reversed. Fixing it means regenerating a part that has been cut.

**`bell.py`'s docstring says "the same 201mm horn"**. The bells are 210, 210, 204 and 201mm
— only the 67-ring is 201. Docstring only; no effect on geometry.

## Commands

Generators read hardcoded relative filenames — run each from its own directory:

```sh
cd bell && python3 bell.py            # all four bells
cd bell && python3 bell.py 20         # one, at most 20 rings
cd bell && python3 bell-section.py bell-trumpet-17rings.svg
cd bell && python3 verify_bell.py bell-trumpet-17rings.svg
cd mouthpiece && python3 mouthpiece.py
cd mouthpiece && python3 mouthpiece-view.py
```

`bell-section.py` miscounts a hand-labelled sheet — engraved digits register as ring
subpaths, so it reads 25 rings in the 17-ring file. Do not regenerate that section.

**After editing either document** — regenerate the page, then audit both:

```sh
G=../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
python3 $G/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.**
