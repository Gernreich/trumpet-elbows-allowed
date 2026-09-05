# trumpet

Laser-cut trumpets, the toolchain that draws their bores, and the library of
bores it has drawn. Everything is millimetre-true at `1 user unit = 1mm`, so it
prints and cuts at real size.

A trumpet is a mouthpiece, a length of tube and a bell. Only the tube differs
between them, which is why the tube is the only thing that belongs to an
instrument here and the two ends are shared.

## The tree

    tools/        the generator, the splitter, and the pre-cut gate
    parts/        the bell and the mouthpiece, shared by every bore
    bores/
      coiled/     25mm, coils flat and drops twice, no elbows
      octagonal/  25mm, the flare form of the octagonal torus
      switchback/ 10mm, folds back on itself twice
      stretched/  10mm, straights longer than turns -- a test, not an instrument
      ribbon/     constant section swept along a planar curve
    designs/      every walk built out as cut files and a page
    spirals/      coiling walks, with the numbers that say what each one costs

The **[octagonal torus](https://github.com/Gernreich/torus-octagonal)** stays in
its own repository. It is a torus, not a trumpet, and `bores/octagonal` is the
cut that opens it out into one.

## The gate

Nothing is cut until it passes. From `tools/`:

    ~/boxes/venv/bin/python regress.py

    28 designs, 9135 checks

Two lattices run through the same gate: `UNIFORM` designs sit on a cubic block,
`STRETCHED` ones run their straights longer than their turns. They were checked
by two forked copies of this toolchain until 2026-09-05, which meant a change to
`bore_split.py` was only ever proved against whichever half you happened to be
standing in. There is one copy now.

`snakebox.py` and `snakeboxvar.py` are [Boxes.py](https://github.com/florianfesti/boxes)
generators and are not standalone -- Boxes.py provides the finger joints, the
burn compensation and the SVG writer. See `tools/README.md` to set that up.

## No elbows

An **elbow** is a turn stranded as its own one-block piece, and it is a cost: it
leaves a piece with no flat face to glue against, so a neighbour's plate has to
be flattened and butt-glued to it. Every bore built here since the first trumpet
splits with none. `bore_split.py --refuse-elbows` will not draw one.

## History

This repository replaces nine, merged 2026-09-05 with their full history intact:
`bore-generator`, `bore-designs`, `bore-stretched`, `bore-ribbon`, `spirals`,
`trumpet-coiled`, `trumpet-octagonal`, `trumpet-switchback` and `trumpet-parts`.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where
the cutting and the playing are shown.
