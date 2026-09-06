# Hilbert Open

    python3 hilbert.py 2 --scale 3

190 blocks, 3040mm of bore in a 310mm cube. 27 sections, 220 parts, 28 cut
files. 1010 checks pass. No elbows, no flattened plates, no tongues, no
unfilled corners, and nothing touching anything.

The order-2 Hilbert curve with three blocks to a step instead of two. Three is
what the turns need: each step of the curve ends in a turn, so a step of three
blocks leaves one arm for the piece behind and one for the piece ahead.

    order/scale    blk    bore   sect  parts  elbow  flats  voids   cube
    2 at x1         64  1984mm     63    252     56     48     44   124mm
    2 at x2        127  3937mm     45    248     22     26      4   217mm
    2 at x3        190  3040mm     27    220      0      0      0   310mm
    2 at x4        253  7843mm     27    220      0      0      0   403mm

Scale 3 is where it saturates: 4 costs 63 more blocks and changes nothing about
the cutting. Below it every step is too short and the curve pays for it - at
scale 1 the thing is 63 sections and 56 elbows for 64 blocks.

Note that the elbow-free version is also the cheapest to cut. 220 parts against
248 at scale 2 and 252 at scale 1, for half as much bore again. An elbow saves
parts on its own piece and spends them on the two either side of it.

The ends are 279mm apart, both on the same face of the cube - the Hilbert
curve starts and ends on one edge at every order. To carry the bell away from
the mouth, add a snorkel: see hilbert_snorkel in the walks folder for the shape
of it.
