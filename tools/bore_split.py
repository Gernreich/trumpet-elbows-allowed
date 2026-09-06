"""Split a bore written in the agreed notation into cuttable part files.

World axes: +X right, +Y up, +Z toward you.

    N S  away / toward you    E W  east / west    U D  up / down
    (facing north, east is on your right)

    <entry> <run> <run> ... <exit>

    You walk the tunnel's centreline.

        first   the way you are facing at the tunnel mouth
        N.5     half a block in, to the centre of block 1 (second term)
        E       re-point east, standing in the block you are already in
        E2      move two blocks east
        E.5     half a block out, leaving through that face (last term)

    You leave facing whatever direction you last pointed. A trailing letter is
    therefore optional: repeating your heading is a no-op, but naming a new one
    turns the final block into an elbow and you exit that way.

    A bare letter costs no block: the turn happens inside the block you
    arrived at. So the tunnel is 1 + the sum of the numbers blocks long, and
    every block where the heading changes is an elbow.

        D E1 S          elbow(D->E), straight 1, elbow(E->S)
        D E4 U2 E6 S    4 elbows and 3 straights

    A bare heading in the middle is rejected: it would mean two elbows butted
    with no run between, which is only sound when both turns lie in the same
    plane. Write U1 if you really want a one-cell jog.

Every elbow is the same part whichever way it turns, so a bore needs one elbow
file plus one file per distinct run length.

    python3 bore_split.py "D R1 F"              report only
    python3 bore_split.py "D R1 F" --write DIR  also cut the files
    python3 bore_split.py "D R1 F" --refuse-elbows   refuse a walk with any
    python3 bore_split.py "D R1 F" --bore=10          the airway, square,
        rather than the block outside: --bore=10 is --blocksize=16 at 3mm ply.
    python3 bore_split.py "D R1 F" --bore=10 --straight=30
        straights 30mm long with the turns left cubic, so the bore lengthens
        without the walk changing. The cross-section stays square either way.
    python3 bore_split.py "D R1 F" --blocksize=31    a larger bore: the
        pitch is the sound square plus two walls, so 31 is 25mm of air in 3mm
        stock, where the default 16 is 10mm of air. Pass the same number to
        check.py or the gate measures the wrong design.
"""
import html, os, re, subprocess, sys, xml.etree.ElementTree as ET

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import svgpath as V

# Facing north, east is to your right. North is away from you, south toward you.
DIRS = {'E': (1, 0, 0), 'W': (-1, 0, 0), 'U': (0, 1, 0),
        'D': (0, -1, 0), 'S': (0, 0, 1), 'N': (0, 0, -1)}
# Point BOXES at your Boxes.py checkout with snakebox.py installed (see the
# install section of README.md). Override with the SNAKEBOX_BOXES env var.
BOXES = os.environ.get('SNAKEBOX_BOXES', os.path.expanduser('~/boxes'))
PY = os.environ.get('SNAKEBOX_PY', os.path.join(BOXES, 'venv/bin/python'))
BED_W, BED_H = 600.0, 308.0   # xTool P2S work area, mm
BLOCK, PIN, BURN = 16.0, 1.5, 0.1   # block pitch, tab reach, kerf allowance
# 16mm is 10mm of air in 3mm stock. It became the default on 2026-09-05, when
# the 25mm bore was retired: the bell and the mouthpiece went 10mm-only on
# 2026-09-03 and nothing was left to put on the end of a 25mm tube.
FEWEST_ELBOWS = True          # --fewest-pieces turns this off
# Fewest is not none. A build repository wants none, and wants to be told rather
# than handed a folder to inspect, so --refuse-elbows stops before anything is
# written. Off by default: most of the library exists to exercise elbows.
REFUSE_ELBOWS = False
BED = BED_W                   # sheets wrap to the bed width
OUTDIR = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                      '..', '..', 'test')
THICKNESS = 3.0
# A port lets a change of plane happen inside a piece, but the joint has not
# survived assembly: the plate it opens leaves the walls of that cell supported
# on one side, with their fingers facing nothing. Off unless asked for.
ALLOW_PORTS = False
FLAT = False        # --flat: plain butt ends, no tabs and no notches
TITLE = None        # --title: the page's title, when the folder makes a poor one
# Built from the constants above rather than typed out, because BLOCK is the
# pitch the plan is laid out on and --blocksize is the pitch SnakeBox cuts to.
# Set one without the other and the sheet and the plan quietly disagree.
# SnakeBox's own --pin_width default is 12mm, chosen when 31 was the only
# block there was. It is a fraction of the opening it has to sit in, not a
# length, so it has to shrink with the block: at 16mm the end frame is 10mm
# across and a 12mm tab does not fit in it at all. 0.48 reproduces 12 exactly
# at the 25mm square, which was the default until 2026-09-05.
PIN_FRAC = 0.48
MIN_SHOULDER = 2.0   # what the automatic sizing aims to leave beside the tab
MIN_FEATURE = 1.5    # and what check.py will actually refuse below
# SnakeBox leaves --pin_play at 0, on the grounds that it matches the finger
# joints, which carry no designed play either. It does not follow. A finger
# joint is a dozen teeth sharing an edge and the errors average out; the
# section seam is ONE tab in ONE notch, drawn exactly its own width, so a press
# fit before you add char, glue or any kerf the burn allowance did not predict.
# Reported from the bench 2026-08-31: section 1 would not enter section 2. This
# widens the notch only -- the tab keeps its full width and strength.
# Per side, so the notch opens by twice this. Measured per bore, not guessed and
# not modelled -- both rows below are assembled objects:
#
#     bore 25mm (tab 12.0 in a 25mm frame)   0.0   assembles, fits well
#     bore 10mm (tab  6.0 in a 10mm frame)   0.025 assembles, fits well
#                                            0.0   will not go together
#                                            0.05  very slightly loose
#                                            0.15  perceptible rock
#
# The same zero clearance that jams the small joint is fine on the large one, so
# the requirement is neither absolute nor constant.
#
# The 10mm bore is the extreme case, not merely a small one: the two sizing rules
# cross at exactly 10mm, where the tab is a full finger-joint tooth AND the
# shoulder is at its 2mm minimum together. Its shoulder is 0.67 of a ply against
# 2.17 at 25mm. Required clearance FALLS as the joint grows, which rules out both
# a constant absolute clearance and a constant fraction of the tab; what fits is
# a fabrication error fixed in millimetres against elastic take-up that scales.
# Hypothesis, not result -- the full reasoning and the coupon test that would
# settle it are in README.md, beside this file.
#
# This is a lookup of what has been cut, not a curve through it. It was kept in
# step by hand across two copies of this file until 2026-09-05, when the fork
# was collapsed; there is one copy now and the table moves with it. A bore not
# in the table gets the small-joint value, because too loose is a worse joint
# and too tight is no joint at all -- and pin_play() says on stderr that it is
# doing so, because a guess that looks like a measurement is the dangerous kind.
#
# The test that would settle the hypothesis needs no bore: cut a coupon of the
# two mating end frames at 22mm block -- the 16mm bore, where the tab is 7.68mm
# and the 48% fraction binds -- at 0, 0.0125 and 0.025 per side. If elastic
# take-up scales, the middle one fits.
#
#   W="N N2 U3 E3 E"
#   for t in A:0 B:0.0125 C:0.025; do
#     bore_split.py --blocksize=22 --play=${t#*:} --tag=${t%%:*} \
#         --refuse-elbows "$W" --write ../test/coupon-16mm/notch-${t%%:*}
#   done
#
# FOUR SHEETS, and the tag is not decoration. Section 2 carries the tab and is
# identical at all three clearances -- verified, not assumed: with the tag held
# constant, play 0 and play 0.025 give byte-identical cut geometry, because play
# is taken out of the notch and never off the tab. So cut section 2 once, cut
# section 1 three times, and try each against it. Those three come off the bed
# as three indistinguishable piles, the difference between them being 0.0125mm
# of notch, so --tag=A engraves an A beside every number on that run. Without it
# the test cannot be read, only performed.
PLAY_BY_BORE = {25.0: 0.0, 10.0: 0.025}
PLAY_UNMEASURED = 0.025


PLAY_OVERRIDE = None            # --play=, for cutting the coupon below
_UNMEASURED_SAID = set()


def pin_play():
    """The measured joint clearance for this bore, or the fallback -- out loud.

    A bore that has not been cut and assembled has no measurement. The fallback
    is the small-joint value, because too loose is a worse joint and too tight
    is no joint at all, and that direction is deliberate. What was not
    deliberate is that it was silent: the guess came back indistinguishable
    from a measurement, so a sheet could be cut at a clearance nobody had ever
    tested without anything saying so. It says so now, once per bore per run,
    on stderr so it cannot be mistaken for output a tool is parsing.
    """
    if PLAY_OVERRIDE is not None:
        return PLAY_OVERRIDE
    bore = round(BLOCK - 2 * THICKNESS, 3)
    if bore in PLAY_BY_BORE:
        return PLAY_BY_BORE[bore]
    if bore not in _UNMEASURED_SAID:
        _UNMEASURED_SAID.add(bore)
        print(f'note: the {bore:g}mm bore has no measured joint clearance. '
              f'Using {PLAY_UNMEASURED:g}mm per side, the small-joint value, '
              f'which is the safe direction but a guess. Cut a coupon, assemble '
              f'it, and add the row to PLAY_BY_BORE.', file=sys.stderr)
    return PLAY_UNMEASURED
