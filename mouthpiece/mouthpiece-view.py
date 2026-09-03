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
# Named on the command line, with the 25mm trumpet layout as the default. It
# used to open "mouthpiece-parts-cut-files.svg" by name; that file was renamed
# on 2026-09-03 to carry its bore and layout, and this went on naming a file
# that no longer exists.
# --src names the cut file; a bare argument is still the OUTPUT, which is what
# this script has always taken. Making argv[1] the source instead silently
# turned the output path into the input path, and the run wrote a display SVG
# straight over a cut file.
_s = [a for a in sys.argv[1:] if a.startswith("--src=")]
SRC = _s[0].split("=", 1)[1] if _s else \
      "mouthpiece-bore25-trumpet-parts-cut-files.svg"
sys.argv = [sys.argv[0]] + [a for a in sys.argv[1:] if not a.startswith("--src=")]
src = pathlib.Path(SRC).read_text()

# A RING IS TWO PATHS NOW, NOT ONE. The apertures moved into their own orange
# group on 2026-09-03 so they cut before the outline that frees the part, and
# this expected both radii on one path: it found 0 rings where it wanted 23.
def ends(d):
    """A path's command ENDPOINTS.

    Not every number in the string. An arc reads `A rx,ry rot large,sweep x,y`
    and `large,sweep` looks exactly like a coordinate pair to a regex that
    flattens the numbers - which put an aperture's bounding box at 138mm
    across on a 152mm sheet and left it inside no outline at all. Parse per
    command, or measure with the tool that wrote the file.
    """
    out, x, y = [], 0.0, 0.0
    for cmd, arg in re.findall(r'([MLAZmlaz])([^MLAZmlaz]*)', d):
        n = [float(v) for v in re.findall(r'-?\d*\.?\d+(?:[eE][-+]?\d+)?', arg)]
        u = cmd.isupper()
        c = cmd.upper()
        if c == 'M' or c == 'L':
            for i in range(0, len(n) - 1, 2):
                x, y = (n[i], n[i+1]) if u else (x + n[i], y + n[i+1])
                out.append((x, y))
        elif c == 'A':
            for i in range(0, len(n) - 6, 7):
                ex, ey = n[i+5], n[i+6]
                x, y = (ex, ey) if u else (x + ex, y + ey)
                out.append((x, y))
    return out


def circles(chunk):
    """Each path as (centre x, centre y, half-width, diameters it draws)."""
    out = []
    for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', chunk):
        pts = ends(d)
        if not pts:
            continue
        xs = [q[0] for q in pts]
        ys = [q[1] for q in pts]
        cx, cy = (min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2
        half = max(max(xs) - min(xs), max(ys) - min(ys)) / 2
        r = sorted({round(float(v) * 2, 3)
                    for v in re.findall(r'[Aa]\s*([\d.]+),', d)}, reverse=True)
        out.append((cx, cy, half, r))
    return out


def group(colour):
    m = re.search(r'<g[^>]*stroke="%s"[^>]*>(.*?)</g>' % colour, src, re.S)
    return circles(m.group(1)) if m else []

# Order by position on the sheet, which is assembly order, NOT by size. Sorting by
# diameter interleaves the cup and the backbore, because both pass through the same
# diameters on the way down and back up -- it drew a plausible object that was not this
# mouthpiece.
#
# PAIR BY CONTAINMENT, not by rank. Zipping the two sorted lists assumes an
# aperture and its outline hold the same position in each - true only if the
# rings never overlap in x on the sheet, which on a nested layout they do. It
# drew the cup as a stack of separate bulges: the same failure the paragraph
# above is about, arrived at from the other direction.
aps, outs = group("#ff8000"), group("#000000")
rings = []
if aps:
    # PAIR BY INDEX, IN FILE ORDER. The generator writes the apertures and the
    # outlines from one list, so ring i is aps[i] with outs[i], and that order
    # is assembly order. Sorting by centre x scrambles it - this sheet is six
    # rows deep, so x alone interleaves them and the cup came out as a stack of
    # alternating bulges. Pairing by containment is no better: it is true of a
    # ring nested in another ring's spare space as well as of its own outline.
    assert len(aps) == len(outs), f"{len(aps)} apertures, {len(outs)} outlines in {SRC}"
    for (ax, ay, ah, ar), (ox, oy, oh, orr) in zip(aps, outs):
        assert abs(ax - ox) < 0.5 and abs(ay - oy) < 0.5, \
            f"aperture at ({ax:.1f},{ay:.1f}) is not concentric with its outline"
        # a radius only exists where the path has arcs; station one is the
        # square bore in the square plate, so fall back to what was measured
        rings.append((ar[0] if ar else 2 * ah,
                      orr[0] if orr else 2 * oh,
                      bool(ar)))
else:                                 # a file from before the split
    tmp = []
    for cx, cy, half, r in circles(src):
        if len(r) == 2:
            tmp.append((cx, r[1], r[0], True))
        elif len(r) == 1:
            tmp.append((cx, r[0], 31.0, False))
    tmp.sort()
    rings = [(a, o, rnd) for _, a, o, rnd in tmp]
assert rings, f"no rings in {SRC}"

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
    f'     aria-label="The mouthpiece assembled from {len(rings)} rings, seen from '
    f'above the cup: '
    f'the cup narrowing to a throat, then the backbore opening slowly below it">\n'
    f'  <!-- DISPLAY ONLY - not a cut file. Generated by mouthpiece-view.py. -->\n'
    f'  <rect x="{x0-M:.1f}" y="{y0-M:.1f}" width="{W:.1f}" height="{H:.1f}" fill="{BG}"/>\n'
    f'{body}\n</svg>\n')
throat = min(a for a, _, _ in rings)
print(f"  {out}: {len(rings)} rings, {rings[0][1]:.0f}mm plate at the instrument to "
      f"ø{rings[-1][0]:.2f} at the lip, ø{throat:.2f} throat, {W:.0f} x {H:.0f}mm")
