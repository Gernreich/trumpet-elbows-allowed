"""Read the point list back out of an SVG path.

The gate re-reads what the generator wrote, which means parsing path data
rather than trusting it. Boxes.py emits absolute M/L/H/V with the occasional
C, so this handles those and skips anything else - it is a reader for one
writer's output, not a general SVG parser.

Split out of octomino-snakes' verify_boxes.py, which needed the same reader
for a different question and is archived with the rest of that work.
"""
import re

NS = '{http://www.w3.org/2000/svg}'


def pts(d):
    """Every point a path visits, in order."""
    toks = re.findall(r'[MLHVCZzl]|-?\d+\.?\d*', d)
    P, i, cur = [], 0, None
    while i < len(toks):
        c = toks[i]
        if c in 'ML':
            cur = (float(toks[i+1]), float(toks[i+2])); P.append(cur); i += 3
        elif c == 'H' and cur is not None:
            cur = (float(toks[i+1]), cur[1]); P.append(cur); i += 2
        elif c == 'V' and cur is not None:
            cur = (cur[0], float(toks[i+1])); P.append(cur); i += 2
        elif c == 'C':
            cur = (float(toks[i+5]), float(toks[i+6])); P.append(cur); i += 7
        else:
            i += 1
    return P
