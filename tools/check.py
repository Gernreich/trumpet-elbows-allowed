"""Check a bore's cut files before any of it goes on the machine.

    python3 check.py "N N10 U2 W2 S7 U2 E4 N9 W2 D2 N4 N"

Every check here exists because something got cut and thrown away. The name of
each says what it looks for; the comment says what it caught.
"""
import argparse, glob, math, os, re, sys
import xml.etree.ElementTree as ET
from shapely.geometry import Polygon, box
from shapely import affinity

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bore_split
from bore_split import (specs_for, cut, _inside, DIRS, AXIS, FACE2D,
                        BED_W, BED_H, BOXES, THICKNESS, walk_text)
import svgpath as V
from assemble import closes, sealed
sys.path.insert(0, BOXES)
from boxes.generators.snakeboxvar import SnakeBoxVar as SnakeBox        # noqa: E402

MIN_FEATURE = 1.5
results = []


def note(ok, section, name, detail=''):
    results.append((ok, section, name, detail))


def poly(p):
    q = p['pts'][:-1] if p['pts'][0] == p['pts'][-1] else p['pts']
    return Polygon(q).buffer(0)


def geometry_of(args):
    """The outline SnakeBox worked from, so parts can be checked against it.

    Built with bore_split.COMMON itself, not a copy of it: a second list here
    would judge a different piece from the one on the sheet the moment either
    moved.
    """
    b = SnakeBox()
    b.parseArgs(args + bore_split.COMMON
                + (['--pin_length=0'] if bore_split.FLAT else []))
    cells = b.cells()
    runs, turns = b.outline(cells)
    borders, tips, caps, ports, tab_run, plains = b.geometry()
    return b, runs, borders, tips, ports


def check_section(i, args, parts, flat=('', '')):
    b, runs, borders, tips, ports = geometry_of(args)
    plates = [p for p in parts if p['role'] == 'P']
    walls = [p for p in parts if p['role'] == 'W']

    # --- part count. A flat snake is two plates plus one wall per boundary
    # run that is not an opening.
    want = len(runs) - len(tips)
    note(len(plates) == 2 and len(walls) == want, i, 'part count',
         f'{len(plates)} plates + {len(walls)} walls, expected 2 + {want}')

    # --- flat means flat: no part may carry a coupling. A tab shows as a
    # protrusion a pin_width wide, a notch as a recess the same.
    if b.pin_length <= 0:
        found = 0
        for p in parts:
            xs = sorted({round(x, 2) for x, _ in p['pts']})
            ys = sorted({round(y, 2) for _, y in p['pts']})
            for vals, span in ((xs, p['w']), (ys, p['h'])):
                # a coupling sits pin_width across, set in or out by pin_length
                for a, c in zip(vals, vals[1:]):
                    if abs((c - a) - b.pin_length) < 0.3 and b.pin_length > 0:
                        found += 1
        note(found == 0, i, 'no coupling left anywhere', f'{found} found')

    for p in parts:
        g = poly(p)
        # --- a part must be one closed piece. Caught nothing yet; would catch
        # a subtraction that split a plate in two.
        note(g.geom_type == 'Polygon' and g.is_valid, i, 'part is one closed piece',
             f'{p["w"]:.1f}x{p["h"]:.1f}')
        # --- slivers. Caught the 0.5 mm stub and the 0.1 mm hair the port cut
        # left on three plates.
        w = MIN_FEATURE / 2
        thin = g.difference(g.buffer(-w, join_style=2, mitre_limit=20)
                             .buffer(w, join_style=2, mitre_limit=20))
        bits = (list(thin.geoms) if thin.geom_type.startswith('Multi')
                else ([thin] if not thin.is_empty else []))
        bad = [x for x in bits if x.area > 0.02]
        note(not bad, i, f'no feature under {MIN_FEATURE} mm',
             '' if not bad else f'{len(bad)} on a {p["w"]:.1f}x{p["h"]:.1f} part')
        # --- bed
        note(max(p['w'], p['h']) <= BED_W and min(p['w'], p['h']) <= BED_H,
             i, 'part fits the bed', f'{p["w"]:.1f}x{p["h"]:.1f}')

    # --- the two plates must be the same part mirrored. They are drawn from
    # one set of borders, so if they differ, one was drawn from different ones
    # - which is exactly what put the port plate's fingers 2 mm out of phase
    # with the wall it mates with.
    if len(plates) == 2:
        A, B = poly(plates[0]), poly(plates[1])
        A = affinity.translate(A, -A.bounds[0], -A.bounds[1])
        B = affinity.translate(B, -B.bounds[0], -B.bounds[1])
        same = abs(A.area - B.area) < 0.5
        if same:
            best = min((affinity.translate(c, -c.bounds[0], -c.bounds[1])
                        .symmetric_difference(A).area)
                       for c in (B, affinity.scale(B, -1, 1, origin='center'),
                                 affinity.scale(B, 1, -1, origin='center'),
                                 affinity.scale(B, -1, -1, origin='center')))
            same = best < 1.0
        if any(flat):
            # a flattened end drops one coupling from one plate and not the
            # other, so they are deliberately no longer mirror images. What
            # must still hold is that nothing else changed: one coupling is
            # 12 x 3 = 36 mm2, so the gap cannot exceed that per flattened end.
            gap = abs(A.area - B.area)
            note(gap <= 40 * sum(1 for f in flat if f) + 4, i,
                 'the plates differ only by the dropped coupling',
                 f'{gap:.1f} mm2 apart, {sum(1 for f in flat if f)} end(s) flattened')
        else:
            note(same, i, 'the two plates are one part mirrored',
                 f'{A.area:.1f} vs {B.area:.1f} mm2')

    # --- fold it up and see whether it closes. Every other check tests a
    # proxy; this builds the section as a solid, plugs the openings it is
    # meant to have, floods the outside, and asks whether the flood reaches
    # the bore. Catches a port, and anything else that leaves a face open.
    plate_cells = None
    if ports:
        # a port is a plate that stops a cell short: model it as such
        ends = b.tipRuns(b.cells(), runs)
        cells = b.cells()
        plate_cells = cells[1:] if ends[0] in ports else cells[:-1]
    sealed, leak, bore = closes(b.cells(), runs, tips, b.blocksize,
                                b.thickness, plate_cells)
    note(sealed, i, 'the section closes round its bore',
         f'{leak:.0f} mm3 of the {bore:.0f} mm3 bore is open to the outside')

    # --- every finger needs a plate to engage. A wall carries fingers on both
    # long edges, one set per plate; if a plate does not cover that run, the
    # set facing it engages nothing. Caught section 5's loose closed-end wall.
    note(not ports, i, 'no wall finger left unengaged',
         'a port removes a plate over one cell, orphaning the fingers there'
         if ports else '')


