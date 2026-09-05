"""Render a bore written in the agreed notation as a 3D view.

Coloured by piece, which is what you cut, or by direction of travel, which is
what you follow when you are trying to read the shape:

    python3 bore_render.py "W D3 E4 N"
    python3 bore_render.py --directions "W D3 E4 N"
"""
import math, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bore_split import coplanar_pieces, specs_for, walk, DIRS

PAL = ['#2f6f4e', '#b5563a', '#3a5f8a', '#8a6b2f', '#6b3a7a',
       '#2f7a7a', '#7a4a2f', '#4a6b2f']
ELB = '#d94f2b'          # elbows stand out
OPEN = {'IN': '#1c1c20', 'OUT': '#e0457b'}

# one hue per direction of travel. Opposites are far apart in hue so a leg and
# its return leg never read as the same colour.
# Down is grey rather than a sixth hue: the bore is built in Minecraft first,
# and grey is a block you can actually get. It also cannot be mistaken for
# south's orange once the faces are shaded, which yellow can.
DIRCOL = {'N': '#2f6ea8', 'S': '#e08a2e', 'E': '#3f8f5e', 'W': '#9b4a9c',
          'U': '#cf4433', 'D': '#8b9096'}
DIRNAME = {'N': 'north', 'S': 'south', 'E': 'east', 'W': 'west',
           'U': 'up', 'D': 'down'}
LIGHT = (0.35, 1.0, 0.55)          # over your left shoulder, from above


def wheel(n):
    """n colours that stay apart, for when there are more pieces than palette.

    Hues at the golden angle, so neighbours in the list are never neighbours on
    the wheel, with lightness alternating as well: eleven pieces round an
    eight-colour palette put the same green on 1 and 9.
    """
    if n <= len(PAL):
        return PAL[:n]
    out = []
    for i in range(n):
        h = (i * 0.618033988749895) % 1.0
        li = 0.42 + 0.13 * (i % 3) / 2.0
        c = (1 - abs(2 * li - 1)) * 0.62
        x = c * (1 - abs((h * 6) % 2 - 1))
        m = li - c / 2
        r, g, b = [(c, x, 0), (x, c, 0), (0, c, x),
                   (0, x, c), (x, 0, c), (c, 0, x)][int(h * 6) % 6]
        out.append('#%02x%02x%02x' % tuple(int((v + m) * 255) for v in (r, g, b)))
    return out


def shade(col, nrm):
    """The face's colour under a fixed light, so the solid reads as a solid.

    Without this every face of a block is the same flat colour and a long run
    of one direction turns into a silhouette you cannot count blocks along.
    """
    n = sum(a * a for a in LIGHT) ** 0.5
    d = sum(a * b for a, b in zip(LIGHT, nrm)) / n
    f = 0.68 + 0.32 * (d + 1) / 2 + (0.08 if nrm[1] > 0 else 0)
    r, g, b = (int(col[i:i+2], 16) for i in (1, 3, 5))
    return '#%02x%02x%02x' % tuple(min(255, int(v * f)) for v in (r, g, b))


def layout(text):
    """Cells in walk order, tagged with the piece they belong to.

    Uses the same grouping bore_split cuts from, so the picture and the cut
    list cannot drift apart.
    """
    rec, groups, plans, plan, unfilled = specs_for(text)
    cells, pieces = [], []
    for i, (g, p) in enumerate(zip(groups, plan)):
        pieces.append((p['kind'], len(g), rec[g[0]]['in'], rec[g[-1]]['out'],
                       p['code']))
        for j in g:
            cells.append((rec[j]['pos'], i))
    return cells, pieces, pieces


