"""Fold a section up in three dimensions and see whether it closes.

Every other check tests a proxy - part counts, edge lengths, tooth spacing.
This builds the section as a solid and asks the question directly: is the bore
sealed everywhere except at the openings it is supposed to have?

The parts are placed as the box actually assembles: the two face plates fill
the piece's outline over the material's thickness at each face, set in wherever
a wall sits; the walls stand between them, a thickness wide, along every
boundary run that is not an opening. Then the openings are plugged and the
outside is flooded. If the flood reaches the bore, something is missing.
"""
import numpy as np
from scipy import ndimage
from shapely.geometry import box
from shapely.ops import unary_union

PX = 1.0            # voxel, mm


def band(v, d, n, s, depth, outward):
    """Axis-aligned band along a boundary run, `depth` mm to one side.

    Walking the outline counter-clockwise the interior is on the left, so the
    outward normal is to the right of travel: (dy, -dx).
    """
    o = (d[1], -d[0]) if outward else (-d[1], d[0])
    ax, ay = v[0] * s, v[1] * s
    bx, by = (v[0] + d[0] * n) * s, (v[1] + d[1] * n) * s
    cx, cy = ax + o[0] * depth, ay + o[1] * depth
    dx, dy = bx + o[0] * depth, by + o[1] * depth
    xs = (ax, bx, cx, dx); ys = (ay, by, cy, dy)
    return box(min(xs), min(ys), max(xs), max(ys))


def regions(cells, runs, tips, s, t, plate_cells=None):
    """The 2D footprint of the plates and of the walls, in mm.

    A plate covers its whole face and a wall stands the full depth of the tube.
    They overlap in the corner columns, which is what the finger joints are:
    modelling the plate as set in there instead leaves a band of air the real
    box does not have, and every section reads as leaking.

    plate_cells, if given, is the set of cells one plate covers - a port is a
    plate that stops short of a cell, and this is how that gets tested.
    """
    solid = unary_union([box(i * s, j * s, (i + 1) * s, (j + 1) * s)
                         for i, j in cells])
    walls = []
    for k, (v, d, n) in enumerate(runs):
        if k in tips:
            continue
        walls.append(band(v, d, n, s, t, outward=False).intersection(solid))
    near = solid
    far = solid if plate_cells is None else unary_union(
        [box(i * s, j * s, (i + 1) * s, (j + 1) * s) for i, j in plate_cells])
    return near, far, unary_union(walls) if walls else None, solid


def voxels(region, nx, ny, ox, oy):
    if region is None or region.is_empty:
        return np.zeros((ny, nx), bool)
    import shapely
    gx, gy = np.meshgrid((np.arange(nx) + 0.5) * PX + ox,
                         (np.arange(ny) + 0.5) * PX + oy)
    return shapely.contains_xy(region, gx.ravel(), gy.ravel()).reshape(ny, nx)


def closes(cells, runs, tips, s, t, plate_cells=None):
    """(sealed, leak_volume_mm3, bore_volume_mm3) for one section."""
    near, far, walls, solid = regions(cells, runs, tips, s, t, plate_cells)
    minx, miny, maxx, maxy = solid.bounds
    M = 3.0
    ox, oy = minx - M, miny - M
    nx = int((maxx - minx + 2 * M) / PX)
    ny = int((maxy - miny + 2 * M) / PX)
    nz = int(s / PX)

    N = voxels(near, nx, ny, ox, oy)
    F = voxels(far, nx, ny, ox, oy)
    W = voxels(walls, nx, ny, ox, oy)
    S = voxels(solid, nx, ny, ox, oy)

    mat = np.zeros((nz, ny, nx), bool)
    ti = int(round(t / PX))
    mat[:ti] = N                       # one face plate
    mat[-ti:] = F                      # the other
    mat |= W[None, :, :]               # walls, the full depth of the tube

    # plug every opening: a slab just outside that run, full depth, so the
    # flood can only get in somewhere it should not
    plug = np.zeros_like(mat)
    for k in tips:
        v, d, n = runs[k]
        outside = band(v, d, n, s, M, outward=True).difference(solid)
        plug |= voxels(outside, nx, ny, ox, oy)[None, :, :]

    free = ~(mat | plug)
    lab, _ = ndimage.label(free)
    outside = set(np.unique(np.concatenate([
        lab[0].ravel(), lab[-1].ravel(), lab[:, 0].ravel(), lab[:, -1].ravel(),
        lab[:, :, 0].ravel(), lab[:, :, -1].ravel()]))) - {0}

    bore = (~mat) & S[None, :, :]
    leaked = bore & np.isin(lab, list(outside))
    v = PX ** 3
    return (not leaked.any(), leaked.sum() * v, bore.sum() * v)