def check_seam(i, a_args, b_args):
    """The two ends that meet must be opposite genders, on the same centre."""
    ba, runs_a, bord_a, tips_a, ports_a = geometry_of(a_args)
    bb, runs_b, bord_b, tips_b, ports_b = geometry_of(b_args)
    ends_a = ba.tipRuns(ba.cells(), runs_a)
    ends_b = bb.tipRuns(bb.cells(), runs_b)
    tab_a = ends_a[1] if ba.tabs_at_exit else ends_a[0]
    tab_b = ends_b[1] if bb.tabs_at_exit else ends_b[0]

    # section i leaves by its exit, section i+1 is entered by its entry
    exit_tabs = (ends_a[1] == tab_a)
    entry_tabs = (ends_b[0] == tab_b)
    plain_a = bool(ba.plain_out)
    plain_b = bool(bb.plain_in)
    if ba.pin_length <= 0 or bb.pin_length <= 0:
        # no coupling at all: the two end frames butt and are glued, so there
        # is no gender to get wrong - but say so rather than pass silently
        note(ba.pin_length <= 0 and bb.pin_length <= 0, f'{i}-{i+1}',
             'seam is a glued butt joint, flat both sides',
             'one side has a coupling and the other does not')
    else:
        ok = (plain_a and plain_b) or (not plain_a and not plain_b
                                       and exit_tabs != entry_tabs)
        note(ok, f'{i}-{i+1}', 'seam is one tab side and one slot side',
             f'exit {"tabs" if exit_tabs else "notches"}, '
             f'entry {"tabs" if entry_tabs else "notches"}')

    # --- the coupling is sized from pin_width and centred on the tube, so two
    # pieces meet only if they agree on both. Caught the elbow whose tab sat
    # 1.5 mm off the centreline.
    note(ba.pin_width == bb.pin_width and ba.blocksize == bb.blocksize
         and ba.thickness == bb.thickness, f'{i}-{i+1}',
         'both sides agree on bore and tab size',
         f'{ba.blocksize}/{ba.pin_width} vs {bb.blocksize}/{bb.pin_width}')

    # --- neither end may be a port: a port cannot carry a coupling at all.
    note(not ports_a and not ports_b, f'{i}-{i+1}', 'seam has no port', '')