# Set NOTCH to size the joint from the female side instead: the tab is then
# whatever fits it, NOTCH - 2 * pin_play(). It overrides the tooth floor below,
# so a notch small enough puts the tab back under the finger teeth -- say so
# rather than silently refusing, because on a small frame the shoulder either
# side of the notch may matter more than the tab's own width.
NOTCH = None


def set_notch(mm):
    """Size the joint from the notch. Tab follows at notch - 2 * play."""
    global NOTCH, COMMON
    NOTCH = float(mm)
    COMMON = _common()


def set_play(mm):
    """Override the table, for measuring rather than for cutting.

    The table is a lookup of what has been cut. This flag exists so the coupon
    the comment above asks for can be cut at values that are NOT in the table -
    which is the whole point of a coupon - without editing the table to values
    nobody has measured yet.

    COMMON is built once at import, so setting the override alone leaves the
    old figure in the SnakeBox arguments and the flag does nothing at all: the
    first three coupons came out byte-identical for that reason, which read as
    "play does not matter" rather than "the flag is not connected". Rebuild it,
    exactly as set_blocksize does.

    Dropped on 2026-09-05 when the stretched fork's copy of this file was
    promoted -- that fork never had it -- and restored the same day, because
    the coupon procedure in the comment above cannot be run without it.
    """
    global PLAY_OVERRIDE, COMMON
    PLAY_OVERRIDE = float(mm)
    COMMON = _common()


def pin_width():
    """The coupling tab, which must never be the weakest feature on the sheet.

    A fraction of the end frame it sits in -- but floored at the finger joint
    tooth, which Boxes.py sizes at 2 x thickness and which therefore does NOT
    shrink with the block. Scaling the tab alone took it below that: at the
    10mm bore the fraction gives 4.8mm against a 6mm tooth, so the one tab
    carrying the joint between two sections came out narrower than the
    ordinary teeth beside it. Reported from the cut file, not caught by the
    gate -- the gate's floor is MIN_FEATURE, 1.5mm, and 4.8 clears it.

    At the 25mm bore the fraction wins at 12mm and nothing moves.
    """
    tooth = 2.0 * THICKNESS                     # Boxes.py FingerJointSettings
    frame = BLOCK - 2 * THICKNESS               # the opening the tab sits in
    if NOTCH is not None:
        # MIN_SHOULDER is the target the automatic sizing aims for, not a limit
        # of the material: the shipped 10mm design already leaves 1.85mm beside
        # its notch and cuts fine. An explicit --notch is someone overriding
        # that target on purpose, so hold it to what the gate actually enforces,
        # MIN_FEATURE. Checking the notch against the TAB's cap refused a notch
        # the automatic path had itself produced.
        w = NOTCH - 2 * pin_play()
        if w <= 0:
            sys.exit(f'--notch must be wider than the {2*pin_play():g}mm of play')
        thin = min((frame - NOTCH) / 2, (frame - w) / 2)
        if thin < MIN_FEATURE:
            sys.exit(f'--notch={NOTCH:g} leaves {thin:.2f}mm beside it in a '
                     f'{frame:g}mm frame, under the {MIN_FEATURE:g}mm the gate '
                     f'allows')
        return w
    return min(max(PIN_FRAC * frame, tooth), frame - 2 * MIN_SHOULDER)


def _common():
    return [f'--blocksize={BLOCK:g}', f'--thickness={THICKNESS:g}',
            f'--burn={BURN:g}',
            f'--pin_width={pin_width():g}', f'--pin_play={pin_play():g}',
            '--labels=0', '--reference=0',
            '--inner_corners=corner', '--spacing=0.5']


COMMON = _common()


def set_blocksize(mm):
    """Change the block pitch. The only supported way: it moves both."""
    global BLOCK, COMMON, STRAIGHT
    was_cubic = STRAIGHT == BLOCK if 'STRAIGHT' in globals() else True
    BLOCK = float(mm)
    if was_cubic:
        STRAIGHT = BLOCK        # stay a cube unless --straight says otherwise
    COMMON = _common()


# A block's CROSS-SECTION is always square: BLOCK outside, BLOCK - 2t of air.
# Its length along the way the bore travels need not match. A block that turns
# is a cube -- the turn has to happen in something square or the two openings
# would sit at different heights -- and a block that runs straight is STRAIGHT
# long. With STRAIGHT == BLOCK every cell is a cube and nothing below changes.
#
# This is the one place the lattice stops being uniform, and it costs the
# integer grid: two columns of the same index can want different widths in
# different parts of the bore. That is legal within a piece (checked) and not
# globally, which is why positions are carried in mm from here on and not in
# cells.
STRAIGHT = BLOCK


def set_straight(mm):
    """How long a straight block runs. Turn blocks stay cubic at BLOCK."""
    global STRAIGHT
    STRAIGHT = float(mm)


def set_bore(mm):
    """The airway, square. The block outside is that plus a wall each side."""
    set_blocksize(float(mm) + 2 * THICKNESS)


def cubic():
    return STRAIGHT == BLOCK


def extent(r, axis):
    """One block's outer size along a world axis."""
    if r['in'] == r['out'] and AXIS[r['out']] == axis:
        return STRAIGHT
    return BLOCK


def block_boxes(rec):
    """Every block as a real (lo, hi) box in mm, by following the chain.

    Once straights are longer than turns the lattice index no longer maps to a
    coordinate: the same column wants two widths in different parts of the
    bore. So position is carried in mm from the mouth instead, each block
    butting its entry face onto the previous block's exit face. For a cubic
    cell this reproduces `index * BLOCK` exactly.
    """
    half = BLOCK / 2.0
    boxes, p = [], (0.0, 0.0, 0.0)      # p: centre of the entry face
    for r in rec:
        din, dout = DIRS[r['in']], DIRS[r['out']]
        a_in = AXIS[r['in']]
        run = extent(r, a_in)           # along the way it arrives
        centre = tuple(p[i] + din[i] * run / 2.0 for i in range(3))
        hw = [half] * 3
        hw[a_in] = run / 2.0
        if r['in'] != r['out']:         # a turn is a cube, square every way
            hw = [half] * 3
            centre = tuple(p[i] + din[i] * half for i in range(3))
        boxes.append((tuple(centre[i] - hw[i] for i in range(3)),
                      tuple(centre[i] + hw[i] for i in range(3))))
        a_out = AXIS[r['out']]
        out = extent(r, a_out) if r['in'] == r['out'] else BLOCK
        p = tuple(centre[i] + dout[i] * out / 2.0 for i in range(3))
    return boxes


def piece_widths(rec, group, k):
    """--widths_* for a piece, in SnakeBox's cell frame.

    SnakeBox lays its own cells out from (0,0) following --path, while these
    are world lattice coordinates, so the indices have to be rebased on the
    piece's first cell. Gaps in the range are filled with BLOCK; a boundary run
    only ever crosses occupied columns, so a gap is never actually measured,
    but leaving a hole in the list would make the span lookup raise.
    """
    if cubic():
        return []
    ax = [j for j in range(3) if j != k]
    base = [rec[group[0]]['pos'][j] for j in ax]
    out = []
    for a, (axis, b) in enumerate(zip(ax, base)):
        w = {}
        for j in group:
            i = rec[j]['pos'][axis] - b
            e = extent(rec[j], axis)
            if w.setdefault(i, e) != e:
                raise ValueError(
                    f'piece at blocks {group[0]+1}-{group[-1]+1} needs column '
                    f'{"uv"[a]}={i} to be both {w[i]:g} and {e:g}mm wide. A '
                    'block is long only along the axis it runs straight on, so '
                    'a straight and a turn sharing a column cannot both fit. '
                    'Shorten --straight to the block size, or change the walk.')
        lo, hi = min(w), max(w)
        vals = ','.join(f'{w.get(i, BLOCK):g}' for i in range(lo, hi + 1))
        out += [f'--widths_{"uv"[a]}={vals}', f'--origin_{"uv"[a]}={lo}']
    return out

