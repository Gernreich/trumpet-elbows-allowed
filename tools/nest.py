"""Nest a bore's parts onto sheets, four ways, and report which wins.

    python3 nest.py "N N10 U2 ..." --out ../designs/nest.svg

  shelf     rows of parts, tallest first          (what cram.py did)
  maxrects  free-rectangle packing, best short side fit
  skyline   bottom-left against a skyline profile
  raster    irregular: the real outline on a 1 mm raster, four rotations

The first three pack bounding boxes, so they can never do better than the sum
of those boxes. Only `raster` can put a wall inside an L-shaped plate's notch,
which is where most of the waste is.
"""
import argparse, os, re, sys
import numpy as np
from scipy.signal import fftconvolve
import shapely
from shapely.geometry import Polygon

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bore_split import specs_for, cut, part_labels, BED_W, BED_H

GAP = 4.0
PX = 1.0        # raster pixel, mm


# ---------------------------------------------------------------- the parts

def parts_of(text):
    """Every part of every section, tagged with its section number."""
    _, _, _, plan, _ = specs_for(text)
    out = []
    for i, p in enumerate(plan, 1):
        nb = {}
        if i > 1:
            nb['in'] = f'{i-1}{i}'
        if i < len(plan):
            nb['out'] = f'{i}{i+1}'
        for part in cut(p['args'], str(i)):
            part['label'] = str(i)
            part['args'] = p['args']
            part['nb'] = nb
            out.append(part)
    return out


# ------------------------------------------------------------ rectangle fits

def shelf(parts, bw, bh):
    items = sorted(((p, p['w'], p['h']) for p in parts),
                   key=lambda t: -max(t[1], t[2]))
    sheets, placed = [], []
    for p, w, h in items:
        done = False
        for si, sh in enumerate(sheets):
            for s in sh:
                for r, a, b in ((0, w, h), (90, h, w)):
                    if s['x'] + a <= bw - GAP and b <= s['h']:
                        placed.append((si, p, r, s['x'], s['y']))
                        s['x'] += a + GAP
                        done = True
                        break
                if done:
                    break
            if done:
                break
            r, a, b = (0, w, h) if w >= h else (90, h, w)
            y = max((s['y'] + s['h'] for s in sh), default=0.0) + GAP
            if y + b <= bh - GAP and a <= bw - 2 * GAP:
                sh.append({'x': GAP + a + GAP, 'y': y, 'h': b})
                placed.append((si, p, r, GAP, y))
                done = True
                break
        if not done:
            r, a, b = (0, w, h) if w >= h else (90, h, w)
            sheets.append([{'x': GAP + a + GAP, 'y': GAP, 'h': b}])
            placed.append((len(sheets) - 1, p, r, GAP, GAP))
    return placed


def maxrects(parts, bw, bh):
    items = sorted(((p, p['w'], p['h']) for p in parts),
                   key=lambda t: -t[1] * t[2])
    bins, placed = [], []

    def split(f, r):
        fx, fy, fw, fh = f
        rx, ry, rw, rh = r
        if rx >= fx + fw or rx + rw <= fx or ry >= fy + fh or ry + rh <= fy:
            return [f]
        out = []
        if ry > fy:
            out.append((fx, fy, fw, ry - fy))
        if ry + rh < fy + fh:
            out.append((fx, ry + rh, fw, fy + fh - ry - rh))
        if rx > fx:
            out.append((fx, fy, rx - fx, fh))
        if rx + rw < fx + fw:
            out.append((rx + rw, fy, fx + fw - rx - rw, fh))
        return out

    def prune(fs):
        keep = []
        for i, a in enumerate(fs):
            if a[2] <= 0 or a[3] <= 0:
                continue
            if any(i != j and a[0] >= b[0] and a[1] >= b[1]
                   and a[0] + a[2] <= b[0] + b[2] and a[1] + a[3] <= b[1] + b[3]
                   for j, b in enumerate(fs) if b[2] > 0 and b[3] > 0):
                continue
            keep.append(a)
        return keep

    for p, w, h in items:
        best = None
        for bi, free in enumerate(bins):
            for f in free:
                for r, a, b in ((0, w + GAP, h + GAP), (90, h + GAP, w + GAP)):
                    if a <= f[2] and b <= f[3]:
                        key = (min(f[2] - a, f[3] - b), max(f[2] - a, f[3] - b))
                        if best is None or key < best[0]:
                            best = (key, bi, f, r, a, b)
        if best is None:
            # pack inside a margin, so adding it back cannot overrun the bed
            bins.append([(0.0, 0.0, bw - 2 * GAP, bh - 2 * GAP)])
            bi, f = len(bins) - 1, (0.0, 0.0, bw - 2 * GAP, bh - 2 * GAP)
            r, a, b = ((0, w + GAP, h + GAP) if w >= h else (90, h + GAP, w + GAP))
        else:
            _, bi, f, r, a, b = best
        x, y = f[0], f[1]
        placed.append((bi, p, r, x + GAP, y + GAP))
        new = []
        for g in bins[bi]:
            new.extend(split(g, (x, y, a, b)))
        bins[bi] = prune(new)
    return placed