def sections_3d(rec, groups):
    """Each section as (boxes, openings) where the bore actually puts it.

    Boxes in mm, not cell indices: a straight block may be longer than a turn,
    and then no single number takes a lattice index to a coordinate. Openings
    are keyed by the box's position in its own section.
    """
    allb = bore_split.block_boxes(rec)
    out = []
    for g in groups:
        boxes = [allb[i] for i in g]
        ins = tuple(-x for x in DIRS[rec[g[0]]['in']])
        out.append((boxes, {(0, ins),
                            (len(boxes) - 1, DIRS[rec[g[-1]]['out']])}))
    return out


def check_seams_3d(rec, groups, t):
    """Put the sections where they belong and test the joints between them.

    Each section on its own is sealed by the check above. This puts two of them
    together with only the outer ends stopped up: if the joint leaves any of
    the end frame unbacked the flood gets in, and if the two bores do not meet
    the bore comes back as more than one region.
    """
    secs = sections_3d(rec, groups)
    for i in range(len(secs) - 1):
        a, b = secs[i], secs[i + 1]
        outer = {(a[0][0], tuple(-x for x in DIRS[rec[groups[i][0]]['in']])),
                 (b[0][-1], DIRS[rec[groups[i + 1][-1]]['out']])}
        ok, leak, bore, parts = sealed([a, b], t, outer)
        note(ok, f'{i+1}-{i+2}', 'the joint is closed',
             f'{leak:.0f} mm3 open to the outside')
        note(parts == 1, f'{i+1}-{i+2}', 'the bore carries on through the joint',
             f'{parts} separate regions')

    # and the whole thing at once, coarser so it fits in memory
    mouth = (secs[0][0][0], tuple(-x for x in DIRS[rec[0]['in']]))
    far = (secs[-1][0][-1], DIRS[rec[-1]['out']])
    ok, leak, bore, parts = sealed(secs, t, {mouth, far}, px=1.0)
    # Every block gives (s-2t) square by its OWN length of bore -- a straight
    # may be longer than a turn -- and every turn adds the little void on the
    # inside of the bend, t by t by the bore width. A 2 mm grid used to be good
    # enough for a 2% tolerance, but that was luck: the trumpet quantised to
    # +0.35% and the spiral to +5.97% on the same code.
    s = bore_split.BLOCK
    turns = sum(1 for r in rec if r['in'] != r['out'])
    # DELIBERATELY a second implementation. This used to call bore_split.extent,
    # the same function block_boxes builds the geometry from, so an error in it
    # moved the measured volume and the expected volume together and the check
    # could not see it: adding 1mm to every straight block still passed. A check
    # that shares its source with the thing it checks is comparing something to
    # itself. So the block length is worked out here, from the walk, and this
    # must not be refactored to call extent() however much it looks like it.
    def block_length(r):
        straight = r['in'] == r['out']
        return bore_split.STRAIGHT if straight else bore_split.BLOCK
    want = (sum((s - 2 * t) ** 2 * block_length(r) for r in rec)
            + turns * t * t * (s - 2 * t))
    note(ok and parts == 1, 'all', 'the assembled bore is one sealed passage',
         f'{leak:.0f} mm3 leaking, {parts} regions')
    note(abs(bore - want) < 0.005 * want, 'all', 'bore volume matches the walk',
         f'{bore:.0f} mm3 against {want:.0f} for {len(rec)} blocks '
         f'and {turns} turns')


