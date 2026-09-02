#!/usr/bin/env python3
"""Draw a bell as it looks built: rings stacked, seen from above the rim.

    python3 bell-view.py bell-square25-204mm-17rings-x4-rim129-cut-files.svg [OUT.svg]
    python3 bell-view.py bell-round25-204mm-17rings-x4-rim145-cut-files.svg

The section drawings show the profile; this shows the object. Isometric, throat at the
back, rim at the front, drawn back to front so each ring occludes the one behind and you
look down into the bore. Ring sizes come out of the cut file, so the picture cannot drift
from what gets cut.

IT USED TO DRAW EVERY RING AS A SQUARE. That was true of the four bells bell.py makes and
of nothing else, so pointed at bell-round.py's square-to-round sheets it drew a confident
picture of the wrong object -- a round bell with a square rim. The corner radius is sitting
right there in the cut file, as the radius of the arcs the corners are drawn with, so it is
read alongside the width: no arcs means a sharp square, an arc radius equal to the
half-width means a true circle, and anything between is the rounded square it says it is.

A rounded square has no creases -- its arcs meet its flats tangentially -- so the outside
of a ring is drawn as one band rather than as separate faces, and the vertical corner edge
is drawn only where there really is a corner.

DISPLAY ONLY. Not a cut file.
"""
import sys, re, math, pathlib

C30, S30 = math.cos(math.radians(30)), math.sin(math.radians(30))
def iso(X, Y, Z):
    return ((X - Y)*C30, (X + Y)*S30 - Z)

