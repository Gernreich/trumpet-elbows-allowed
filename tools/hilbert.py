"""Write a Hilbert cube as a walk in the bore notation.

A Hilbert curve visits every cell of a 2^n cube exactly once, in unit steps,
which makes it the densest bore the notation can describe: no cell of the box
is wasted. Order 1 is 8 blocks in a 2x2x2, order 2 is 64 in a 4x4x4, order 3 is
512 in an 8x8x8.

It is also the most expensive to cut. A space-filling curve turns almost every
block, and a block that turns is a corner, so an order-2 cube comes out as 63
sections for 64 blocks.

    python3 hilbert.py 2                    # print the walk
    python3 hilbert.py 2 --split            # and how it cuts
"""
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bore_split import BLOCK, DIRS, walk, touching              # noqa: E402

NAME = {v: k for k, v in DIRS.items()}


def transpose_to_axes(X, bits):
    """Skilling's transform, the inverse of the Hilbert index.

    Gray-decode, then undo the excess work the encoding does, swapping and
    reflecting the sub-cubes back into place.
    """
    N = 2 << (bits - 1)
    t = X[2] >> 1
    for i in range(2, 0, -1):
        X[i] ^= X[i - 1]
    X[0] ^= t
    Q = 2
    while Q != N:
        P = Q - 1
        for i in range(2, -1, -1):
            if X[i] & Q:
                X[0] ^= P
            else:
                t = (X[0] ^ X[i]) & P
                X[0] ^= t
                X[i] ^= t
        Q <<= 1
    return X


def cube(order):
    """The cells of a 3D Hilbert curve of this order, in order."""
    pts = []
    for h in range(1 << (3 * order)):
        X = [0, 0, 0]
        for i in range(3 * order):
            b = (h >> (3 * order - 1 - i)) & 1
            X[i % 3] |= b << (order - 1 - i // 3)
        pts.append(tuple(transpose_to_axes(X, order)))
    check(pts, order)
    return pts


def check(pts, order):
    """The curve must be a path before it can be a bore."""
    n = 1 << order
    if len(set(pts)) != len(pts):
        raise ValueError('the curve visits a cell twice')
    if any(not (0 <= c < n) for p in pts for c in p):
        raise ValueError(f'the curve leaves the {n}x{n}x{n} cube')
    for a, b in zip(pts, pts[1:]):
        if sum(abs(b[k] - a[k]) for k in range(3)) != 1:
            raise ValueError(f'the curve jumps from {a} to {b}')


def to_walk(pts, scale=1):
    """Cells to notation: run-length encode the steps.

    At scale 1 every step is one block, so the curve fills its box solid and
    you cannot see it. At scale 2 each step is two blocks, which leaves an
    empty block between parallel runs - the curve as it is usually drawn, a
    tube you can see through. It also puts a straight block between every pair
    of turns, which is what stops the corners meeting each other.
    """
    steps = [NAME[tuple(b[k] - a[k] for k in range(3))]
             for a, b in zip(pts, pts[1:])]
    steps = [d for d in steps for _ in range(scale)]
    runs, cur, n = [], steps[0], 0
    for s in steps:
        if s == cur:
            n += 1
        else:
            runs.append((cur, n))
            cur, n = s, 1
    runs.append((cur, n))
    return runs[0][0] + ' ' + ' '.join(f'{d}{k}' for d, k in runs)


def main(order, split=False, scale=1):
    pts = cube(order)
    text = to_walk(pts, scale)
    rec = walk(text)
    n = ((1 << order) - 1) * scale + 1
    # BLOCK, not a second copy of the pitch: this printed millimetres at a pitch
    # bore_split had stopped cutting at, and nothing compared the two.
    vol = (n * BLOCK) ** 3 / 1e6
    print(text)
    print(f'\n  order {order} at scale {scale}: {len(rec)} blocks, '
          f'{len(rec)*BLOCK:g} mm of centreline in a {n*BLOCK:g} mm cube')
    print(f'  {len(rec)*BLOCK/vol:.0f} mm of bore per litre, '
          f'{len(touching(rec))} pairs of blocks touching')
    if split:
        import bore_split
        bore_split.main(text, None)


if __name__ == '__main__':
    argv = sys.argv[1:]
    scale = 1
    if '--scale' in argv:
        i = argv.index('--scale')
        scale = int(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    a = [x for x in argv if x != '--split']
    try:
        main(int(a[0]) if a else 2, '--split' in argv, scale)
    except (ValueError, IndexError) as e:
        print(f'error: {e}')
        sys.exit(1)
