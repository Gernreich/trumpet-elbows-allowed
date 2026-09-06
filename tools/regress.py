"""Run the pre-cut gate over every design we have built or tried.

Designs diverge more than they look: the first trumpet never called the lap
code, so a crash in it went unseen until a spiral needed one, and a volume
check that passed the trumpet at +0.35% was 6% out on a spiral of identical
construction. A change is only known good against the whole set.

    python3 regress.py            # every design
    python3 regress.py trumpet    # those whose name matches

Two lattices, one gate. UNIFORM designs sit on a cubic block; STRETCHED ones
run their straights longer than their turns. They were gated by two forked
copies of this toolchain until 2026-09-05, which meant a change to bore_split
was only ever proved against whichever half you happened to be standing in.
The fork is gone; this runs both.

Every design carries the switches it is cut with. Passing the wrong ones gates
a design nobody is cutting -- --files never looks at the pitch, so it will not
notice.
"""
import json
import os
import re
import subprocess
import sys

# (name, walk or walks/*.txt, folder of cut files or None[, block pitch mm])
UNIFORM = [
    ('first trumpet', 'N N10 U2 W2 S7 U2 E4 N9 W2 D2 N4 N',
     '../parts/bore/concept/walk/elbows/first_trumpet'),
    # No folder: ../parts/bore/concept/walk/elbows/spiral_trumpet holds a page and two photographs and
    # no cut files at all. It was named here until 2026-09-03, and check.py
    # globbed nothing, added no sheet checks and still reported 0 failed - the
    # design looked covered for as long as nobody counted. check.py now fails a
    # folder it was pointed at and found empty, which is how this surfaced.
    ('spiral trumpet', 'U U3 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U3 U',
     None),
    ('helix, rise 2', 'N N4 U2 E4 U2 S4 U2 W4 U2 N4 N', None),
    ('helix, rise 1', 'N N4 U1 E4 U1 S4 U1 W4 U1 N4 N', None),
    ('helix, side 6', 'N N6 U2 E6 U2 S6 U2 W6 U2 N6 U2 E6 E', None),
    ('test bore', 'U U2 E2 S2 U2 U', None),
    ('three blocks', 'W D3 E4 N', None),
    # corners in a row. A leg of one block makes its block a corner, so these
    # are chains of touching corners - the case that used to raise rather than
    # cut, because the lap was named in the walk's frame and an elbow is drawn
    # in a canonical one.
    ('4 corners, flat', 'N N2 U1 N1 U1 N2 N', None),
    ('4 corners, solid', 'N N2 U1 E1 S1 E4 E', None),
    ('5 corners, solid', 'N N2 U1 E1 S1 W1 S3 S', None),
    ('6 corners, solid', 'N N2 E1 U1 E1 U1 E1 U3 U', None),
    # coils tight enough to touch themselves. The second used to be refused by
    # the generator: a piece came back alongside its own blocks, so one cell
    # had three neighbours and it was no longer a snake.
    ('tightest coil', 'U U3 N1 E1 S1 U1 W1 U1 N1 E1 S1 U1 W1 U1 '
                      'N1 E1 S1 U1 W1 U1 U', None),
    ('touching coil', 'N N2 E1 S2 U1 W1 U1 N2 E1 S2 U1 W1 U1 '
                      'N2 E1 S2 U1 W1 U1 U', None),
    # a space-filling curve: every cell of a 2x2x2 used, so it touches itself
    # everywhere it can. python3 hilbert.py 1 prints it.
    ('hilbert cube 1', 'S S1 U1 N1 E1 S1 D1 N1', None),
    # the same curve at scale 2: the open knot rather than the solid block.
    # Too long to sit in this list, so it is kept beside it.
    ('hilbert cube 2', 'walks/hilbert_cube.txt', None),
    # the same 4x4x4 filled corner to opposite corner instead, so the mouth and
    # the bell are as far apart as the box allows
    ('corner to corner', 'walks/corner_to_corner.txt', None),
    # the knot with the bell taken out under it and up past the top, so the
    # two ends are at opposite corners of the whole thing
    ('hilbert snorkel', 'walks/hilbert_snorkel.txt', None),
    # a piece spiralling inward touches its own arms at a corner, which the
    # generator refuses as a pinch. This one used to raise.
    ('double spiral', 'N N4 W4 S3 E3 N2 W2 U2 E2 N3 W3 S4 E4 E', None),
    ('metre spring', 'walks/metre_spring.txt', None),
    # a folded run that doubles back twice inside a 4x4 cross-section, cut
    # short at both ends to leave room for the mouthpiece and the bell. Four
    # of its six sections come out as one of two shapes, so it is the set's
    # check that duplicates still get their own section number engraved.
    # On a 16mm block - 10mm of air inside 3mm walls. The only design here not
    # cut at the stock pitch, so it is the one thing keeping --blocksize
    # honest: everything scaled with the block except SnakeBox's 12mm tab,
    # which does not fit a 10mm frame. Its 25mm twin was retired with
    # trumpet-switchback's 25mm folder on 2026-09-03.
    ('trumpet switchback',
     'walks/trumpet_switchback.txt',
     '../parts/bore/concept/walk/no-elbows/switchback/10mm/bore', 16),
    # The elbow-free walks. Every design above either contains elbows or is too
    # small to be interesting, so nothing was checking that a long walk still
    # splits without one - the property every build is chosen for.
    # 190 blocks and 27 pieces, no elbows: the open Hilbert knot is the largest
    # elbow-free walk here by a factor of three, and gates 1010 checks.
    ('hilbert open', 'walks/hilbert_open.txt', '../parts/bore/concept/walk/no-elbows/hilbert_open'),
    # a telescope whose legs grow 4, 5, 6 ... so every turn is a fold and the
    # risers between loops are the only thing leaving the plane
    ('wide telescope', 'walks/wide_telescope.txt', '../parts/bore/concept/walk/no-elbows/wide_telescope'),
    # A flat meander -- the Greek key wound all the way in and brought back out
    # beside itself. 68 blocks that split into ONE piece, so it has no section
    # seam at all. '4 corners, flat' is single-piece too, but at 8 blocks; this
    # one exercises that path at a size where it matters, and is the only design
    # here whose cut files run to two sheets.
    ('greek spiral', 'walks/greek_spiral.txt',
     '../parts/bore/concept/walk/no-elbows/greek-spiral/bore', ['--bore=10']),
    # two-piece walks that turn: the smallest cases where a bend has to carry
    # its corner internally rather than strand it
    ('three block turn', 'walks/three_block_turn.txt', None),
    ('hook check', 'walks/hook_check.txt', None),
]