def render(cells, pieces, az, w, h, title, el=0.0, openings=None,
           colour=None):
    openings = openings or {}
    ca, sa = math.cos(math.radians(az)), math.sin(math.radians(az))
    ce, se = math.cos(math.radians(el)), math.sin(math.radians(el))
    def rot(p):
        x, y, z = p
        x, z = x*ca + z*sa, -x*sa + z*ca
        y, z = y*ce - z*se, y*se + z*ce
        return (x, y, z)
    def proj(p):
        x, y, z = rot(p)
        return ((x - z)*0.866, (x + z)*0.5 - y)
    occ = {c for c, _ in cells}
    FACES = [((0,1,0),[(0,1,0),(1,1,0),(1,1,1),(0,1,1)]),
             ((0,-1,0),[(0,0,0),(0,0,1),(1,0,1),(1,0,0)]),
             ((1,0,0),[(1,0,0),(1,0,1),(1,1,1),(1,1,0)]),
             ((-1,0,0),[(0,0,0),(0,1,0),(0,1,1),(0,0,1)]),
             ((0,0,1),[(0,0,1),(0,1,1),(1,1,1),(1,0,1)]),
             ((0,0,-1),[(0,0,0),(1,0,0),(1,1,0),(0,1,0)])]
    faces = []
    for c, pid in cells:
        for nrm, cor in FACES:
            nb = tuple(c[i] + nrm[i] for i in range(3))
            if nb in occ:
                continue
            rn = rot(nrm)
            if rn[0] + rn[1] + rn[2] <= 0.01:
                continue
            pts = [tuple(c[i] + v[i] for i in range(3)) for v in cor]
            cen = tuple(sum(p[i] for p in pts)/4 for i in range(3))
            rc = rot(cen)
            faces.append((rc[0]+rc[1]+rc[2], pts, pid, c, nrm))
    faces.sort(key=lambda t: t[0])
    pid_of = {c: i for c, i in cells}
    pp = [proj(p) for f in faces for p in f[1]]
    if not pp:
        return '', 1, 1
    minx = min(a for a, _ in pp); maxx = max(a for a, _ in pp)
    miny = min(b for _, b in pp); maxy = max(b for _, b in pp)
    S = min((w-40)/(maxx-minx or 1), (h-60)/(maxy-miny or 1))
    ox = 20 - minx*S + ((w-40) - (maxx-minx)*S)/2
    oy = 40 - miny*S
    def P(p):
        a, b = proj(p); return (ox + a*S, oy + b*S)
    out = [f'<text x="{w/2:.0f}" y="26" class="c">{title}</text>']
    def cross(a, b):
        return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
    for _, pts, pid, cell, nrm in faces:
        tag = openings.get((cell, nrm))
        base = colour(cell, pid) if colour else PAL[pid % len(PAL)]
        col = OPEN[tag] if tag else shade(base, nrm)
        s = ' '.join(f'{a:.1f},{b:.1f}' for a, b in map(P, pts))
        out.append(f'<polygon points="{s}" fill="{col}" fill-opacity="'
                   f'{0.98 if tag else 1.0}" stroke="#333" stroke-width='
                   f'"{1.8 if tag else 0.8}" stroke-linejoin="round"/>')
        if tag:
            ax = sum(a for a, _ in map(P, pts))/4
            ay = sum(b for _, b in map(P, pts))/4
            out.append(f'<text x="{ax:.0f}" y="{ay+5:.0f}" class="op">{tag}</text>')
        # heavy line wherever this face crosses from one piece to the next
        for k in range(4):
            p0, p1 = pts[k], pts[(k+1) % 4]
            e = tuple(p1[i]-p0[i] for i in range(3))
            d = cross(nrm, e)
            nb = tuple(cell[i]+d[i] for i in range(3))
            if nb in pid_of and pid_of[nb] != pid:
                (ax, ay), (bx, by) = P(p0), P(p1)
                out.append(f'<line x1="{ax:.1f}" y1="{ay:.1f}" x2="{bx:.1f}" '
                           f'y2="{by:.1f}" stroke="#111" stroke-width="3.2" '
                           f'stroke-linecap="round"/>')
    # piece numbers, at each piece's centroid
    for i in range(len(pieces)):
        cs = [c for c, q in cells if q == i]
        cen = tuple(sum(c[k] for c in cs)/len(cs) + 0.5 for k in range(3))
        x, y = P(cen)
        out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="11" fill="#fff" '
                   f'fill-opacity="0.9" stroke="#111" stroke-width="1.2"/>')
        out.append(f'<text x="{x:.1f}" y="{y+4.5:.1f}" class="pn">{i+1}</text>')
    return '\n'.join(out), S, (ox, oy)


