#!/usr/bin/env python3
"""Generate the adapter that carries the 7 x 14 port up to the bell's throat.

    python3 bell-adapter.py                  # the default 6-ring adapter
    python3 bell-adapter.py --rings=8        # a longer, gentler transition
    python3 bell-adapter.py --plies=2        # 6mm of rise a ring instead of 3
    python3 bell-adapter.py --port=7x14      # a different port
    python3 bell-adapter.py --out=FILE.svg   # somewhere other than cut-files/

A ported bore does not open at its mouth; it opens through a 7 x 14mm slot in the
cheek, and the bell's throat is the bore's own 10mm square. Those two are the same
area to within 2mm2 and nothing like the same shape, so something has to turn one
into the other. This is that part, and it is the reason the bell itself needs no
change: the top ring here presents exactly the face the bore's end presents, a
10mm square hole in a 16mm square, so bell-round.py's ring 0 lands on it as it was
always going to land on the bore.

WHY THE PORT IS NOT SQUARE is set out in ribbon_bore.py beside PORT_ACROSS, and it
is worth repeating because it is what this part exists to undo. The tab slots run
up both cheek walls the whole length of the bore, so what a port has to clear is
measured ACROSS the run and nothing else. Length ALONG the run is free. A 7 x 14
slot therefore clears the slots by 1.66mm where a bore-square 10 x 10 clears them
by 0.16mm, and buys back the area it gave away by growing in the direction that
costs nothing. The price is that the opening is the wrong shape for a horn, and
the price is paid here.

AREA IS THE SCHEDULE, and the aspect ratio is what moves. Each station encloses
the area the schedule asks for, 98mm2 at the port to 100mm2 at the throat, while
the ratio of the two half-widths runs 2:1 down to 1:1. Holding area is the whole
point: a horn that pinches anywhere is a horn with a step in it, and interpolating
the two half-widths straight -- the obvious thing -- dips the area 4% in the
middle, because the long axis loses faster than the short axis gains.

BOTH ENDS ARE TANGENT, which is what "smooth" has to mean for a part whose ends
are bolted to other parts. The schedule is a smoothstep, 3t^2 - 2t^3, on the area
and on the aspect alike, so the section leaves the port and arrives at the throat
with zero rate of change. A linear schedule meets the port at full tilt and puts a
visible crease in the first joint.

THE SECTION IS A SHARP RECTANGLE ALL THE WAY UP and never a rounded one. Both ends
are sharp -- the port is a slot with square corners, the throat is the bore's
square -- so there is nothing to round and nothing to round back. bell-round.py
rounds because it has to reach a circle; this does not.

RINGS STACK; THEY DO NOT TELESCOPE, the same as every other ring in this project.
Each ring's outer contour is the NEXT station's aperture grown by LAP, so each
seats on the one below by LAP per side, and a ring's wall is its own gain plus the
lap. That is also what sets the floor on how fast the section may move: a
half-width may SHRINK by at most LAP - MINWALL in one ring, or the wall on that
side thins past the minimum. The long axis has 2mm to lose and may lose 1mm a
ring, so three rings is the hard floor and the default is six.

NO KERF COMPENSATION, matching bell.py and bell-round.py. These rings are glued
laminations, not a press fit: a tenth of a millimetre goes into the glue line.
The bore's sheets compensate because their tabs have to enter their mortices.
"""
import math, os, pathlib, sys

RISE, LAP, MINWALL = 3.0, 3.0, 2.0   # ply, the seat per side, the thinnest wall
BORE  = 10.0        # the bore's air channel, and the bell's throat
PLATE = 16.0        # the bore's outside, and the face the bell's ring 0 covers
PORT_ACROSS, PORT_ALONG = 7.0, 14.0  # ribbon_bore.py's port, as CUT

args = [a for a in sys.argv[1:]]
opts = dict(a[2:].split("=", 1) for a in args if a.startswith("--") and "=" in a)
for k in opts:
    if k not in ("rings", "plies", "port", "out"):
        sys.exit(f"error: --{k} is not an option here. "
                 f"Try --rings, --plies, --port or --out.")
if "port" in opts:
    try:
        PORT_ACROSS, PORT_ALONG = (float(v) for v in opts["port"].lower().split("x"))
    except ValueError:
        sys.exit(f'error: --port={opts["port"]} should read like --port=7x14.')
PLIES = int(opts.get("plies", 1))
STEP = RISE * PLIES


def smoothstep(t):
    """0 to 1 with zero slope at both ends. The whole of "smooth" is in here."""
    return t * t * (3.0 - 2.0 * t)


