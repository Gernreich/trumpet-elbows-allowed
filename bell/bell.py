#!/usr/bin/env python3
"""Generate a trumpet-profile bell as stacking rings, at a chosen ring budget.

    python3 bell.py                 # all four
    python3 bell.py 20              # one, at most 20 rings
    python3 bell.py --length=100    # a 100mm bell, same throat and rim
    python3 bell.py --length=100 --rim=80    # ... and a narrower rim to suit

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

THE ANGLE REPORTED IS THE STEEPEST RING, NOT THE LAST. The profile is defined over L, but
a whole number of rings at each rise usually overshoots it: the 14-ring bell is 14 x 15 =
210mm against a 201mm curve. A ring past L is still a full `step` tall -- it is `plies`
laminations of 3mm ply like every other -- and simply has less curve left to draw, so it
flares less than the ring below it. The 14-ring's steepest ring is 36.8 degrees and its rim
ring 24.6. Reporting the last one made that bell look like the shallowest of the four when
it is the second steepest, so both are printed and the steepest is the headline.
"""
import sys, math, pathlib, subprocess

RISE, LAP, MINWALL = 3.0, 3.0, 2.0
GAMMA, RT, RIM, L  = 0.7, 12.5, 61.5, 201.0     # 25mm throat = the bore's channel, 201mm
BORE     = 25.0      # the bore's air channel; the horn continues it rather than stepping
PLATE    = 31.0      # the bore's outside. Ring 0 has to cover this whole end face
OVERHANG = 3.0       # and stand proud of it, to glue against and to locate the joint

# A bell cannot be scaled. The throat is ø31 because the bore is 31mm outside, and a ring
# rises 3mm because the ply is 3mm: neither number is ours to halve. What is free is the
# profile, so a smaller bell is a shorter L and whatever rim you want at the end of it, and
# the flare between them steepens to suit. --rim is the bore's diameter AT the rim; the cut
# ring's outer edge lands a wall further out, which is the figure the report prints.
_args = [a for a in sys.argv[1:] if not a.startswith("--")]
_opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in _opts:
    if k not in ("length", "rim", "gamma", "lap", "overhang", "numbers"):
        sys.exit(f"unknown option --{k}: length, rim, gamma, lap, overhang or numbers")
# Numbering is part of writing the sheet. Every sheet this run writes gets its own rings
# numbered from 0 -- each is a complete bell, not a continuation of the one before.
NUMBERS = _opts.get("numbers", "yes")
if NUMBERS not in ("yes", "no"):
    sys.exit(f"--numbers: yes or no, not {NUMBERS!r}")


def number(path, count):
    """Engrave each ring's index, or write nothing at all.

    --order=document: a bell telescopes, so the order it is written in is the order it is
    glued in. A sheet left on disk gets cut, so a sheet that cannot be numbered is removed
    rather than handed over as rings nobody can put in order."""
    if NUMBERS != "yes":
        return "NONE — --numbers=no; the rings carry nothing"
    tool = pathlib.Path(__file__).resolve().parent / "number_rings.py"
    r = subprocess.run([sys.executable, str(tool), path, "--order=document"],
                       capture_output=True, text=True)
    if r.returncode:
        pathlib.Path(path).unlink(missing_ok=True)
        sys.exit(f"  {path} NOT written: the rings could not be numbered.\n"
                 f"{r.stdout}{r.stderr}"
                 f"  Fix that, or pass --numbers=no if you meant a bare sheet.")
    return f"{count} rings engraved, 0 at the bore"

L     = float(_opts.get("length", L))
RIM   = float(_opts.get("rim", RIM*2)) / 2.0
GAMMA = float(_opts.get("gamma", GAMMA))
LAP      = float(_opts.get("lap", LAP))
OVERHANG = float(_opts.get("overhang", OVERHANG))
if L < 2*RISE:  sys.exit(f"--length must be at least two rings, {2*RISE:g}mm")
if RIM <= RT:   sys.exit(f"--rim must open past the ø{2*RT:.0f}mm throat")

U0 = L / ((RIM/RT)**(1/GAMMA) - 1)
b  = RT*(L+U0)**GAMMA
rad = lambda z: b*((L-z)+U0)**(-GAMMA)

# A non-default profile gets its length into the filename. Without that a 100mm bell that
# happened to land on 17 rings would quietly overwrite bell-trumpet-17rings.svg, which is
# hand-nested and hand-labelled and not reproducible from this script.
STEM = "bell-trumpet" if (L, 2*RIM, GAMMA) == (201.0, 123.0, 0.7) else f"bell-trumpet-{L:.0f}mm"

def rings(plies):
    step = RISE*plies
    r, z, out = RT, 0.0, []
    while z < L - 1e-9:
        g = max(rad(min(z+step, L)) - rad(z), MINWALL-LAP)
        out.append((2*r, 2*(r+g+LAP), g+LAP, math.degrees(math.atan(g/step))))
        r += g; z += step

    # Ring 0 is the flange onto the bore, and the one ring whose outer is not set by the
    # profile. The bore ends in a square annulus of ply 3mm wide -- 25mm inside, 31mm out --
    # and the flange has to cover all of it and stand proud, or there is nothing to glue to
    # but the outside of the tube and the joint opens up.
    a0, o0, _, ang = out[0]
    o0 = max(o0, PLATE + 2*OVERHANG)
    out[0] = (a0, o0, (o0 - a0)/2.0, ang)
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

budgets = [int(_args[0])] if _args else [67, 20, 15, 10]
for want in budgets:
    plies = 1
    while True:
        rs, step = rings(plies)
        if len(rs) <= want: break
        plies += 1
    name = f"{STEM}-{len(rs)}rings.svg"
    W, H = emit(rs, plies, step, name)
    numbered = number(name, len(rs))
    angles = [r[3] for r in rs]
    steep = max(angles)
    rim = "" if abs(steep - angles[-1]) < 0.05 else f" (rim ring {angles[-1]:.1f}°)"
    print(f"  {name}  {len(rs):>2} rings x {plies} ply ({step:g}mm rise)  "
          f"ø{rs[0][0]:.0f}->ø{rs[-1][1]:.1f}mm  {min(angles):.1f}°->{steep:.1f}°{rim}  "
          f"wall {min(r[2] for r in rs):.1f}-{max(r[2] for r in rs):.1f}mm  sheet {W:.0f}x{H:.0f}mm")
    print(f"      numbers: {numbered}")

    # A whole number of rings rarely lands on L, and the leftover is a flat collar at the
    # rim: the last ring is a full step tall but only draws what curve was left. At 201mm
    # that is cosmetic. On a short bell the same few millimetres are a large fraction of
    # the profile, and the rim ring can come out nearly cylindrical -- so say so, and say
    # which length would have divided evenly.
    over = len(rs)*step - L
    if over > 0.01:
        near = min((abs(L - m*step), m*step) for m in (len(rs) - 1, len(rs)) if m)[1]
        hint = "" if over <= step/2 else f"   --length={near:g} divides evenly"
        print(f"      {len(rs)*step:g}mm of rings on a {L:g}mm profile: the rim ring is "
              f"{step:g}mm tall and draws {step-over:g}mm of curve{hint}")
