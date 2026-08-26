#!/usr/bin/env python3
"""Engrave a hex index inside each ring of a bell sheet, smallest ring 0.

    python3 number_rings.py SHEET.svg
    python3 number_rings.py SHEET.svg --cap=3.0          # smaller digits than the wall allows
    python3 number_rings.py SHEET.svg --order=document   # a profile that doubles back
    python3 number_rings.py SHEET.svg --start=26         # continue an existing stack

Eleven rings glued in the wrong order is eleven rings unglued, and once the parts are off
the bed nothing about a ring says where it belongs: consecutive rings differ by two
millimetres and the sheet is laid out smallest-first, which is exactly the order they stop
being in the moment someone picks them up. So each ring carries its own number.

SMALLEST FIRST ONLY WHERE THAT IS ALSO ASSEMBLY ORDER. A bell telescopes, so its rings
can only get bigger and the two orders agree. A mouthpiece narrows to the throat and opens
again, so its backbore and its cup pass through the same diameters and sorting by size
shuffles one into the other -- which is the one thing a number on a ring must never do.
When the sizes are not monotonic in the file this refuses to guess; --order=document
numbers them as written, which is assembly order for anything these generators produce.

HEX, ONE CHARACTER PER RING. A 67-ring bell in decimal needs two digits and twice the wall
to put them on; in hex it needs one up to ring 15 and never more than two. The count starts
at 0 so a ring's number is its index in the stack, not its position in a list.

BLUE ENGRAVES, and this is the repository's blue: #0000ff, its own stage, written before
the black that frees the part. Engraving after the cut is engraving on a part that has
already dropped.

THE DIGITS ARE STROKES, NOT TEXT. Seven-segment characters drawn as line paths, so nothing
depends on a font being installed, on text-to-path conversion, or on an importer's idea of
what <text> means. They are also single-stroke, which is what an engraver wants.

FITTING IS MEASURED, NOT ASSUMED. The wall here runs from 2.29mm on ring 0 to 9.15mm on the
rim, and the rim's aperture is nearly a circle, so there is no flat to sit a digit on and no
single size that suits both ends. Each digit is placed at the bottom of its ring and grown
to the largest size where EVERY point of it still lands between that ring's aperture and its
outer edge, measured against the real curves. A digit that cannot be fitted is reported and
nothing is written.
"""
import re
import sys
import math
import pathlib

