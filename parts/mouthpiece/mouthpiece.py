#!/usr/bin/env python3
"""Generate the mouthpiece sheet: cup, throat, backbore, as stacking rings.

    python3 mouthpiece.py [OUT.svg]

The profile is a list of apertures in assembly order. Every ring is one path with two
concentric subpaths -- outer then aperture -- so the pairing is stated in the file rather
than inferred from two circles that happen to sit near each other.

Station one is square, at PLATE mm, matching the elbow's closing face so the mouthpiece
meets the bore flat. The rest are round.

WALL is the ring width. Against a given aperture step it fixes the seat: a ring rests on
the one below over an annulus of (WALL - step/2) per side, so the 1mm cup step gives 2.50mm
and the 0.4mm backbore step gives 2.80mm. Nothing may drop through the ring beneath it,
which this checks before writing.
"""
import sys, math, pathlib, subprocess

WALL   = 3.0        # ring width, mm — also the ply thickness, so a ring is as thick as it is wide
PLATE  = 16.0       # station one, square, matching the elbow's closing face
BORE   = 10.0       # the square bore this meets
THROAT = 3.66       # a #27 drill, the standard trumpet mouthpiece throat
GAP    = 2.0        # spacing between parts on the sheet
MARGIN = 3.0

NECK   = 5.0        # the waist the cup runs down to before the backbore opens again
NCUP   = 6          # stations in the cup, station one included

# Derived from BORE rather than typed, so the cup starts where the bore ends. Typing it
# is how station one came to be wider than its own plate: the aperture stayed at a bore
# this no longer cuts while PLATE followed the new one, and the seat check -- correctly --
# refused to write, with "station 2 falls through station 1".
cup      = [round(BORE - (BORE - NECK) * i / (NCUP - 1), 3) for i in range(NCUP)]
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
parts, holes, x = [], [], MARGIN
for i, (o, a) in enumerate(zip(outers, profile)):
    cx = x + o / 2.0
    outer = square(cx, cy, o) if i == 0 else circle(cx, cy, o)
    # aperture and outline in separate stages -- holes before rims
    holes.append(f'  <path d="{circle(cx, cy, a)}"/>')
    parts.append(f'  <path d="{outer}"/>')
    x += o + GAP

svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
       f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
       f'  <title>Mouthpiece - {len(profile)} rings, aperture '
       f'{profile[0]:g} to {profile[len(cup)-1]:g} to {profile[-1]:g}mm</title>\n'
       f'  <desc>1 user unit = 1mm. {len(cup)} cup rings in 4mm steps, then {len(backbore)} '
       f'backbore rings in 0.40mm steps. Wall {WALL:g}mm. Stack {len(profile)*WALL:g}mm at '
       f'{WALL:g}mm a layer.</desc>\n'
       f'  <g fill="none" stroke="#ff8000" stroke-width="0.1">\n'
       + "\n".join(holes) + "\n  </g>\n"
       f'  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
       + "\n".join(parts) + "\n  </g>\n</svg>\n")

# This script took sys.argv[1] as the output path with no option parsing at all, so
# --numbers=no would have been written to a file of that name. Options are separated now.
_opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in _opts:
    if k != "numbers":
        sys.exit(f"unknown option --{k}: numbers")
NUMBERS = _opts.get("numbers", "yes")
if NUMBERS not in ("yes", "no"):
    sys.exit(f"--numbers: yes or no, not {NUMBERS!r}")
out = next((a for a in sys.argv[1:] if not a.startswith("--")),
           "mouthpiece-parts-cut-files.svg")
open(out, "w").write(svg)

# --order=document: this profile narrows to the throat and opens again, so it passes
# through the same diameters twice and sorting the rings by size would interleave the two
# halves. A sheet that cannot be numbered is removed rather than left on disk to be cut.
if NUMBERS == "yes":
    _tool = pathlib.Path(__file__).resolve().parent.parent / "bell" / "number_rings.py"
    _r = subprocess.run([sys.executable, str(_tool), out, "--order=document"],
                        capture_output=True, text=True)
    if _r.returncode:
        pathlib.Path(out).unlink(missing_ok=True)
        sys.exit(f"  {out} NOT written: the rings could not be numbered.\n"
                 f"{_r.stdout}{_r.stderr}"
                 f"  Fix that, or pass --numbers=no if you meant a bare sheet.")

print(f"  {out}: {len(profile)} rings, {W:.1f} x {H:.1f}mm, stack {len(profile)*WALL:g}mm")
print(f"  apertures {profile[0]:g} -> {THROAT:g} -> {profile[-1]:g}mm")
print(f"  numbers   " + (f"{len(profile)} rings engraved, 0 at the bore"
                         if NUMBERS == "yes" else
                         "NONE — --numbers=no; the rings carry nothing"))