# (name, walk file, folder of cut files, switches it is cut with)
STRETCHED = [
    # Truncations of one coil at its N spacers. A turn is four circuit terms and
    # an N lands every three, so the reachable lengths are multiples of 3/4:
    # 0.75, 1.5, 2.25, 3, all four of which are built. Parts are cut from the 1.5t, so a change that moves
    # it needs asking about.
    #
    # The 1.5t is also ../parts/bore/concept/walk/no-elbows/switchback's walk exactly. It arrived here as
    # a borrowed test before anyone noticed it was a coil.
    ('coil 10x10x30 0.75t', 'walks/coil-0.75t.txt', '../parts/bore/concept/walk/no-elbows/stretched/coil-10x10x30-0.75t',
     ['--bore=10', '--straight=30']),
    ('coil 10x10x30 1.5t', 'walks/coil-1.5t.txt', '../parts/bore/concept/walk/no-elbows/stretched/coil-10x10x30-1.5t',
     ['--bore=10', '--straight=30']),
    ('coil 10x10x30 2.25t', 'walks/coil-2.25t.txt', '../parts/bore/concept/walk/no-elbows/stretched/coil-10x10x30-2.25t',
     ['--bore=10', '--straight=30']),
    # WUED repeated with an N spacer every three terms: a square circuit in
    # cross-section that steps north. The first walk laid out for this lattice.
    ('coil 10x10x30 3t', 'walks/coil-3t.txt', '../parts/bore/built/coil-10x10x30-3t',
     ['--bore=10', '--straight=30']),
]

DESIGNS = UNIFORM + STRETCHED

# The stock pitch, read once at import and before any design has moved it.
_DEFAULT_BLOCK = 16.0


def _norm(entry):
    """Both tables as (name, walk, folder, switches).

    UNIFORM's optional fourth field is a bare block pitch in mm, from before
    designs carried switch lists. Kept as it is so the table reads unchanged.
    """
    name, walk, folder, *rest = entry
    if not rest:
        return name, walk, folder, []
    sw = rest[0]
    if isinstance(sw, list):
        return name, walk, folder, sw
    return name, walk, folder, [f'--blocksize={sw}']


def walk_of(spec, here):
    """A design is a walk, or the name of a file holding one."""
    if spec.endswith('.txt'):
        return open(os.path.join(here, spec)).read().strip()
    return spec


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
    # bore_split keeps the pitch in module globals and set_blocksize only
    # moves STRAIGHT along with BLOCK while the block is still cubic. Once a
    # stretched design has set STRAIGHT away from BLOCK, a later design's
    # set_blocksize leaves it there -- so the pitch leaks forward across the
    # corpus. It never showed while the two lattices were gated separately and
    # every design in a run shared its switches. Reset to the cubic default
    # first, explicitly, so each design is measured on its own lattice.
    B.STRAIGHT = B.BLOCK
    B.set_blocksize(_DEFAULT_BLOCK)
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
    for entry in DESIGNS:
        name, walk, folder, switches = _norm(entry)
        if pattern and pattern.lower() not in name.lower():
            continue
        text = walk_of(walk, here)
        args = [sys.executable, 'check.py', text] + switches
        if folder:
            # A NAMED FOLDER THAT IS NOT THERE IS A FAILURE, not a skip. This
            # used to fall through to a geometry-only run: when
            # trumpet-switchback's 25mm folder was deleted, this design went
            # from 195 checks to 176 and still said "pass". check.py's own
            # guard cannot help - it only fires on a folder that exists and is
            # empty, and this one had stopped existing.
            #
            # The stretched fork dropped this guard and skipped instead. It got
            # away with it only because check_page below then crashed on the
            # missing directory - a traceback standing in for a check.
            if not os.path.isdir(os.path.join(here, folder)):
                print(f'  FAIL  {name:<18} names {folder}, which is not there')
                bad += 1
                continue
            args += ['--files', folder]
        r = subprocess.run(args, cwd=here, capture_output=True, text=True)
        last = (r.stdout.strip().splitlines() or ['no output'])[-1]
        page = check_page(here, folder, text, switches) if folder else None
        ok = r.returncode == 0 and '0 failed' in last and page is None
        bad += not ok
        print(f'  {"pass" if ok else "FAIL"}  {name:<18} {last}'
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
