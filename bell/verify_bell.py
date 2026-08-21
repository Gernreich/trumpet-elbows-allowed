#!/usr/bin/env python3
"""Check a bell sheet that has been edited by hand, before it is cut.

    python3 verify_bell.py SHEET.svg [REFERENCE.svg]

With one argument it checks the sheet against itself: ring count, the 1.5mm
lap, what is nested inside what, whether any two outlines cross, and whether
the cut stage is one colour. With a second it also compares every ring to the
sizes bell.py produced -- useful after the paths have been converted to curves,
when a byte-for-byte diff no longer says anything.

Position-independent, so it still answers after the parts have been nested.

Sizes are measured per command. Flattening a path's numbers and pairing them
off breaks the moment it uses H, V or a curve, and these files use all three.
Quadratic control points are counted into a box, making it a superset of the
true one; for telling a ring from a digit that is the safe way to be wrong.
"""
import re
import sys
import pathlib

NUM = re.compile(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?")
RING_MIN = 20.0                     # smallest ring is ø31; every digit is under 3mm
LAP = 1.5                           # each ring overhangs the next aperture by this


def subpaths(d):
    out, cur = [], None
    x = y = sx = sy = 0.0
    for cmd, arg in re.findall(r"([MmLlHhVvQqTtCcSsAaZz])([^A-Za-z]*)", d):
        n = [float(v) for v in NUM.findall(arg)]
        up, rel = cmd.upper(), cmd.islower()
        if up == "M":
            if cur:
                out.append(cur)
            x, y = (x + n[0], y + n[1]) if rel else (n[0], n[1])
            sx, sy = x, y
            cur = [(x, y)]
            for i in range(2, len(n), 2):
                x, y = (x + n[i], y + n[i + 1]) if rel else (n[i], n[i + 1])
                cur.append((x, y))
        elif up == "L":
            for i in range(0, len(n), 2):
                x, y = (x + n[i], y + n[i + 1]) if rel else (n[i], n[i + 1])
                cur.append((x, y))
        elif up == "H":
            for v in n:
                x = x + v if rel else v
                cur.append((x, y))
        elif up == "V":
            for v in n:
                y = y + v if rel else v
                cur.append((x, y))
        elif up in ("Q", "T", "C", "S", "A"):
            step = {"Q": 4, "T": 2, "C": 6, "S": 4, "A": 7}[up]
            for i in range(0, len(n) - step + 1, step):
                seg = n[i:i + step]
                if up == "A":
                    x, y = (x + seg[5], y + seg[6]) if rel else (seg[5], seg[6])
                    cur.append((x, y))
                    continue
                pts = [(seg[k], seg[k + 1]) for k in range(0, step, 2)]
                for dx, dy in pts:
                    cur.append((x + dx, y + dy) if rel else (dx, dy))
                lx, ly = pts[-1]
                x, y = (x + lx, y + ly) if rel else (lx, ly)
        elif up == "Z":
            x, y = sx, sy
    if cur:
        out.append(cur)
    return out


def box(pts):
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    return min(xs), min(ys), max(xs), max(ys)


def span(pts):
    x0, y0, x1, y1 = box(pts)
    return max(x1 - x0, y1 - y0)


def read(path):
    """(rings, marks). A ring is a path holding two big concentric squares."""
    src = pathlib.Path(path).read_text()
    rings, marks = [], []
    for i, attrs in enumerate(re.findall(r"<path\b([^>]*)>", src)):
        d = re.search(r'\bd="([^"]+)"', attrs)
        if not d:
            continue
        sp = subpaths(d.group(1))
        stroke = re.search(r"stroke\s*[:=]\s*.?(#[0-9a-fA-F]{6})", attrs)
        colour = stroke.group(1).lower() if stroke else None
        big = sorted([p for p in sp if span(p) >= RING_MIN], key=span)
        if len(big) == 2:
            rings.append({"i": i, "outer": box(big[1]), "ap": box(big[0]),
                          "o": span(big[1]), "a": span(big[0]), "c": colour})
        else:
            marks.append({"i": i, "box": box([q for p in sp for q in p]), "c": colour})
    return rings, marks


def inside(b, o, pad=0.01):
    return (b[0] >= o[0] - pad and b[1] >= o[1] - pad
            and b[2] <= o[2] + pad and b[3] <= o[3] + pad)


def crosses(a, b):
    return a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]


def is_bell_sheet(path):
    """Bell rings are squares drawn with H and V. The mouthpiece's are circles,
    drawn with elliptical arcs, and it laps by whatever its 3mm wall leaves
    rather than by a fixed 1.5mm -- so this tool has nothing true to say about
    it. Pointed at one it used to report a problem, which is worse than
    silence: a checker that answers about a file it does not understand teaches
    you to ignore it."""
    src = pathlib.Path(path).read_text()
    ds = re.findall(r'\bd="([^"]+)"', src)
    if any(c in "Aa" for d in ds for c in re.findall(r"[A-Za-z]", d)):
        return False, "rings are drawn with arcs, so they are round — not a bell sheet"
    rings, _ = read(path)
    if len(rings) < 2:
        return False, (f"only {len(rings)} concentric-square ring(s) — a section drawing "
                       "or a sheet of something else")
    return True, ""


