# `built/` is the instruments that exist

One folder per instrument that has been cut and press-fit together. **Everything here is
wood somebody is holding**; everything in `../parts/bore/concept/` is a drawing.

- Folders are named for the **design** (the names their generators write), not the page.
  `../three-turn/` is the writeup. No two directories in the
  repository share a name.
- **The `coil-` prefix is load-bearing.** `folder_stack()` builds sheet names from the
  output path, and `built` names no family, so the leaf must carry it. Drop it and
  `repro.py` redraws twelve sheets under names nothing on disk has.
- **Both ends are shared**: one bell and one mouthpiece design, in `../parts/`, see
  `../ends/`. Never copy them in here.

## `coil-fold2-long-straight-3t/`: the three-turn trumpet

44 blocks, 12 sections, 1096mm, three turns about a north–south axis. It plays (one note
is F4). Walk in `../tools/walks/coil-3t.txt`, writeup at `../three-turn/`.

| | |
| --- | --- |
| bell | `../parts/bell/` — `bell-round10-153mm-17rings-x3-rim86`, seated in the tube end |
| mouthpiece | `../parts/mouthpiece/` — `mouthpiece-bore10-trumpet-parts`, seated in the tube end, **24 rings where the design cuts 30** |
| finish | scorched birch under several coats of shellac |

**`cut-files/` is a REDRAW, not the record.** Since 2026-09-13 it is drawn at the 0.15mm
kerf; the wood was cut at 0.1mm, so every drawn part is 0.05mm smaller per axis. The
sheets as cut are in `cut-files/old/` (gitignored, local) and in git before that date.
Never read `cut-files/` as a description of the object.

## The spiral bore moved out

The 1000mm swept-curve spiral, `ribbon-spiral-bore10-45deg-R35to113/`, was here until
2026-10-14 and is now in the swept-curve repository (`~/LaserMadeMusic/GIT/swept-curve`,
`built/`), with its page. It still takes this repository's bell and mouthpiece.

## Gates

`../tools/regress.py` gates the coil (`coil 10x10x30 3t`). It needs the Boxes.py venv, or
every design fails on shapely:

```
cd ../tools && SNAKEBOX_BOXES=~/Software/boxes \
  SNAKEBOX_PY=~/Software/boxes/venv/bin/python \
  ~/Software/boxes/venv/bin/python regress.py
```