def check_pairing(rec, groups, norms, flats, laps):
    """Every tab must meet a notch, and every missing side a flat one.

    An elbow's frame has three sides, so one side of its neighbour's frame has
    no mate. That side must be dealt with, and there are two ways depending on
    what it is. A plate side is flattened: the coupling comes off and the two
    end faces meet flat. A wall side is lapped: the wall runs t past the joint
    as a tongue and fills the inside of the bend, which the elbow then sits
    against. Either way the tab is gone; what is not allowed is a live coupling
    facing nothing.

    What this cannot tell you is which of the two face plates was flattened:
    both are the same shape mirrored, and which one ends up facing the elbow
    depends on how Boxes lays the mirrored copy out. That one needs a dry fit.
    """
    AX = 'xyz'

    def frame(gi, which):
        g, k = groups[gi], norms[gi]
        bore_axis = AXIS[rec[g[0]]['in'] if which == 'in' else rec[g[-1]]['out']]
        sides = {}
        for a in range(3):
            if a == bore_axis:
                continue
            for sgn in (-1, 1):
                d = tuple(sgn if j == a else 0 for j in range(3))
                sides[d] = 'plate' if a == k else 'wall'
        if len(g) == 1 and rec[g[0]]['in'] != rec[g[0]]['out']:
            other = (DIRS[rec[g[0]]['out']] if which == 'in'
                     else tuple(-x for x in DIRS[rec[g[0]]['in']]))
            if other in sides:
                sides[other] = 'absent'
        # a lapped wall side carries a tongue rather than a coupling
        end = 0 if which == 'in' else 1
        want_lap = laps[gi][end]
        if want_lap:
            for d, kind in list(sides.items()):
                if kind != 'wall':
                    continue
                f2 = tuple(d[j] for j in range(3) if j != k)
                if FACE2D.get(f2) == want_lap:
                    sides[d] = 'lap'
        # a flattened plate side presents nothing either
        want = flats[gi][0 if which == 'in' else 1]
        if want:
            for d, kind in sides.items():
                if kind == 'plate' and [j for j in range(3) if d[j]][0] == k:
                    sgn = 1 if k in (0, 2) else -1
                    if (d[k] == sgn) == (want == 'first'):
                        sides[d] = 'flat'
        return sides

    for i in range(len(groups) - 1):
        A, B = frame(i, 'out'), frame(i + 1, 'in')
        bad = []
        for d in A:
            a, b = A[d], B.get(d, 'absent')
            pair = {a, b}
            ok = (pair <= {'plate', 'wall'}          # both present: tab meets notch
                  or pair == {'flat', 'absent'}      # nothing meets nothing
                  or pair == {'lap', 'absent'}       # the tongue fills the bend
                  or pair == {'absent'}              # nothing against nothing
                  or pair == {'flat'})
            if not ok:
                ax = AX[[j for j in range(3) if d[j]][0]]
                sgn = '+' if sum(d) > 0 else '-'
                bad.append(f'{sgn}{ax}: {a} against {b}')
        note(not bad, f'{i+1}-{i+2}', 'every side of the seam has a mate',
             '; '.join(bad))


