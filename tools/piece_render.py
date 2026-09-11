"""Draw one assembled piece from several angles, each face coloured by its part.

Reads the piece's outline from SnakeBoxVar itself -- the generator that cuts it --
and its part sizes from the cut files, so the picture cannot drift from what is
actually cut.

    python3 piece_render.py --path UUR --label B1
    python3 piece_render.py --open_faces N,E --label E1
"""
import argparse, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bore_split                          # noqa: E402
from bore_split import BOXES, cut          # noqa: E402
sys.path.insert(0, BOXES)
# SnakeBoxVar, which is the generator that actually CUTS these pieces:
# bore_split shells out to `scripts/boxes SnakeBoxVar`, and check.py reads
# its geometry too. This file read the outline from plain SnakeBox while
# taking the parts from cut(), which runs the Var -- so the topology and the
# parts came from two different generators, under a docstring saying the
# picture cannot drift from what is cut. They agree today, cell for cell and
# run for run; nothing was watching whether they went on agreeing.
from boxes.generators.snakeboxvar import SnakeBoxVar as SnakeBox   # noqa: E402

PLATE_COL = '#b8b2a4'
WALL_COLS = ['#3a5f8a', '#3f7d55', '#c8532f', '#d9962f',
             '#6b3a7a', '#2f7a7a', '#7a4a2f', '#4a6b2f']
IN_COL, OUT_COL = '#4a4a52', '#7a4a86'
FACES = [((0, 0, 1),  [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]),
         ((0, 0, -1), [(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)]),
         ((1, 0, 0),  [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)]),
         ((-1, 0, 0), [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)]),
         ((0, 1, 0),  [(0, 1, 0), (0, 1, 1), (1, 1, 1), (1, 1, 0)]),
         ((0, -1, 0), [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)])]
VIEWS = [(0, 0, 'front-right'), (90, 0, 'back-right'), (180, 0, 'back-left'),
         (270, 0, 'front-left'), (20, -52, 'from below, front'),
         (200, -52, 'from below, back')]


def describe(path, open_faces):
    """Outline, tips and measured part sizes for one piece."""
    # --open_faces IS FOR A SINGLE CELL, and a single cell has no path. Passing
    # both made SnakeBoxVar refuse -- "--open_faces is for a single cell only" --
    # so the second example in the docstring above could never have run: --path
    # carries a default, so it was always sent alongside. An elbow is drawn by
    # naming its two open faces and nothing else.
    args = ([f'--open_faces={open_faces}', '--path=']
            if open_faces else [f'--path={path}'])
    b = SnakeBox()
    b.parseArgs(args + bore_split.COMMON)
    cells = b.cells()
    runs, _ = b.outline(cells)
    tips = b.tipRuns(cells, runs)

    parts = cut(args, 'render')
    walls = [p for p in parts if p['role'] == 'W']
    wall_runs = [k for k in range(len(runs)) if k not in tips]
    if len(walls) != len(wall_runs):
        raise SystemExit(f'{len(walls)} wall parts but {len(wall_runs)} wall runs')
    plate = next(p for p in parts if p['role'] == 'P')
    return cells, runs, tips, dict(zip(wall_runs, walls)), plate


def face_run(runs, cell, m):
    """The boundary run carrying this cell face."""
    cx, cy = cell
    seg = {(1, 0): (cx + 1, cy), (-1, 0): (cx, cy + 1),
           (0, 1): (cx + 1, cy + 1), (0, -1): (cx, cy)}[m]
    for k, (v, d, n) in enumerate(runs):
        if (d[1], -d[0]) != m:
            continue
        for i in range(n):
            if (v[0] + i * d[0], v[1] + i * d[1]) == seg:
                return k
    raise AssertionError(f'no run for cell {cell} face {m}')