G = {
 '0': [[(.5,1),(.85,.8),(.85,.2),(.5,0),(.15,.2),(.15,.8),(.5,1)]],
 '1': [[(.30,.78),(.52,1),(.52,0)],[(.28,0),(.78,0)]],
 '2': [[(.10,.78),(.30,1),(.70,1),(.90,.78),(.90,.60),(.10,0),(.90,0)]],
 '3': [[(.10,1),(.90,1),(.45,.55)],[(.45,.55),(.90,.55),(.90,.16),(.72,0),(.28,0),(.10,.16)]],
 '4': [[(.70,0),(.70,1),(.12,.32),(.92,.32)]],
 '5': [[(.85,1),(.20,1),(.15,.55),(.50,.62),(.80,.50),(.88,.28),(.75,.06),(.40,0),(.15,.12)]],
 '6': [[(.82,.92),(.55,1),(.25,.85),(.15,.45),(.15,.18),(.35,0),(.62,0),(.85,.18),(.85,.38),(.62,.55),(.30,.55),(.15,.45)]],
 '7': [[(.12,1),(.90,1),(.42,0)]],
 '8': [[(.5,.55),(.22,.68),(.22,.87),(.5,1),(.78,.87),(.78,.68),(.5,.55),(.18,.40),(.18,.14),(.5,0),(.82,.14),(.82,.40),(.5,.55)]],
 '9': [[(.18,.08),(.45,0),(.75,.15),(.85,.55),(.85,.82),(.65,1),(.38,1),(.15,.82),(.15,.62),(.38,.45),(.70,.45),(.85,.55)]],
 'E': [[(.90,1),(.15,1),(.15,0),(.90,0)],[(.15,.5),(.68,.5)]],
 'S': [[(.90,.85),(.70,1),(.30,1),(.10,.85),(.10,.65),(.90,.35),(.90,.15),(.70,0),(.30,0),(.10,.15)]],
 'P': [[(.15,0),(.15,1),(.70,1),(.90,.80),(.90,.64),(.70,.44),(.15,.44)]],
 'W': [[(.05,1),(.26,0),(.50,.70),(.74,0),(.95,1)]],
 'N': [[(.15,0),(.15,1),(.85,0),(.85,1)]],
 'C': [[(.90,.80),(.70,1.0),(.30,1.0),(.10,.80),(.10,.20),(.30,0.0),(.70,0.0),(.90,.20)]],
 'A': [[(.10,0),(.50,1),(.90,0)],[(.26,.40),(.74,.40)]],
 'F': [[(.88,1),(.15,1),(.15,0)],[(.15,.52),(.68,.52)]],
 'G': [[(.90,.80),(.70,1),(.30,1),(.10,.80),(.10,.20),(.30,0),(.70,0),(.90,.20),(.90,.45),(.55,.45)]],
 'H': [[(.15,1),(.15,0)],[(.85,1),(.85,0)],[(.15,.5),(.85,.5)]],
 'B': [[(.15,0),(.15,1),(.68,1),(.88,.83),(.88,.68),(.68,.55),(.15,.55)],
       [(.15,.55),(.72,.55),(.90,.40),(.90,.16),(.70,0),(.15,0)]],
 'L': [[(.18,1),(.18,0),(.88,0)]],
 'D': [[(.15,0),(.15,1),(.58,1),(.88,.74),(.88,.26),(.58,0),(.15,0)]],
 'U': [[(.15,1),(.15,.24),(.36,0),(.64,0),(.85,.24),(.85,1)]],
 'R': [[(.15,0),(.15,1),(.70,1),(.90,.80),(.90,.66),(.70,.50),(.15,.50)],
       [(.50,.50),(.90,0)]],
}


def parse(text):
    t = text.strip().upper()
    bad = re.sub(r'[NSEWUD0-9.\s,]', '', t)
    if bad:
        raise ValueError(f'unexpected characters: {bad!r}')
    if '.' in t:
        raise ValueError(
            'half blocks are no longer written: the first term already puts '
            'you half a block in, at the centre of block 1')
    toks = re.findall(r'([NSEWUD])(\d*)', t)
    if len(toks) < 2:
        raise ValueError('need the way you came in, then at least one move')
    if toks[0][1]:
        raise ValueError(
            f'"{toks[0][0]}{toks[0][1]}": the first term is only the way you '
            'came in. Put that travel in the next term.')
    out = [(d, int(n) if n else 0) for d, n in toks]
    for i, (d, n) in enumerate(out):
        if 0 < i < len(out) - 1 and n == 0:
            raise ValueError(
                f'"{d}" in the middle does nothing. Turning without travelling '
                'either repeats the term after it or tries to bend one block '
                f'twice. Give it a distance, or drop it.')
    return out


def walk(text):
    """Walk the tunnel. One record per block: position, heading in, heading out.

    You start half a block in, at the centre of block 1, facing the first term.
    Every term after that turns you where you stand and then moves you n blocks,
    so the turn costs nothing and the tunnel is 1 + the sum of the numbers.
    """
    toks = parse(text)
    heading = toks[0][0]
    pos = (0, 0, 0)
    rec = [{'pos': pos, 'in': heading, 'out': heading}]
    seen = {pos: 1}
    for d, n in toks[1:]:
        if d != heading:
            if sum(a*b for a, b in zip(DIRS[heading], DIRS[d])) != 0:
                raise ValueError(f'{heading} -> {d} reverses the bore, not a turn')
            if rec[-1]['in'] != rec[-1]['out']:
                raise ValueError(
                    f'"{d}{n or ""}" turns again in a block that already turns '
                    f'{rec[-1]["in"]} to {rec[-1]["out"]}. A block bends once; '
                    'give the previous term at least one block of travel.')
            rec[-1]['out'] = d
            heading = d
        for _ in range(n):
            pos = tuple(pos[k] + DIRS[heading][k] for k in range(3))
            if pos in seen:
                raise ValueError(
                    f'block {len(rec)+1} runs into block {seen[pos]}: the bore '
                    f'is back at {pos}. Two blocks cannot share a cell - the '
                    'walk has to go round, not through.')
            seen[pos] = len(rec) + 1
            rec.append({'pos': pos, 'in': heading, 'out': heading})
    return rec


def touching(rec):
    """Blocks that sit face to face without being neighbours along the bore.

    Legal, and often unavoidable in a tight coil, but worth saying out loud: the
    two blocks each keep their own wall, so the bore runs through 6 mm of wood
    there rather than 3, and a walk that folds back hard enough can end up with
    its openings blocked by the piece it folded past.
    """
    pos = [r['pos'] for r in rec]
    out = []
    for i in range(len(pos)):
        for j in range(i + 2, len(pos)):
            if sum(abs(pos[i][k] - pos[j][k]) for k in range(3)) == 1:
                out.append((i + 1, j + 1))
    return out


def blocks(text):
    """Group the walk into pieces: each turning block is an elbow, runs of
    straight blocks group together."""
    rec = walk(text)
    out2, run, h = [], 0, None
    for r in rec:
        if r['in'] != r['out']:
            if run:
                out2.append(('straight', run, h, h)); run = 0
            out2.append(('elbow', 1, r['in'], r['out']))
        else:
            if run == 0:
                h = r['in']
            run += 1
    if run:
        out2.append(('straight', run, h, h))
    if not out2:
        raise ValueError('that describes no blocks at all')
    return out2


AXIS = {'E': 0, 'W': 0, 'U': 1, 'D': 1, 'N': 2, 'S': 2}
FACE2D = {(0, -1): 'S', (1, 0): 'E', (0, 1): 'N', (-1, 0): 'W'}
PATH2D = {(1, 0): 'R', (-1, 0): 'L', (0, 1): 'U', (0, -1): 'D'}
VEC2D = {v: k for k, v in FACE2D.items()}       # letter -> unit vector


def turn2d(v, q):
    """v turned a quarter turn anticlockwise, q times."""
    for _ in range(q % 4):
        v = (-v[1], v[0])
    return v


def simple_snake(cells):
    """Do these 2D cells form one path, rather than a fork or a ring?

    A piece is cut as a flat snake, so every cell must have at most two
    neighbours in the piece and exactly two must have one. A walk that touches
    itself can bring a piece back alongside its own earlier blocks - a U-turn
    two blocks wide does it - and that cell then has three neighbours. The
    generator refuses such a piece; the search must not offer it.
    """
    s = set(cells)
    if len(s) == 1:
        return True
    deg = {c: sum(1 for d in ((1, 0), (-1, 0), (0, 1), (0, -1))
                  if (c[0] + d[0], c[1] + d[1]) in s) for c in s}
    if any(v > 2 for v in deg.values()):
        return False
    if sum(1 for v in deg.values() if v == 1) != 2:
        return False
    # and it must not pinch. Two cells touching only at a corner leave a point
    # where the tube has no width - the generator refuses it - and a piece
    # spiralling inward does exactly that as its arms pass. If one of the two
    # cells between them is present the corner is an ordinary L and fine.
    for c in s:
        for dx, dy in ((1, 1), (1, -1), (-1, 1), (-1, -1)):
            if (c[0] + dx, c[1] + dy) in s:
                if (c[0] + dx, c[1]) not in s and (c[0], c[1] + dy) not in s:
                    return False
    return True