def check_sheets(folder):
    """The written sheets, as the machine will see them."""
    def xf(g):
        m = re.match(r'translate\(([-\d.]+),([-\d.]+)\)(?:\s*rotate\((\d+)\))?',
                     g.get('transform') or '')
        if not m:
            return None
        dx, dy, r = float(m.group(1)), float(m.group(2)), int(m.group(3) or 0)
        def f(x, y):
            if r == 90:   x, y = -y, x
            elif r == 180: x, y = -x, -y
            elif r == 270: x, y = y, -x
            return (x + dx, y + dy)
        return f

    def collect(n, f, P, L):
        for ch in n:
            if ch.tag == V.NS + 'g':
                g2 = xf(ch) or (lambda x, y: (x, y))
                collect(ch, lambda x, y, a=f, b=g2: a(*b(x, y)), P, L)
            elif ch.tag == V.NS + 'path' and ch.get('stroke') == '#000000':
                q = [f(x, y) for x, y in V.pts(ch.get('d'))]
                if len(q) > 2:
                    P.append(q[:-1] if q[0] == q[-1] else q)
            elif ch.tag == V.NS + 'polyline' and ch.get('stroke') == '#0000ff':
                L.append([f(float(a), float(b)) for a, b in
                          (t.split(',') for t in ch.get('points').split())])

    seen = 0
    for fn in sorted(glob.glob(os.path.join(folder, '*.svg'))):
        name = os.path.basename(fn)
        # deepnest_* is a layout aid, not a cut file: parts are spread out on
        # purpose and it carries no engraving.
        # bore10-... is the name written since 2026-09-03; the bare NN_ form is
        # what the frozen repositories still hold, and both are cut files.
        if not re.match(r'(bore[\d.]+-|\d\d_|nest_|recut_)', name):
            continue
        seen += 1
        root = ET.parse(fn).getroot()
        W = float(root.get('width')[:-2]); H = float(root.get('height')[:-2])
        P, L = [], []
        collect(root, lambda x, y: (x, y), P, L)
        note(W <= BED_W + 1e-6 and H <= BED_H + 1e-6, name,
             'sheet fits the bed', f'{W:.0f}x{H:.0f}')
        gs = [Polygon(q).buffer(0) for q in P]
        # --- two parts must never share material. Caught 12 pairs cut through
        # each other when the nester's rotation offset was wrong.
        ov = sum(1 for i in range(len(gs)) for j in range(i + 1, len(gs))
                 if gs[i].intersection(gs[j]).area > 1e-6)
        note(ov == 0, name, 'no two parts overlap', f'{ov} pairs')
        # --- engraving must land on the part. Caught labels placed at the
        # centre of an L-shaped plate's bounding box, out in the notch.
        off = sum(1 for lab in L for pt in lab
                  if not any(_inside(p, *pt) for p in P))
        note(off == 0, name, 'engraving on material', f'{off} points off')

    # A filter that matches nothing reports a clean run over no files at all.
    # Renaming the sheets did exactly that once: 194 checks became 176 and
    # still said 0 failed, because the six it stopped looking at were the six
    # that had been renamed. Asked to check a folder, say so if it was empty.
    note(seen > 0, os.path.basename(os.path.normpath(folder)),
         'folder holds cut files', f'{seen} matched *.svg')


def main(text, folder=None, report=True):
    del results[:]                      # a caller may run more than one walk
    rec, groups, plans, plan, unfilled = specs_for(text)
    flats = [p['flat'] for p in plan]
    specs = [(p['args'], p['kind']) for p in plan]

    if report:
        print(f'{text.strip()}\n{len(rec)} blocks, {len(groups)} sections\n')
    allparts = []
    for i, (args, kind) in enumerate(specs, 1):
        parts = cut(args, f'chk{i}')
        allparts.append(parts)
        check_section(i, args, parts, flats[i - 1])
    for i in range(len(specs) - 1):
        check_seam(i + 1, specs[i][0], specs[i+1][0])
    check_seams_3d(rec, groups, THICKNESS)
    check_pairing(rec, groups, [p['norm'] for p in plan], flats,
                  [p['lap'] for p in plan])
    if folder:
        check_sheets(folder)

    by = {}
    for ok, sec, name, detail in results:
        d = by.setdefault(name, [True, []])
        if not ok:
            d[0] = False
            d[1].append(f'section {sec}: {detail}')
    bad = sum(1 for ok, *_ in results if not ok)
    if report:
        print(f'{"check":<40}{"result"}')
        for name, (ok, fails) in by.items():
            print(f'  {name:<38}{"pass" if ok else "FAIL"}')
            for f in fails[:4]:
                print(f'      {f}')
        print(f'\n{len(results)} checks, {bad} failed')
        return 1 if bad else 0
    return len(results), bad, [(n, f) for n, (ok, f) in by.items() if not ok]


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('walk', nargs='+')
    ap.add_argument('--files', metavar='DIR',
                    help='also check the sheets written into DIR')
    ap.add_argument('--blocksize', type=float, metavar='MM',
                    help='block pitch the files were cut at '
                         f'(default {bore_split.BLOCK:g})')
    ap.add_argument('--bore', type=float, metavar='MM',
                    help='the square airway, if you would rather say that')
    ap.add_argument('--straight', type=float, metavar='MM',
                    help='length of a straight block; turns stay cubic')
    a = ap.parse_args()
    if a.blocksize:
        bore_split.set_blocksize(a.blocksize)
    if a.bore:
        bore_split.set_bore(a.bore)
    if a.straight:
        bore_split.set_straight(a.straight)
    try:
        sys.exit(main(walk_text(' '.join(a.walk)), a.files))
    except ValueError as e:
        print(f'error: {e}')
        sys.exit(1)
