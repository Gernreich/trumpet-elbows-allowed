"""Bundle a folder of per-section cut files onto as few P2S bed sheets as it can.

    python3 bed_sheets.py CUT_DIR OUT_DIR STEM [--tries N] [--seed S]

Writes STEM-sheet1ofN.svg ... into OUT_DIR. Each section file is placed WHOLE
-- its parts stay together, so the number engraved on every part still finds
its neighbours -- at 0 or 90 degrees, GAP apart and GAP from the bed edge.
The paths are copied verbatim inside a translate (and rotate): what is cut is
what the section files say, and the section files are what bore_split.py
wrote and gated.

Packing is maxrects, best short side fit, on each section's real content box
rather than its canvas, which carries a margin. Maxrects is greedy, so the
order sections arrive in decides how many sheets it needs. Largest first is
tried, then up to --tries seeded random orders, stopping as soon as one
reaches the area bound. The seed makes the search, and so the sheet,
reproducible: the same command writes the same bytes.

Every sheet is re-read before the tool reports success, and it exits 1 if
any check fails:
  every section is placed exactly once
  every path and polyline is byte-identical to its section file's
  every section lies inside the bed
  no two sections come closer than GAP
That last check is not decoration. The first version of the packer reserved
the gap only to the right of and below each placed section, so a later one
could butt against its left or top edge; a one-sheet layout of the tight coil
came out with seven pairs touching, and this check is what said so.
"""
import argparse, math, os, random, re, sys
import xml.etree.ElementTree as ET

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from svgpath import pts
from bore_split import BED_W, BED_H

NS = '{http://www.w3.org/2000/svg}'
GAP = 4.0
ET.register_namespace('', 'http://www.w3.org/2000/svg')


# ------------------------------------------------------------ geometry

def _mul(M, N):
    return [M[0]*N[0] + M[2]*N[1], M[1]*N[0] + M[3]*N[1],
            M[0]*N[2] + M[2]*N[3], M[1]*N[2] + M[3]*N[3],
            M[0]*N[4] + M[2]*N[5] + M[4], M[1]*N[4] + M[3]*N[5] + M[5]]


def _matrix(t):
    M = [1, 0, 0, 1, 0, 0]
    for op, args in re.findall(r'(\w+)\(([^)]*)\)', t or ''):
        a = [float(v) for v in re.split(r'[ ,]+', args.strip())]
        if op == 'translate':
            N = [1, 0, 0, 1, a[0], a[1] if len(a) > 1 else 0.0]
        elif op == 'rotate' and len(a) == 1:
            c, s = math.cos(math.radians(a[0])), math.sin(math.radians(a[0]))
            N = [c, s, -s, c, 0, 0]
        else:
            raise SystemExit(f'unexpected transform {t!r}')
        M = _mul(M, N)
    return M


def geometry(el, M=(1, 0, 0, 1, 0, 0), acc=None):
    """Every point drawn under el, in el's parent space, and every path's data."""
    if acc is None:
        acc = {'pts': [], 'data': []}
    M = _mul(list(M), _matrix(el.get('transform')))
    tag = el.tag.replace(NS, '')
    P = []
    if tag == 'path':
        P = pts(el.get('d'))
        acc['data'].append((el.get('d'), el.get('stroke')))
    elif tag == 'polyline':
        v = [float(n) for n in re.split(r'[\s,]+', el.get('points').strip())]
        P = list(zip(v[0::2], v[1::2]))
        acc['data'].append((el.get('points'), el.get('stroke')))
    for x, y in P:
        acc['pts'].append((M[0]*x + M[2]*y + M[4], M[1]*x + M[3]*y + M[5]))
    for c in el:
        geometry(c, M, acc)
    return acc


def box(points):
    xs, ys = [p[0] for p in points], [p[1] for p in points]
    return min(xs), min(ys), max(xs), max(ys)


# ------------------------------------------------------------ packing