def piece_plan(rec, a, b):
    """(k, port_in, port_out) for blocks a..b, or None if it cannot be cut.

    A piece is one flat slab, so its cells must share a constant coordinate -
    that axis is the plane normal, k. Each opening is then either on the rim,
    which needs the bore to arrive along the plane, or through a face plate,
    which is a port and needs the bore to arrive along k.

    A piece that will not fit the bed is not cuttable either, so it is refused
    here rather than reported afterwards: the search then returns the fewest
    pieces that can actually be made, instead of the fewest that could be if
    the machine were bigger.
    """
    pos = [rec[i]['pos'] for i in range(a, b + 1)]
    consts = [j for j in range(3) if len({p[j] for p in pos}) == 1]
    single = (a == b)
    best = None
    for k in consts:
        if not simple_snake([tuple(p[j] for j in range(3) if j != k)
                             for p in pos]):
            continue
        if not fits_bed(plate_mm(plate_span_mm(rec, range(a, b + 1), k))):
            continue
        pi = AXIS[rec[a]['in']] == k
        po = AXIS[rec[b]['out']] == k
        if pi and po:
            continue            # no rim opening left to couple by
        if (pi or po) and not ALLOW_PORTS:
            continue
        if not single:
            # a rim opening sits opposite its neighbour cell, so an end block
            # that bends can only open through a plate
            if rec[a]['in'] != rec[a]['out'] and not pi:
                continue
            if rec[b]['in'] != rec[b]['out'] and not po:
                continue
        elif pi or po:
            continue            # single cell + port not supported yet
        cand = (pi + po, k, pi, po)
        if best is None or cand < best:
            best = cand
    return None if best is None else best[1:]


def coplanar_pieces(text):
    """Split the walk into the fewest cuttable pieces.

    Shortest path over block positions. What it minimises is set by
    FEWEST_ELBOWS: by default the single-block elbows first and the number of
    pieces second, and with --fewest-pieces the other way round. Ties after
    that go to fewer ports. Returns the plan for each piece too, since which
    plane it lies in and which ends are ports are decided here.

    Fewest elbows is not the cheapest in parts - measured over 133 walks it
    trades 23 elbows for 46 more parts, because folding a turn into a bend adds
    two walls to that bend while a lone elbow is only four parts for its whole
    block. It is what is wanted here regardless: an elbow is a one-block piece
    with three tabs, fiddly to hold and weak at the seam, and parts are cheap
    by comparison.
    """
    rec = walk(text)
    n = len(rec)
    INF = float('inf')
    best = [(INF, INF, INF)] * (n + 1)
    best[0] = (0, 0, 0)
    cut = [None] * (n + 1)
    for i in range(1, n + 1):
        for a in range(i - 1, -1, -1):
            plan = piece_plan(rec, a, i - 1)
            if plan is None:
                continue
            lone = (i - a == 1 and rec[a]['in'] != rec[a]['out'])
            head = ((best[a][0] + lone, best[a][1] + 1) if FEWEST_ELBOWS
                    else (best[a][0] + 1, best[a][1] + lone))
            cost = (head[0], head[1], best[a][2] + plan[1] + plan[2])
            if cost <= best[i]:
                best[i] = cost
                cut[i] = (a, plan)
    if cut[n] is None:
        raise ValueError('no way to cut this bore into flat pieces')
    groups, plans, i = [], [], n
    while i > 0:
        a, plan = cut[i]
        groups.append(list(range(a, i)))
        plans.append(plan)
        i = a
    groups.reverse(); plans.reverse()
    return rec, groups, plans


def flat_sides(rec, groups, norms):
    """Which plate, at which end, loses its coupling.

    An elbow's opening frame has three sides: the fourth is its other opening.
    So at every seam with an elbow one side of the neighbour's frame has no
    mate - a notch nothing will fill, or a tab with nowhere to go. That side is
    always a face plate of the neighbour, so one of its two plates loses the
    coupling at that end and the other keeps it.

    Which of the two: the un-mirrored plate is the piece seen from the side
    given by the cross product of the axes flat() keeps - +x when the normal is
    x, -y for y, +z for z.
    """
    out = [['', ''] for _ in groups]
    for i in range(len(groups) - 1):
        for who, other, end in ((i, i + 1, 1), (i + 1, i, 0)):
            g = groups[other]
            if len(g) != 1 or rec[g[0]]['in'] == rec[g[0]]['out']:
                continue                     # the neighbour is not an elbow
            b = rec[g[0]]
            miss = (DIRS[b['out']] if other > who
                    else tuple(-x for x in DIRS[b['in']]))
            k = norms[who]
            axis = next(j for j in range(3) if miss[j])
            if axis != k:
                continue                     # not a plate side; nothing to do
            sign = 1 if k in (0, 2) else -1
            out[who][end] = 'first' if miss[axis] == sign else 'mirror'
    return [tuple(x) for x in out]


def plain_ends(plans):
    """Which openings butt plain rather than couple.

    A port is a bare hole - it cannot carry tabs, and it cannot carry notches
    either, because a tab needs material in a band where the plate has already
    stopped. So the opening that meets a port is plain on both sides: the two
    butt flat and are glued. Returns (plain_in, plain_out) per piece.

    The bore's two OUTER ends are plain for a different reason: there is no
    next section for them to couple to. What meets them is the mouthpiece at
    one end and the bell at the other, and both present a flat plate that
    glues onto the end face - the mouthpiece's station one, the bell's ring 0.
    A tab standing 3mm proud of that face holds the plate off it and leaves
    the joint on the tab alone. Reported from the bench 2026-08-31.
    """
    n = len(plans)
    plain = [[False, False] for _ in range(n)]
    for i, (k, pi, po) in enumerate(plans):
        if pi and i > 0:
            plain[i - 1][1] = True      # the piece before butts on this port
        if po and i < n - 1:
            plain[i + 1][0] = True
    plain[0][0] = True                  # the mouth: meets the mouthpiece plate
    plain[n - 1][1] = True              # the bell end: meets the bell flange
    return [tuple(p) for p in plain]


def lap_args(laps):
    a = []
    if laps[0]:
        a.append(f'--lap_in={laps[0]}')
    if laps[1]:
        a.append(f'--lap_out={laps[1]}')
    return a


def lap_tag(laps):
    """Suffix for a piece carrying a tongue. Font-safe: L plus the face."""
    if not any(laps):
        return ''
    return '@' + (laps[0] or '-') + (laps[1] or '-')


def plane_of(rec, idx):
    used = set()
    for i in idx:
        used.add(AXIS[rec[i]['in']])
        used.add(AXIS[rec[i]['out']])
    return used


def assign_laps(rec, groups, plans):
    """Which piece carries the tongue at each elbow, and how each is rolled.

    The tongue must land on a WALL, so the piece is rolled to suit where it can
    be: a straight's roll is free, a bend's is fixed by its own turn. Returns
    the forced plane normal per piece and the lap face at each of its ends.
    """
    n = len(groups)
    used = [plane_of(rec, g) for g in groups]
    norm = [pl[0] for pl in plans]
    free = [len(u) < 2 and not (pl[1] or pl[2]) for u, pl in zip(used, plans)]
    laps = [['', ''] for _ in groups]
    unfilled = []

    for e, g in enumerate(groups):
        if len(g) != 1 or rec[g[0]]['in'] == rec[g[0]]['out']:
            continue                                    # not an elbow
        b = rec[g[0]]
        want = []
        if e > 0:
            want.append((e - 1, DIRS[b['out']], 1))     # neighbour before, its exit
        if e < n - 1:
            want.append((e + 1, tuple(-x for x in DIRS[b['in']]), 0))
        filled = 0
        for p, d, end in want:
            gp = groups[p]
            # The tongue runs past a wall of the end frame it sits on. Only a
            # one-cell elbow can have an opening in that frame - its other
            # opening - and there is no wall to run past there. On any longer
            # piece the far opening is at the other end and irrelevant, so this
            # must not be tested against the piece's openings in general.
            if len(gp) == 1 and rec[gp[0]]['in'] != rec[gp[0]]['out']:
                if d in (tuple(-x for x in DIRS[rec[gp[0]]['in']]),
                         DIRS[rec[gp[0]]['out']]):
                    continue
            axis = next(j for j in range(3) if d[j])
            if axis == norm[p] and free[p]:
                # roll this straight so the void side becomes a wall
                travel = next(iter(used[p]))
                norm[p] = next(j for j in range(3) if j not in (travel, axis))
                free[p] = False
            if axis == norm[p]:
                continue                                # lands on a plate
            flat = tuple(d[j] for j in range(3) if j != norm[p])
            if flat not in FACE2D:
                continue
            laps[p][end] = FACE2D[flat]
            free[p] = False
            filled += 1
        if not filled:
            unfilled.append(e + 1)
    return norm, [tuple(l) for l in laps], unfilled