def skyline(parts, bw, bh):
    items = sorted(((p, p['w'], p['h']) for p in parts),
                   key=lambda t: -max(t[1], t[2]))
    sheets, placed = [], []

    def fit(sky, a, b):
        best = None
        for i in range(len(sky)):
            x = sky[i][0]
            if x + a > bw:
                continue
            rem, y, j = a, 0.0, i
            while rem > 1e-9 and j < len(sky):
                y = max(y, sky[j][1])
                rem -= sky[j][2]
                j += 1
            if rem > 1e-9 or y + b > bh:
                continue
            key = (y + b, x)
            if best is None or key < best[0]:
                best = (key, x, y, i)
        return best

    def add(sky, x, y, a, b):
        out, placedseg = [], (x, y + b, a)
        for sx, sy, sw in sky:
            if sx + sw <= x or sx >= x + a:
                out.append((sx, sy, sw))
                continue
            if sx < x:
                out.append((sx, sy, x - sx))
            if sx + sw > x + a:
                out.append((x + a, sy, sx + sw - x - a))
        out.append(placedseg)
        out.sort()
        merged = [out[0]]
        for seg in out[1:]:
            if abs(seg[1] - merged[-1][1]) < 1e-9:
                merged[-1] = (merged[-1][0], merged[-1][1], merged[-1][2] + seg[2])
            else:
                merged.append(seg)
        return merged

    for p, w, h in items:
        done = False
        for si, sky in enumerate(sheets):
            for r, a, b in ((0, w + GAP, h + GAP), (90, h + GAP, w + GAP)):
                got = fit(sky, a, b)
                if got:
                    _, x, y, _ = got
                    placed.append((si, p, r, x + GAP / 2, y + GAP / 2))
                    sheets[si] = add(sky, x, y, a, b)
                    done = True
                    break
            if done:
                break
        if not done:
            sheets.append([(0.0, 0.0, bw)])
            r, a, b = ((0, w + GAP, h + GAP) if w >= h else (90, h + GAP, w + GAP))
            placed.append((len(sheets) - 1, p, r, GAP / 2, GAP / 2))
            sheets[-1] = add(sheets[-1], 0.0, 0.0, a, b)
    return placed


# ------------------------------------------------------------ irregular fit

def raster_masks(p):
    """Boolean masks of the real outline at 0/90/180/270, true size.

    The gap is kept by growing what is already placed, not by growing the part
    being placed: a grown mask can overhang the sheet edge even when the part
    itself fits.
    """
    pts = p['pts'][:-1] if p['pts'][0] == p['pts'][-1] else p['pts']
    poly = Polygon([(x - p['x0'], y - p['y0']) for x, y in pts])
    out = []
    for rot in (0, 90, 180, 270):
        g = shapely.affinity.rotate(poly, rot, origin=(0, 0))
        minx, miny, maxx, maxy = g.bounds
        g = shapely.affinity.translate(g, -minx, -miny)
        nx = max(1, int(np.ceil((maxx - minx) / PX)))
        ny = max(1, int(np.ceil((maxy - miny) / PX)))
        gx, gy = np.meshgrid((np.arange(nx) + 0.5) * PX, (np.arange(ny) + 0.5) * PX)
        m = shapely.contains_xy(g, gx.ravel(), gy.ravel()).reshape(ny, nx)
        out.append((rot, m, -minx, -miny))
    return out


