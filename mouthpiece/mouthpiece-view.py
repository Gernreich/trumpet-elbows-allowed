#!/usr/bin/env python3
"""Draw the mouthpiece as built: rings stacked, seen from above the cup.

    python3 mouthpiece-view.py [OUT.svg]

bell-view.py draws square section and would render these as squares. These rings are
round -- read from the arc radii in the cut file -- except station one, which is the 31mm
square that meets the elbow's end face. Circles are sampled and projected, so a circle in
plan becomes a true ellipse in isometric rather than an approximation of one.

DISPLAY ONLY. Not a cut file.
"""
import sys, re, math, pathlib

RISE, C30, S30 = 3.0, math.cos(math.radians(30)), math.sin(math.radians(30))
iso = lambda X, Y, Z: ((X - Y) * C30, (X + Y) * S30 - Z)

# Order by position on the sheet, which is assembly order, NOT by size. Sorting by
# diameter interleaves the cup and the backbore, because both pass through the same
# diameters on the way down and back up -- it drew a plausible object that was not this
# mouthpiece.
src = pathlib.Path("mouthpiece-parts.svg").read_text()
rings = []
for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', src):
    r = sorted({round(float(v) * 2, 3) for v in re.findall(r'[Aa]\s*([\d.]+),', d)}, reverse=True)
    x = float(re.search(r'[Mm]\s*(-?[\d.]+)', d).group(1))
    if len(r) == 2:
        rings.append((x, r[1], r[0], True))                    # x, aperture, outer, round
    elif len(r) == 1:
        rings.append((x, r[0], 31.0, False))                   # the square plate and its bore
assert len(rings) == 23, len(rings)
rings.sort()
rings = [(a, o, rnd) for _, a, o, rnd in rings]

def ring_pts(a, o, round_, z):
    """outer boundary then aperture, each as projected points"""
    out = []
    for dia, N in ((o, 96), (a, 96)):
        r = dia / 2.0
        if round_ or dia == a:
            out.append([iso(r*math.cos(2*math.pi*i/N), r*math.sin(2*math.pi*i/N), z) for i in range(N)])
        else:
            out.append([iso(sx*r, sy*r, z) for sx, sy in ((-1,-1), (1,-1), (1,1), (-1,1))])
    return out

def wall(edge, z, outward):
    """The visible half of one cylindrical or prismatic wall, as one path.

    Two rules, and the render was wrong until both held.

    **Cull the faces you cannot see.** A wall is a closed surface; from above you see
    the near half of an outer wall and the far half of a bore. Drawing all of it puts
    the near half of the bore on top of the view down the bore.

    **Fill nonzero, never evenodd.** Under evenodd, two quads that overlap in
    projection cancel to a hole, and the outer wall and the bore of the same ring
    overlap wherever the aperture is wide -- which is every ring in the backbore. That
    is what opened the white stripes across the base tiers, the hatching inside the
    cup, and the sliver through the corner of the square plate. Nothing was missing
    from the geometry; the fill rule was subtracting it. Under nonzero, abutting
    subpaths of one path fill as their union with no seam between them, so the wall
    comes out solid without a stroke to hide the joins.
    """
    subs = []
    for i in range(len(edge)):
        (ax, ay), (bx, by) = edge[i], edge[(i + 1) % len(edge)]
        # outward normal of a CCW edge; the viewer is at +X +Y +Z
        nx, ny = (by - ay), -(bx - ax)
        if ((nx + ny) if outward else -(nx + ny)) <= 0:
            continue
        q = [iso(ax, ay, z), iso(bx, by, z), iso(bx, by, z + RISE), iso(ax, ay, z + RISE)]
        area = sum(q[j][0]*q[(j+1) % 4][1] - q[(j+1) % 4][0]*q[j][1] for j in range(4))
        if area < 0:                              # one winding, so nonzero cannot cancel
            q.reverse()
        subs.append("M" + "L".join(f"{p[0]:.2f},{p[1]:.2f}" for p in q) + "Z")
    return " ".join(subs)


faces, z = [], 0.0
TOP, SIDE, EDGE, BG = "#d8b445", "#9c7c1c", "#5e4a10", "#eef2f8"
N = 96
for a, o, rnd in rings:
    r = o / 2.0
    if rnd:
        edge = [(r*math.cos(2*math.pi*i/N), r*math.sin(2*math.pi*i/N)) for i in range(N)]
    else:
        # Station one is a square plate. Its wall was drawn as a cylinder like every other
        # ring, so a round skirt appeared under a square top and the base read as a
        # separate part sitting below the stack. A square plate has four flat sides.
        edge = [(-r,-r), (r,-r), (r,r), (-r,r)]
    ri = a / 2.0
    bore = [(ri*math.cos(2*math.pi*i/N), ri*math.sin(2*math.pi*i/N)) for i in range(N)]
    # the bore first: its far half is what you see down the hole, and the near half of
    # the outer wall is drawn over the top of it
    faces.append((z, 0, wall(bore, z, False), SIDE))
    faces.append((z, 1, wall(edge, z, True), SIDE))
    outer, inner = ring_pts(a, o, rnd, z + RISE)
    dd = ("M" + "L".join(f"{q[0]:.2f},{q[1]:.2f}" for q in outer) + "Z"
          "M" + "L".join(f"{q[0]:.2f},{q[1]:.2f}" for q in inner) + "Z")
    faces.append((z, 2, dd, TOP))
    z += RISE
faces.sort(key=lambda f: (f[0], f[1]))

pts = [(float(x), float(y)) for x, y in re.findall(r'(-?[\d.]+),(-?[\d.]+)', " ".join(f[2] for f in faces))]
x0, x1 = min(p[0] for p in pts), max(p[0] for p in pts)
y0, y1 = min(p[1] for p in pts), max(p[1] for p in pts)
M = 8.0
W, H = x1-x0+2*M, y1-y0+2*M
# A wall is a run of sampled quads; stroking each one drew a hairline per sample and the
# walls read as corrugated metal. Filled nonzero they need no outline at all -- abutting
# subpaths of one path fill as their union. The flat faces keep a stroke, where it
# separates one ring from the next, and the annulus keeps evenodd, where the aperture is
# nested inside the rim and evenodd is what punches it out.
body = "\n".join(
    f'    <path d="{d}" fill="{c}" '
    + ('fill-rule="nonzero"/>' if c == SIDE
       else f'fill-rule="evenodd" stroke="{EDGE}" stroke-width="0.2"/>')
    for _, _, d, c in faces)
out = sys.argv[1] if len(sys.argv) > 1 else "mouthpiece-view.svg"
pathlib.Path(out).write_text(
    f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.1f}mm" height="{H:.1f}mm"\n'
    f'     viewBox="{x0-M:.1f} {y0-M:.1f} {W:.1f} {H:.1f}" role="img"\n'
    f'     aria-label="The mouthpiece assembled from 23 rings, seen from above the cup: '
    f'the cup narrowing to a throat, then the backbore opening slowly below it">\n'
    f'  <!-- DISPLAY ONLY - not a cut file. Generated by mouthpiece-view.py. -->\n'
    f'  <rect x="{x0-M:.1f}" y="{y0-M:.1f}" width="{W:.1f}" height="{H:.1f}" fill="{BG}"/>\n'
    f'{body}\n</svg>\n')
throat = min(a for a, _, _ in rings)
print(f"  {out}: {len(rings)} rings, {rings[0][1]:.0f}mm plate at the instrument to "
      f"ø{rings[-1][0]:.2f} at the lip, ø{throat:.2f} throat, {W:.0f} x {H:.0f}mm")