def piece_spec(rec, idx, k=None, laps=('', ''), ports=(False, False),
               plains=(False, False), flats=('', '')):
    """SnakeBox arguments for one piece, in its own 2D frame.

    Returns (code, args, note). Raises if the piece is one SnakeBox cannot cut:
    a flat snake opens each end on the face opposite its neighbour, so an end
    cell that bends is out of reach.
    """
    pos = [rec[i]['pos'] for i in idx]
    first, last = rec[idx[0]], rec[idx[-1]]
    # The plane is set by the directions the piece travels and turns through,
    # not by which coordinates happen to be constant: a piece whose cells lie
    # on a line has two constant axes and only one of them is the right one.
    port_in, port_out = ports
    if k is None:
        used = set()
        for i in idx:
            used.add(AXIS[rec[i]['in']])
            used.add(AXIS[rec[i]['out']])
        if len(used) > 2:
            raise ValueError(
                f'piece at blocks {idx[0]+1}-{idx[-1]+1} is not planar')
        k = next(j for j in range(3) if j not in used)

    def flat(v):
        return tuple(v[j] for j in range(3) if j != k)

    a_in = flat(tuple(-x for x in DIRS[first['in']]))     # hole the bore enters
    a_out = flat(DIRS[last['out']])                       # hole it leaves by
    if (not port_in and a_in not in FACE2D) or (not port_out and a_out not in FACE2D):
        raise ValueError(f'piece at blocks {idx[0]+1}-{idx[-1]+1} opens on a '
                         'face plate but was not given a port')

    def port_bits():
        """--port_* flags, including which of the two plates to cut.

        The un-mirrored plate is the piece seen from the side given by the
        cross product of the two axes kept by flat(): +x when k is x, -y when
        k is y, +z when k is z. The port has to be on the face the bore
        crosses, so compare the two.
        """
        out = []
        if not (port_in or port_out):
            return out
        out.append('--port_in' if port_in else '--port_out')
        want = (tuple(-x for x in DIRS[first['in']]) if port_in
                else DIRS[last['out']])
        sign = 1 if k in (0, 2) else -1
        unmirrored = tuple(sign if j == k else 0 for j in range(3))
        if want != unmirrored:
            out.append('--port_mirror')
        return out

    extra = port_bits() + piece_widths(rec, idx, k)
    if plains[0]:
        extra.append('--plain_in')
    if plains[1]:
        extra.append('--plain_out')
    if flats[0]:
        extra.append(f'--flat_in={flats[0]}')
    if flats[1]:
        extra.append(f'--flat_out={flats[1]}')
    # the gender of a piece's ends changes its geometry, so it has to show in
    # the name or two different parts would share a file
    ptag = (('~i' if port_in else '') + ('~o' if port_out else '')
            + ('~a' if plains[0] else '') + ('~b' if plains[1] else '')
            + ('~f' if flats[0] else '') + ('~g' if flats[1] else ''))

    if len(idx) == 1:
        # One-cell pieces are shared between rotations: every straight is the
        # same part, and four rotations of a turn share one file. Both rest on
        # the cell being a cube. On a cuboid a straight's section depends on
        # which axis it runs along, and rotating a turn swaps two pitches, so
        # the files would no longer be congruent and one drawing would stand
        # for parts that are not the same. Refuse rather than draw that.
        if not cubic():
            raise ValueError(
                f'block {idx[0]+1} is a one-cell piece and the cell is not a '
                f'cube (section {BLOCK:g}, straight {STRAIGHT:g}). Straights and '
                'elbows share one file per shape, which only holds for a cube. '
                'Lengthen the run, or use a cubic cell.')
        if first['in'] == first['out']:
            # Every straight is the same part: all four face pairs are congruent.
            return ('S1' + lap_tag(laps),
                    ['--path=', f'--open_faces={FACE2D[a_in]},{FACE2D[a_out]}']
                    + lap_args(laps) + extra, 'straight')
        # Elbows are NOT all one part. The four rotations of a turn are
        # congruent, but the two senses are not, so which way the bore turns
        # in its own plane picks one of exactly two parts. Sign of the cross
        # product of the two face normals tells them apart; each sense gets a
        # canonical face pair so the same file serves every rotation of it.
        cross = a_in[0] * a_out[1] - a_in[1] * a_out[0]
        faces = ('N', 'E') if cross < 0 else ('E', 'N')
        # The drawn elbow is a canonical rotation of the one in the walk - that
        # is how four rotations of a turn share one file. A lap was named in
        # the walk's frame, so it has to be turned into the drawn frame too or
        # it names the wrong side of the part, and half the time that side is
        # an opening and the generator cannot find a wall there.
        q = next(r for r in range(4)
                 if FACE2D[turn2d(a_in, r)] == faces[0]
                 and FACE2D[turn2d(a_out, r)] == faces[1])
        laps = tuple(FACE2D[turn2d(VEC2D[L], q)] if L else '' for L in laps)
        return ('E' + ''.join(faces) + lap_tag(laps),
                ['--path=', f'--open_faces={faces[0]},{faces[1]}']
                + lap_args(laps) + extra, 'elbow')

    cells = [flat(p) for p in pos]
    steps = [tuple(b[j] - a[j] for j in range(2)) for a, b in zip(cells, cells[1:])]
    path = ''.join(PATH2D[d] for d in steps)
    # SnakeBox opens each end opposite its neighbour; check that is what we need
    if not port_in and a_in != tuple(-x for x in steps[0]):
        raise ValueError(
            f'piece at blocks {idx[0]+1}-{idx[-1]+1}: the bore enters its end '
            'block from the side, so that block bends. SnakeBox opens an end '
            'only on the face opposite its neighbour.')
    if not port_out and a_out != steps[-1]:
        raise ValueError(
            f'piece at blocks {idx[0]+1}-{idx[-1]+1}: the bore leaves its end '
            'block sideways, so that block bends.')
    straight = len(set(path)) == 1
    code = ((f'S{len(cells)}' if straight else 'B' + path)
            + ptag + lap_tag(laps))
    return (code, [f'--path={path}'] + lap_args(laps) + extra,
            ('straight' if straight else 'bend'))


def glyphs(text, h):
    """The label, and a tick that says which way up it was engraved.

    Turned 180 degrees this table's 9 is exactly its 6 -- the same point list rotated, by
    construction -- so on the 9-section and 12-section coils, where both digits appear, a
    section 6 and a section 9 are the same mark on a rectangular side wall. The tick sits
    on the baseline right of the last digit.
    """
    w, gap, out, cx = h*0.62, h*0.20, [], 0.0
    for ch in text:
        for st in G[ch]:
            pts = ' '.join(f'{cx+px*w:.2f},{-py*h:.2f}' for px, py in st)
            out.append(f'<polyline points="{pts}" fill="none" stroke="#0000ff" '
                       f'stroke-width="0.3" stroke-linecap="round" stroke-linejoin="round"/>')
        cx += w + gap
    right = cx - gap
    tg, tl = h*0.16, h*0.22
    out.append(f'<polyline points="{right+tg:.2f},0.00 {right+tg+tl:.2f},0.00" '
               f'fill="none" stroke="#0000ff" stroke-width="0.3" '
               f'stroke-linecap="round" stroke-linejoin="round"/>')
    return out, right + tg + tl


def cut(args, tag):
    """Run SnakeBox and read back its parts.

    Two bits of tidying happen here. A path wholly inside another is a hole in
    it, not a part. And a magenta rectangle marks a region to cut away from the
    plate it sits on - that is how a port is made: the plate is drawn full size
    so its finger joints keep the spacing of the wall they mate with, and the
    port cell is subtracted afterwards, leaving every surviving edge exactly as
    drawn.
    """
    out = f'/tmp/snakebox_{tag}.svg'
    if os.path.exists(out):
        os.remove(out)          # never read a previous run's parts
    r = subprocess.run([PY, 'scripts/boxes', 'SnakeBoxVar'] + args + COMMON
                       + (['--pin_length=0'] if FLAT else [])
                       + [f'--output={out}'], cwd=BOXES, capture_output=True,
                      text=True)
    if r.returncode != 0 or not os.path.exists(out):
        raise RuntimeError('SnakeBox failed for %s\n  %s\n%s'
                           % (tag, ' '.join(args),
                              (r.stderr or r.stdout or '').strip()[-600:]))
    root = ET.parse(out).getroot()
    ps = []
    for g in root.iter(V.NS + 'g'):
        role = 'P' if g.get('id') in ('p-0', 'p-1') else 'W'
        paths, marks = [], []
        for p in g.iter(V.NS + 'path'):
            pts = V.pts(p.get('d'))
            xs = [q[0] for q in pts]; ys = [q[1] for q in pts]
            rec = {'d': p.get('d'), 'pts': pts, 'role': role,
                   'x0': min(xs), 'y0': min(ys),
                   'w': max(xs) - min(xs), 'h': max(ys) - min(ys)}
            if p.get('stroke') == 'rgb(255,0,255)':
                marks.append(rec)
            elif p.get('stroke') == 'rgb(0,0,0)':
                paths.append(rec)
        host = {}
        for q in paths:
            inside = [o for o in paths if o is not q
                      and o['x0'] <= q['x0'] + 1e-6 and o['y0'] <= q['y0'] + 1e-6
                      and o['x0'] + o['w'] >= q['x0'] + q['w'] - 1e-6
                      and o['y0'] + o['h'] >= q['y0'] + q['h'] - 1e-6]
            if inside:
                host[id(q)] = min(inside, key=lambda o: o['w'] * o['h'])
        for o in paths:
            if id(o) in host:
                continue
            o['holes'] = [q for q in paths if host.get(id(q)) is o]
            for m in marks:
                o = subtract(o, m)
            ps.append(o)
    # name them: the two face plates in the order drawn, then the walls in the
    # order of the boundary runs they lie along
    np_, nw = 0, 0
    for o in ps:
        if o['role'] == 'P':
            np_ += 1
            o['name'] = f'P{np_}'
        else:
            o['name'] = chr(ord('A') + nw)     # the wall on the nth run
            nw += 1
    return ps