def maxrects(items, bw, bh):
    """items: (key, w, h), placed in the order given -> [(sheet, key, rot, x, y)]."""
    sheets, out = [], []
    for key, w, h in items:
        best = None
        for si, free in enumerate(sheets):
            for fx, fy, fw, fh in free:
                for rot, a, b in ((0, w, h), (90, h, w)):
                    if a <= fw and b <= fh:
                        s = (min(fw - a, fh - b), max(fw - a, fh - b))
                        if best is None or (si, s) < (best[0], best[1]):
                            best = (si, s, rot, fx, fy, a, b)
        if best is None:
            sheets.append([(GAP, GAP, bw - 2 * GAP, bh - 2 * GAP)])
            si = len(sheets) - 1
            fx, fy, fw, fh = sheets[si][0]
            for rot, a, b in ((0, w, h), (90, h, w)):
                if a <= fw and b <= fh:
                    best = (si, None, rot, fx, fy, a, b)
                    break
            if best is None:
                raise SystemExit(f'{key} ({w:.1f} x {h:.1f}mm) does not fit the bed')
        si, _, rot, x, y, a, b = best
        out.append((si, key, rot, x, y))
        # Split every free rectangle the placed one overlaps, grown by GAP on
        # ALL four sides.
        px0, py0, px1, py1 = x - GAP, y - GAP, x + a + GAP, y + b + GAP
        new = []
        for fx, fy, fw, fh in sheets[si]:
            fx1, fy1 = fx + fw, fy + fh
            if px0 >= fx1 or px1 <= fx or py0 >= fy1 or py1 <= fy:
                new.append((fx, fy, fw, fh))
                continue
            if px0 > fx:
                new.append((fx, fy, px0 - fx, fh))
            if px1 < fx1:
                new.append((px1, fy, fx1 - px1, fh))
            if py0 > fy:
                new.append((fx, fy, fw, py0 - fy))
            if py1 < fy1:
                new.append((fx, py1, fw, fy1 - py1))
        new = [r for r in new if r[2] > 0 and r[3] > 0]
        sheets[si] = [r for i, r in enumerate(new) if not any(
            j != i and o[0] <= r[0] and o[1] <= r[1]
            and o[0] + o[2] >= r[0] + r[2] and o[1] + o[3] >= r[1] + r[3]
            and (o != r or j < i)
            for j, o in enumerate(new))]
    return out


def sheets_needed(placed):
    return max(si for si, *_ in placed) + 1


def search(items, tries, seed):
    """Largest first, then seeded random orders; the first order that reaches
    the area bound wins, otherwise the one with fewest sheets."""
    first = sorted(items, key=lambda t: -t[1] * t[2])
    best = maxrects(first, BED_W, BED_H)
    floor = math.ceil(sum(w * h for _, w, h in items)
                      / ((BED_W - 2 * GAP) * (BED_H - 2 * GAP)))
    rng = random.Random(seed)
    for i in range(tries):
        if sheets_needed(best) <= floor:
            break
        order = items[:]
        rng.shuffle(order)
        if i % 2:
            order.sort(key=lambda t: -max(t[1], t[2]) + rng.uniform(0, 40))
        try:
            placed = maxrects(order, BED_W, BED_H)
        except SystemExit:
            continue
        if sheets_needed(placed) < sheets_needed(best):
            best = placed
    return best, floor


# ------------------------------------------------------------ writing

def section_of(name):
    return re.search(r'(\d+of\d+)', name)[1]


