# CLAUDE.md

**Nothing in this repository has been cut.** It is gated at 85 checks, and a passing
gate means no check failed, not that the part is buildable — see
`../../../../../../../../tools/CLAUDE.md` for what the gate cannot see. Say "gated"
and not "built" until one exists.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The bore of a trumpet whose walk is a flat meander — the Greek key. It ships cut
files only; the thing that makes them is **`../../../../../../../../tools`**, and the mouthpiece
and bell are **`../../../../..`**.

**There is no README in this folder.** The reading page for this bore is
`../../../../../../../../greek-spiral/`, and the repository's writeup is at the
root. This file is the note that sits beside the cut files.


## One section is the whole design

Every other bore here splits into several sections that couple tab-into-notch. This
one is planar, so it splits into **one**, and that changes what matters:

- **There is no assembly order.** Two sheets, one section. The `01of01` in the
  filenames is not a sequence.
- **The seam clearance is inert.** `PLAY_BY_BORE` gives 0.025 per side at the 10mm
  bore and nothing on this design uses it — there is no section-to-section joint.
  Do not cite `coil/fold2`'s measured play as evidence about this bore.
- **Both ends are plain**, not just one. `plain_ends()` marks the first piece's
  entry and the last piece's exit; on a one-piece bore that is the same piece, so
  it carries both `~a` and `~b` and the filename says `-buttin-buttout-`.

A change that makes this design split into more than one section has changed the
walk, not the toolchain. Check the walk first.

## Regenerate under the venv python

`check.py` imports shapely, which the system `python3` does not have. `bore_split.py`
**writes every file before it gates them**, so a system-python `--write` leaves a
folder of finished-looking cut files and a traceback where the gate should be — which
is exactly how this folder was first produced. Both commands in the README's rebuild
block use `~/Software/boxes/venv/bin/python`. Keep it that way.

16mm is the default block, so neither command needs `--blocksize`. Pass the same pitch to
*both* if you ever pass it at all: the gate's geometry half is recut in-process, so gating
at a pitch the sheets were not cut at reports a clean run on a design nobody cut.

## Sheet 1 is 592mm on a 600mm bed

The largest sheet in the repository **by area** — 592.0 x 284.4mm, 1684cm2 — with 8.0mm
to spare on the width. It is **not** the widest, and the width is what runs out first.

Three sheets sit within 1.5mm of the 600mm bed, and they are the ones to watch:

| sheet | width | margin | gated by |
|---|---|---|---|
| `telescope-wide` section 9 | 598.9mm | 1.1mm | `check.py` |
| `volute` narrow panels | 598.9mm | 1.1mm | `ribbon_bore.py` |
| `hilbert/open` section 14 | 598.6mm | 1.4mm | `check.py` |

**The volute panel sheet is a swept-curve sheet, so a different generator gates it** —
`ribbon_bore.py` refuses at `BED_W`, `check.py` never sees it. Two gates, one bed: a
change that moves either one has to be read against both, and neither reports the other's
sheets. It ties `telescope-wide` for the widest thing in the repository.

The gate's `sheet fits the bed` check passes the block sheets, and will keep passing right
up to 600.0. The nester may split differently rather than failing, so compare the reported
sheet sizes after any change and do not assume two sheets stays two.

## In the generator's corpus

`../../../../../../../../tools/regress.py` gates every design in `walks/` on every
toolchain change, and this walk is one of them: `walks/greek_spiral.txt`, with a row
pointing at `./bore` at `--bore=10`. A toolchain change that breaks this bore is
caught there rather than on the bench.

## Colour is the cut order

Shared across these repositories: **blue engraves, then green → orange → cyan →
black**; black frees the part; **violet `#8000ff` means skip**. Bore nets use two
stages — blue engraves the section number, black cuts. **Not this one.** It is
a single section, so the number would read `1` on all twenty-four parts and
answer a question nobody can ask; nothing is engraved and black is the only
colour on the sheet. Marking each wall with its own length would name the stick
and nothing else: the plate carries no matching mark, and the one that would
complete it cannot be derived. The plate is the jig — a wall of length L fits
only the run of length L, and all eleven lengths differ.

## Publishing

This folder publishes nothing of its own. The page that describes this bore is
`../../../../../../../../greek-spiral/`, and Pages deploys it from `main` through
`.github/workflows/pages.yml`, keyed per commit. Its `index.html` is `README.md`
rendered by `md2html.py` and committed, not built on the server, so regenerate it
there after editing that README and read the audit before pushing:

```sh
cd ../../../../../../../../greek-spiral
G=../../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
```
