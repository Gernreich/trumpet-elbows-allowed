#!/usr/bin/env python3
"""Draw a bell in axial section, so the inside surface and the outside are both visible.

    python3 bell-section.py cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg [OUT.svg]

A stack of rings is hard to read from the cut sheet: the file is a row of squares and
says nothing about the shape they make. Sliced down the axis, both surfaces show at once
-- the bore climbing one staircase and the outside climbing another, with the wall
between them and the axis for reference. Ring sizes are read out of the cut file, so this
cannot drift from what gets cut.

DISPLAY ONLY. Not a cut file.
"""
import os, sys, re, math, pathlib


def display_out(src, suffix):
    """Where a display drawing goes, given the cut file it was read from.

    Beside the PART, not inside cut-files/ and not beside this script. cut-files/ holds
    cut files and nothing else -- a section drawing sent to a laser is scrap -- so the
    output climbs out of it to the part's own directory. Source-derived rather than
    script-derived because bell-section.py and bell-view.py are read by the mouthpiece
    too, and a script-derived path filed a mouthpiece drawing under bell/.

    The "-cut-files" the source carries is dropped, so
    bell-round10-...-rim86-cut-files.svg gives bell-round10-...-rim86-section.svg and not
    a doubled -cut-files-section.
    """
    stem = pathlib.Path(src).stem
    if stem.endswith("-cut-files"):
        stem = stem[:-len("-cut-files")]
    d = pathlib.Path(src).resolve().parent
    if d.name == "cut-files":
        d = d.parent
    return os.path.relpath(d / (stem + suffix), os.getcwd())