def main(text, by_direction=False):
    cells, pieces, bl = layout(text)
    rec, groups, plans = coplanar_pieces(text)
    seen, clash = {}, []
    for c, pid in cells:
        if c in seen:
            clash.append(c)
        seen[c] = pid
    print(f'bore: {text}')
    print(f'  {len(cells)} cells, {len(pieces)} pieces')
    for i, (k, n, a_, b_, code) in enumerate(pieces, 1):
        label = k if k == 'elbow' else f'{k} {n}'
        print(f'   {i:<3} {label:<13} {code:<8} {a_} -> {b_}')
    print('\n  ' + (f'!! the bore runs into itself at {len(clash)} cell(s)'
                    if clash else 'no self-intersection: every cell is used once'))

    first, last = rec[0], rec[-1]
    openings = {(first['pos'], tuple(-x for x in DIRS[first['in']])): 'IN',
                (last['pos'], DIRS[last['out']]): 'OUT'}

    pal = wheel(len(pieces))
    if by_direction:
        # a turn belongs to both legs; colour it by the way it leaves, which is
        # the way you are going as you look along the picture
        dirof = {r['pos']: r['out'] for r in rec}
        turns = {r['pos'] for r in rec if r['in'] != r['out']}
        colour = lambda c, pid: DIRCOL[dirof[c]]
    else:
        colour = lambda c, pid: pal[pid % len(pal)]

    VIEWS = [(0, 0, 'front-right'), (90, 0, 'back-right'), (180, 0, 'back-left'),
             (270, 0, 'front-left'), (0, 62, 'from above'),
             (0, -34, 'from below')]
    W, H = 400, 360
    body = []
    for i, (az, el, nm) in enumerate(VIEWS):
        g, _, _ = render(cells, pieces, az, W, H, nm, el, openings, colour)
        body.append(f'<g transform="translate({(i%3)*W},{(i//3)*H})">{g}</g>')

    ly = 2*H + 16
    leg = [f'<text x="24" y="{ly}" class="hd">{text.strip()} — {len(rec)} blocks, '
           f'{len(pieces)} pieces, '
           + ('coloured by direction of travel' if by_direction else 'assembled')
           + '</text>']
    if by_direction:
        used = [d for d in 'NSEWUD' if any(r['out'] == d for r in rec)]
        rows = [(DIRCOL[d],
                 f'{d}  {DIRNAME[d]:<7}'
                 f'{sum(1 for r in rec if r["out"] == d):>3} blocks'
                 + (f', {sum(1 for r in rec if r["out"] == d and r["pos"] in turns)}'
                    ' turning' if any(r['out'] == d and r['pos'] in turns
                                      for r in rec) else ''))
                for d in used]
    else:
        rows = [(pal[i % len(pal)],
                 f'{i+1}. {code:<6} {(k if k == "elbow" else k + " " + str(cnt)):<12}'
                 f'{a_} to {b_}')
                for i, (k, cnt, a_, b_, code) in enumerate(pieces)]
    rows.append((OPEN['IN'], 'IN   the bore mouth, tabs'))
    rows.append((OPEN['OUT'], 'OUT  the far end, notches'))
    for i, (col, name) in enumerate(rows):
        cx, cy = 24 + (i % 2)*560, ly + 28 + (i//2)*26
        leg.append(f'<rect x="{cx}" y="{cy-11}" width="20" height="14" rx="2" '
                   f'fill="{col}" stroke="#222" stroke-width="0.9"/>')
        leg.append(f'<text x="{cx+30}" y="{cy}" class="lg">{name}</text>')
    HH = ly + 28 + ((len(rows)+1)//2)*26 + 14

    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W*3}" height="{HH}" '
           f'viewBox="0 0 {W*3} {HH}"><style>'
           f'.c{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222;text-anchor:middle}}'
           f'.hd{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222}}'
           f'.lg{{font:13px ui-monospace,Menlo,monospace;fill:#333}}'
           f'.op{{font:700 15px ui-sans-serif,sans-serif;fill:#fff;text-anchor:middle}}'
           f'.pn{{font:700 12px ui-sans-serif,sans-serif;fill:#111;text-anchor:middle}}'
           f'</style><rect width="{W*3}" height="{HH}" fill="#fff"/>'
           + ''.join(body) + ''.join(leg) + '</svg>')
    name = 'bore3d_directions.svg' if by_direction else 'bore3d.svg'
    open(name, 'w').write(svg)
    print(f'\n  wrote {name}  ({W*3} x {HH})')


if __name__ == '__main__':
    argv = sys.argv[1:]
    by_dir = False
    for flag in ('--directions', '--by-direction'):
        if flag in argv:
            argv.remove(flag)
            by_dir = True
    try:
        main(' '.join(argv), by_dir)
    except ValueError as e:
        print(f'error: {e}')
        sys.exit(1)