def stations(m):
    """The m+1 apertures, port at 0 and the bell's throat at m, as (a, b) half-widths.

    a is ACROSS the cheek's band, b is ALONG the duct - the same two directions the
    port is drawn in, so a reader can put the sheet against the instrument.
    """
    a0, b0 = PORT_ACROSS / 2.0, PORT_ALONG / 2.0
    a1 = b1 = BORE / 2.0
    A0, A1 = 4.0 * a0 * b0, 4.0 * a1 * b1
    r0, r1 = b0 / a0, b1 / a1
    out = []
    for k in range(m + 1):
        s = smoothstep(k / m)
        area = A0 + (A1 - A0) * s
        ratio = r0 + (r1 - r0) * s
        a = math.sqrt(area / (4.0 * ratio))
        out.append((a, ratio * a))
    # The ends are solved, not interpolated towards: say so exactly rather than
    # leaving a reader to trust the arithmetic above.
    out[0], out[-1] = (a0, b0), (a1, b1)
    return out


def rings_from(sts):
    """One ring per gap, plus the collar that presents the bore's own end face.

    The collar is the part that makes the bell fit: aperture BORE square, outer
    PLATE square, which is exactly what the bell's ring 0 expects to be glued to.
    It is the last station repeated, so it has no gain and is pure seat.
    """
    rings = []
    for k in range(len(sts) - 1):
        (aa, ab), (na, nb) = sts[k], sts[k + 1]
        rings.append({"aa": aa, "ab": ab, "oa": na + LAP, "ob": nb + LAP,
                      "kind": "morph"})
    aa, ab = sts[-1]
    rings.append({"aa": aa, "ab": ab, "oa": PLATE / 2.0, "ob": PLATE / 2.0,
                  "kind": "collar"})
    return rings


def check(sts, rings):
    notes, fails = [], []

    walls = [(r[o] - r[a], n, d) for r in rings
             for o, a, d in (("oa", "aa", "across"), ("ob", "ab", "along"))
             for n in [r["kind"]]]
    thin = min(walls)
    if thin[0] < MINWALL - 1e-9:
        fails.append(f"wall falls to {thin[0]:.2f}mm {thin[2]}, under the "
                     f"{MINWALL:g}mm minimum")
    notes.append(f"wall        {thin[0]:.2f}-{max(walls)[0]:.2f}mm, thinnest "
                 f"{thin[2]} on the {thin[1]} ring")

    seats = [min(a["oa"] - b["aa"], a["ob"] - b["ab"])
             for a, b in zip(rings, rings[1:])]
    if seats and min(seats) < LAP - 1e-9:
        fails.append(f"a joint seats on {min(seats):.2f}mm, under the {LAP:g}mm lap")
    notes.append(f"seat        {min(seats):.2f}-{max(seats):.2f}mm per side over "
                 f"{len(seats)} joint(s)")

    areas = [4.0 * a * b for a, b in sts]
    notes.append(f"area        {areas[0]:.1f} to {areas[-1]:.1f}mm2, "
                 f"{min(areas):.1f} at its narrowest")
    if min(areas) < min(areas[0], areas[-1]) - 1e-9:
        fails.append(f"the airway pinches to {min(areas):.1f}mm2, under both ends")

    # A step inward is a lip for the air to catch on, and the schedule can produce
    # one in ONE direction while the area still rises - which is exactly what the
    # area law buys and exactly what nobody would look for.
    lips = sum(1 for p, q in zip(sts, sts[1:]) if q[0] < p[0] - 1e-9)
    notes.append(f"across      {2*sts[0][0]:.2f} to {2*sts[-1][0]:.2f}mm, "
                 f"opening at every station" if not lips else
                 f"across      steps inward at {lips} station(s)")
    if lips:
        fails.append(f"the section steps inward across the band at {lips} station(s)")

    # The long axis MUST close, and does so by design; it is reported, not checked.
    notes.append(f"along       {2*sts[0][1]:.2f} to {2*sts[-1][1]:.2f}mm, closing "
                 f"{2*(sts[0][1]-sts[-1][1]):.2f}mm over {len(rings)-1} ring(s)")

    top = rings[-1]
    if abs(2 * top["aa"] - BORE) > 1e-9 or abs(2 * top["oa"] - PLATE) > 1e-9:
        fails.append(f"the collar is {2*top['aa']:.2f} in {2*top['oa']:.2f}, not the "
                     f"{BORE:g} in {PLATE:g} the bell's ring 0 seats on")
    notes.append(f"collar      ø{2*top['aa']:.0f} aperture in a ø{2*top['oa']:.0f} "
                 f"face - the bore's own end, so the bell stacks unchanged")

    foot = (2 * rings[0]["oa"], 2 * rings[0]["ob"])
    notes.append(f"footprint   {foot[0]:.2f} x {foot[1]:.2f}mm on the cheek, around a "
                 f"{PORT_ACROSS:g} x {PORT_ALONG:g} port")
    return notes, fails