def raster(parts, bw, bh, order=None):
    from scipy.ndimage import binary_dilation
    # keep the edge margin inside the grid, so a part cannot be placed where
    # the margin would then push the sheet past the bed
    NX, NY = int((bw - 2 * GAP) / PX), int((bh - 2 * GAP) / PX)
    n = 2 * int(np.ceil(GAP / PX)) + 3
    grow = np.ones((n, n), bool)
    sheets, blocked, placed = [], [], []
    for p in (order or sorted(parts, key=lambda q: -q['w'] * q['h'])):
        masks = raster_masks(p)
        done = False
        for si in range(len(sheets)):
            best = None
            for rot, m, ox, oy in masks:
                my, mx = m.shape
                if my > NY or mx > NX:
                    continue
                hit = fftconvolve(blocked[si].astype(float),
                                  m[::-1, ::-1].astype(float), mode='valid')
                free = np.argwhere(hit < 0.5)
                if len(free) == 0:
                    continue
                iy, ix = free[np.lexsort((free[:, 1], free[:, 0]))][0]
                if best is None or (iy, ix) < best[0]:
                    best = ((iy, ix), rot, m, ox, oy)
            if best:
                (iy, ix), rot, m, ox, oy = best
                my, mx = m.shape
                sheets[si][iy:iy + my, ix:ix + mx] |= m
                blocked[si] = binary_dilation(sheets[si], grow)
                placed.append((si, p, rot, GAP + ix * PX, GAP + iy * PX))
                done = True
                break
        if not done:
            sheets.append(np.zeros((NY, NX), bool))
            blocked.append(np.zeros((NY, NX), bool))
            si = len(sheets) - 1
            fits = [t for t in masks if t[1].shape[0] <= NY and t[1].shape[1] <= NX]
            if not fits:
                raise SystemExit(f'a part is {p["w"]:.0f}x{p["h"]:.0f} mm and '
                                 f'will not fit the bed in any rotation')
            rot, m, ox, oy = fits[0]
            my, mx = m.shape
            sheets[si][0:my, 0:mx] |= m
            blocked[si] = binary_dilation(sheets[si], grow)
            placed.append((si, p, rot, GAP, GAP))
    return placed


# ------------------------------------------------------------------ output

def emit(placed, path, tag):
    bysheet = {}
    for si, p, rot, x, y in placed:
        bysheet.setdefault(si, []).append((p, rot, x, y))
    out = []
    for si in sorted(bysheet):
        body, xs, ys = [], [], []
        for p, rot, x, y in bysheet[si]:
            w, h = p['w'], p['h']
            t = {0: f'translate({x:.3f},{y:.3f})',
                 90: f'translate({x+h:.3f},{y:.3f}) rotate(90)',
                 180: f'translate({x+w:.3f},{y+h:.3f}) rotate(180)',
                 270: f'translate({x:.3f},{y+w:.3f}) rotate(270)'}[rot]
            inner = ''.join(f'<path d="{q["d"]}" fill="none" stroke="#000000" '
                            f'stroke-width="0.2"/>' for q in p.get('holes', ()))
            local = (f'<g transform="translate({-p["x0"]:.3f},{-p["y0"]:.3f})">'
                     f'{inner}<path d="{p["d"]}" fill="none" stroke="#000000" '
                     f'stroke-width="0.2"/></g>')
            lbl = part_labels(p, p['label'], p.get('args'), p.get('nb'))
            body.append(f'<g transform="{t}">{local}{lbl}</g>')
            ww, hh = (h, w) if rot in (90, 270) else (w, h)
            xs += [x, x + ww]
            ys += [y, y + hh]
        W, H = max(xs) + GAP, max(ys) + GAP
        stem, ext = os.path.splitext(path)
        this = f'{stem}_{tag}_{si+1}{ext}' if len(bysheet) > 1 else f'{stem}_{tag}{ext}'
        open(this, 'w').write(
            f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
            f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
            f'<!-- nested by {tag}, {GAP:.0f} mm apart; each part carries its '
            f'section number -->\n' + '\n'.join(body) + '\n</svg>\n')
        out.append((this, W, H, len(bysheet[si])))
    return out