def shave_stubs(poly, t, minw=3.0):
    """Trim slivers the cut leaves behind, wherever they are.

    Subtracting the port cell runs the cut through whatever finger is there and
    leaves a hair of it: half a millimetre of finger, and a tenth of a
    millimetre more where burn had stepped the edge. Both are too thin to
    survive cutting and sit on the face the mating piece slides past.

    Found by opening the shape - eroding then dilating - and taking the
    difference, which is the material too thin to survive the erosion, wherever
    on the outline it sits. Only those pieces are removed. Replacing the whole
    shape with the opened one instead would round every corner of every finger,
    and mitred joins can even add material at a reflex corner.
    """
    w = minw / 4.0
    for _ in range(6):          # a trimmed sliver can expose a thinner one
        opened = (poly.buffer(-w, join_style=2, mitre_limit=20)
                      .buffer(w, join_style=2, mitre_limit=20))
        thin = poly.difference(opened)
        bits = (list(thin.geoms) if thin.geom_type.startswith('Multi')
                else ([thin] if not thin.is_empty else []))
        took = False
        for b in bits:
            if b.area <= 0.02:
                continue
            trimmed = poly.difference(b.buffer(0.01))
            # take it if it removed the sliver and little else, and did not
            # sever the plate somewhere narrow
            if (trimmed.geom_type == 'Polygon'
                    and poly.area - trimmed.area < 1.5 * b.area + 0.5):
                poly, took = trimmed, True
        if not took:
            break
    return poly


def subtract(part, mark):
    """Cut the marked region out of a part, keeping its remaining edges."""
    from shapely.geometry import Polygon
    from shapely.ops import unary_union
    pp = part['pts'][:-1] if part['pts'][0] == part['pts'][-1] else part['pts']
    mm = mark['pts'][:-1] if mark['pts'][0] == mark['pts'][-1] else mark['pts']
    # the marker is drawn as a hole, so burn has shrunk it; grow it back a
    # little or a sliver of plate survives along the cut
    left = Polygon(pp).buffer(0).difference(Polygon(mm).buffer(0.2))
    if left.geom_type == 'MultiPolygon':
        left = max(left.geoms, key=lambda g: g.area)
    left = shave_stubs(left, THICKNESS)
    ring = list(left.exterior.coords)[:-1]
    d = ('M ' + ' L '.join(f'{x:.3f} {y:.3f}' for x, y in ring) + ' Z')
    xs = [x for x, _ in ring]; ys = [y for _, y in ring]
    out = dict(part)
    out.update({'d': d, 'pts': ring + [ring[0]], 'x0': min(xs), 'y0': min(ys),
                'w': max(xs) - min(xs), 'h': max(ys) - min(ys)})
    return out


def _inside(poly, x, y):
    """Even-odd point-in-polygon."""
    c = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > y) != (y2 > y):
            if x < x1 + (y - y1) * (x2 - x1) / (y2 - y1):
                c = not c
    return c


def _clearance(poly, x, y):
    """Distance from (x,y) to the nearest edge of the outline."""
    best = float('inf')
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        dx, dy = x2 - x1, y2 - y1
        L = dx * dx + dy * dy
        t = 0.0 if L == 0 else max(0.0, min(1.0, ((x - x1) * dx + (y - y1) * dy) / L))
        px, py = x1 + t * dx, y1 + t * dy
        d = ((x - px) ** 2 + (y - py) ** 2) ** 0.5
        if d < best:
            best = d
    return best


def strokes_fit(part, gs, ox, oy):
    """Is every point of the drawn label on the part, and off its holes?"""
    poly = part['pts'][:-1] if part['pts'][0] == part['pts'][-1] else part['pts']
    holes = []
    for q in part.get('holes', ()):
        hp = q['pts']
        holes.append(hp[:-1] if hp[0] == hp[-1] else hp)
    for g in gs:
        for pair in re.findall(r'(-?[\d.]+),(-?[\d.]+)', g):
            x, y = ox + float(pair[0]), oy + float(pair[1])
            if not _inside(poly, x, y) or any(_inside(h, x, y) for h in holes):
                return False
    return True


def label_spot(part, gw, gh, step=1.5):
    """Centre for a gw x gh label that lands on material.

    A part's bounding-box centre is only on the part when the part is convex
    enough: an L-shaped face plate has its centre out in the notch, and a
    ported plate has a hole through the middle of it. So take the inside point
    with the most room around it, shrink the label if it is tight, and give up
    rather than engrave off the part.
    """
    poly = part['pts']
    if poly[0] == poly[-1]:
        poly = poly[:-1]
    holes = []
    for q in part.get('holes', ()):
        hp = q['pts']
        holes.append(hp[:-1] if hp[0] == hp[-1] else hp)
    x0, y0, w, h = part['x0'], part['y0'], part['w'], part['h']
    best, bx, by = -1.0, x0 + w / 2, y0 + h / 2
    y = y0 + step
    while y < y0 + h:
        x = x0 + step
        while x < x0 + w:
            if _inside(poly, x, y) and not any(_inside(hp, x, y) for hp in holes):
                c = min([_clearance(poly, x, y)]
                        + [_clearance(hp, x, y) for hp in holes])
                if c > best:
                    best, bx, by = c, x, y
            x += step
        y += step
    need = ((gw / 2) ** 2 + (gh / 2) ** 2) ** 0.5
    scale = 1.0
    while need * scale > best and scale > 0.45:
        scale -= 0.05
    if need * scale > best:
        return None
    return bx, by, scale


TAG = ''             # --tag=, appended to every part's engraved number


def part_labels(p, code, args=None, neighbours=None):
    """The engraving for one part: its section number, and TAG if there is one.

    A coupon cut three times at three clearances comes off the bed as three
    identical piles - the difference is 0.0125mm of notch, which no one can
    see. --tag=A puts an A beside every number on that run, so the piles stay
    told apart. It changes the engraving and nothing else: the cut paths are
    the same with it and without.

    Part names and edge marks were tried and were harder to read than they
    were worth on parts this size. The number says which section a loose part
    belongs to, which is what actually gets lost on the bench.
    """
    gh = 5.0 / 3.0
    code = f'{code}{TAG}'
    _, gw0 = glyphs(code, gh)
    spot = label_spot(p, gw0, gh)
    if not spot:
        return ''
    cx, cy, sc = spot
    gs, gw = glyphs(code, gh * sc)
    if not strokes_fit(p, gs, cx - gw / 2, cy + gh * sc / 2):
        return ''
    return (f'<g transform="translate({cx-p["x0"]-gw/2:.2f},'
            f'{cy-p["y0"]+gh*sc/2:.2f})">' + ''.join(gs) + '</g>')


def provenance(meta, code, n, sheets, parts, sw, sh):
    """The <title> and <desc> a section's sheet carries.

    A cut file that has been downloaded, renamed or printed should still be
    able to say what it is. The name carries the bore and the design; this
    carries the rest, in the units the drawing is actually in - including the
    straight length, which is the whole point of this repository and is the
    one thing a stretched sheet cannot be told from a uniform one without.
    """
    if not meta:
        return ''
    bore = BLOCK - 2 * THICKNESS
    of = f' of {int(meta["total"])}'
    part = '' if len(sheets) == 1 else f', sheet {n} of {len(sheets)}'
    title = (f'{meta["design"]} bore - section {int(code)}{of}'
             f'{part}, {meta["kind"]} {meta["raw"][1:]}, '
             f'{len(parts)} parts on {sw:.0f}x{sh:.0f}mm')
    desc = (f'1 user unit = 1mm. {bore:g}mm square bore in {THICKNESS:g}mm '
            f'stock, so a {BLOCK:g}mm block pitch; a block that runs straight '
            f'is {STRAIGHT:g}mm long and a block that turns is a {BLOCK:g}mm '
            f'cube. Blocks {meta["span"]} of the walk, entering on '
            f'{meta["in"]} and leaving on {meta["out"]}, a {meta["plate"]} '
            f'block plate laid flat. Two face plates (mirror images) and the '
            f'side walls; every part carries the section number only. Inner '
            f'cuts are the port and come before their outline. '
            f'black #000000 cuts, blue #0000ff engraves.')
    esc = lambda t: t.replace('&', '&amp;').replace('<', '&lt;')
    return f'<title>{esc(title)}</title>\n<desc>{esc(desc)}</desc>\n'


