# Wide Telescope

    U U3 N4 W4 S5 E5 N6 U3 W6 S7 E7 N8 U3 W8 S9 E9 N10 U3 U

101 blocks, 3131mm of bore in 310 x 403 x 341mm. Ten sections, 72 parts,
14 cut files. 354 checks pass.

The Telescope Spiral redrawn so that no leg is shorter than three blocks.
Three nested loops, the innermost with four-block legs, each loop two blocks
wider than the one inside it, and three blocks of climb between them:

    loop 1   N4 W4 S5 E5      legs 4 4 5 5
    loop 2   W6 S7 E7 N8      legs 6 7 7 8
    loop 3   W8 S9 E9 N10     legs 8 9 9 10

Mouth opens down at the bottom, bell opens up at the top, 682mm apart.

    #    blocks    plate        file                             parts  sheets
    1    1-5       4x2 bl       01_bend_RRRD.svg                   6      1
    2    6-23      6x6 bl       02_bend_DDLLLLUUUUURRRRRD.svg     10      2
    3    24-29     2x5 bl       03_bend_DDDDR.svg                  6      1
    4    30-32     2x2 bl       04_bend_UL.svg                     6      1
    5    33-52     8x8 bl       05_bend_LLLLUUUUUUURRRRRRRD.svg   10      2
    6    53-60     2x7 bl       06_bend_DDDDDDR.svg                6      1
    7    61-63     2x2 bl       07_bend_UL.svg                     6      1
    8    64-71     7x2 bl       08_bend_LLLLLLU.svg                6      1
    9    72-89     10x8 bl      09_bend_UUUUUUURRRRRRRRRD.svg      8      3
    10   90-101    4x9 bl       10_bend_DDDDDDDDRRR.svg            6      2

## Why it is cheaper than the Telescope Spiral

Every piece is a bend. There is not one elbow, no flattened plate, no tongue
and no unfilled corner - all of which exist only where a section meets an
elbow.

    Telescope Spiral   114 blocks  3534mm  19 sections  106 parts  11 elbows
    Wide Telescope     101 blocks  3131mm  10 sections   72 parts   0 elbows

The difference is entirely leg length. The Telescope's innermost hook is
N1 W1 S2 E2 - four legs too short to give their turns an arm - and lengthening
them alone is impossible, because the loop outside is only two blocks away and
the walk runs into itself. The whole nest has to start wider, which is what
this does.

Eight pairs of blocks touch where the loops pass each other. Legal and sealed;
it means 6mm of wood between those runs rather than 3.
