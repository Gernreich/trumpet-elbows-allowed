"""Open-ended tube following a polyomino snake, one unit cube per cell.

Both ends are open. The end frame is a square annulus of width `thickness`:
the two face plates cover the middle of the top and bottom sides, the two
side walls cover the full left and right sides. That partition is only
2-fold symmetric, but the annulus *region* is 4-fold symmetric, so one tab
centred on each of the four sides mates with the matching notches in any of
the four rotations. One end carries the four tabs, the other the notches.
"""

import copy
import math

from boxes import *


class SnakeBoxVar(Boxes):
    """Open-ended tube in the shape of a polyomino snake"""

    ui_group = "Misc"

    def __init__(self):
        Boxes.__init__(self)

        self.addSettingsArgs(edges.FingerJointSettings)

        self.argparser.add_argument(
            "--widths_u", action="store", type=str, default="",
            help="comma-separated cell widths along the plate's first axis, "
                 "one per column, lowest index first. Empty means every "
                 "column is --blocksize. A snake whose straights are longer "
                 "than its turns has columns of two widths, and only the "
                 "caller knows which is which, because it depends on the "
                 "plane the piece lies in and which way the bore runs in it.")
        self.argparser.add_argument(
            "--widths_v", action="store", type=str, default="",
            help="the same along the plate's second axis.")
        self.argparser.add_argument(
            "--origin_u", action="store", type=int, default=0,
            help="lattice index of the first entry in --widths_u.")
        self.argparser.add_argument(
            "--origin_v", action="store", type=int, default=0,
            help="lattice index of the first entry in --widths_v.")
        self.argparser.add_argument(
            "--blocksize", action="store", type=float, default=40.,
            help="edge length of one cube (in mm)(used for x, y and z alike)")
        self.argparser.add_argument(
            "--path", action="store", type=str, default="RRRRRRR",
            help="cell-to-cell moves as U/D/L/R (7 moves = 8 cells); "
                 "empty for a single cell, which needs --open_faces")
        self.argparser.add_argument(
            "--open_faces", action="store", type=str, default="",
            help="single-cell pieces only: which two faces are open, as two of "
                 "N/S/E/W (e.g. S,E for an elbow). Longer paths take their two "
                 "ends from the path itself.")
        self.argparser.add_argument(
            "--pin_width", action="store", type=float, default=12.,
            help="width of each connecting tab (in mm)")
        self.argparser.add_argument(
            "--pin_length", action="store", type=float, default=3.,
            help="how far each tab sticks out (in mm)")
        self.argparser.add_argument(
            "--pin_play", action="store", type=float, default=0.0,
            help="extra width of the notches, per side (in mm) "
                 "(0 matches the finger joints, which carry no designed play)")
        self.argparser.add_argument(
            "--port_in", action="store_true",
            help="the bore ENTERS through a hole in a face plate instead of "
                 "the rim, so that end of the snake is closed off. Lets a run "
                 "whose cells are coplanar be one piece even where the bore "
                 "arrives perpendicular to it")
        self.argparser.add_argument(
            "--port_out", action="store_true",
            help="same as --port_in, at the far end")
        self.argparser.add_argument(
            "--flat_in", action="store", type=str, default="",
            help="drop the coupling from ONE face plate at the entry: 'first' "
                 "or 'mirror'. For the side that meets an elbow's missing "
                 "face, where a tab has nothing to enter and a notch nothing "
                 "to fill")
        self.argparser.add_argument(
            "--flat_out", action="store", type=str, default="",
            help="same as --flat_in, at the exit opening")
        self.argparser.add_argument(
            "--plain_in", action="store_true",
            help="no tabs or notches on the ENTRY opening: a plain edge that "
                 "butts and is glued, which is what meets a port")
        self.argparser.add_argument(
            "--plain_out", action="store_true",
            help="same as --plain_in, at the exit opening")
        self.argparser.add_argument(
            "--port_mirror", action="store_true",
            help="cut the port in the mirrored plate rather than the first "
                 "one, i.e. in the other face of the piece")
        self.argparser.add_argument(
            "--tabs_at_exit", action="store_true",
            help="put the tabs on the exit opening rather than the entry; "
                 "needed when the piece before this one couples to a port")
        self.argparser.add_argument(
            "--lap_in", action="store", type=str, default="",
            help="face (N/S/E/W) whose wall runs t past the ENTRY opening as a "
                 "full-width tongue instead of a tab; fills the inside of a "
                 "bend when this piece sits next to an elbow")
        self.argparser.add_argument(
            "--lap_out", action="store", type=str, default="",
            help="same as --lap_in, at the EXIT opening")
        self.argparser.add_argument(
            "--pin_seat", action="store", type=float, default=0.2,
            help="extra depth cut into each notch (in mm); clearance so the "
                 "tab cannot bottom out before the two end faces meet")

    # ----------------------------------------------------------- the shape

    def cells(self):
        step = {'R': (1, 0), 'L': (-1, 0), 'U': (0, 1), 'D': (0, -1)}
        pos, out = (0, 0), [(0, 0)]
        for i, c in enumerate(self.path.strip().upper()):
            if c not in step:
                raise ValueError(
                    f"move {i+1} is {c!r}; only U, D, L and R are allowed")
            pos = (pos[0] + step[c][0], pos[1] + step[c][1])
            if pos in out:
                raise ValueError(f"the path revisits cell {pos} at move {i+1}")
            out.append(pos)
        if len(out) < 2 and not self.open_faces.strip():
            raise ValueError(
                "need at least one move, or --open_faces to say which two "
                "faces of a single cell are open")
        return out

    def outline(self, cells):
        """Boundary runs as (start vertex, direction, steps), plus turns."""
        S = set(cells)
        nxt = {}
        for (i, j) in cells:
            for present, a, b in (
                    ((i, j - 1), (i, j), (i + 1, j)),
                    ((i + 1, j), (i + 1, j), (i + 1, j + 1)),
                    ((i, j + 1), (i + 1, j + 1), (i, j + 1)),
                    ((i - 1, j), (i, j + 1), (i, j))):
                if present not in S:
                    if a in nxt:
                        raise ValueError(
                            f"the shape pinches at {a}: two cells meet only "
                            "at a corner, so it cannot be built as a tube")
                    nxt[a] = b

        start = min(nxt, key=lambda v: (v[1], v[0]))
        walk, v = [start], nxt[start]
        while v != start:
            walk.append(v)
            v = nxt[v]
        if len(walk) != len(nxt):
            raise ValueError("the shape encloses a hole")

        runs = []
        for k in range(len(walk)):
            a, b = walk[k], walk[(k + 1) % len(walk)]
            d = (max(-1, min(1, b[0] - a[0])), max(-1, min(1, b[1] - a[1])))
            n = abs(b[0] - a[0]) + abs(b[1] - a[1])
            if runs and runs[-1][1] == d:
                runs[-1][2] += n
            else:
                runs.append([a, d, n])
        if len(runs) > 1 and runs[0][1] == runs[-1][1]:
            last = runs.pop()
            runs[0] = [last[0], runs[0][1], runs[0][2] + last[2]]

        turns = [runs[k][1][0] * runs[(k + 1) % len(runs)][1][1] -
                 runs[k][1][1] * runs[(k + 1) % len(runs)][1][0]
                 for k in range(len(runs))]
        assert sum(turns) == 4, f"turning number {sum(turns)}, expected 4"
        return runs, turns

    def tipRuns(self, cells, runs):
        """Indices of the two open runs, the first of which gets the tabs."""
        faces = self.open_faces.strip().upper().replace(',', ' ').split()
        if faces:
            if len(cells) != 1:
                raise ValueError(
                    "--open_faces is for a single cell only; with a path the "
                    "two ends are set by where the path ends")
            if len(faces) != 2:
                raise ValueError(
                    f"--open_faces wants exactly two of N/S/E/W, got {faces}")
            # walking counterclockwise, the outward normal is to the right
            normal = {(1, 0): 'S', (0, 1): 'E', (-1, 0): 'N', (0, -1): 'W'}
            byface = {normal[tuple(d)]: k for k, (v, d, n) in enumerate(runs)}
            out = []
            for f in faces:
                if f not in byface:
                    raise ValueError(f"unknown face {f!r}; use N, S, E or W")
                out.append(byface[f])
            if out[0] == out[1]:
                raise ValueError("the two open faces must be different")
            return out

        S = set(cells)
        adj = {c: [n for n in ((c[0]+1, c[1]), (c[0]-1, c[1]),
                               (c[0], c[1]+1), (c[0], c[1]-1)) if n in S]
               for c in cells}
        ends = [c for c in cells if len(adj[c]) == 1]
        if len(ends) != 2:
            raise ValueError(f"not a simple snake: {len(ends)} loose ends")
        ends.sort(key=lambda c: cells.index(c))  # path start first

        out = []
        for e in ends:
            i, j = e
            d = (e[0] - adj[e][0][0], e[1] - adj[e][0][1])
            cap = {(1, 0): ((i+1, j), (0, 1)),
                   (-1, 0): ((i, j+1), (0, -1)),
                   (0, 1): ((i+1, j+1), (-1, 0)),
                   (0, -1): ((i, j), (1, 0))}[d]
            for k, (v, dd, n) in enumerate(runs):
                if v == cap[0] and dd == cap[1] and n == 1:
                    out.append(k)
                    break
            else:
                raise AssertionError(f"no cap run found for end cell {e}")
        return out

    # ----------------------------------------------------- the cell's sides
    # A snake piece is FLAT, so a cell has two in-plane pitches and one
    # out-of-plane size. For a cube all three are --blocksize and nothing here
    # changes. For a cuboid the caller has to say which is which, because only
    # it knows what plane this piece lies in: the same 16x16x30 cell is
    # 16 x 16 in plan with a 30 height for a piece in the XY plane, and
    # 16 x 30 in plan with a 16 height for one in XZ.
    def widths(self, axis):
        """{lattice index: width} for one in-plane axis."""
        raw = (self.widths_u if axis == 0 else self.widths_v).strip()
        if not raw:
            return {}
        o = self.origin_u if axis == 0 else self.origin_v
        return {o + i: float(w) for i, w in enumerate(raw.split(','))}

    def cubic(self):
        return not self.widths(0) and not self.widths(1)

    def span(self, start, d, n):
        """Real length of a boundary run: the widths of the n cells it
        crosses, not n times one pitch.

        A run starts at lattice VERTEX `start` and steps n cells in direction
        d. Going forward it crosses cells start .. start+n-1; going backward it
        crosses start-1 .. start-n, because a vertex is the low corner of the
        cell above it.
        """
        axis = 0 if d[1] == 0 else 1
        w = self.widths(axis)
        if not w:
            return n * self.blocksize
        step = d[axis]
        i0 = start[axis]
        idx = ([i0 + k for k in range(n)] if step > 0
               else [i0 - 1 - k for k in range(n)])
        missing = [i for i in idx if i not in w]
        if missing:
            raise ValueError(
                f"--widths_{'uv'[axis]} has no width for column(s) {missing}; "
                f"it covers {sorted(w)}")
        return sum(w[i] for i in idx)

    def geometry(self):
        """Border lengths/angles and the two open (cap) runs."""
        t, s = self.thickness, self.blocksize
        cells = self.cells()
        runs, turns = self.outline(cells)
        ends = self.tipRuns(cells, runs)
        ports = ([ends[0]] if self.port_in else []) + \
                ([ends[1]] if self.port_out else [])
        tips = [k for k in ends if k not in ports]
        if not tips:
            raise ValueError("both ends cannot be ports: the piece would have "
                             "no rim opening to couple by")
        # which opening carries the tabs; a port can only ever offer slots
        tab_run = ends[1] if self.tabs_at_exit else ends[0]
        plains = set()
        if self.plain_in:
            plains.add(ends[0])
        if self.plain_out:
            plains.add(ends[1])

        # A plate edge is set in by the material thickness only where a wall
        # actually sits; along an open face there is no wall, so the plate runs
        # flush out to the true boundary. Each run therefore loses its
        # *neighbour's* inset at each end, signed by the corner.
        #
        # Deriving it this way rather than special-casing the caps is what lets
        # the two open faces be adjacent, as they are on a single-cell elbow.
        # The old rule left such a polygon 2t short in both axes.
        n = len(runs)
        inset = [0.0 if k in tips else t for k in range(n)]
        borders = []
        caps = {}
        for k in range(n):
            head = turns[k - 1] * inset[(k - 1) % n]
            tail = turns[k] * inset[(k + 1) % n]
            full = self.span(runs[k][0], runs[k][1], runs[k][2])
            borders.append(full - head - tail)
            borders.append(90 * turns[k])
            if k in tips:
                # how much of the full run this edge lost at its start, and the
                # run's full length, so the tab can be centred on the tube
                caps[k] = (head, full)
        return borders, tips, caps, ports, tab_run, plains

    # ------------------------------------------------------------- drawing

    def portMark(self, edge_length):
        """Magenta outline of the cell to cut away from this plate.

        Drawn at the start of the closed end's border, so x runs along that
        edge and y into the piece. Not a cut: bore_split subtracts this region
        from the plate and drops the marker.
        """
        s, t = self.blocksize, self.thickness
        if not self.cubic():
            raise ValueError(
                "a port marker on a non-cubic cell is not implemented: the "
                "hole is square in one pitch and the cell is not.")
        # the plate's edge here is set in by t, because a wall sits along it,
        # so the cell starts t before this border rather than on it
        self.rectangularHole(edge_length / 2., s / 2. - t, edge_length + 4 * t,
                             s, color=Color.MAGENTA)

    def lapRuns(self, runs, ends, ports=()):
        """{(run, 'l'|'r')} for wall ends that run past an opening as a tongue.

        A tab on the side facing an elbow's missing face meets nothing. Making
        that whole side run t further instead fills the inside of the bend, and
        the next piece's end face lands flat on it.
        """
        normal = {(1, 0): 'S', (0, 1): 'E', (-1, 0): 'N', (0, -1): 'W'}
        laps = set()
        n = len(runs)
        for tip, face in ((ends[0], self.lap_in), (ends[1], self.lap_out)):
            face = (face or "").strip().upper()
            if not face or tip in ports:
                continue
            for k in ((tip - 1) % n, (tip + 1) % n):
                if k in ends:
                    continue          # the neighbouring run is the other opening
                if normal[tuple(runs[k][1])] == face:
                    laps.add((k, 'l' if (k - 1) % n == tip else 'r'))
                    break
            else:
                raise ValueError(
                    f"no wall faces {face} at that opening; the lap must name "
                    f"one of the two walls beside it")
        return laps

    def lapProfile(self, length):
        """A tongue t deep across the whole opening, in place of a tab.

        Only as wide as the wall itself: the finger teeth must not run on past
        the joint plane or they would foul the next piece's plates.
        """
        t = self.thickness
        self.corner(-90)
        self.edge(t)
        self.corner(90)
        self.edge(length)
        self.corner(90)
        self.edge(t)
        self.corner(-90)

    def endProfile(self, length, pin):
        """A centred tab (pin=True) or notch (pin=False) across `length`."""
        if self.pin_length <= 0:
            # No coupling: a plain end. Drawing the profile anyway would leave
            # zero-width spikes, because polygonWall's corner correction eats a
            # thickness at each -90 turn and the zero-length sides go negative.
            self.edge(length)
            return
        w = self.pin_width + (0. if pin else 2. * self.pin_play)
        # A notch is cut deeper than the tab is long. With equal depths the
        # tab bottoms out exactly as the faces meet, so char at the notch
        # floor or a thick sheet holds the seam open.
        p = self.pin_length + (0. if pin else self.pin_seat)
        a = (length - w) / 2.
        if a < 0:
            raise ValueError(
                f"pin_width {self.pin_width} is too wide for the "
                f"{length:.1f}mm end frame")
        d = -1 if pin else 1
        self.edge(a)
        self.corner(d * 90)
        self.edge(p)
        self.corner(-d * 90)
        self.edge(w)
        self.corner(-d * 90)
        self.edge(p)
        self.corner(d * 90)
        self.edge(a)

    def plateBorders(self, borders, tips, caps, tab_run, plains=(), short=()):
        """Borders with the cap runs replaced by a tab / notch profile.

        `short` names port runs this plate must stop a cell short of. The
        plate is only as wide as the bore - it sits between the side walls -
        so a bore-sized hole through it would sever it. Ending it a cell early
        instead leaves that cell's face open, bounded by the piece's own walls,
        with the plate never cut through.
        """
        out, edgetypes = [], []
        w_pin = self.pin_width
        w_slot = self.pin_width + 2. * self.pin_play
        n = len(borders) // 2
        for k in range(n):
            length, angle = borders[2 * k], borders[2 * k + 1]
            # A run beside the removed cell loses that cell, but gains back
            # the thickness it was set in by: the new edge has no wall behind
            # it, so it runs out flush to the cell boundary.
            if short and not self.cubic():
                raise ValueError(
                    "a port on a non-cubic cell is not implemented: the run "
                    "beside the removed cell gives back one pitch and this "
                    "does not yet know which. Use a cubic cell, or no port.")
            length -= ((self.blocksize - self.thickness)
                       * sum(1 for j in ((k - 1) % n, (k + 1) % n) if j in short))
            if k in short:
                # the open end of the shortened plate: no wall behind it
                out += [length, angle]
                edgetypes += ["e"]
            elif k in tips and (self.pin_length <= 0 or k in plains):
                # plain open end, no coupling
                out += [length, angle]
                edgetypes += ["e"]
            elif k in tips:
                pin = (k == tab_run)
                w = w_pin if pin else w_slot
                p = self.pin_length + (0. if pin else self.pin_seat)
                # Centre the tab on the TUBE, not on this edge. A plate edge is
                # inset by t only where a wall sits, so on an elbow - whose two
                # openings share a corner - the edge is shorter at one end only.
                # Centring on the edge would put the tab t/2 off the centreline
                # and it would miss the notch it has to meet.
                head, full = caps[k]
                a = full / 2. - head - w / 2.
                a2 = length - w - a
                if a < 0 or a2 < 0:
                    raise ValueError(
                        f"pin_width {self.pin_width} is too wide for the "
                        f"{length:.1f}mm end frame")
                d = -1 if pin else 1
                out += [a, d * 90, p, -d * 90, w, -d * 90, p, d * 90, a2, angle]
                edgetypes += ["e"] * 5
            else:
                out += [length, angle]
                edgetypes += ["f"]
        return out, edgetypes

    def snakeWalls(self, borders, tips, laps=(), tab_run=None, plains=()):
        """Walls for every run except the two caps, with pin/slot ends."""
        t = self.thickness
        h = self.blocksize - 2 * t
        n = len(borders) // 2
        bottom = top = self.edges["F"]
        plain = self.edges["e"]

        lset = copy.deepcopy(self.edges["f"].settings)
        lf, lF, lh = lset.edgeObjects(self, add=False)
        rset = copy.deepcopy(self.edges["f"].settings)
        rf, rF, rh = rset.edgeObjects(self, add=False)

        self.moveTo(0, bottom.margin())
        for k in range(n):
            if k in tips:
                continue
            length = borders[2 * k]
            a_before, a_after = borders[2 * k - 1], borders[2 * k + 1]
            # a cap run on either side leaves that end of this wall open
            l_open = (k - 1) % n in tips
            r_open = (k + 1) % n in tips
            # tips[0] is the tab end, tips[1] the notch end
            l_pin = ((k - 1) % n) == tab_run
            r_pin = ((k + 1) % n) == tab_run
            l_lap = (k, 'l') in laps
            r_lap = (k, 'r') in laps

            left = plain if l_open else lf
            right = plain if r_open else rF

            if not l_open:
                lset.setValues(t, angle=a_before)
                if a_before < 0:
                    length -= t * math.tan(math.radians(-a_before / 2))
            if not r_open:
                rset.setValues(t, angle=a_after)
                if a_after < 0:
                    length -= t * math.tan(math.radians(-a_after / 2))

            self.moveTo(left.spacing() + self.spacing +
                        (t if l_lap else
                         (self.pin_length if l_open and l_pin
                          and ((k - 1) % n) not in plains else 0.)), 0)
            with self.saved_context():
                bottom(length)
                self.edgeCorner(bottom, right, 90)
                if r_lap:
                    self.lapProfile(h)
                elif r_open and ((k + 1) % n) not in plains:
                    self.endProfile(h, r_pin)
                else:
                    right(h)
                self.edgeCorner(right, top, 90)
                top(length)
                self.edgeCorner(top, left, 90)
                if l_lap:
                    self.lapProfile(h)
                elif l_open and ((k - 1) % n) not in plains:
                    self.endProfile(h, l_pin)
                else:
                    left(h)
                self.edgeCorner(left, bottom, 90)
                self.ctx.stroke()
            self.moveTo(length + right.spacing() + self.spacing +
                        (t if r_lap else
                         (self.pin_length if r_open and r_pin
                          and ((k + 1) % n) not in plains else 0.)))

    # -------------------------------------------------------------- render

    def render(self):
        # Every path is emitted black: a single cut stage. Nothing is drawn in
        # Boxes.py's green ETCHING colour, because in these repositories colour
        # is the cut order (green cuts before black) and an etched cell
        # division would be sliced clean through the plate.
        borders, tips, caps, ports, tab_run, plains = self.geometry()
        pb, pe = self.plateBorders(borders, tips, caps, tab_run, plains)
        runs, _ = self.outline(self.cells())

        ends = self.tipRuns(self.cells(), runs)

        # At a port ONE plate must stop a cell short, leaving that cell's face
        # open. It is drawn full size and marked instead of being drawn short:
        # Boxes spaces finger joints to fit each edge, so a plate drawn 31 mm
        # shorter gets its own spacing and no longer meshes with the wall it
        # has to mate with. The marker is a magenta rectangle over the cell to
        # remove; bore_split subtracts it, which leaves every surviving edge
        # exactly as it was drawn, teeth included.
        seg, i = {}, 0
        for k in range(len(borders) // 2):
            seg[k] = i
            i += 5 if k in tips else 1
        port_seg = {seg[k]: borders[2 * k] for k in ports}

        def mark(i):
            if i in port_seg:
                self.portMark(port_seg[i])

        first = None if self.port_mirror else (mark if ports else None)
        second = (mark if ports else None) if self.port_mirror else None

        # One plate can lose its coupling at an end while the other keeps it -
        # for the side that meets an elbow's missing face, where a tab has
        # nothing to enter and a notch nothing to fill. The two plates stop
        # being mirror images, so they are drawn from separate border lists.
        fa = {e for e, w in ((ends[0], self.flat_in),
                             (ends[1], self.flat_out)) if w == 'first'}
        fb = {e for e, w in ((ends[0], self.flat_in),
                             (ends[1], self.flat_out)) if w == 'mirror'}
        ab, ae = (self.plateBorders(borders, tips, caps, tab_run,
                                    set(plains) | fa) if fa else (pb, pe))
        bb, be = (self.plateBorders(borders, tips, caps, tab_run,
                                    set(plains) | fb) if fb else (pb, pe))
        self.polygonWall(borders=ab, edge=ae, move="right", callback=first)
        self.polygonWall(borders=bb, edge=be, move="mirror right",
                         callback=second)
        self.snakeWalls(borders, tips, self.lapRuns(runs, ends, ports), tab_run,
                        plains)