def main():
    ok, why = is_bell_sheet(sys.argv[1])
    if not ok:
        print(f"  skipped {pathlib.Path(sys.argv[1]).name}: {why}")
        return 0
    rings, marks = read(sys.argv[1])
    fails = []
    print(f"  rings {len(rings)}   other paths {len(marks)}")

    order = sorted(rings, key=lambda r: r["o"])
    print("\n  the 1.5mm lap, ring to ring:")
    laps = [round((order[i]["o"] - order[i + 1]["a"]) / 2, 3) for i in range(len(order) - 1)]
    bad = [v for v in laps if abs(v - LAP) > 0.01]
    print(f"    {sorted(set(laps))}mm per side" + ("" if not bad else f"   OFF: {bad}"))
    if bad:
        fails.append("lap is not 1.5mm at every joint")

    # Two legitimate schemes. One colour for every ring is a single cut stage. A ramp
    # is one stage per ring, and then the colour IS the cut order -- so what matters is
    # that it rises with the ring, smallest first, since that is inner-before-outer for
    # every nest on the sheet. A ramp that does not sort with the sizes is worse than
    # no ramp: it looks deliberate and orders the cut wrongly.
    print("\n  cut stage:")
    mc = sorted({m["c"] for m in marks})
    by_size = sorted(rings, key=lambda r: r["o"])
    cols = [r["c"] for r in by_size]
    if len(set(cols)) == 1:
        print(f"    rings  one stage, {cols[0]}")
    else:
        print(f"    rings  {len(set(cols))} stages, {cols[0]} (ø{by_size[0]['o']:.1f}) "
              f"→ {cols[-1]} (ø{by_size[-1]['o']:.1f})")
        if len(set(cols)) != len(cols):
            dupe = [c for c in set(cols) if cols.count(c) > 1]
            fails.append(f"rings share a stage, so their order is undecided: {sorted(dupe)}")
        keys = [int(c[1:], 16) for c in cols]
        if keys != sorted(keys):
            fails.append("ring colours do not rise with ring size — the ramp orders "
                         "the cut wrongly")
        else:
            print("    ramp rises with size, so the smallest cuts first and the rim last")
    print(f"    labels {mc}")
    if len(mc) > 1:
        fails.append(f"marks use {len(mc)} shades, so they will not all land in one stage: {mc}")
    if set(mc) & set(cols):
        fails.append(f"a label shares a stage with a ring: {sorted(set(mc) & set(cols))}")

    # Reported as a note, not a failure. Document order is only the cut order on an
    # importer that follows it, and most do not: LightBurn and RDWorks order inner
    # shapes first by default whatever the file says. A freed disc this size also sits
    # flat on the bed rather than dropping. Calling it a problem alongside a genuine
    # one -- two shades of blue, which no software will rescue you from -- taught the
    # reader to weigh them the same, and they are not the same.
    print("\n  nesting:")
    nested = late = 0
    for a in rings:
        for b in rings:
            if a is b or not inside(a["outer"], b["ap"]):
                continue
            nested += 1
            if a["i"] > b["i"]:
                late += 1
                print(f"    – ø{a['o']:.1f} (path #{a['i']}) sits in ø{b['o']:.1f} "
                      f"(path #{b['i']}) and is written after it")
    print(f"    {nested} parts nested, {nested - late} written before the ring that frees them")
    if late:
        print(f"    {late} written after. That only decides anything if your software cuts in")
        print("    document order — most order inner shapes first regardless.")

    clash = 0
    for i in range(len(rings)):
        for j in range(i + 1, len(rings)):
            a, b = rings[i], rings[j]
            if (crosses(a["outer"], b["outer"])
                    and not inside(a["outer"], b["ap"]) and not inside(b["outer"], a["ap"])):
                clash += 1
                print(f"    ✗ ø{a['o']:.1f} and ø{b['o']:.1f} overlap and neither is nested")
    if clash:
        fails.append(f"{clash} pair(s) of outlines cross")
    print(f"    {clash} overlapping outlines")

    if len(sys.argv) > 2:
        ref, _ = read(sys.argv[2])
        print(f"\n  against {pathlib.Path(sys.argv[2]).name}:")
        a = sorted(round(r["a"], 2) for r in rings)
        b = sorted(round(r["a"], 2) for r in ref)
        same = a == b and len(rings) == len(ref)
        print(f"    {len(rings)} rings vs {len(ref)}; apertures "
              f"{'identical' if same else 'DIFFER'}")
        if not same:
            fails.append("ring sizes differ from the reference")

    print()
    for f in fails:
        print(f"  ✗ {f}")
    print(f"  {'✓ sheet is ready to cut' if not fails else f'{len(fails)} problem(s)'}")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