def sheet(parts, code, path, bed=BED, bed_h=None, args=None,
          neighbours=None, meta=None):
    """Lay the parts out, turning and re-sheeting as the bed demands.

    A part taller than the bed is laid on its side; rows wrap at the bed width;
    and when the next row would run past the bed height the sheet is closed and
    another started, because a piece's parts together are often taller than the
    bed even when every one of them fits it.
    Returns [(path, w, h), ...], one entry per sheet written.
    """
    bed_h = BED_H if bed_h is None else bed_h
    M, GAP = 12.0, 10.0

    laid = []
    for p in parts:
        rot = p['h'] > bed_h - 2 * M and p['w'] <= bed_h - 2 * M
        laid.append((p, rot, (p['h'], p['w']) if rot else (p['w'], p['h'])))

    def pack(bw, bh):
        """Row-wrap into sheets; returns a list of sheets, each a list of
        (part, rot, w, h, x, y)."""
        sheets, cur = [], []
        x, y, rh = M, M, 0.0
        for p, rot, (w, h) in laid:
            if x > M and x + w + M > bw:
                if y + rh + GAP + h + M > bh:
                    sheets.append(cur); cur = []
                    x, y, rh = M, M, 0.0
                else:
                    x, y, rh = M, y + rh + GAP, 0.0
            cur.append((p, rot, w, h, x, y))
            x += w + GAP
            rh = max(rh, h)
        if cur:
            sheets.append(cur)
        return sheets

    sheets = pack(bed, bed_h)
    out = []
    for n, placed in enumerate(sheets, 1):
        body = []
        sw = max(x + w for _, _, w, _, x, _ in placed) + M
        sh = max(y + h for _, _, _, h, _, y in placed) + M
        for p, rot, w, h, x, y in placed:
            inner = ''.join(f'<path d="{q["d"]}" fill="none" stroke="#000000" '
                            f'stroke-width="0.2"/>' for q in p.get('holes', ()))
            local = (f'<g transform="translate({-p["x0"]:.3f},{-p["y0"]:.3f})">'
                     f'{inner}<path d="{p["d"]}" fill="none" stroke="#000000" '
                     f'stroke-width="0.2"/></g>')
            lbl = part_labels(p, code, args, neighbours)
            t = (f'translate({x+w:.3f},{y:.3f}) rotate(90)' if rot
                 else f'translate({x:.3f},{y:.3f})')
            body.append(f'<g transform="{t}">{local}{lbl}</g>')
        stem, ext = os.path.splitext(path)
        this = path if len(sheets) == 1 else f'{stem}-sheet{n}{ext}'
        open(this, 'w').write(
            f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{sw:.2f}mm" '
            f'height="{sh:.2f}mm" viewBox="0 0 {sw:.2f} {sh:.2f}">\n'
            + provenance(meta, code, n, sheets,
                         [p for p, *_ in placed], sw, sh)
            + '\n'.join(body) + '\n</svg>\n')
        out.append((this, sw, sh))
    return out


# Only a folder that says nothing but "bore" or a size is dull. 'bore-10mm'
# is; 'bore-stretched' is not, and climbing past it once titled a page
# "Git Bore Stretched Bore" off the parent directory.
DULL = re.compile(r'bores?([-_][\d.]+mm)?|[\d.]+mm')


def folder_stack(outdir):
    """(folder name, the folder names that say what the design is).

    A design folder inside a build repository is often just 'bore', which names
    the page fine and titles it uselessly - a browser tab reading "Bore" says
    nothing. Borrow the parent in that case: trumpet-coiled/bore reads as
    "Trumpet Coiled Bore". 'bore-10mm' is as parentless as 'bore' is, and so is
    the '10mm' that a candidate/10mm/bore layout puts between them: what those
    name is the size, and the instrument is still further up. So climb until a
    folder names something, rather than borrowing exactly one level.
    """
    full = os.path.normpath(os.path.abspath(outdir))
    name = os.path.basename(full)
    stack, at, cur = [name], full, name
    while DULL.fullmatch(cur.lower()):
        at = os.path.dirname(at)
        cur = os.path.basename(at)
        if not cur:
            break
        stack.insert(0, cur)
    return name, stack


def design_slug(stack):
    """The design's part of a cut file's name.

    The size terms come out, because the bore is stated separately and a name
    that says it twice - bore10-switchback-10mm - reads as a mistake.
    """
    keep = [w for w in stack if not DULL.fullmatch(w.lower())]
    return '-'.join(keep).lower().replace('_', '-')


def bore_tag():
    """bore25 / bore10 - the sound square, not the block pitch.

    Derived rather than typed, so it cannot disagree with the geometry the
    same run is cutting. On a stretched lattice the straight length varies and
    the bore does not, which is why the bore is the half that names the file.
    """
    return f'bore{BLOCK - 2 * THICKNESS:g}'


def filename(code):
    """The shape half of a file name, surviving the path letters.

    The suffixes are kept off the alphabet a path uses (U D L R), so a piece
    whose path contains an L cannot have its name shredded by the lap marker.
    Hyphens, not underscores, because the bell and mouthpiece sheets in
    trumpet-parts are hyphenated and one project reads better in one style.
    """
    base = code
    bits = []
    if '@' in base:
        base, faces = base.split('@', 1)
        bits.append('lap' + faces.replace('-', ''))
    for tag, name in (('~i', 'portin'), ('~o', 'portout'),
                      ('~a', 'buttin'), ('~b', 'buttout'),
                      ('~f', 'flatin'), ('~g', 'flatout')):
        if tag in base:
            base = base.replace(tag, '')
            bits.append(name)
    tail = ('-' + '-'.join(bits)) if bits else ''
    if base.startswith('E'):
        return f'elbow-{base[1:]}{tail}'
    if base.startswith('S'):
        return f'straight{base[1:]}{tail}'
    return f'bend-{base[1:]}{tail}'


def cutname(code, total, raw, stack):
    """The whole name of one section's sheet.

    bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg

    Every part of that earns its place. The bore, because the same walk at two
    bores makes two different sets of parts under one name - which is what
    these files did until 2026-09-03, in two repositories, and the only tell
    was the sheet size. The design, because a Downloads folder holds files from
    more than one, and because a stretched coil and a uniform one share both
    the bore and the walk. NNofTT, because a set with a sheet missing should
    say so.
    """
    lead = '-'.join(x for x in (bore_tag(), design_slug(stack)) if x)
    return (f'{lead}-{int(code):02d}of{int(total):02d}-'
            f'{filename(raw)}-cut-files.svg')


def specs_for(text):
    """Every section's SnakeBox arguments, in assembly order.

    One place decides this. It was worked out separately in bore_split, nest
    and check, and each time a switch was added - ports, flat ends, then the
    flattened sides - the other two carried on generating the old piece and
    checking or nesting something that was not being cut.
    """
    rec, groups, plans = coplanar_pieces(text)
    plains = plain_ends(plans)
    # laps first: rolling a free straight so its tongue lands on a wall changes
    # which sides of it are plates, and that is what decides the flats.
    norm, laps, unfilled = assign_laps(rec, groups, plans)
    flats = flat_sides(rec, groups, norm)
    out = []
    for i, g in enumerate(groups):
        code, args, note = piece_spec(rec, g, norm[i], laps[i],
                                      (plans[i][1], plans[i][2]), plains[i],
                                      flats[i])
        # norm[i], not plans[i][0]: assign_laps rolls a straight whose roll is
        # free so its tongue lands on a wall, and the piece is cut in the
        # rolled frame. Anything reasoning about which side is a plate has to
        # use the same one or it is describing a different piece.
        out.append({'group': g, 'code': code, 'args': args, 'kind': note,
                    'flat': flats[i], 'lap': laps[i], 'norm': norm[i],
                    'plan': plans[i]})
    return rec, groups, plans, out, unfilled


def walk_text(arg):
    """A walk, or something that holds one.

    Takes the notation itself, a .txt file with it in, or one of the viewer
    pages - which carry the walk verbatim in their title rail, so a page can be
    turned back into cut files without keeping the walk anywhere else.
    """
    if not os.path.exists(arg):
        return arg
    body = open(arg).read()
    if arg.lower().endswith(('.html', '.htm')):
        m = re.search(r'<div class="walk">([^<]+)</div>', body)
        if not m:
            raise ValueError(f'{arg} is not one of these pages: no walk in it')
        return html.unescape(m.group(1)).strip()
    return body.strip()


def plate_span(cells, k):
    """The piece's bounding box in its own plane, in blocks."""
    ax = [j for j in range(3) if j != k]
    return tuple(max(c[j] for c in cells) - min(c[j] for c in cells) + 1
                 for j in ax)


def plate_span_mm(rec, group, k):
    """The same box in mm, summing each column's own width."""
    ax = [j for j in range(3) if j != k]
    out = []
    for axis in ax:
        w = {}
        for j in group:
            w[rec[j]['pos'][axis]] = extent(rec[j], axis)
        lo, hi = min(w), max(w)
        out.append(sum(w.get(i, BLOCK) for i in range(lo, hi + 1)))
    return tuple(out)


def plate_mm(span_mm):
    """That box in mm, plus burn, plus a tab at an end that has one. Counted
    at both ends, which is 3 mm pessimistic on one axis - the right way round
    for deciding whether a piece can be made."""
    return tuple(q + 2 * PIN + BURN * 2 for q in span_mm)


def plate_size(rec, group, k):
    """How big a piece's face plate is, from the walk alone."""
    cells = [rec[j]['pos'] for j in group]
    return plate_span(cells, k), plate_mm(plate_span_mm(rec, group, k))


def fits_bed(mm):
    """A part can be turned, so it is the longer side that meets the longer bed."""
    a, b = max(mm), min(mm)
    return a <= BED_W and b <= BED_H


