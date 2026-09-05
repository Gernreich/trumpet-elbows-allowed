#!/usr/bin/env python3
"""Run the pre-cut gate over every design in this repository.

    python3 regress.py            # every design
    python3 regress.py coil       # those whose name matches

The frozen corpus in ../../bore-generator gates the frozen toolchain and knows
nothing about this fork: it has no stretched lattice in it, so it cannot catch a
change to `bore_split.py` here. Each design is otherwise gated only when it is
written, which means a change to tools/ is proved against whichever design you
happened to rebuild and no other. This runs both.

Every design carries the switches it is cut with. Passing the wrong ones gates a
design nobody is cutting -- --files never looks at the pitch, so it will not
notice.
"""
import json
import os
import re
import subprocess
import sys

# (name, walk file, folder of cut files, switches it is cut with)
DESIGNS = [
    # Truncations of one coil at its N spacers. A turn is four circuit terms and
    # an N lands every three, so the reachable lengths are multiples of 3/4:
    # 0.75, 1.5, 2.25, 3, all four of which are built. Parts are cut from the 1.5t, so a change that moves
    # it needs asking about.
    #
    # The 1.5t is also ../trumpet-switchback's walk exactly. It arrived here as
    # a borrowed test before anyone noticed it was a coil.
    ('coil 10x10x30 0.75t', 'walks/coil-0.75t.txt', '../coil-10x10x30-0.75t',
     ['--bore=10', '--straight=30']),
    ('coil 10x10x30 1.5t', 'walks/coil-1.5t.txt', '../coil-10x10x30-1.5t',
     ['--bore=10', '--straight=30']),
    ('coil 10x10x30 2.25t', 'walks/coil-2.25t.txt', '../coil-10x10x30-2.25t',
     ['--bore=10', '--straight=30']),
    # WUED repeated with an N spacer every three terms: a square circuit in
    # cross-section that steps north. The first walk laid out for this lattice.
    ('coil 10x10x30 3t', 'walks/coil-3t.txt', '../coil-10x10x30-3t',
     ['--bore=10', '--straight=30']),
]


def check_page(here, folder, text, switches):
    """The 3D page beside the cut files, which nothing else looks at.

    check.py reads sheets and geometry and never opens the viewer, so a render
    can be wrong while every check passes -- it has been. The page drew a
    uniform lattice for a stretched one, and 392 checks said nothing, because
    the mistake was in what the page was told rather than in any part. Only a
    screenshot caught it.

    This does not judge the picture. It asserts the page was handed the walk it
    sits beside: the block count and the centreline it prints. A page built from
    a stale walk, or before a geometry change, fails here.
    """
    import bore_split as B
    for sw in switches:
        k, _, v = sw.lstrip('-').partition('=')
        {'bore': B.set_bore, 'straight': B.set_straight,
         'blocksize': B.set_blocksize}[k](v)
    rec, groups, _, _, _ = B.specs_for(B.walk_text(text))
    want_blocks = len(rec)
    want_mm = round(sum(B.extent(r, B.AXIS[r['out']]) for r in rec))

    d = os.path.join(here, folder)
    pages = [f for f in os.listdir(d) if f.endswith('.html')]
    if len(pages) != 1:
        return f'{len(pages)} html pages in {folder}, expected 1'
    body = open(os.path.join(d, pages[0])).read()
    m = re.search(r'const SETS = (\[.*?\]);\n', body, re.S)
    if not m:
        return f'{pages[0]}: no SETS block -- not a viewer page?'
    sets = json.loads(m.group(1))
    if len(sets) != 1:
        return f'{pages[0]}: {len(sets)} coils in a per-coil page'
    d0 = sets[0]['d']
    if d0['blocks'] != want_blocks:
        return (f'{pages[0]}: page says {d0["blocks"]} blocks, '
                f'the walk has {want_blocks}')
    if d0['mm'] != want_mm:
        return (f'{pages[0]}: page says {d0["mm"]}mm of centreline, '
                f'the walk gives {want_mm}')
    if sets[0]['walk'] != text.strip():
        return f'{pages[0]}: the page carries a different walk'
    return None


def main(pattern=None):
    here = os.path.dirname(os.path.abspath(__file__))
    bad = 0
    for name, walk, folder, switches in DESIGNS:
        if pattern and pattern.lower() not in name.lower():
            continue
        text = open(os.path.join(here, walk)).read().strip()
        args = [sys.executable, 'check.py', text] + switches
        if folder and os.path.isdir(os.path.join(here, folder)):
            args += ['--files', folder]
        r = subprocess.run(args, cwd=here, capture_output=True, text=True)
        last = (r.stdout.strip().splitlines() or ['no output'])[-1]
        page = check_page(here, folder, text, switches) if folder else None
        ok = r.returncode == 0 and '0 failed' in last and page is None
        bad += not ok
        print(f'  {"pass" if ok else "FAIL"}  {name:<16} {last}'
              + ('' if page is None else '   + page'))
        if page:
            print(f'          page: {page}')
        if not ok:
            for line in r.stdout.splitlines():
                if 'FAIL' in line or line.startswith('      '):
                    print(f'          {line.strip()}')
            if r.stderr.strip():
                print(f'          {r.stderr.strip().splitlines()[-1]}')
    print(f'\n  {bad} design(s) failing' if bad else '\n  all designs pass')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else None))