def write(placed, sections, out_dir, stem):
    n = sheets_needed(placed)
    os.makedirs(out_dir, exist_ok=True)
    written = []
    for si in range(n):
        here = sorted((p for p in placed if p[0] == si), key=lambda p: p[1])
        groups = []
        for _, f, rot, x, y in here:
            x0, y0, w, h, body = sections[f]
            inner = ''.join(ET.tostring(c, encoding='unicode') for c in body)
            if rot == 0:
                t = f'translate({x - x0:.3f},{y - y0:.3f})'
            else:   # rotate 90 clockwise about the origin, then place
                t = f'translate({x + h + y0:.3f},{y - x0:.3f}) rotate(90)'
            groups.append(f'<g id="section-{section_of(f)}" transform="{t}">{inner}</g>')
        nums = ', '.join(section_of(p[1]).split('of')[0].lstrip('0') for p in here)
        path = os.path.join(out_dir, f'{stem}-sheet{si+1}of{n}.svg')
        with open(path, 'w') as fh:
            fh.write(
                '<?xml version="1.0" encoding="utf-8"?>\n'
                f'<svg xmlns="http://www.w3.org/2000/svg" width="{BED_W:.2f}mm" '
                f'height="{BED_H:.2f}mm" viewBox="0 0 {BED_W:.2f} {BED_H:.2f}">\n'
                f'<title>{stem} - sheet {si+1} of {n}, sections {nums}</title>\n'
                f'<desc>1 user unit = 1mm, the whole xTool P2S bed ({BED_W:.0f} x '
                f'{BED_H:.0f}mm). Each section file is copied whole and unchanged, '
                f'{GAP:.0f}mm apart; written by tools/bed_sheets.py. '
                'black #000000 cuts, blue #0000ff engraves.</desc>\n'
                + '\n'.join(groups) + '\n</svg>\n')
        written.append((path, len(here), nums))
    return written


def check(written, originals):
    """Re-read what was written. Returns a list of failures."""
    bad, seen = [], {}
    for path, _, _ in written:
        root = ET.parse(path).getroot()
        boxes = []
        for g in root.findall(NS + 'g'):
            sec = g.get('id').split('-', 1)[1]
            if sec in seen:
                bad.append(f'section {sec} placed twice')
            seen[sec] = path
            acc = geometry(g)
            if sorted(acc['data']) != originals[sec]:
                bad.append(f'section {sec}: paths differ from its section file')
            b = box(acc['pts'])
            boxes.append((sec, b))
            if b[0] < 0 or b[1] < 0 or b[2] > BED_W or b[3] > BED_H:
                bad.append(f'section {sec} leaves the bed')
        for i in range(len(boxes)):
            for j in range(i + 1, len(boxes)):
                a, b = boxes[i][1], boxes[j][1]
                g = GAP - 0.01
                if (a[0] < b[2] + g and b[0] < a[2] + g
                        and a[1] < b[3] + g and b[1] < a[3] + g):
                    bad.append(f'sections {boxes[i][0]} and {boxes[j][0]} '
                               f'closer than {GAP:g}mm')
    for sec in sorted(set(originals) - set(seen)):
        bad.append(f'section {sec} not placed')
    return bad


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('cut_dir')
    ap.add_argument('out_dir')
    ap.add_argument('stem')
    ap.add_argument('--tries', type=int, default=20000)
    ap.add_argument('--seed', type=int, default=1)
    a = ap.parse_args(argv)

    sections, originals, items = {}, {}, []
    for f in sorted(os.listdir(a.cut_dir)):
        if not f.endswith('.svg'):
            continue
        root = ET.parse(os.path.join(a.cut_dir, f)).getroot()
        acc = geometry(root)
        x0, y0, x1, y1 = box(acc['pts'])
        body = [c for c in root if c.tag.replace(NS, '') not in ('title', 'desc')]
        sections[f] = (x0, y0, x1 - x0, y1 - y0, body)
        originals[section_of(f)] = sorted(acc['data'])
        items.append((f, x1 - x0, y1 - y0))
    if not items:
        raise SystemExit(f'no section files in {a.cut_dir}')

    placed, floor = search(items, a.tries, a.seed)
    written = write(placed, sections, a.out_dir, a.stem)
    for path, k, nums in written:
        print(f'  {os.path.basename(path)}: {k} sections ({nums})')
    print(f'  {len(written)} sheet(s); the area bound is {floor}')
    bad = check(written, originals)
    for b in bad:
        print(f'  FAIL {b}')
    print(f'  {len(items)} sections, {len(bad)} failed')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
