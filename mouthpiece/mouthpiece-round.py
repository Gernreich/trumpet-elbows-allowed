#!/usr/bin/env python3
"""Generate a whole mouthpiece: backbore, throat, entrance and cup, in one sheet.

    python3 mouthpiece-round.py [OUT.svg]
    python3 mouthpiece-round.py --taper=2.0     # finer steps down the bore-side run
    python3 mouthpiece-round.py --seat=1.25     # demand more seat, round more slowly
    python3 mouthpiece-round.py --rim=17.0      # a wider rim
    python3 mouthpiece-round.py --bowl=5        # a 15mm cup instead of 12mm

`mouthpiece.py` puts a 25mm ROUND aperture in its 31mm square plate, so the joint that
meets the bore throws away the bore's corners: a 25mm square channel opens into a 25mm
circle and 21% of the area steps out of the airway at station one. This one starts as the
bore actually is -- a sharp 25mm square inside a sharp 31mm plate -- and rounds the corners
away going up, exactly as bell-round.py does going out. Same rounded squares, same exact
offsets, opposite direction.

THE 4mm TAPER HAD TO GO. A ring's outer is its aperture offset by WALL, so the seat the
next ring lands on is WALL minus how much the two apertures differ in that direction. On
the flats that is half the aperture step: the old 4mm steps left 3 - 2 = 1.00mm. Through
the corners of a SHARP square the same step costs step x root2 = 2.83mm, leaving 0.17mm --
and rounding a corner pulls the diagonal in further, which takes it negative. So a square
mouthpiece cannot be built on a 4mm taper at all. At 2.5mm the corner seat is 1.23mm with
room to round. The run is the same 25 -> 5mm cone either way, sampled finer: three more
rings, 9mm more mouthpiece, and no change to the profile the air sees.

ROUNDNESS IS NOT SCHEDULED, IT IS AS FAST AS THE SEAT ALLOWS. Each station takes the
largest corner radius that still leaves MINSEAT in both directions, in closed form. The
part therefore rounds as quickly as its own geometry permits and no quicker -- on the
default profile it reaches a true circle at the 7.5mm station, 24mm up from the bore, and
the throat and the whole cup are round.

THE CUP IS A BOWL, NOT A CONE. This used to stop at ø10.06 opening 0.40mm a ring -- a 3.8
degree half-angle, a tube, against the 16 to 17mm a trumpet rim is across inside. The bowl
is an ellipse arc whose slope is zero at the rim, so the wall stands parallel to the axis
where the lip sits and turns toward the throat going down; its first ring opens 3.21mm and
its rim ring 0.33mm. A cone would meet the rim at an angle and feel like a funnel.

The three runs are named for the anatomy, which `mouthpiece.py` gets backwards:

    BACKBORE   25 -> 5mm, the end that closes onto the bore, square becoming round
    ENTRANCE   the 3.66mm throat, then 0.40mm a ring out to 10.06 -- 48mm of it, which is
               long for a mouthpiece and is where to look first if it plays stuffy
    BOWL       10.06 -> 16.5mm over 12mm, the cup proper, ending at the rim

A sheet from here is the same part as mouthpiece-round-parts.svg plus mouthpiece-cup.py's
four rings, ring for ring, so both routes number identically. `mouthpiece-cup.py` stays for
retrofitting a mouthpiece already glued without a bowl.

The bowl is a staircase of 3mm ply. Sand or fill the steps and round the rim over before
playing it; as cut the rim edge is a square corner and your lip will say so.
"""
import sys, math

WALL    = 3.0        # ring width, and the ply is 3mm so a ring is as thick as it is wide
PLATE   = 31.0       # station one, square, matching the bore's closing face
BORE    = 25.0       # the square bore this meets, corners and all
NECK    = 5.0        # where the backbore run ends and the throat begins
THROAT  = 3.66       # a #27 drill, the standard trumpet mouthpiece throat
TAPER   = 2.5        # aperture step down the backbore run; 4.0 cannot be made square
RIM     = 16.5       # the rim your lip sits on; a trumpet is 16 to 17mm inside
BOWL    = 4          # rings of cup above the entrance: 4 x 3mm = a 12mm bowl
MINSEAT = 1.0        # every joint, flats and corners alike
GAP     = 2.0
MARGIN  = 3.0
DIAG    = math.sqrt(2.0)

opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in opts:
    if k not in ("taper", "seat", "rim", "bowl"):
        sys.exit(f"unknown option --{k}: taper, seat, rim or bowl")
TAPER   = float(opts.get("taper", TAPER))
MINSEAT = float(opts.get("seat", MINSEAT))
RIM     = float(opts.get("rim", RIM))
BOWL    = int(opts.get("bowl", BOWL))
out_path = next((a for a in sys.argv[1:] if not a.startswith("--")), "mouthpiece-round-parts.svg")


def support(h, c, s):
    """How far a rounded square of half-width h and corner radius c reaches in a direction,
    s = |cos| + |sin| of it: 1 straight out through a flat, root2 out through a corner."""
    return (h - c)*s + c


# ── the profile, in assembly order from the bore ─────────────────────────────
steps    = int(round((BORE - NECK)/TAPER))
backbore = [BORE - TAPER*i for i in range(steps + 1)]          # 25 .. 5, the bore end
entrance = [round(THROAT + 0.40*i, 3) for i in range(17)]      # the throat, then 3.66 .. 10.06

# The bowl: an ellipse arc through the (depth, radius) plane, depth measured DOWN from the
# rim. Its slope is -k d / r, which is zero at d = 0, so the wall stands parallel to the
# axis where the lip sits and turns toward the throat going down. A cone would meet the rim
# at an angle and feel like a funnel.
r0, R, D = entrance[-1]/2.0, RIM/2.0, BOWL*WALL
if RIM <= entrance[-1]:
    sys.exit(f"--rim must open past the ø{entrance[-1]:g}mm the entrance reaches")
bowl     = [2.0*math.sqrt(R*R - (R*R - r0*r0)*((D - WALL*(k+1))/D)**2) for k in range(BOWL)]
profile  = backbore + entrance + bowl
hs       = [a/2.0 for a in profile]

# ── roundness: the largest corner radius each station can afford ─────────────
# seat(s) = WALL - |support_i(s) - support_{i-1}(s)|, so keeping MINSEAT bounds how far
# this station's diagonal reach may move from the one below. Within that band we want the
# reach as SMALL as possible, because smaller reach is a rounder corner.
cs = [0.0]                                                     # station one is the bore
for i in range(1, len(profile)):
    h, room = hs[i], WALL - MINSEAT
    prev = support(hs[i-1], cs[i-1], DIAG)
    lo, hi = max(h, prev - room), min(h*DIAG, prev + room)
    if lo > hi + 1e-9:
        sys.exit(f"station {i+1} cannot hold {MINSEAT}mm of seat at any corner radius — "
                 f"try --taper below {TAPER:g} or --seat below {MINSEAT:g}")
    cs.append((h*DIAG - lo)/(DIAG - 1.0))

# ── outers, and the checks, before anything is drawn ─────────────────────────
# Station one's outer is the sharp plate, not the offset: it has to cover the bore's face
# corner to corner. Everywhere else the outer is the aperture offset by WALL, which for a
# rounded square is another rounded square, exactly WALL away in every direction.
OH = [PLATE/2.0] + [h + WALL for h in hs[1:]]
OC = [0.0]       + [c + WALL for c in cs[1:]]

fails, seats = [], []
for i in range(1, len(profile)):
    for s in (1.0, DIAG):
        # both are linear in s, so min-of-two minus max-of-two is concave and its smallest
        # value on [1, root2] is at an end: checking the flats and the corners is enough
        room = (min(support(OH[i], OC[i], s), support(OH[i-1], OC[i-1], s))
                - max(support(hs[i], cs[i], s), support(hs[i-1], cs[i-1], s)))
        seats.append(room)
        if room <= 0:
            fails.append(f"station {i+1} falls through station {i} "
                         f"{'across the flats' if s == 1.0 else 'through the corners'}")
        elif room < MINSEAT - 1e-6:
            fails.append(f"station {i+1} seats on only {room:.2f}mm at station {i}")

