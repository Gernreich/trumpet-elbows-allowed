#!/usr/bin/env python3
"""Generate the mouthpiece sheet: cup, throat, backbore, as stacking rings.

    python3 mouthpiece.py [OUT.svg]

The profile is a list of apertures in assembly order. Every ring is one path with two
concentric subpaths -- outer then aperture -- so the pairing is stated in the file rather
than inferred from two circles that happen to sit near each other.

Station one is square, at PLATE mm, matching the elbow's closing face so the mouthpiece
meets the bore flat. The rest are round.

WALL is the ring width. Against a given aperture step it fixes the seat: a ring rests on
the one below over an annulus of (WALL - step/2) per side, so the 4mm cup step gives 1.00mm
and the 0.4mm backbore step gives 2.80mm. Nothing may drop through the ring beneath it,
which this checks before writing.
"""
import sys, math

WALL   = 3.0        # ring width, mm — also the ply thickness, so a ring is as thick as it is wide
PLATE  = 31.0       # station one, square, matching the elbow's closing face
BORE   = 25.0       # the square bore this meets
THROAT = 3.66       # a #27 drill, the standard trumpet mouthpiece throat
GAP    = 2.0        # spacing between parts on the sheet
MARGIN = 3.0

cup      = [25.0, 21.0, 17.0, 13.0, 9.0, 5.0]          # 4mm steps down to the throat
backbore = [round(THROAT + 0.40 * i, 3) for i in range(17)]   # 17 rings, 0.40mm steps
profile  = cup + backbore                               # apertures, in assembly order

def circle(cx, cy, d):
    r = d / 2.0
    return (f"M {cx+r:.4f},{cy:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx:.4f},{cy+r:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx-r:.4f},{cy:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx:.4f},{cy-r:.4f} "
            f"A {r:.4f},{r:.4f} 0 0 1 {cx+r:.4f},{cy:.4f} Z")

def square(cx, cy, s):
    h = s / 2.0
    return f"M {cx-h:.4f},{cy-h:.4f} H {cx+h:.4f} V {cy+h:.4f} H {cx-h:.4f} Z"

outers = [PLATE] + [a + 2 * WALL for a in profile[1:]]

# every ring must seat on the one below rather than falling through its aperture
for i in range(1, len(profile)):
    lo = max(profile[i], profile[i-1]) / 2.0
    hi = min(outers[i], outers[i-1]) / 2.0
    if outers[i] <= profile[i-1] or hi - lo <= 0:
        sys.exit(f"station {i+1} falls through station {i}")

W = MARGIN * 2 + sum(outers) + GAP * (len(outers) - 1)
H = MARGIN * 2 + max(outers)
cy = H / 2.0
parts, x = [], MARGIN
for i, (o, a) in enumerate(zip(outers, profile)):
    cx = x + o / 2.0
    outer = square(cx, cy, o) if i == 0 else circle(cx, cy, o)
    parts.append(f'  <path d="{outer} {circle(cx, cy, a)}"/>')
    x += o + GAP

svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
       f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
       f'  <title>Mouthpiece - {len(profile)} rings, aperture '
       f'{profile[0]:g} to {profile[len(cup)-1]:g} to {profile[-1]:g}mm</title>\n'
       f'  <desc>1 user unit = 1mm. {len(cup)} cup rings in 4mm steps, then {len(backbore)} '
       f'backbore rings in 0.40mm steps. Wall {WALL:g}mm. Stack {len(profile)*WALL:g}mm at '
       f'{WALL:g}mm a layer.</desc>\n'
       f'  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
       + "\n".join(parts) + "\n  </g>\n</svg>\n")

out = sys.argv[1] if len(sys.argv) > 1 else "mouthpiece-parts.svg"
open(out, "w").write(svg)
print(f"  {out}: {len(profile)} rings, {W:.1f} x {H:.1f}mm, stack {len(profile)*WALL:g}mm")
print(f"  apertures {profile[0]:g} -> {THROAT:g} -> {profile[-1]:g}mm")
