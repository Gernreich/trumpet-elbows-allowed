# Telescope Spiral

    U U3 N1 W1 S2 E2 N3 U1 W2 S3 E3 N4 U1 W4 S5 E5 N6 U1 W6 S7 E7 N8 U1
    W8 S9 E9 N10 U1 U

114 blocks, 3534 mm of bore in 310 x 279 x 341 mm. 19 sections, 104 parts,
23 cut files. 567 checks pass.

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
loop, both vertical, 589 mm apart.

    #    blocks    kind      plate         file
    1    1-3       straight  3x1 bl        01_straight3_lapS.svg
    2-4  4-6       elbow x3                02..04_elbow_EN.svg
    5    7-12      bend      3x3 bl        05_bend_URRDD...
    6-7  13-14     elbow x2                06..07_elbow_NE.svg
    8    15-25     bend      4x4 bl        08_bend_LUUURRRDDD...
    9-10 26-27     elbow x2                09..10_elbow_NE.svg
    11   28-46     bend      6x6 bl        11_bend_LLLUUUUURRRRRDDDDD...  2 sheets
    12-13 47-48    elbow x2                12..13_elbow_NE.svg
    14   49-75     bend      8x8 bl        14_bend_LLLLL...  2 sheets
    15-16 76-77    elbow x2                15..16_elbow_NE.svg
    17   78-102    bend      9x10 bl       17_bend_LLLLLLL...  3 sheets
    18   103-103   elbow                   18_elbow_EN.svg
    19   104-114   bend      2x10 bl       19_bend_DDDDDDDDDR...

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
run alongside each other. Legal and sealed; it means 6 mm of wood between those
runs rather than 3.