def sections(d):
    """(width, corner radius) for each subpath, whatever commands drew it.

    The corner radius is the arc radius: 0 where the corners are drawn with H and V, and
    half the width where the whole outline is arcs and the ring is a circle."""
    toks = re.findall(r'[MmLlHhVvZzCcSsQqAa]|-?\d*\.?\d+(?:[eE]-?\d+)?', d)
    i = 0; x = y = 0.0; cmd = None; subs = []; cur = None; arcs = []; arc = None
    def n():
        nonlocal i
        v = float(toks[i]); i += 1; return v
    while i < len(toks):
        if re.fullmatch(r'[A-Za-z]', toks[i]): cmd = toks[i]; i += 1; continue
        if cmd is None: i += 1; continue
        rel = cmd.islower(); c = cmd.upper()
        if c == 'M':
            dx,dy=n(),n(); x,y=(x+dx,y+dy) if rel else (dx,dy)
            cur=[x]; subs.append(cur); arc=[0.0]; arcs.append(arc); cmd='l' if rel else 'L'
        elif c == 'L': dx,dy=n(),n(); x,y=(x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        elif c == 'H': v=n(); x=x+v if rel else v; cur.append(x)
        elif c == 'V': v=n(); y=y+v if rel else v; cur.append(x)
        elif c == 'C': n();n();n();n(); dx,dy=n(),n(); x,y=(x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        elif c in ('S','Q'): n();n(); dx,dy=n(),n(); x,y=(x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        elif c == 'A':
            rx=n(); n();n();n();n(); dx,dy=n(),n(); x,y=(x+dx,y+dy) if rel else (dx,dy)
            cur.append(x); arc.append(abs(rx))
        else: i += 1
    return [(max(v)-min(v), max(a)) for v, a in zip(subs, arcs) if len(v) > 1]

def outline(h, c, per=14):
    """A rounded square of half-width h and corner radius c, counter-clockwise.

    c = 0 gives the four corners of a square and nothing else; c = h gives a circle. The
    arcs are tangent to the flats at both ends, so no crease is introduced anywhere the
    file does not have one."""
    c = min(max(c, 0.0), h)
    s, pts = h - c, []
    for (cx, cy), a0 in (((s,-s), -90), ((s,s), 0), ((-s,s), 90), ((-s,-s), 180)):
        if c <= 1e-9:
            pts.append((cx, cy))
        else:
            pts += [(cx + c*math.cos(math.radians(a0 + 90.0*k/per)),
                     cy + c*math.sin(math.radians(a0 + 90.0*k/per))) for k in range(per + 1)]
    return pts

def facing(pts):
    """The run of the outline that faces the viewer, as an ordered list of points.

    The outward normal of a counter-clockwise edge (dx, dy) is (dy, -dx), and the camera
    looks down -(1,1), so an edge shows when dy > dx. On a convex ring those edges form
    one unbroken run, which is the silhouette band."""
    n = len(pts)
    vis = []
    for i in range(n):
        (ax, ay), (bx, by) = pts[i], pts[(i+1) % n]
        vis.append((by - ay) - (bx - ax) > 0)
    if all(vis):
        return pts + [pts[0]]
    start = next(i for i in range(n) if vis[i] and not vis[i-1])
    run = [pts[start]]
    i = start
    while vis[i % n]:
        i += 1
        run.append(pts[i % n])
    return run

def poly(points):
    return "".join(f"{'M' if i == 0 else 'L'}{q[0]:.2f},{q[1]:.2f}"
                   for i, q in enumerate(points)) + "Z"

src = sys.argv[1]
s = pathlib.Path(src).read_text()
rings = []
for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', s):
    w = sections(d)
    if len(w) == 2: rings.append(tuple(sorted(w)))     # (aperture, outer), each (width, corner)
rings.sort()
assert rings, f"no rings in {src}"
m = re.search(r'([\d.]+)mm of rise', s)
rise = float(m.group(1)) if m else (1.5 if len(rings) == 5 else 3.0)

TOP, SIDE, EDGE, BG = "#d8b445", "#9c7c1c", "#5e4a10", "#eef2f8"
faces = []
z = 0.0
for (aw, ac), (ow, oc) in rings:
    ho, hoc = ow/2.0, oc
    ha, hac = aw/2.0, ac
    out_pts, ap_pts = outline(ho, hoc), outline(ha, hac)

    # the outside of the ring, as one band: the facing run at the bottom, back along the top
    band = facing(out_pts)
    faces.append((z, poly([iso(px, py, z) for px, py in band]
                          + [iso(px, py, z + rise) for px, py in reversed(band)]), SIDE))

    # a sharp corner is a real edge and gets a line; a rounded one is not and does not
    if hoc <= 1e-9:
        a, b = iso(ho, ho, z), iso(ho, ho, z + rise)
        faces.append((z + 0.0005, f"M{a[0]:.2f},{a[1]:.2f}L{b[0]:.2f},{b[1]:.2f}", None))

    # top annulus: the outer outline with the aperture as a hole
    faces.append((z + 0.001, poly([iso(px, py, z + rise) for px, py in out_pts])
                           + poly([iso(px, py, z + rise) for px, py in ap_pts]), TOP))
    z += rise
faces.sort(key=lambda f: f[0])

pts = [(float(x), float(y)) for x, y in re.findall(r'([-\d.]+),([-\d.]+)', "".join(f[1] for f in faces))]
x0 = min(p[0] for p in pts); x1 = max(p[0] for p in pts)
y0 = min(p[1] for p in pts); y1 = max(p[1] for p in pts)
M = 10.0
W, H = x1-x0+2*M, y1-y0+2*M
body = "\n".join(
    f'    <path d="{d}" fill="none" stroke="{EDGE}" stroke-width="0.4"/>' if c is None else
    f'    <path d="{d}" fill="{c}" fill-rule="evenodd" stroke="{EDGE}" stroke-width="0.25"/>'
    for _, d, c in faces)

# The label has to describe the object in the file, not the object this script was first
# written for. A ring is square when its corners have no radius and round when that radius
# is half its width; a bell can start as one and end as the other.
def shape(w, c):
    return "square" if c <= 1e-9 else ("round" if abs(c - w/2.0) < 0.01 else "rounded-square")
first, last = shape(*rings[0][0]), shape(*rings[-1][1])   # the bore's throat, the rim's edge
what = (f"{first} rings stacked" if first == last else
        f"rings stacked and morphing from {first} at the throat to {last} at the rim")

dst = sys.argv[2] if len(sys.argv) > 2 else pathlib.Path(src).stem + "-view.svg"
pathlib.Path(dst).write_text(
    f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.1f}mm" height="{H:.1f}mm"\n'
    f'     viewBox="{x0-M:.1f} {y0-M:.1f} {W:.1f} {H:.1f}" role="img"\n'
    f'     aria-label="A {len(rings)}-ring bell seen from above the rim: {what}, '
    f'widening toward the viewer, the bore visible down the middle">\n'
    f'  <!-- DISPLAY ONLY - not a cut file. Generated by bell-view.py from {pathlib.Path(src).name}. -->\n'
    f'  <rect x="{x0-M:.1f}" y="{y0-M:.1f}" width="{W:.1f}" height="{H:.1f}" fill="{BG}"/>\n'
    f'{body}\n</svg>\n')
print(f"  {dst}: {len(rings)} rings, {first} throat to {last} rim, {W:.0f} x {H:.0f}mm")