def rounded(cx, cy, h, c):
    """c = 0 is a sharp square, c = h is a true circle, and everything between is both."""
    c = min(max(c, 0.0), h)
    if c <= 1e-9:
        return f"M {cx-h:.4f},{cy-h:.4f} H {cx+h:.4f} V {cy+h:.4f} H {cx-h:.4f} Z"
    s = h - c
    arc = lambda x, y: f"A {c:.4f},{c:.4f} 0 0 1 {x:.4f},{y:.4f}"
    run = lambda cmd, v: f"{cmd} {v:.4f} " if s > 1e-9 else ""
    return (f"M {cx-s:.4f},{cy-h:.4f} "
            + run("H", cx+s) + arc(cx+h, cy-s) + " "
            + run("V", cy+s) + arc(cx+s, cy+h) + " "
            + run("H", cx-s) + arc(cx-h, cy+s) + " "
            + run("V", cy-s) + arc(cx-s, cy-h) + " Z")


def layout(per):
    placed, y, W = [], MARGIN, 0.0
    for k in range(0, len(profile), per):
        idx = range(k, min(k + per, len(profile)))
        x, rowh = MARGIN, max(2*OH[i] for i in idx)
        for i in idx:
            placed.append((i, x + OH[i], y + rowh/2.0))
            x += 2*OH[i] + GAP
        W = max(W, x - GAP + MARGIN); y += rowh + GAP
    return placed, W, y - GAP + MARGIN


if fails:
    for f in fails:
        print(f"  x {f}")
    sys.exit(f"  {out_path} not written: {len(fails)} problem(s)")

per = min(range(1, len(profile) + 1),
          key=lambda p: (round(max(layout(p)[1:]), 6), round(layout(p)[1]*layout(p)[2], 6)))
placed, W, H = layout(per)
parts = [f'  <path d="{rounded(cx, cy, OH[i], OC[i])} {rounded(cx, cy, hs[i], cs[i])}"/>'
         for i, cx, cy in placed]

round_at = next((i for i in range(len(profile)) if abs(cs[i] - hs[i]) < 1e-6), None)
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
       f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
       f'  <title>Square-to-round mouthpiece - {len(profile)} rings, {BORE:g}mm square bore '
       f'to {THROAT:g}mm throat to ø{profile[-1]:g}mm lip</title>\n'
       f'  <desc>1 user unit = 1mm. Station one is a sharp {BORE:g}mm square aperture in a '
       f'sharp {PLATE:g}mm square plate, matching the bore corner for corner. The corners '
       f'round away going up and the section is a true circle from the '
       f'{profile[round_at]:g}mm station on, so the {THROAT:g}mm throat and the whole cup '
       f'are round. {len(backbore)} backbore rings in {TAPER:g}mm steps, {len(entrance)} entrance '
       f'rings in 0.40mm steps, then {len(bowl)} bowl rings to a ø{RIM:g}mm rim. Wall {WALL:g}mm, 3mm of rise a ring, stack '
       f'{len(profile)*WALL:g}mm. Every joint seats on at least {min(seats):.2f}mm per side, '
       f'flats and corners alike. Rings stack; they do not telescope.</desc>\n'
       f'  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
       + "\n".join(parts) + "\n  </g>\n</svg>\n")
open(out_path, "w").write(svg)

print(f"  {out_path}: {len(profile)} rings, {W:.1f} x {H:.1f}mm, stack {len(profile)*WALL:g}mm")
print(f"    airway     {BORE:g}mm square -> {THROAT:g} throat -> ø{profile[-1]:g} lip")
print(f"    backbore   {len(backbore)} rings in {TAPER:g}mm steps to the ø{NECK:g} neck")
print(f"    entrance   {len(entrance)} rings, ø{THROAT:g} throat opening to ø{entrance[-1]:g}")
print(f"    bowl       {len(bowl)} rings, ø{entrance[-1]:g} to a ø{RIM:g}mm rim over {D:g}mm")
print(f"    round from station {round_at+1} (ø{profile[round_at]:g}mm), "
      f"{round_at*WALL:g}mm up from the bore")
print(f"    seat       {min(seats):.2f}-{max(seats):.2f}mm per side, flats and corners alike")
print(f"    ok, written")
