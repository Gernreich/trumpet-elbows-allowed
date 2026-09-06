# Telescope Spiral

    U U3 N1 W1 S2 E2 N3 U1 W2 S3 E3 N4 U1 W4 S5 E5 N6 U1 W6 S7 E7 N8 U1
    W8 S9 E9 N10 U1 U

114 blocks, 1824mm of bore in 160 x 144 x 176mm. 18 sections, 102 parts,
18 cut files. 482 checks pass.

The loops after the first grow by exactly two blocks each, legs 2k, 2k+1,
2k+1, 2k+2 for k of 1 to 4:

    loop 2   W2 S3 E3 N4
    loop 3   W4 S5 E5 N6
    loop 4   W6 S7 E7 N8
    loop 5   W8 S9 E9 N10

That regularity is a progression, not a symmetry. Of the 48 symmetries of a
cube only the identity maps this shape onto itself, and no adjustment will
change that: the coil always turns the same way, so no reflection can fit it,
and the loops differ in size, so no rotation can either.

Five square loops, each one wider than the last, stacked as they grow - which
is where the name comes from, one drawing out of the next. The mouth is a
3-block stem at the bottom and the bell leaves through the top of the last
loop, both vertical, 304mm apart.

Each file is `bore10-telescope-spiral-NNof18-<kind>-<shape>-cut-files.svg`.

    #    blocks   kind     plate
    1    1-3      straight 3x1 bl     51x19mm
    2    4-4      elbow    1x1 bl     19x19mm
    3    5-5      elbow    1x1 bl     19x19mm
    4    6-6      elbow    1x1 bl     19x19mm
    5    7-12     bend     3x3 bl     51x51mm
    6    13-13    elbow    1x1 bl     19x19mm
    7    14-14    elbow    1x1 bl     19x19mm
    8    15-25    bend     4x4 bl     67x67mm
    9    26-26    elbow    1x1 bl     19x19mm
    10   27-27    elbow    1x1 bl     19x19mm
    11   28-46    bend     6x6 bl     99x99mm
    12   47-47    elbow    1x1 bl     19x19mm
    13   48-48    elbow    1x1 bl     19x19mm
    14   49-75    bend     8x8 bl   131x131mm
    15   76-76    elbow    1x1 bl     19x19mm
    16   77-77    elbow    1x1 bl     19x19mm
    17   78-104   bend     10x10 bl 163x163mm
    18   105-114  bend     2x9 bl    147x35mm

0.91 parts a block: six bends carry 99 of the 114 blocks, and a loop costs the
same one file however wide it gets. Sections 11, 14 and 17 need more than one
sheet - their parts together are taller than the bed even though every part
fits it.

## What the tool flags

Nine inside corners are left open, at the elbows either side of each loop
transition, and the four `U1` transitions each put two single-block elbows back
to back, where only 2 of 3 tabs engage. Every one of those comes from the `U1`.
Writing `U2` instead costs 4 blocks, folds each elbow pair into the neighbouring
bend, and removes all nine open corners.

Thirteen pairs of blocks touch without being joined along the bore - the loops
run alongside each other. Legal and sealed; it means 6mm of wood between those
runs rather than 3.