# Every generated bell states its own rise in the file; the mouthpiece does not, and
# is 3mm ply a ring. The table this used to consult held one entry, for a 5-ring
# bell.svg that no longer exists, and nothing left has 5 rings.
DEFAULT_RISE = 3.0
def subpath_widths(d):
    """Width of each subpath, whatever commands drew it."""
    toks = re.findall(r'[MmLlHhVvZzCcSsQqAa]|-?\d*\.?\d+(?:[eE]-?\d+)?', d)
    i = 0; x = y = 0.0; cmd = None; subs = []; cur = None
    def n():
        nonlocal i
        v = float(toks[i]); i += 1; return v
    while i < len(toks):
        if re.fullmatch(r'[A-Za-z]', toks[i]): cmd = toks[i]; i += 1; continue
        if cmd is None: i += 1; continue
        rel = cmd.islower(); c = cmd.upper()
        if c == 'M':
            dx, dy = n(), n(); x, y = (x+dx, y+dy) if rel else (dx, dy)
            cur = [x]; subs.append(cur); cmd = 'l' if rel else 'L'
        elif c == 'L': dx, dy = n(), n(); x, y = (x+dx, y+dy) if rel else (dx, dy); cur.append(x)
        elif c == 'H': v = n(); x = x+v if rel else v; cur.append(x)
        elif c == 'V': v = n(); y = y+v if rel else v; cur.append(x)
        elif c == 'C': n();n();n();n(); dx,dy = n(),n(); x,y = (x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        elif c in ('S','Q'): n();n(); dx,dy = n(),n(); x,y = (x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        elif c == 'A': n();n();n();n();n(); dx,dy = n(),n(); x,y = (x+dx,y+dy) if rel else (dx,dy); cur.append(x)
        else: i += 1
    return [max(v) - min(v) for v in subs if len(v) > 1]

def ring_sizes(path):
    s = pathlib.Path(path).read_text()
    out = []
    # A sheet that cuts its holes before its rims keeps the aperture and the outline in
    # different colours, so a ring is no longer one path with two subpaths. Both groups are
    # written in the same ring order, so pair them by position.
    ap_g = re.search(r'<g[^>]*stroke="#ff8000"[^>]*>(.*?)</g>', s, re.S)
    out_g = re.search(r'<g[^>]*stroke="#000000"[^>]*>(.*?)</g>', s, re.S)
    if ap_g and out_g:
        aps = [subpath_widths(d) for d in re.findall(r'd="([^"]+)"', ap_g.group(1))]
        outs = [subpath_widths(d) for d in re.findall(r'd="([^"]+)"', out_g.group(1))]
        for a, o in zip(aps, outs):
            if len(a) == 1 and len(o) == 1:
                out.append((a[0], o[0]))
        if out:
            out.sort()
            r = re.search(r'([\d.]+)mm of rise', s)
            return out, float(r.group(1)) if r else 3.0

    for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', s):
        # Walk the path. There used to be a shortcut here that read the width straight out
        # of "M x,y H x2", which is the full width of a SQUARE ring and only the flat
        # between the corner arcs of a rounded one. Every square-to-round bell was measured
        # short by two corner radii, and by more as the rings rounded: a 17-ring bell
        # reported a 4.87mm throat where its throat is the whole bore, and its section showed
        # a near-flat taper with the rim floating detached from it.
        e = subpath_widths(d)
        if len(e) == 2:
            out.append((min(e), max(e)))
    out.sort()
    r = re.search(r'([\d.]+)mm of rise', s)
    rise = float(r.group(1)) if r else DEFAULT_RISE
    return out, rise

src = sys.argv[1]
rings, rise = ring_sizes(src)
assert rings, f"no rings found in {src}"
dst = sys.argv[2] if len(sys.argv) > 2 else display_out(src, "-section.svg")
# This draws whatever it is pointed at, and it was pointed at the mouthpiece. Saying
# 'bell' unconditionally made mouthpiece-section.svg announce itself to screen readers
# as a bell, and a hand-edit to the SVG would have been undone by the next run.

L = len(rings) * rise
RMAX = rings[-1][1] / 2
M, SCALE = 14.0, 1.0
W, H = (L + 2*M) * SCALE, (2*RMAX + 2*M) * SCALE
cy = H / 2

WALL, BORE, AXIS = "#c9a227", "#faf7f0", "#8a8f96"
body = []
z = 0.0
for a, o in rings:
    ri, ro = a/2, o/2
    for sgn in (-1, 1):
        y0 = cy + sgn*ri*SCALE
        y1 = cy + sgn*ro*SCALE
        body.append(f'<rect x="{M + z*SCALE:.3f}" y="{min(y0,y1):.3f}" '
                    f'width="{rise*SCALE:.3f}" height="{abs(y1-y0):.3f}" '
                    f'fill="{WALL}" stroke="#6b5416" stroke-width="0.4"/>')
    z += rise

LABEL = (f'Axial section of the {len(rings)}-ring mouthpiece: the bore narrowing to the '
 'throat and opening again, the outside stepping the other way, wall between them'
 if 'mouthpiece' in pathlib.Path(src).stem else
 f'Axial section of a {len(rings)}-ring bell: the bore climbing one staircase and '
 'the outside another, wall between them')

svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.1f}mm" height="{H:.1f}mm"\n'
       f'     viewBox="0 0 {W:.1f} {H:.1f}" role="img"\n'
       f'     aria-label="{LABEL}">\n'
       f'  <!-- DISPLAY ONLY - not a cut file. Generated by bell-section.py from {pathlib.Path(src).name}. -->\n'
       f'  <rect width="{W:.1f}" height="{H:.1f}" fill="{BORE}"/>\n'
       + "\n".join("  " + b for b in body) + "\n"
       f'  <line x1="{M:.1f}" y1="{cy:.1f}" x2="{M + L*SCALE:.1f}" y2="{cy:.1f}" '
       f'stroke="{AXIS}" stroke-width="0.6" stroke-dasharray="6 3"/>\n'
       f'  <text x="{M:.1f}" y="{H - 4:.1f}" font-family="ui-sans-serif,system-ui,sans-serif" '
       f'font-size="7" fill="#334155">{len(rings)} rings · '
       f'ø{rings[0][0]:.0f} bore to ø{rings[-1][1]:.1f} rim · {L:.0f}mm long</text>\n'
       f'</svg>\n')
pathlib.Path(dst).write_text(svg)
print(f"  {dst}: {len(rings)} rings, {rise:g}mm rise, {W:.0f} x {H:.0f}mm")
