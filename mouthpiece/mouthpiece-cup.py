#!/usr/bin/env python3
"""Extra rings that turn the end of the mouthpiece into a cup.

    python3 mouthpiece-cup.py [OUT.svg]
    python3 mouthpiece-cup.py --rim=17.0        # a wider rim
    python3 mouthpiece-cup.py --rings=5         # a deeper bowl, 15mm instead of 12
    python3 mouthpiece-cup.py --onto=10.06      # stack onto a different aperture

`mouthpiece-round.py` ends its cup run at ø10.06, opening 0.40mm a ring: a 3.8 degree
half-angle, which is a tube with ambitions. A trumpet rim is 16 to 17mm across inside, so
the bowl is not shallow, it is absent. These rings are what is missing, and they STACK ON
TOP of a mouthpiece that is already glued -- nothing here replaces a part you have cut.

A CUP IS A BOWL, NOT A CONE, and the difference is where the wall stands up. The profile
here is an ellipse arc through the (depth, radius) plane:

    r(d) = sqrt(R^2 - (R^2 - r0^2) (d/D)^2)

with d measured DOWN from the rim, R the rim radius, r0 the aperture it lands on and D the
depth. Its slope dr/dd is -k d / r, which is zero at d = 0: the wall is parallel to the axis
at the rim and turns hard toward the throat as it goes down. That is what your lip sits on.
A straight cone would meet the rim at an angle and feel like a funnel.

THE FIRST RING HAS TO SEAT ON GLUED WORK. Its outer is the aperture below it plus two
walls, so the seat is WALL minus half the aperture step -- the same rule as the rest of the
mouthpiece, but the ring below is one you cannot re-cut. Every joint is checked, the joint
onto the existing stack included, and nothing is written if one is short.

Rings are 3mm ply and the bowl is a staircase of them. Sand or fill the steps and round the
rim over before playing it; the rim edge as cut is a square corner and your lip will say so.
"""
import sys, math

WALL    = 3.0        # ring width, and the ply thickness
RISE    = 3.0        # one ring, one lamination
ONTO    = 10.06      # the aperture this stacks onto: mouthpiece-round.py's last ring
RIM     = 16.5       # rim aperture; a trumpet is 16 to 17mm inside
RINGS   = 4          # 4 x 3mm = a 12mm bowl
MINSEAT = 1.0
GAP     = 2.0
MARGIN  = 3.0

opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in opts:
    if k not in ("rim", "rings", "onto"):
        sys.exit(f"unknown option --{k}: rim, rings or onto")
RIM   = float(opts.get("rim", RIM))
ONTO  = float(opts.get("onto", ONTO))
RINGS = int(opts.get("rings", RINGS))
out_path = next((a for a in sys.argv[1:] if not a.startswith("--")), "mouthpiece-cup-parts.svg")
if RIM <= ONTO:
    sys.exit(f"--rim must open past the ø{ONTO:g}mm it stacks onto")
if RINGS < 1:
    sys.exit("--rings must be at least 1")

r0, R, D = ONTO/2.0, RIM/2.0, RINGS*RISE
depth = lambda k: D - RISE*(k + 1)                       # k = 0 is the first ring above
apertures = [2.0*math.sqrt(R*R - (R*R - r0*r0)*(depth(k)/D)**2) for k in range(RINGS)]
outers = [a + 2*WALL for a in apertures]

# every joint, the one onto the glued stack first
below, seats, fails = ONTO, [], []
for i, a in enumerate(apertures):
    seat = (below + 2*WALL - a)/2.0
    seats.append(seat)
    where = "the glued stack" if i == 0 else f"ring {i}"
    if seat <= 0:
        fails.append(f"ring {i+1} falls through {where}")
    elif seat < MINSEAT - 1e-9:
        fails.append(f"ring {i+1} seats on only {seat:.2f}mm at {where}")
    below = a
if fails:
    for f in fails:
        print(f"  x {f}")
    sys.exit(f"  {out_path} not written: {len(fails)} problem(s) — try --rings above {RINGS}")


def circle(cx, cy, d):
    r = d/2.0
    return (f"M {cx+r:.4f},{cy:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx:.4f},{cy+r:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx-r:.4f},{cy:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx:.4f},{cy-r:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx+r:.4f},{cy:.4f} Z")


W = MARGIN*2 + sum(outers) + GAP*(len(outers) - 1)
H = MARGIN*2 + max(outers)
cy, x, parts = H/2.0, MARGIN, []
for a, o in zip(apertures, outers):
    parts.append(f'  <path d="{circle(x + o/2.0, cy, o)} {circle(x + o/2.0, cy, a)}"/>')
    x += o + GAP

svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
       f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
       f'  <title>Mouthpiece cup extension - {RINGS} rings, ø{ONTO:g} to ø{RIM:g}mm rim</title>\n'
       f'  <desc>1 user unit = 1mm. {RINGS} rings that stack on an existing mouthpiece ending '
       f'at ø{ONTO:g}mm and open it into a {D:g}mm cup with a ø{RIM:g}mm rim. The bowl is an '
       f'ellipse arc, so the wall stands parallel to the axis at the rim and turns toward the '
       f'throat going down. Wall {WALL:g}mm, {RISE:g}mm of rise a ring. Every joint seats on at '
       f'least {min(seats):.2f}mm per side, the joint onto the glued stack included. These are '
       f'ADDITIONAL rings; they replace nothing.</desc>\n'
       f'  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
       + "\n".join(parts) + "\n  </g>\n</svg>\n")
open(out_path, "w").write(svg)

print(f"  {out_path}: {RINGS} rings, {W:.1f} x {H:.1f}mm, adds {D:g}mm of cup")
print(f"    stacks onto ø{ONTO:g} and opens to a ø{RIM:g}mm rim")
half = math.degrees(math.atan(((apertures[0] - ONTO)/2)/RISE))
print(f"    first ring opens {apertures[0]-ONTO:.2f}mm ({half:.0f}° half-angle), "
      f"the rim ring {apertures[-1]-apertures[-2] if RINGS > 1 else 0:.2f}mm — a bowl, not a cone")
for i, (a, o) in enumerate(zip(apertures, outers)):
    print(f"    ring {i+1}  aperture ø{a:6.2f}  outer ø{o:6.2f}  seats {seats[i]:.2f}mm")