BLUE   = "#0000ff"          # blue engraves
MARGIN = 0.30               # clear of both edges, mm
CAP    = 4.0                # no digit larger than this however wide the wall
GROUP  = "ring-numbers"     # replaced wholesale if the sheet already has one
NUM    = re.compile(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?")
TOK    = re.compile(r"([MmLlHhVvAaZz])([^A-Za-z]*)")

# a b c d e f g, clockwise from the top bar then the middle
SEG = {"0": "abcdef", "1": "bc",     "2": "abged",  "3": "abgcd",
       "4": "fgbc",   "5": "afgcd",  "6": "afgecd", "7": "abc",
       "8": "abcdefg", "9": "abcfgd", "A": "abcefg", "b": "fgecd",
       "C": "afed",   "d": "bgcde",  "E": "afged",  "F": "afge"}


def flatten(d, per=64):
    """Every subpath as a closed ring of points, arcs sampled and Z honoured."""
    out, cur = [], None
    x = y = sx = sy = 0.0
    for cmd, arg in TOK.findall(d):
        n = [float(v) for v in NUM.findall(arg)]
        if cmd == "M":
            if cur:
                out.append(cur)
            x, y = n[0], n[1]; sx, sy = x, y; cur = [(x, y)]
        # Straight runs are sampled, not just stepped to. A radius-by-angle lookup
        # interpolates between whatever points it is given, and a straight edge is not
        # straight in polar coordinates: with only its two ends, the bottom of a square
        # reads as its corner radius, 21.9mm instead of 15.5mm.
        elif cmd == "H":
            for v in n:
                cur += [(x + (v-x)*i/per, y) for i in range(1, per + 1)]; x = v
        elif cmd == "V":
            for v in n:
                cur += [(x, y + (v-y)*i/per) for i in range(1, per + 1)]; y = v
        elif cmd == "A":
            for i in range(0, len(n), 7):
                r, _, _, la, sw, ex, ey = n[i:i + 7]
                cur += arc(x, y, r, int(la), int(sw), ex, ey, per)
                x, y = ex, ey
        elif cmd == "Z":
            x, y = sx, sy
    if cur:
        out.append(cur)
    return [close(c) for c in out]


def arc(x0, y0, r, large, sweep, x1, y1, per):
    dx, dy = (x0 - x1)/2.0, (y0 - y1)/2.0
    lam = (dx*dx + dy*dy)/(r*r)
    if lam > 1:
        r *= math.sqrt(lam)
    k = math.sqrt(max((r*r*r*r - r*r*(dy*dy + dx*dx))/(r*r*(dy*dy + dx*dx)), 0.0))
    k *= -1 if large == sweep else 1
    cx, cy = k*r*dy/r + (x0 + x1)/2.0, -k*r*dx/r + (y0 + y1)/2.0
    a0 = math.atan2(y0 - cy, x0 - cx)
    a1 = math.atan2(y1 - cy, x1 - cx)
    da = a1 - a0
    if sweep and da < 0:
        da += 2*math.pi
    if not sweep and da > 0:
        da -= 2*math.pi
    return [(cx + r*math.cos(a0 + da*i/per), cy + r*math.sin(a0 + da*i/per))
            for i in range(1, per + 1)]


def close(pts, per=64):
    """Fill in the segment Z leaves implicit, or a sharp square is three sides and a gap."""
    ax, ay = pts[-1]; bx, by = pts[0]
    if math.hypot(bx - ax, by - ay) < 1e-9:
        return pts
    return pts + [(ax + (bx-ax)*i/per, ay + (by-ay)*i/per) for i in range(1, per)]


def polar(pts, cx, cy):
    return sorted((math.atan2(py - cy, px - cx), math.hypot(px - cx, py - cy))
                  for px, py in pts)


def radius(tab, th):
    lo, hi = 0, len(tab) - 1
    if th <= tab[0][0] or th >= tab[-1][0]:
        return (tab[0][1] + tab[-1][1])/2.0
    while hi - lo > 1:
        mid = (lo + hi)//2
        lo, hi = (mid, hi) if tab[mid][0] <= th else (lo, mid)
    (t0, r0), (t1, r1) = tab[lo], tab[hi]
    return r0 if t1 == t0 else r0 + (r1 - r0)*(th - t0)/(t1 - t0)


def digit(ch, x, y, w, h):
    """Seven-segment character, top-left at (x, y), as a list of line paths."""
    L, R, T, M, B = x, x + w, y, y + h/2.0, y + h
    seg = {"a": (L, T, R, T), "b": (R, T, R, M), "c": (R, M, R, B), "d": (L, B, R, B),
           "e": (L, M, L, B), "f": (L, T, L, M), "g": (L, M, R, M)}
    return [f"M {seg[s][0]:.4f},{seg[s][1]:.4f} L {seg[s][2]:.4f},{seg[s][3]:.4f}"
            for s in SEG[ch]]


def hexlabel(i):
    """Ring index as seven-segment hex.

    b and d are lower case and have to be: on seven segments an upper-case B is the same
    shape as 8 and an upper-case D is the same shape as 0, so a sheet numbered in capitals
    would have two pairs of rings wearing each other's labels."""
    return f"{i:X}".translate(str.maketrans("BD", "bd")) if i < 16 \
        else f"{i:02X}".translate(str.maketrans("BD", "bd"))


def paths_with_stroke(src):
    """(d, stroke) for every path, taking its colour from the element or its group."""
    out, gstroke, stack = [], None, []
    pat = re.compile(r'<g\b[^>]*>|</g\s*>|<path\b[^>]*>')
    hue = re.compile(r'stroke\s*[:=]\s*"?\s*(#[0-9a-fA-F]{6})')
    for m in pat.finditer(src):
        el = m.group(0)
        if el.startswith("</g"):
            gstroke = stack.pop() if stack else None
        elif el.startswith("<g"):
            stack.append(gstroke)
            c = hue.search(el)
            if c:
                gstroke = c.group(1).lower()
        else:
            d = re.search(r'\bd="([^"]+)"', el)
            if d:
                c = hue.search(el)
                out.append((d.group(1), c.group(1).lower() if c else gstroke))
    return out


def points_of(paths):
    out = []
    for p in paths:
        out += [(float(a), float(b)) for a, b in re.findall(r"(-?[\d.]+),(-?[\d.]+)", p)]
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    cap = float(opts.get("cap", CAP))
    # An extension sheet continues the stack it will be glued to, so its first ring is not
    # ring 0. Numbering it from 0 would put a second "0" in the same mouthpiece.
    start = int(opts.get("start", 0))
    for k in opts:
        if k not in ("cap", "order", "start"):
            sys.exit(f"  unknown option --{k}: cap, order or start")
    src_path = pathlib.Path(args[0])
    src = src_path.read_text()

    old = re.search(rf'\s*<g id="{GROUP}".*?</g>', src, re.S)
    if old:
        src = src[:old.start()] + src[old.end():]

    # A ring is two concentric outlines -- and so is an engraved 0, 4, 6, 8 or 9, because a
    # digit's counter is a subpath too. That is exactly how bell-section.py comes to read
    # 25 rings in the hand-labelled 17-ring sheet. Counting subpaths cannot tell them
    # apart; colour can, and in this repository it already does. Blue is engraving, never a
    # part, so blue paths are not candidates.
    elements = paths_with_stroke(src)
    existing = sum(1 for _, stroke in elements if stroke == BLUE)
    if existing:
        sys.exit(f"  {src_path.name} already carries {existing} engraved mark(s) in {BLUE}.\n"
                 f"  Numbering it would put a second set of labels on the same parts. Those\n"
                 f"  are hand-placed — clear them in Inkscape first if you want these instead.")

    # Refuse what cannot be read rather than misread it. A sheet saved out of Inkscape comes
    # back in relative commands and quadratic curves; this parser is built for what the
    # generators write, and a half-parsed outline would place a digit somewhere plausible
    # and wrong instead of failing.
    cuts = [d for d, _ in elements]
    unknown = sorted({c for d in cuts for c in re.findall(r"[A-Za-z]", d)} - set("MHVAZ"))
    if unknown:
        sys.exit(f"  {src_path.name} uses path commands this cannot read: {' '.join(unknown)}\n"
                 f"  Relative commands and curves mean the sheet has been through an editor.\n"
                 f"  The generators write M, H, V, A and Z only.")

    rings = []
    for d in cuts:
        sp = flatten(d)
        if len(sp) != 2:
            continue
        span = lambda p: max(max(q[0] for q in p) - min(q[0] for q in p),
                             max(q[1] for q in p) - min(q[1] for q in p))
        ap, out = sorted(sp, key=span)
        cx = (min(q[0] for q in out) + max(q[0] for q in out))/2.0
        cy = (min(q[1] for q in out) + max(q[1] for q in out))/2.0
        rings.append({"ap": polar(ap, cx, cy), "out": polar(out, cx, cy),
                      "cx": cx, "cy": cy, "o": span(out)})
    if not rings:
        sys.exit(f"  no rings in {src_path.name}")
    if len(rings) > 256:
        sys.exit(f"  {len(rings)} rings needs more than two hex characters")

    # A number is only worth engraving if it is the ASSEMBLY order. On a bell that is also
    # size order -- the thing telescopes, so it can only get bigger. A mouthpiece does not:
    # its bore narrows 25 -> 5 and then opens 3.66 -> 10.06, so the backbore and the cup
    # pass through the same diameters and sorting by size shuffles one into the other. Both
    # generators write their rings in assembly order, so when the sizes are not monotonic
    # the file's own order is the truth and sorting would destroy it.
    order = opts.get("order")
    spans = [r["o"] for r in rings]
    rising = all(b > a for a, b in zip(spans, spans[1:]))
    if order is None:
        order = "size" if rising else None
        if order is None:
            sys.exit("  these rings are not in size order in the file. A mouthpiece doubles\n"
                     "  back through the same diameters; a bell's flange ring is wider than the\n"
                     "  rings just above it; a hand-nested sheet can be in any order at all.\n"
                     "  Numbering by size would not be assembly order. Pass --order=document to\n"
                     "  number them as the file lists them, which IS assembly order for a\n"
                     "  generated sheet, or --order=size to confirm you meant smallest-first.")
    if order not in ("size", "document"):
        sys.exit(f"  --order: size or document, not {order!r}")
    if order == "size":
        rings.sort(key=lambda r: r["o"])

    marks, report, fails = [], [], []
    for i, r in enumerate(rings):
        label = hexlabel(start + i)
        down = math.pi/2                       # SVG y grows downward, so this is the bottom
        inner, outer = radius(r["ap"], down), radius(r["out"], down)
        placed = None
        for step in range(40):                 # largest size that actually lands on material
            h = cap - step*(cap - 0.4)/39.0
            # the gap tracks the digit size. A seven-segment 1 is just the right-hand
            # bar of its cell, so a fixed gap leaves '10' reading as a stroke welded to
            # a nought once the digits shrink.
            gap = 0.34*h
            w = 0.55*h*len(label) + gap*(len(label) - 1)
            mid = (inner + outer)/2.0
            x0, y0 = r["cx"] - w/2.0, r["cy"] + mid - h/2.0
            paths, xoff = [], x0
            for ch in label:
                paths += digit(ch, xoff, y0, 0.55*h, h)
                xoff += 0.55*h + gap
            ok = True
            for px, py in points_of(paths):
                th = math.atan2(py - r["cy"], px - r["cx"])
                d = math.hypot(px - r["cx"], py - r["cy"])
                if not (radius(r["ap"], th) + MARGIN <= d <= radius(r["out"], th) - MARGIN):
                    ok = False
                    break
            if ok:
                placed = (paths, h)
                break
        if placed is None:
            fails.append(f"ring {i} ({label}): no digit fits between ø{2*inner:.1f} "
                         f"and ø{2*outer:.1f}")
            continue
        marks += placed[0]
        report.append((i, label, r["o"], outer - inner, placed[1]))

    if fails:
        for f in fails:
            print(f"  x {f}")
        sys.exit(f"  {src_path.name} not written: {len(fails)} ring(s) unlabelled")

    body = "\n".join(f'    <path d="{d}"/>' for d in marks)
    group = (f'\n  <g id="{GROUP}" fill="none" stroke="{BLUE}" stroke-width="0.1">\n'
             f'{body}\n  </g>')
    at = src.index("  <g ")                    # engrave before the black that frees the part
    out = src[:at] + group.lstrip("\n") + "\n" + src[at:]

    before = re.findall(r'\bd="([^"]+)"', src)
    after = re.findall(r'\bd="([^"]+)"', out)
    if after[len(marks):] != before:
        sys.exit("  cut path data changed — refusing to write")


    src_path.write_text(out)
    print(f"  {src_path.name}: {len(rings)} rings numbered {report[0][1]}-{report[-1][1]}, "
          f"{len(marks)} engraved strokes in {BLUE}")
    for i, label, o, wall, h in report:
        print(f"    ring {i:>2}  '{label}'  ø{o:>6.2f}  wall {wall:>5.2f}mm  digit {h:.2f}mm")


if __name__ == "__main__":
    main()
