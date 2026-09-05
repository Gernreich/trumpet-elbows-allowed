# CLAUDE.md

## Working in this repository

A laser-cutting build repository, not a software project. The deliverable is **one cut
sheet** — `octagonal-trumpet.svg`, 444.077 × 484.599mm — plus the page describing it.

**There is no generator here, and that is the important thing about this repository.** Every
other trumpet repository writes its sheets from a script; this one does not. The sheet is
the trumpet form of the **[octagonal torus](https://github.com/Gernreich/torus-octagonal)**,
derived from that plate by hand and edited in Inkscape — the file still carries its layer
markers. Nothing regenerates it, so nothing can restore it.

**So the cut file is authored, and it is the author's.** Do not tidy it, re-nest it, or
"clean up" its path data. It has been cut. If something in it looks wrong, say so and stop.

- **Stage by name.** Never `git add -A` or `git add .`.
- Treat the SVG as concurrently modified in Inkscape.

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**, with black always the cut that frees
the part and violet always skip. That sequence is shared by every LaserMadeMusic repository.

This sheet uses **two** of those stages and nothing engraves:

| | Colour | Count | What |
|---|---|---:|---|
| 1 | **orange `#ff8000`** | 1 | the plate's central hole, cut while the plate is still held by the sheet |
| 2 | **black `#000000`** | 4 | the outlines that free the parts |
| — | **violet `#8000ff`** | 16 | skip — one line at the middle of every flat and one at every corner |

**The violet lines are the point, not an oversight.** They divide the wall into sixteen
segments and are carried in the drawing rather than cut, so "not cut" is a decision recorded
in the file instead of a colour someone forgot to map. Turn one green to cut it. **A
per-colour job silently skips any colour you leave unmapped**, which is why the count above
is worth checking against the file before a run:

```sh
grep -o 'stroke:#[0-9a-f]*' octagonal-trumpet.svg | sort | uniq -c
```

## The plate is shared with the torus

Rim at apothem 86.149, hole at 58.149, and the same joint phase — so a panel cut for the
torus mates with one cut here. Those numbers belong to `torus-octagonal`; if they ever move,
they move there first and this sheet follows. That repository's `verify_torus.js` is what
checks them.

## The page

`README.md` is the source and `index.html` is built from it, as in every sibling repository:

```sh
G=~/LaserMadeMusic/GIT/lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html \
    --rebuild "python3 $G/md2html.py {md} {out}" --links
python3 $G/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.** It ends with a pass/fail tally.

**This repository kept a separate `index.md` until 2026-09-02** and was the only one of the
family that did. The two documents had drifted — `README.md` was an abridged rewording of
`index.md`, sharing barely a sentence with it, and only `index.md` reached the published
page. They are one file now. Do not reintroduce the split: a second source is a second thing
to keep in step, and it was not kept.
