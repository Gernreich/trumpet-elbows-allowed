# spiral trumpet

    U U3 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U3 U

95 blocks, 1520mm of centreline. An expanding square spiral: three loops, each
wider than the last, rising two blocks between them, entering and leaving
upward. 10 sections, 64 flat parts, 10 sheets.

Each file is `bore10-spiral-trumpet-NNof10-<kind>-<shape>-cut-files.svg`.

    #    blocks   kind     plate
    1    1-3      straight 3x1 bl     51x19mm
    2    4-4      elbow    1x1 bl     19x19mm
    3    5-13     bend     3x5 bl     83x51mm
    4    14-17    bend     3x2 bl     51x35mm
    5    18-18    elbow    1x1 bl     19x19mm
    6    19-39    bend     7x9 bl   147x115mm
    7    40-47    bend     7x2 bl    115x35mm
    8    48-48    elbow    1x1 bl     19x19mm
    9    49-81    bend     11x13 bl 211x179mm
    10   82-95    bend     11x4 bl   179x67mm

At the 25mm bore this was eleven sections, not ten: one loop of the spiral had
to be cut in two because as a single piece it came to 406 x 344mm, 36mm over the
600mm bed. At 10mm the same loop is well inside it -- the largest plate here is
211 x 179mm -- so the splitter no longer needs to divide it. The bed rule has not
changed; the pieces got smaller.

Sections 6, 10 and 11 need more than one sheet - their parts together are
taller than the bed even though every part fits it.

Section 1 carries a lap: its wall runs 3mm past the joint as a tongue and
fills the inside of the bend below the first elbow.

The insides of the bends at sections 5 and 8 are left open. Neither neighbour
can carry a tongue on a wall there, so it would have to go on a plate, which is
not supported. Each is a 3 x 3 x 25mm void on the inside of the turn, sealed
from outside by the surrounding walls - a notch in the bore rather than a leak.

## The pictures

`spiral_directions.png` — six views, each block coloured by the way the bore
travels through it: north blue, south orange, east green, west purple, up red,
down teal. Opposite directions are far apart in hue so a leg and its return leg
never read alike. The "from above" panel is the one that shows the three loops
nested.

`spiral_pieces.png` — the same six views coloured by section instead, numbered
to match the cut list, with heavy lines on the seams.

`spiral.html` — the same bore to turn around with the mouse. Open it in a
browser; nothing is installed and nothing is fetched but the fonts. Drag to
turn, scroll to zoom, right-drag to pan. It colours by direction or by section,
the legend isolates one of either when clicked, and the slider walks the bore
from the mouthpiece a block at a time - which is the one thing a still picture
cannot do, and the reason a spiral is hard to read on paper.

    python3 bore_render.py --directions "U U3 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U3 U"
    python3 bore_render.py "U U3 ..."          # by section
    python3 viewer.py "U U3 ..." --out ../../../test/spiral_trumpet/spiral.html \
        --title "Spiral Trumpet Bore"

## Checking

    python3 check.py "U U3 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U3 U" \
        --files ../../../test/spiral_trumpet

355 checks, 0 failed.
