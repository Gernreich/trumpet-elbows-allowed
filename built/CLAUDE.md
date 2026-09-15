# `built/` is the instruments that exist

One folder per instrument that has been cut, glued and assembled. **Everything
here is wood somebody is holding**; everything in `../parts/bore/concept/` is a
drawing. That is the whole distinction, and it is why this tree sits at the
repository root rather than under `../parts/`: an instrument is a bore *and* the
two ends on it, so it is not a thing that belongs inside `parts/bore/`.

**The folders are named for the design, not for the page.** `../three-turn/` and
`../ribbon-spiral/` are the writeups; `coil-fold2-long-straight-3t/` and
`ribbon-spiral-bore10-45deg-R35to113/` are the parts, under the names their
generators write. No two directories in this repository share a name.

**THE `coil-` PREFIX IS LOAD-BEARING.** `bore_split.py`'s `folder_stack()` builds
every sheet's filename out of the output path, borrowing the parent when it names
a family — `coil`, `meander`, `spiral`, `hilbert`, `swept-curve`. Under the old
`parts/bore/built/coil/fold2-long-straight-3t` the parent supplied it. Here the
parent is `built`, which names no family, so the leaf carries it instead and the
sheets keep the `bore10-coil-fold2-long-straight-3t-` names they were cut under.
Drop the prefix and `repro.py` redraws twelve sheets under names nothing on disk
has, which is what it did the first time this folder moved.

**Both ends are shared, so neither is copied in here.** There is one bell design
and one mouthpiece design, both in `../parts/`, and both instruments wear them —
see `../ends/`. What differs is where they seat, which is recorded per
instrument below.

## `coil-fold2-long-straight-3t/` — the three-turn trumpet

A 1096mm coil, 44 blocks in 12 sections, winding three whole turns about a
north–south axis. **It plays; one of its notes is F4.** Walk in
`../tools/walks/coil-3t.txt`, writeup at `../three-turn/`.

| | |
| --- | --- |
| bell | `../parts/bell/` — `bell-round10-153mm-17rings-x3-rim86`, seated in the tube end |
| mouthpiece | `../parts/mouthpiece/` — `mouthpiece-bore10-trumpet-parts`, seated in the tube end |
| finish | scorched birch under several coats of shellac |

**The mouthpiece on it is 24 rings where the design cuts 30** — 72mm rather than
90. The instrument is short there, not the drawing.

**`cut-files/` is a REDRAW, not the record.** On 2026-09-13 the twelve pins for
this coil were removed from `../tools/as-built.sha256` on the author's
instruction to redraw it at the current ply and kerf, so `cut-files/` is drawn
at 3.0mm and 0.15mm while the wood was cut at 3.0mm and 0.1mm. **The ply is the
same; only the kerf moved.** Boxes.py insets each outline by BURN = KERF/2, so
0.1 → 0.15 takes 0.025mm off every side and every part comes out 0.05mm smaller
in each axis. Measured against `cut-files/old/`, that is the whole difference —
each sheet is 0.05mm shorter, and narrower by 0.05mm times the parts across it
(0.30mm on the six-part sheets, 0.40mm on the eight-part ones). The twelve
sheets as actually cut are in `cut-files/old/` — gitignored, a local archive —
and in git before that date. Do not read `cut-files/` as a description of the
object.

## `ribbon-spiral-bore10-45deg-R35to113/` — the spiral

1000mm of centreline in 17 facets of 45°, winding out from R34.7 to R112.9. A
swept curve, not a lattice walk: two faces flat, two faceted. Writeup at
`../ribbon-spiral/`.

| | |
| --- | --- |
| bell | `../parts/bell/` — same design, seated in the **square port**, its shank standing out of the plane of the coil through a 10 x 10mm hole in one 3mm cheek |
| mouthpiece | `../parts/mouthpiece/` — same design, on the straight lead, seated in the TUBE END where four walls grip the full 16mm |
| finish | bare ply, and **not airtight** — it has not had the shellac the three-turn has |

**Only one of the three variants in `cut-files/` is the wood.** The pair named
`ported-square-narrow` is what was cut; `narrow` and `ported-narrow` are
drawings of the same design under other flags, kept because the generator
insists a sheet cannot overwrite its full-width twin. Reproduce the cut pair
byte-identically with the command in `../ribbon-spiral/README.md`.

**Its generator lives in the concept tree** — `ribbon_bore.py` and
`ribbon_view.py` under `../parts/bore/concept/swept-curve/` — because it draws
every swept curve, cut or not. Only the design moved here, not the tool that
draws it.

## Gates

`previews/` came with the spiral and must stay with it: `all-gates.sh` walks
every `previews/` directory and asks each file for its cut file, so a preview
whose sheet has moved reads as stale. Rebuild the spiral's previews from **this**
directory, not from `swept-curve/`, whose regeneration loop no longer reaches
them:

```
G=~/LaserMadeMusic/GIT/lasermade-tools
for f in $(find . -path '*/cut-files/*.svg' -not -path '*/old/*'); do
  python3 $G/make-preview.py "$f" "$(dirname $(dirname $f))/previews/$(basename $f)"
done
```

The walk gate covers this tree through `../tools/regress.py`, whose
`coil 10x10x30 3t` entry names `../built/coil-fold2-long-straight-3t`. It needs the
Boxes.py venv, or every design fails with `No module named 'shapely'`:

```
cd ../tools && SNAKEBOX_BOXES=~/Software/boxes \
  SNAKEBOX_PY=~/Software/boxes/venv/bin/python \
  ~/Software/boxes/venv/bin/python regress.py
```
