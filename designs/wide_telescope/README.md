# Wide Telescope

    U U3 N4 W4 S5 E5 N6 U3 W6 S7 E7 N8 U3 W8 S9 E9 N10 U3 U

101 blocks, 1616mm of bore in 160 x 208 x 176mm. 9 sections, 68 parts,
9 cut files, one sheet each. 290 checks pass.

The Telescope Spiral redrawn so that no leg is shorter than three blocks.
Three nested loops, the innermost with four-block legs, each loop two blocks
wider than the one inside it, and three blocks of climb between them:

    loop 1   N4 W4 S5 E5      legs 4 4 5 5
    loop 2   W6 S7 E7 N8      legs 6 7 7 8
    loop 3   W8 S9 E9 N10     legs 8 9 9 10

Mouth opens down at the bottom, bell opens up at the top, 352mm apart.

Each file is `bore10-wide-telescope-NNof09-<shape>-cut-files.svg`.

    #    blocks    plate      shape                           parts  sheets
    1    1-5       4x2 bl     bend-RRRD-buttin                    6       1
    2    6-23      6x6 bl     bend-DDLLLLUUUUURRRRRD             12       1
    3    24-29     2x5 bl     bend-DDDDR                          6       1
    4    30-32     2x2 bl     bend-UL                             6       1
    5    33-52     8x8 bl     bend-LLLLUUUUUUURRRRRRRD           10       1
    6    53-60     2x7 bl     bend-DDDDDDR                        6       1
    7    61-63     2x2 bl     bend-UL                             6       1
    8    64-89     10x10 bl   bend-LLLLLLUUUUUUUUURRRRRRRRRD     10       1
    9    90-101    4x9 bl     bend-DDDDDDDDRRR-buttout            6       1

## Why it is cheaper than the Telescope Spiral

Every piece is a bend. There is not one elbow, no flattened plate, no tongue
and no unfilled corner - all of which exist only where a section meets an
elbow.

    Telescope Spiral   114 blocks  1824mm  19 sections  106 parts  11 elbows
    Wide Telescope     101 blocks  1616mm  10 sections   72 parts   0 elbows

The difference is entirely leg length. The Telescope's innermost hook is
N1 W1 S2 E2 - four legs too short to give their turns an arm - and lengthening
them alone is impossible, because the loop outside is only two blocks away and
the walk runs into itself. The whole nest has to start wider, which is what
this does.

Eight pairs of blocks touch where the loops pass each other. Legal and sealed;
it means 6mm of wood between those runs rather than 3.
