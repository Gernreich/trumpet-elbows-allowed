# CLAUDE.md

## Working in this repository

A laser-cutting build repository, not a software project. The deliverable is
**`BuildA1_90_25.svg`** — the torus's 18 pieces in 20 contours — plus two pages describing
it. The three `RunA1/2/3` files are the raw **boxes.py** outputs it was stitched from, kept
unmodified.

## One group in the cut file is generated, and only that one

`number_pieces.js` writes `<g id="piece-numbers">` — eighteen hex numbers in blue, one per piece.
It is the only thing in this repository that writes to `BuildA1_90_25.svg`, and the way it is
allowed to is by never reading back into a cut path: it strips its own previous group, measures the
pieces, appends a new group, and touches nothing else. A re-run replaces the numbers; `--clear`
removes them.

```sh
node number_pieces.js --dry-run     # where each number would go, and the room around it
node number_pieces.js --table       # the README's key, printed from the file
node number_pieces.js               # write them
node number_pieces.js --clear       # take them out
```

Two things it refuses to do rather than do badly, both demonstrated to fail when they should:

- **A number that does not fit.** Every glyph point is tested against the piece's own outline and
  against anything nested inside it, and the whole run is abandoned if one point falls outside.
  It prints how many points it checked, so a run that tested nothing cannot look like a run that
  passed.
- **A number on waste.** The anchor is not the centroid or the box centre — a plate is a ring and
  both of those land in its hole, which has eight panels nested in it. It samples the piece and
  takes the point furthest from any edge of the piece *or* of anything inside it.

It borrows `verify_torus.js`'s reader rather than parsing paths again, so it cannot disagree with
the verifier about where a part is.

**`verify_torus.js` now ignores blue** as engraving, alongside violet as skip — and that is the move
that once broke it, when six recoloured panels went blue and 20 contours were reported as 14. So
blue does not get ignored on trust: every blue contour is measured, and one larger than 20mm is
reported loudly as a part that is being dropped. A glyph is a mark; a panel is not.

## The rest of the cut file is authored, and it is the author's

Apart from that one group, `BuildA1_90_25.svg` is not generated. It was assembled by hand from the three generator
runs — the eight segments stitched into one closed loop per plate — and edited in an SVG
editor. The history says so plainly: *"saved as Plain SVG: the editor state is gone, the
drawing is not"*, and *"BuildA1_90_25.svg is final"*. Nothing regenerates it, so nothing
can restore it.

- **Stage by name. Never `git add -A` or `git add .`.** Treat every SVG here as open in an
  editor right now. A blanket add commits work in progress that was not offered.
- Do not re-nest, re-order or tidy the path data. If something looks wrong, say so and
  stop.
- The `RunA*` files are **unmodified generator output** and stay that way. `RunA2` doubles
  as the reference `verify_torus.js` measures joint phase against, so changing it breaks
  the checker rather than the drawing.

## Verify after every change, and read the phase line

```sh
node verify_torus.js BuildA1_90_25.svg RunA2_R59Point693.svg
```

It reports the stroke palette, contours, plate and hole geometry, concentricity, **joint
phase**, cut order, nesting clearances, and whether everything sits inside the viewBox.

**The phase check is the one that matters.** It catches a hole whose tabs land where the
panel is solid — a build that measures perfectly and cannot be assembled. A geometric pass
without it is not a pass.

`verify_torus.js` knows exactly one object: an octagonal torus at R 90 with a hole at
apothem 58.149 and a five-colour cut order. Hand it anything else and it prints what it
measured, says it does not recognise the file, and stops — by design, rather than reporting
nonsense with stars on it.

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**, black frees the part, violet is skip.
That sequence is shared by every LaserMadeMusic repository. `BuildA1_90_25.svg` uses five cut
stages and, since 2026-09-03, engraves the piece numbers:

| Colour | Count |
|---|---:|
| blue `#0000ff` (engrave) | 50 |
| green `#00ff00` | 24 |
| orange `#ff8000` | 2 |
| cyan `#00ffff` | 16 |
| black `#000000` | 4 |
| violet `#8000ff` (skip) | 24 |

Counted from the file, not from the prose — do the same before trusting it:

```sh
grep -o 'stroke:#[0-9a-f]\{6\}' BuildA1_90_25.svg | sort | uniq -c
```

**The plate is shared with [trumpet-octagonal](https://github.com/Gernreich/trumpet-octagonal)**,
which cuts the same R 90 plate opened into a curve. Its geometry follows this repository's,
not the other way round: if the apothem or the joint phase ever moves, it moves here first.

## Two documents, and they are two different pages

Unlike the sibling repositories, this one publishes twice, and that is deliberate:

| source | page | what it is |
|---|---|---|
| `README.md` | `index.html` | the short page — cut it, or build it at your size |
| `Octagonal_Torus_Gold.md` | `Octagonal_Torus_Gold.html` | the full writeup, 1030 lines |

They are not two sources for one page. Keep it that way — `trumpet-octagonal` kept a second
source for a *single* page until 2026-09-02, the two drifted until they shared barely a
sentence, and only one of them reached the reader.

```sh
G=~/LaserMadeMusic/GIT/lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html \
    --rebuild "python3 $G/md2html.py {md} {out}" --links
python3 $G/svg-stroke-check.py --dir . --quiet
```

**Read the audit output before pushing.** It ends with a pass/fail tally.