def bake(d, dx, dy):
    """Shift an absolute path's coordinates, keeping every curve exact.

    Nesters vary in how well they apply a <g transform>, so the file hands
    them plain top-level paths with nothing to interpret.
    """
    out, i, n = [], 0, len(d)
    tok = re.findall(r'[MLHVCSQTAZmlhvcsqtaz]|-?\d*\.?\d+', d)
    i = 0
    while i < len(tok):
        c = tok[i]
        if c in 'Zz':
            out.append('Z'); i += 1; continue
        if c not in 'MLHVCSQTA':
            raise ValueError(f'relative or unknown command {c!r}; '
                             'this only shifts absolute paths')
        counts = {'M': 2, 'L': 2, 'H': 1, 'V': 1, 'C': 6, 'S': 4, 'Q': 4,
                  'T': 2, 'A': 7}[c]
        i += 1
        out.append(c)
        vals = []
        while i < len(tok) and tok[i] not in 'MLHVCSQTAZmlhvcsqtaz':
            vals.append(float(tok[i])); i += 1
        for j in range(0, len(vals), counts):
            grp = vals[j:j + counts]
            if c == 'H':
                grp = [grp[0] + dx]
            elif c == 'V':
                grp = [grp[0] + dy]
            elif c == 'A':
                grp = grp[:5] + [grp[5] + dx, grp[6] + dy]
            else:
                grp = [v + (dx if k % 2 == 0 else dy) for k, v in enumerate(grp)]
            out.append(' '.join(f'{v:.3f}' for v in grp))
    return ' '.join(out)


def bake(d, dx, dy):
    """Shift an absolute path's coordinates, keeping every curve exact.

    Nesters vary in how well they apply a <g transform>, so the file hands
    them plain top-level paths with nothing left to interpret.
    """
    tok = re.findall(r'[MLHVCSQTAZmlhvcsqtaz]|-?\d*\.?\d+', d)
    out, i = [], 0
    while i < len(tok):
        c = tok[i]
        if c in 'Zz':
            out.append('Z'); i += 1; continue
        if c not in 'MLHVCSQTA':
            raise ValueError(f'relative or unknown command {c!r}; '
                             'this only shifts absolute paths')
        step = {'M': 2, 'L': 2, 'H': 1, 'V': 1, 'C': 6, 'S': 4, 'Q': 4,
                'T': 2, 'A': 7}[c]
        i += 1
        out.append(c)
        vals = []
        while i < len(tok) and tok[i] not in 'MLHVCSQTAZmlhvcsqtaz':
            vals.append(float(tok[i])); i += 1
        for j in range(0, len(vals), step):
            g = vals[j:j + step]
            if c == 'H':
                g = [g[0] + dx]
            elif c == 'V':
                g = [g[0] + dy]
            elif c == 'A':
                g = g[:5] + [g[5] + dx, g[6] + dy]
            else:
                g = [v + (dx if k % 2 == 0 else dy) for k, v in enumerate(g)]
            out.append(' '.join(f'{v:.3f}' for v in g))
    return ' '.join(out)