# --------------------------------------------------------- in world terms

def build(sections, t, px=1.0):
    """Voxel the given sections where the bore actually puts them.

    A section is (boxes, openings): boxes as real (lo, hi) mm pairs, one per
    block, openings as (box index, direction). Every face of every box that no
    other box in the section covers carries a slab of material a thickness deep
    - the plate on the two faces out of the piece's plane, the wall on the rest
    - except at an opening, which is left as the hole it is.

    Boxes rather than cell indices because a straight block may be longer than
    a turn, and then no single number takes a lattice index to a coordinate.
    Coverage is measured between the boxes themselves, so a face that a
    neighbour only partly covers is walled over the rest of itself rather than
    all-or-nothing.

    Returns (material, bore, grid origin, shape).
    """
    allb = [b for boxes, _ in sections for b in boxes]
    lo = [min(b[0][i] for b in allb) for i in range(3)]
    hi = [max(b[1][i] for b in allb) for i in range(3)]
    M = 2.0 * max(t, 1.0) * 8                    # margin in mm, room to flood
    org = [lo[i] - M for i in range(3)]
    shape = [int(round((hi[i] - lo[i] + 2 * M) / px)) for i in range(3)]
    mat = np.zeros(shape[::-1], bool)
    bore = np.zeros(shape[::-1], bool)

    def sl(a, b, axis):
        """voxel slice of the mm interval [a, b] along `axis`"""
        return slice(int(round((a - org[axis]) / px)),
                     int(round((b - org[axis]) / px)))

    def region(box):
        return [sl(box[0][i], box[1][i], i) for i in range(3)]

    for boxes, openings in sections:
        for n, box in enumerate(boxes):
            r = region(box)
            bore[r[2], r[1], r[0]] = True
            for axis in range(3):
                for sign in (-1, 1):
                    d = tuple(sign if i == axis else 0 for i in range(3))
                    if (n, d) in openings:
                        continue
                    # the slab just inside this face
                    face = list(r)
                    face[axis] = (sl(box[0][axis], box[0][axis] + t, axis)
                                  if sign < 0 else
                                  sl(box[1][axis] - t, box[1][axis], axis))
                    slab = np.zeros(shape[::-1], bool)
                    slab[face[2], face[1], face[0]] = True
                    # take away whatever another box of this section covers
                    for m, other in enumerate(boxes):
                        if m == n:
                            continue
                        touches = (abs(other[1][axis] - box[0][axis]) < 1e-9
                                   if sign < 0 else
                                   abs(other[0][axis] - box[1][axis]) < 1e-9)
                        if not touches:
                            continue
                        o = region(other)
                        cover = list(face)
                        for i in range(3):
                            if i == axis:
                                continue
                            a0 = max(box[0][i], other[0][i])
                            a1 = min(box[1][i], other[1][i])
                            if a1 - a0 <= 1e-9:
                                cover = None
                                break
                            cover[i] = sl(a0, a1, i)
                        if cover is not None:
                            slab[cover[2], cover[1], cover[0]] = False
                    mat |= slab
    return mat, bore & ~mat, org, shape


def sealed(sections, t, plug, px=1.0):
    """Is the bore reachable from outside anywhere but through `plug`?

    `plug` is the openings that are meant to be open - the two ends of what is
    being tested. They are stopped up, the outside is flooded, and anything of
    the bore the flood reaches is a leak. Each entry is (box, direction), the
    box in mm, so a plug fits whatever size that block actually is.
    """
    mat, bore, org, shape = build(sections, t, px)
    stop = mat.copy()

    def sl(a, b, axis):
        return slice(int(round((a - org[axis]) / px)),
                     int(round((b - org[axis]) / px)))

    for box, d in plug:
        axis = [i for i in range(3) if d[i]][0]
        sign = d[axis]
        box3 = [sl(box[0][i], box[1][i], i) for i in range(3)]
        box3[axis] = (sl(box[0][axis] - t, box[0][axis], axis) if sign < 0
                      else sl(box[1][axis], box[1][axis] + t, axis))
        stop[box3[2], box3[1], box3[0]] = True

    lab, _ = ndimage.label(~stop)
    edge = set(np.unique(np.concatenate([
        lab[0].ravel(), lab[-1].ravel(), lab[:, 0].ravel(), lab[:, -1].ravel(),
        lab[:, :, 0].ravel(), lab[:, :, -1].ravel()]))) - {0}
    leak = bore & np.isin(lab, list(edge))
    parts, n = ndimage.label(bore)
    return (not leak.any(), leak.sum() * px ** 3, bore.sum() * px ** 3, n)
