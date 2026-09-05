#!/usr/bin/env python3
"""One page holding a design at every block size it is cut at.

    python3 sizes.py trumpet_switchback /tmp/sizes.html

Nothing here is cut at two sizes any more - trumpet-switchback's 25mm folder
went on 2026-09-03 - so this writes no page that is kept. It is left in place
because the next design cut at two pitches will want it, and because the
locked-scale drawing it does is the only way to see one.

A design cut at two sizes is one shape and two pitches: the walk, the sections
and the shapes are identical, and only the millimetres differ. Two pages make
that hard to see, because you lose the camera opening the second one. Here they
share a viewer and a control swaps the pitch.

Not a second viewer -- viewer.build_many is the same template and the same
drawing code a per-size page goes through, with more than one set in it.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bore_split as B                                        # noqa: E402
import viewer                                                 # noqa: E402

# label, the --blocksize it is cut at. The default first: it is the one the
# design is drawn for, and the other reads as a variation on it. That order
# flipped on 2026-09-05, when 10mm became the default and 25mm was retired.
SIZES = [('10mm', 16.0), ('25mm', 31.0)]


def main(walk_name, out):
    here = os.path.dirname(os.path.abspath(__file__))
    text = open(os.path.join(here, 'walks', walk_name + '.txt')).read().strip()
    sets, stats = [], []
    for label, block in SIZES:
        # Build each set while its own size is set. data_for reads BLOCK when it
        # runs, so collecting walks and building them later hands every set the
        # last size -- which is exactly what happened: both came out at 16mm and
        # one was labelled 682mm regardless.
        B.set_blocksize(block)
        sets.append((label, text, viewer.data_for(text)))
        rec, groups, _, _, _ = B.specs_for(B.walk_text(text))
        stats.append((label, block, len(rec), len(rec) * block, len(groups)))
    title = walk_name.replace('_', ' ').title() + ' Bore'
    open(out, 'w').write(viewer.build_sets(sets, title))
    print(f'  {os.path.basename(out)}')
    for label, block, n, mm, g in stats:
        print(f'    {label:>5}  block {block:g}mm  {n} blocks  {mm:g}mm  {g} sections')


if __name__ == '__main__':
    a = sys.argv[1:]
    if len(a) != 2:
        sys.exit('usage: sizes.py WALK_NAME OUT.html')
    main(a[0], a[1])
