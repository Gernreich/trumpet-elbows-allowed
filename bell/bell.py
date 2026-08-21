#!/usr/bin/env python3
"""Generate a trumpet-profile bell as stacking rings, at a chosen ring budget.

    python3 bell.py                 # all four
    python3 bell.py 20              # one, at most 20 rings

A trumpet bell is not a cone. Its radius follows roughly a Bessel profile,
r = b (u + u0)^-gamma with u measured back from the rim and gamma near 0.7, so the wall
runs nearly parallel to the axis at the throat and turns sharply at the rim.

The ply is 3mm, so a ring rises 3mm. Fewer rings therefore means each ring is several
identical laminations stacked -- PLIES below -- which coarsens the steps. It does not hold
the length fixed: a whole number of rings at each rise lands somewhere slightly different,
so the four come out 210, 210, 204 and 201mm long. Ten rings of seven plies is the same
profile as 67 rings of one, read at lower resolution and cut off at a different place.

WALL is the radius gained plus LAP. Keeping them separate is what allows a shallow flare:
if the wall had to equal the gain, a 2mm minimum ring would force a 34 degree minimum
angle, and a trumpet spends most of its bell below that.
"""
import sys, math, pathlib

RISE, LAP, MINWALL = 3.0, 1.5, 2.0
GAMMA, RT, RIM, L  = 0.7, 15.5, 61.5, 201.0     # ø31 throat, trumpet-scale rim, 201mm long
U0 = L / ((RIM/RT)**(1/GAMMA) - 1)
b  = RT*(L+U0)**GAMMA
rad = lambda z: b*((L-z)+U0)**(-GAMMA)

def rings(plies):
    step = RISE*plies
    r, z, out = RT, 0.0, []
    while z < L - 1e-9:
        g = max(rad(min(z+step, L)) - rad(z), MINWALL-LAP)
        out.append((2*r, 2*(r+g+LAP), g+LAP, math.degrees(math.atan(g/step))))
        r += g; z += step
    return out, step

def sq(cx, cy, s):
    h = s/2.0
    return f"M {cx-h:.4f},{cy-h:.4f} H {cx+h:.4f} V {cy+h:.4f} H {cx-h:.4f} Z"

def emit(rs, plies, step, path):
    GAP = M = 3.0
    per = max(1, int(math.ceil(math.sqrt(len(rs)))))
    lines, y, W = [], M, 0.0
    for k in range(0, len(rs), per):
        chunk = rs[k:k+per]; x = M; rowh = max(o for _, o, _, _ in chunk)
        for a, o, _, _ in chunk:
            cx = x + o/2.0
            lines.append(f'  <path d="{sq(cx, y+rowh/2, o)} {sq(cx, y+rowh/2, a)}"/>')
            x += o + GAP
        W = max(W, x - GAP + M); y += rowh + GAP
    H = y - GAP + M
    pathlib.Path(path).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
        f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
        f'  <title>Trumpet-profile bell - {len(rs)} rings of {plies} ply, '
        f'ø{rs[0][0]:.0f} to ø{rs[-1][1]:.1f}mm</title>\n'
        f'  <desc>1 user unit = 1mm. Bessel horn, gamma {GAMMA}, throat ø{2*RT:.0f} to rim '
        f'ø{rs[-1][1]:.1f}mm over {len(rs)*step:.0f}mm. Each ring is {plies} lamination(s) of '
        f'{RISE:g}mm ply, {step:g}mm of rise, lapping the ring below by {LAP}mm.</desc>\n'
        f'  <g fill="none" stroke="#000000" stroke-width="0.1">\n' + "\n".join(lines) + "\n  </g>\n</svg>\n")
    return W, H

budgets = [int(sys.argv[1])] if len(sys.argv) > 1 else [67, 20, 15, 10]
for want in budgets:
    plies = 1
    while True:
        rs, step = rings(plies)
        if len(rs) <= want: break
        plies += 1
    W, H = emit(rs, plies, step, f"bell-trumpet-{len(rs)}rings.svg")
    print(f"  bell-trumpet-{len(rs)}rings.svg  {len(rs):>2} rings x {plies} ply ({step:g}mm rise)  "
          f"ø{rs[0][0]:.0f}->ø{rs[-1][1]:.1f}mm  {rs[0][3]:.1f}°->{rs[-1][3]:.1f}°  "
          f"wall {min(r[2] for r in rs):.1f}-{max(r[2] for r in rs):.1f}mm  sheet {W:.0f}x{H:.0f}mm")