def rect(cx, cy, a, b):
    return (f"M {cx-a:.4f},{cy-b:.4f} H {cx+a:.4f} V {cy+b:.4f} "
            f"H {cx-a:.4f} Z")


def layout(rings):
    """In a row, port first, so the sheet reads in the order the rings stack."""
    GAP = M = 3.0
    placed, x = [], M
    rowh = max(2 * r["ob"] for r in rings)
    for r in rings:
        w = 2 * r["oa"]
        placed.append((r, x + w / 2.0, M + rowh / 2.0))
        x += w + GAP
    return placed, x - GAP + M, rowh + 2 * M


def emit(sts, rings, path):
    placed, W, H = layout(rings)
    holes = [f'  <path d="{rect(cx, cy, r["aa"], r["ab"])}"/>' for r, cx, cy in placed]
    lines = [f'  <path d="{rect(cx, cy, r["oa"], r["ob"])}"/>' for r, cx, cy in placed]
    tall = len(rings) * STEP
    pathlib.Path(path).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
        f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
        f'  <title>Port-to-bell adapter - {len(rings)} rings of {PLIES} ply, {tall:g}mm, '
        f'{PORT_ACROSS:g} x {PORT_ALONG:g} port to a {BORE:g}mm square throat</title>\n'
        f'  <desc>1 user unit = 1mm. The section runs from the cheek\'s '
        f'{PORT_ACROSS:g} x {PORT_ALONG:g} port to the bore\'s {BORE:g}mm square, '
        f'holding area on a smoothstep so both ends are tangent. The last ring is a '
        f'collar, {BORE:g}mm square in a {PLATE:g}mm square face: that is the bore\'s '
        f'own end face, so the bell stacks on it unchanged. Each ring is {PLIES} '
        f'lamination(s) of {RISE:g}mm ply, {STEP:g}mm of rise, seating on the ring '
        f'below over {LAP:g}mm per side. Rings stack; they do not telescope. No kerf '
        f'compensation: these are glued, as bell.py\'s are. The wide side of each '
        f'ring lies ALONG the duct; the narrow side goes across the cheek\'s band. '
        f'orange #ff8000 cuts the apertures first, black #000000 frees the rings.</desc>\n'
        + '  <g fill="none" stroke="#ff8000" stroke-width="0.1">\n'
        + "\n".join(holes) + "\n  </g>\n"
        + '  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
        + "\n".join(lines) + "\n  </g>\n</svg>\n")
    return W, H


# The long axis has (PORT_ALONG - BORE)/2 to lose and may lose LAP - MINWALL a ring.
FLOOR = max(3, math.ceil(((PORT_ALONG - BORE) / 2.0) / (LAP - MINWALL)))
m = int(opts.get("rings", 6))
if m < FLOOR:
    sys.exit(f"error: --rings={m} cannot hold the wall. The long axis closes "
             f"{(PORT_ALONG - BORE) / 2.0:g}mm and may close {LAP - MINWALL:g}mm a "
             f"ring, so {FLOOR} is the floor.")

sts = stations(m)
rings = rings_from(sts)
notes, fails = check(sts, rings)

name = (f"bell-adapter-port{PORT_ACROSS:g}x{PORT_ALONG:g}-to-bore{BORE:g}"
        f"-{len(rings)}rings-x{PLIES}-cut-files.svg")
if "out" in opts:
    path = opts["out"]
else:
    d = pathlib.Path(__file__).resolve().parent / "cut-files"
    d.mkdir(exist_ok=True)
    path = os.path.relpath(d / name, os.getcwd())

print(f"\n  port-to-bell adapter, {PORT_ACROSS:g} x {PORT_ALONG:g} to "
      f"{BORE:g} x {BORE:g}")
print(f"  {len(rings)} rings of {PLIES} ply, {len(rings)*STEP:g}mm tall\n")
print(f"  {'ring':>4}  {'across':>14}  {'along':>14}  {'area':>8}")
for k, r in enumerate(rings):
    print(f"  {k:>4}  {2*r['aa']:6.2f} in {2*r['oa']:5.2f}  "
          f"{2*r['ab']:6.2f} in {2*r['ob']:5.2f}  {4*r['aa']*r['ab']:7.1f}mm2"
          + ("   the collar the bell sits on" if r["kind"] == "collar" else ""))
print()
for n in notes:
    print(f"  {n}")
print()
if fails:
    for f in fails:
        print(f"  FAIL  {f}")
    print(f"\n  {len(fails)} check(s) failed. Nothing written.")
    sys.exit(1)
W, H = emit(sts, rings, path)
print(f"  wrote {path}  ({W:.0f} x {H:.0f}mm)")
