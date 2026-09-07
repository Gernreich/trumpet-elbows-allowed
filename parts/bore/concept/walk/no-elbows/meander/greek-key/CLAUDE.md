# CLAUDE.md

**Nothing in this repository has been cut.** It was generated on 2026-09-04, moved
here out of the design library the same day, and gated at 85 checks. A passing gate
means no check failed, not that the part is buildable — see `../../../../../../../tools/CLAUDE.md`,
which lists two bench failures the gate could not see. Say "gated" and not "built"
until one exists.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

The bore of a trumpet whose walk is a flat meander — the Greek key. It ships cut
files only; the thing that makes them is **`../../../../../../../tools`**, and the mouthpiece
and bell are **`../../../../..`**.

**The README is gone.** Every `README.md` and `index.html` under `trumpet/` was
removed on 2026-09-05, pending one new writeup for the trumpet as a whole once
the renaming and reorganising is finished. Git has them all. Until it exists,
this file is the documentation, and any recipe below that renders or audits a
README is waiting on that writeup rather than describing something present.


## One section is the whole design

Every other bore here splits into several sections that couple tab-into-notch. This
one is planar, so it splits into **one**, and that changes what matters:

- **There is no assembly order.** Two sheets, one section. The `01of01` in the
  filenames is not a sequence.
- **The seam clearance is inert.** `PLAY_BY_BORE` gives 0.025 per side at the 10mm
  bore and nothing on this design uses it — there is no section-to-section joint.
  Do not cite `meander/fold2`'s measured play as evidence about this bore.
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
block use `~/boxes/venv/bin/python`. Keep it that way.

16mm is the default block, so neither command needs `--blocksize`. Pass the same pitch to
*both* if you ever pass it at all: the gate's geometry half is recut in-process, so gating
at a pitch the sheets were not cut at reports a clean run on a design nobody cut.

## Sheet 1 is 592mm on a 600mm bed

The largest sheet in any of these repositories, with 8mm to spare on the width. The
gate's `sheet fits the bed` check passes it, and it will keep passing it right up to
600.0. Anything that grows the walk, or the block, will fail there first — and the
nester may split differently rather than failing, so compare the reported sheet sizes
after any change and do not assume two sheets stays two.

## Not yet in the generator's corpus

`../../../../../../../tools/regress.py` gates every design in `walks/` on every toolchain
change. This walk is passed on the command line instead, so a regression that broke
it would not be caught by anything. Adding `walks/greek_spiral.txt` and a row in
`regress.py` pointing at `./bore` at pitch 16 is the fix; it has not
been done, and the README says so.

## Colour is the cut order

Shared across these repositories: **blue engraves, then green → orange → cyan →
black**; black frees the part; **violet `#8000ff` means skip**. Bore nets use two
stages — blue engraves the section number, black cuts.

## Publishing

Pages deploys from `main` through `.github/workflows/pages.yml`, keyed per commit.
`index.html` is `README.md` rendered by `md2html.py` and committed, not built on the
server, so a stale `index.html` publishes stale content — regenerate it after editing
the README, and read the audit before pushing:

```sh
G=../../../../../../../../lasermade-tools
python3 $G/md2html.py README.md index.html
python3 $G/doc-audit.py README.md --html index.html
```
