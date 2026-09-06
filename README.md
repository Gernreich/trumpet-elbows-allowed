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
      greek-spiral/ 10mm, a flat meander -- the Greek key, in one piece
      octagonal/  25mm, the flare form of the octagonal torus
      switchback/ 10mm, folds back on itself twice
      stretched/  10mm, straights longer than turns; the 3-turn coil plays F4
      ribbon/     constant section swept along a planar curve
    octagonal/    the octagonal torus, and the trumpet cut out of it
    designs/      every walk built out as cut files and a page
    spirals/      coiling walks, with the numbers that say what each one costs

`octagonal/` holds two objects and one writeup. The torus is a ring of square
section; the trumpet is that same ring opened out into a curve -- the same
25 x 25mm channel, and literally the same R 90 plate. They were two repositories
and then two directories, which meant the shared-plate claim spanned a boundary
and could not be checked in one place. Now it is one command:

    cd octagonal
    node verify_torus.js octagonal-trumpet.svg RunA2_R59Point693.svg
    # -> COMPLEMENTARY the plate's tabs land in the panel's notches

It is not a bore, which is why it sits beside `bores/` rather than inside it.

## The gate

Nothing is cut until it passes. From `tools/`:

    ~/boxes/venv/bin/python regress.py

    29 designs, 9047 checks

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

This repository replaces eleven, merged 2026-09-05 with their full history intact:
`bore-generator`, `bore-designs`, `bore-stretched`, `bore-ribbon`, `spirals`,
`trumpet-coiled`, `trumpet-octagonal`, `trumpet-switchback`, `trumpet-parts`
`torus-octagonal` and `greek-spiral`, the last of which had never been pushed
anywhere and existed only as a local checkout.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where
the cutting and the playing are shown.
