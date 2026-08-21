#!/usr/bin/env python3
"""Recolour a bell sheet's rings as a black-to-red ramp, smallest to largest.

    python3 ramp_bell.py SHEET.svg

Colour is the cut stage, so a ramp ordered by size is an instruction: cut the
smallest ring first and the rim last. That is inner-before-outer for every nest
on the sheet, expressed in the one channel a laser importer always reads.

Touches nothing but the stroke colour of the seventeen ring paths. The labels
keep their blue, the path data is compared before and after, and the file is
not written unless every check holds.
"""
import re
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from verify_bell import read  # noqa: E402

DATA = re.compile(r'\bd="([^"]+)"')
path_el = re.compile(r"<path\b[^>]*>")


def main():
    p = pathlib.Path(sys.argv[1])
    src = p.read_text()
    rings, marks = read(p)
    n = len(rings)
    assert n == 17, f"expected 17 rings, found {n}"

    # rank 0 = smallest -> #000000, rank n-1 = largest -> #ff0000
    ramp = {r["i"]: f"#{round(k * 255 / (n - 1)):02x}0000"
            for k, r in enumerate(sorted(rings, key=lambda x: x["o"]))}
    assert ramp[sorted(rings, key=lambda x: x["o"])[0]["i"]] == "#000000"
    assert ramp[sorted(rings, key=lambda x: x["o"])[-1]["i"]] == "#ff0000"
    assert len(set(ramp.values())) == n, "ramp is not one colour per ring"

    seen = {"i": -1}

    def swap(m):
        seen["i"] += 1
        el = m.group(0)
        if seen["i"] not in ramp:
            return el
        return re.sub(r"(stroke\s*:\s*)#[0-9a-fA-F]{6}",
                      lambda s: s.group(1) + ramp[seen["i"]], el, count=1)

    out = path_el.sub(swap, src)

    if DATA.findall(out) != DATA.findall(src):
        sys.exit("  path data changed — refusing to write")
    new_rings, new_marks = read_text(out, p)
    if [m["c"] for m in new_marks] != [m["c"] for m in marks]:
        sys.exit("  label colours changed — refusing to write")
    got = {r["i"]: r["c"] for r in new_rings}
    if got != ramp:
        sys.exit(f"  ramp not applied as intended\n    want {ramp}\n    got  {got}")

    p.write_text(out)
    for k, r in enumerate(sorted(new_rings, key=lambda x: x["o"]), 1):
        print(f"    {k:>2}  ø{r['o']:>6.2f}  {r['c']}")
    print(f"  {n} rings recoloured, {len(marks)} labels left alone")


def read_text(text, like):
    tmp = like.with_suffix(".ramp-tmp.svg")
    tmp.write_text(text)
    try:
        return read(tmp)
    finally:
        tmp.unlink()


if __name__ == "__main__":
    main()
