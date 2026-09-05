#!/usr/bin/env python3
"""One page holding every coil, with a control to switch between them.

    python3 coils.py            # writes ../coils.html

The four coils are the same walk truncated at its N spacers, so comparing them
means comparing lengths of one thing rather than four designs. Four separate
pages make that awkward: you lose the camera every time you open another. Here
they share a viewer and a control swaps the cells.

This is not a second viewer. It calls viewer.build_many, which is the same
template and the same drawing code the per-coil pages use -- a gallery is one
set or several, not a fork.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bore_split as B                                        # noqa: E402
import viewer                                                 # noqa: E402

# label, walk file. Order is shortest first, which is also the order they
# truncate in, so the control reads as a length rather than a list of designs.
COILS = [
    ('¾',  'walks/coil-0.75t.txt'),
    ('1½',  'walks/coil-1.5t.txt'),
    ('2¼',  'walks/coil-2.25t.txt'),
    ('3',   'walks/coil-3t.txt'),
]
TITLE = '10x10x30 Coils'


def main(out):
    here = os.path.dirname(os.path.abspath(__file__))
    # the cells carry real millimetres, so the geometry has to be set before
    # any walk is read -- a gallery built at the default block would draw
    # cubes and quietly disagree with every cut file in the repository
    B.set_bore(10)
    B.set_straight(30)
    items = [(lab, open(os.path.join(here, f)).read().strip())
             for lab, f in COILS]
    open(out, 'w').write(viewer.build_many(items, TITLE))
    print(f'  {os.path.basename(out)}')
    for lab, text in items:
        rec, groups, _, _, _ = B.specs_for(B.walk_text(text))
        mm = sum(B.extent(r, B.AXIS[r['out']]) for r in rec)
        print(f'    {lab:>2} turns   {len(rec):>2} blocks  {mm:>5g}mm  '
              f'{len(groups):>2} sections')


if __name__ == '__main__':
    a = sys.argv[1:]
    main(a[0] if a else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), '..', 'coils.html'))