def view(occ, runs, colour, az, el, w, h, title):
    ca, sa = math.cos(math.radians(az)), math.sin(math.radians(az))
    ce, se = math.cos(math.radians(el)), math.sin(math.radians(el))

    def rot(p):
        x, y, z = p
        x, z = x * ca + z * sa, -x * sa + z * ca
        y, z = y * ce - z * se, y * se + z * ce
        return (x, y, z)

    def proj(p):
        x, y, z = rot(p)
        return ((x - z) * 0.866, (x + z) * 0.5 - y)

    quads = []
    for c in sorted(occ):
        for nrm, cor in FACES:
            if tuple(c[i] + nrm[i] for i in range(3)) in occ:
                continue
            col, tag = (colour['plate'], None) if nrm[2] else \
                colour[face_run(runs, (c[0], c[1]), (nrm[0], nrm[1]))]
            rn = rot(nrm)
            if rn[0] + rn[1] + rn[2] <= 0.01:
                continue
            pts = [tuple(c[i] + v[i] for i in range(3)) for v in cor]
            rc = rot(tuple(sum(p[i] for p in pts) / 4 for i in range(3)))
            quads.append((rc[0] + rc[1] + rc[2], pts, col, tag))
    quads.sort(key=lambda t: t[0])

    pp = [proj(p) for q in quads for p in q[1]]
    minx, maxx = min(a for a, _ in pp), max(a for a, _ in pp)
    miny, maxy = min(b for _, b in pp), max(b for _, b in pp)
    S = min((w - 46) / (maxx - minx), (h - 74) / (maxy - miny))
    ox = 23 - minx * S + ((w - 46) - (maxx - minx) * S) / 2
    oy = 52 - miny * S

    def P(p):
        a, b = proj(p)
        return (ox + a * S, oy + b * S)

    out = [f'<text x="{w/2:.0f}" y="30" class="t">{title}</text>']
    for _, pts, col, tag in quads:
        s = ' '.join(f'{a:.1f},{b:.1f}' for a, b in map(P, pts))
        out.append(f'<polygon points="{s}" fill="{col}" fill-opacity="'
                   f'{0.98 if tag else 0.9}" stroke="#222" stroke-width='
                   f'"{1.6 if tag else 0.9}" stroke-linejoin="round"/>')
        if tag:
            cx = sum(a for a, _ in map(P, pts)) / 4
            cy = sum(b for _, b in map(P, pts)) / 4
            out.append(f'<text x="{cx:.0f}" y="{cy+5:.0f}" class="o">{tag}</text>')
    return '\n'.join(out)


def main(path, open_faces, label, out):
    cells, runs, tips, wallpart, plate = describe(path, open_faces)
    occ = {(c[0], c[1], 0) for c in cells}

    colour, legend = {}, []
    colour['plate'] = PLATE_COL
    legend.append((PLATE_COL, f'{label}P  x2  {plate["w"]:.2f} x {plate["h"]:.2f}'))
    for i, (k, p) in enumerate(sorted(wallpart.items())):
        col = WALL_COLS[i % len(WALL_COLS)]
        colour[k] = (col, None)
        n = runs[k][2]
        legend.append((col, f'{label}W  {n} cell{"s" if n > 1 else " "}  '
                            f'{max(p["w"], p["h"]):.2f}'))
    colour[tips[0]] = (IN_COL, 'IN')
    colour[tips[1]] = (OUT_COL, 'OUT')
    legend.append((IN_COL, 'entry - open, carries the TABS'))
    legend.append((OUT_COL, 'exit  - open, carries the NOTCHES'))

    W, H = 400, 360
    body = [f'<g transform="translate({(i%3)*W},{(i//3)*H})">'
            f'{view(occ, runs, colour, az, el, W, H, nm)}</g>'
            for i, (az, el, nm) in enumerate(VIEWS)]

    ly = 2 * H + 16
    what = f'--path={path}' + (f' --open_faces={open_faces}' if open_faces else '')
    leg = [f'<text x="24" y="{ly}" class="h">{label} = {what} — '
           f'2 plates + {len(wallpart)} walls</text>']
    for i, (col, name) in enumerate(legend):
        cx, cy = 24 + (i % 2) * 560, ly + 28 + (i // 2) * 26
        leg.append(f'<rect x="{cx}" y="{cy-11}" width="20" height="14" rx="2" '
                   f'fill="{col}" stroke="#222" stroke-width="0.9"/>')
        leg.append(f'<text x="{cx+30}" y="{cy}" class="l">{name}</text>')
    HH = ly + 28 + ((len(legend) + 1) // 2) * 26 + 14

    open(out, 'w').write(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W*3}" height="{HH}" '
        f'viewBox="0 0 {W*3} {HH}"><style>'
        f'.t{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222;text-anchor:middle}}'
        f'.h{{font:600 15px ui-sans-serif,system-ui,sans-serif;fill:#222}}'
        f'.l{{font:13px ui-monospace,Menlo,monospace;fill:#333}}'
        f'.o{{font:700 15px ui-sans-serif,sans-serif;fill:#fff;text-anchor:middle}}'
        f'</style><rect width="{W*3}" height="{HH}" fill="#fff"/>'
        + ''.join(body) + ''.join(leg) + '</svg>')
    print(f'{out}  {W*3} x {HH}   {len(cells)} cells, '
          f'2 plates + {len(wallpart)} walls')


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--path', default='UUR', help='cell-to-cell moves')
    ap.add_argument('--open_faces', default='', help='single cell: two of N/S/E/W')
    ap.add_argument('--label', default='', help='piece code used in the legend')
    ap.add_argument('--out', default='piece_3d.svg')
    a = ap.parse_args()
    main(a.path, a.open_faces, a.label or (a.path or 'piece'), a.out)