def main(text, outdir=None):
    rec, groups, plans, plan, unfilled = specs_for(text)
    specs = [(p['group'], p['code'], p['args'], p['kind']) for p in plan]

    n = len(rec)
    print(f'bore:  {text.strip()}')
    mm = sum(extent(r, AXIS[r['out']]) for r in rec)
    note = ('' if cubic() else
            f'  (section {BLOCK:g}, straight {STRAIGHT:g}, turns cubic)')
    print(f'       {n} blocks ({mm:g}mm of centreline){note}')
    print(f'       {len(specs)} pieces to assemble\n')

    # Number by position along the bore, not by shape: the engraved number is
    # what tells you which section a loose part belongs to, so it has to be the
    # order you assemble in. Two sections of the same shape therefore get their
    # own file rather than sharing one, or the numbers would lie.
    specs = [(g, str(i), args, note, code)
             for i, (g, code, args, note) in enumerate(specs, 1)]

    print('assembly order   (a piece is one flat snake = one SVG)')
    print('  #    blocks   kind       in   out   plate            shape')
    toobig = []
    facts = {}
    for (g, code, args, note, raw), k in zip(specs, [p['plan'][0] for p in plan]):
        r0, r1 = rec[g[0]], rec[g[-1]]
        span = f'{g[0]+1}-{g[-1]+1}'
        bl, mm = plate_size(rec, g, k)
        ok = fits_bed(mm)
        size = f'{bl[0]}x{bl[1]} bl {max(mm):.0f}x{min(mm):.0f}'
        if not ok:
            toobig.append((code, bl, mm))
        print(f'  {code:<4} {span:<8} {note:<10} {r0["in"]}    {r1["out"]}    '
              f'{size:<16} {raw}' + ('  !! over the bed' if not ok else ''))
        facts[code] = {'span': span, 'in': r0['in'], 'out': r1['out'],
                       'kind': note, 'plate': f'{bl[0]}x{bl[1]}'}

    if REFUSE_ELBOWS:
        bad = [code for _, code, _, note, _ in specs if note == 'elbow']
        if bad:
            raise ValueError(
                f'--refuse-elbows: section{"s" if len(bad) > 1 else ""} '
                f'{", ".join(bad)} of {len(specs)} '
                f'{"are elbows" if len(bad) > 1 else "is an elbow"}. '
                'Nothing written. Lengthen the term between the turns: a '
                'hairpin needs 2 and a coil 3.')

    print('\ncut list   (every part engraved with its section number)')
    total = 0
    shapes = {}
    # the name a sheet is written under says which bore and which design it
    # belongs to, so it has to be settled before the first file is written
    fname, stack = folder_stack(outdir if outdir else '.')
    for g, code, args, note, raw in specs:
        fn = cutname(code, len(specs), raw, stack)
        line = f'  {code:>3}  {fn:<64}'
        if outdir:
            os.makedirs(outdir, exist_ok=True)
            parts = cut(args, code)
            n = int(code)
            nb = {}
            if n > 1:
                nb['in'] = f'{n-1}{n}'
            if n < len(specs):
                nb['out'] = f'{n}{n+1}'
            sheets = sheet(parts, code, os.path.join(outdir, fn),
                           args=args, neighbours=nb,
                           meta=dict(facts[code], total=len(specs), raw=raw,
                                     design=design_slug(stack) or fname))
            total += len(parts)
            sizes = ' + '.join(f'{w:.0f}x{h:.0f}' for _, w, h in sheets)
            line += f'{len(parts)} parts   {sizes}mm'
            if len(sheets) > 1:
                line += f'   ({len(sheets)} sheets)'
            over = [t for t in sheets if t[1] > BED_W or t[2] > BED_H]
            if over:
                line += f'   !! over the {BED_W:.0f} x {BED_H:.0f} bed'
        shapes.setdefault(raw, []).append(code)
        print(line)
    if outdir:
        # a design folder gets the page that goes with it, named for the
        # folder, so cut files and the thing you turn around never drift apart
        import viewer
        name = fname
        # the same stack the file names were built from, so a page and the
        # sheets beside it cannot end up naming two different designs
        words = ' '.join(stack)
        # folder names are hyphenated across these repositories
        # (trumpet-coiled, torus-octagonal), so a hyphen is a word break
        # here exactly as an underscore is
        title = ' '.join(
            w if w[:1].isdigit() else w.title()      # '10mm', not '10Mm'
            for w in words.replace('_', ' ').replace('-', ' ').split())
        if 'Bore' not in title:
            title += ' Bore'
        # A folder name has to sort, stay unambiguous and survive a URL;
        # a page title has to read. 'coil-10x10x30-3t' is the right folder
        # and "Coil 10x10x30 3t Bore" is a filename read aloud, so --title
        # lets the two differ rather than forcing one to serve both.
        if TITLE:
            title = TITLE
        path = os.path.join(outdir, name + '.html')
        open(path, 'w').write(viewer.build(text, title))
        print(f'\n  {os.path.basename(path):<44}drag to turn, colour by '
              f'direction or section')
        # and put the written parts through the gate, so a design folder
        # having been made means it has been checked, rather than meaning
        # somebody remembered to check it
        import check
        nchecks, bad, fails = check.main(text, outdir, report=False)
        print(f'  {"checked":<44}{nchecks} checks, {bad} failed')
        for name, msgs in fails:
            print(f'    !! {name}')
            for m in msgs[:3]:
                print(f'       {m}')

    same = {k: v for k, v in shapes.items() if len(v) > 1}
    print(f'\n  {len(specs)} sections'
          + (f', {total} flat parts' if outdir else ''))
    for k, v in same.items():
        print(f'  sections {", ".join(v)} are the same shape, cut separately '
              f'so each carries its own number')

    rub = touching(rec)
    if rub:
        pairs = ', '.join(f'{a}-{b}' for a, b in rub[:6])
        print(f'\n  ! {len(rub)} pair(s) of blocks touch without being joined '
              f'along the bore:\n    {pairs}'
              + (' ...' if len(rub) > 6 else '')
              + '\n    Legal, but the walk folds back on itself there. Check '
                'the 3D view.')

    if toobig:
        print(f'\n  ! piece(s) {", ".join(c for c, _, _ in toobig)} do not fit '
              f'the {BED_W:.0f} x {BED_H:.0f}mm bed.\n'
              f'    A flat piece may span at most {int((BED_W - 2*PIN - 2*BURN)//BLOCK)}'
              f' blocks one way by '
              f'{int((BED_H - 2*PIN - 2*BURN)//BLOCK)} the other. Put a turn out '
              f'of the plane to break it up.')

    if unfilled:
        print(f'\n  ! the inside of the bend is left open at piece(s) '
              f'{", ".join(str(u) for u in unfilled)}: neither neighbour can\n'
              f'    carry the tongue on a wall, so it would have to go on a plate.')

    for i in range(len(specs) - 1):
        if specs[i][3] == 'elbow' and specs[i+1][3] == 'elbow':
            print(f'\n  ! pieces {i+1} and {i+2} are both single elbows meeting '
                  'directly.\n    Only 2 of the 3 tabs engage if their turns are '
                  'in perpendicular\n    planes. Consider a block of straight '
                  'between them.')


if __name__ == '__main__':
    # Run as a script this file is __main__, and check.py's `import bore_split`
    # loads it a second time as a separate module with its own globals. Setting a
    # switch on __main__ therefore set it for the writer and not for the gate:
    # `--ports --write` wrote ported cut files and then gated the unported design,
    # reporting 226 checks and 0 failed on parts nothing had looked at. Everything
    # below runs on the imported copy so there is one set of globals.
    import bore_split as B

    a = sys.argv[1:]
    d = B.OUTDIR
    if '--flat' in a:
        a.remove('--flat')
        B.FLAT = True
    if '--ports' in a:
        a.remove('--ports')
        B.ALLOW_PORTS = True
    if '--fewest-pieces' in a:
        a.remove('--fewest-pieces')
        B.FEWEST_ELBOWS = False
    if '--refuse-elbows' in a:
        a.remove('--refuse-elbows')
        B.REFUSE_ELBOWS = True
    bs = [x for x in a if x.startswith('--blocksize=')]
    if bs:
        a.remove(bs[0])
        B.set_blocksize(bs[0].split('=', 1)[1])
    bo = [x for x in a if x.startswith('--bore=')]
    if bo:
        a.remove(bo[0])
        B.set_bore(bo[0].split('=', 1)[1])
    tg = [x for x in a if x.startswith('--tag=')]
    if tg:
        a.remove(tg[0])
        B.TAG = tg[0].split('=', 1)[1]
    pl = [x for x in a if x.startswith('--play=')]
    if pl:
        a.remove(pl[0])
        B.set_play(pl[0].split('=', 1)[1])
    st = [x for x in a if x.startswith('--straight=')]
    if st:
        a.remove(st[0])
        B.set_straight(st[0].split('=', 1)[1])
    nt = [x for x in a if x.startswith('--notch=')]
    if nt:
        a.remove(nt[0])
        B.set_notch(nt[0].split('=', 1)[1])
    ti = [x for x in a if x.startswith('--title=')]
    if ti:
        a.remove(ti[0])
        B.TITLE = ti[0].split('=', 1)[1]
    if '--no-write' in a:
        a.remove('--no-write'); d = None
    if '--write' in a:
        i = a.index('--write'); d = a[i+1]; a = a[:i] + a[i+2:]
    try:
        B.main(B.walk_text(' '.join(a)) if a else 'D R1 F', d)
    except ValueError as e:
        print(f'error: {e}'); sys.exit(1)