def deepnest_input(parts, path, bw, bh):
    """Sheet and parts as plain closed paths, in two files.

    Two files because the sheet has to be marked as such, and on its own it is
    the only thing there to mark. Engraved numbers are left out entirely: they
    are open paths, and a nester imports them as parts of their own.
    """
    stem, ext = os.path.splitext(path)
    sheet_f, parts_f = f'{stem}_sheet{ext}', f'{stem}_parts{ext}'

    open(sheet_f, 'w').write(
        f'<?xml version="1.0" encoding="utf-8"?>\n'
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{bw:.2f}mm" '
        f'height="{bh:.2f}mm" viewBox="0 0 {bw:.2f} {bh:.2f}">\n'
        f'<!-- The sheet: one {bw:.0f} x {bh:.0f} mm rectangle and nothing '
        f'else, so it is the only thing to mark. -->\n'
        f'<path d="M 0 0 L {bw:.2f} 0 L {bw:.2f} {bh:.2f} L 0 {bh:.2f} Z" '
        f'fill="none" stroke="#ff0000" stroke-width="0.2"/>\n</svg>\n')

    body, x, y, rh = [], 0.0, 0.0, 0.0
    for p in sorted(parts, key=lambda q: -q['w'] * q['h']):
        if x > 0 and x + p['w'] > bw * 2:
            x, y, rh = 0.0, y + rh + 10.0, 0.0
        body.append(f'<path d="{bake(p["d"], x - p["x0"], y - p["y0"])}" '
                    f'fill="none" stroke="#000000" stroke-width="0.2"/>')
        x += p['w'] + 10.0
        rh = max(rh, p['h'])
    W, H = bw * 2 + 20, y + rh + 20
    open(parts_f, 'w').write(
        f'<?xml version="1.0" encoding="utf-8"?>\n'
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
        f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
        f'<!-- {len(parts)} parts, each one closed path at top level, no '
        f'groups, no transforms, no engraving. -->\n'
        + '\n'.join(body) + '\n</svg>\n')
    return sheet_f, parts_f, W, H


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('walk', nargs='+')
    ap.add_argument('--out', default='nest.svg')
    ap.add_argument('--deepnest', metavar='FILE',
                    help='write an input file for Deepnest / SVGnest and stop')
    ap.add_argument('--tries', type=int, default=0, metavar='N',
                    help='search N random orderings for the raster nester, the '
                         'way Deepnest searches: placement is deterministic, '
                         'the order it places in is not')
    a = ap.parse_args()
    ps = parts_of(' '.join(a.walk))
    if a.deepnest:
        sf, pf, W, H = deepnest_input(ps, a.deepnest, BED_W, BED_H)
        print(f'sheet -> {os.path.basename(sf)}  '
              f'({BED_W:.0f} x {BED_H:.0f} mm, one rectangle)')
        print(f'{len(ps)} parts -> {os.path.basename(pf)}  ({W:.0f} x {H:.0f} mm)')
        raise SystemExit

    def area(P):
        n = len(P)
        return abs(sum(P[i][0] * P[(i + 1) % n][1] - P[(i + 1) % n][0] * P[i][1]
                       for i in range(n))) / 2
    poly = sum(area(p['pts']) for p in ps)
    bed = BED_W * BED_H
    print(f'{len(ps)} parts, {poly:.0f} mm2 of material = {poly/bed:.2f} sheets\n')
    print(f'{"method":<10}{"sheets":>7}{"used":>8}   files')
    results, emit_cache = {}, {}
    for name, fn in (('shelf', shelf), ('maxrects', maxrects),
                     ('skyline', skyline), ('raster', raster)):
        placed = fn(ps, BED_W, BED_H)
        emit_cache[name] = set(si for si, *_ in placed)
        files = emit(placed, a.out, name)
        n = len(files)
        used = poly / (n * bed)
        results[name] = n
        print(f'{name:<10}{n:>7}{100*used:>7.0f}%   '
              + ', '.join(os.path.basename(f[0]) for f in files))
    best = min(results, key=lambda k: results[k])
    print(f'\nbest: {best} at {results[best]} sheet(s)')

    if a.tries:
        import random, time
        base = sorted(ps, key=lambda q: -q['w'] * q['h'])
        champ, champ_n, champ_last = None, len(emit_cache['raster']), None
        t0 = time.time()
        print(f'\nsearching {a.tries} orderings for the raster nester')
        for i in range(a.tries):
            order = base[:]
            # shuffle, but keep big parts early: a big part placed late has
            # nowhere to go
            k = random.randint(1, max(2, len(order) // 2))
            for _ in range(k):
                x = random.randrange(len(order))
                y = random.randrange(len(order))
                order[x], order[y] = order[y], order[x]
            placed = raster(ps, BED_W, BED_H, order)
            n = len(set(si for si, *_ in placed))
            last = sum(1 for si, *_ in placed if si == n - 1)
            if n < champ_n or (n == champ_n and champ_last is not None
                               and last < champ_last):
                champ, champ_n, champ_last = placed, n, last
                print(f'   try {i+1}: {n} sheets, {last} parts on the last one')
        print(f'   {a.tries} tries in {time.time()-t0:.0f}s; '
              f'best {champ_n} sheet(s)')
        if champ:
            for f in emit(champ, a.out, 'rastersearch'):
                print(f'   wrote {os.path.basename(f[0])}')
