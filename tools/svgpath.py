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

# Every letter, and numbers in the three spellings SVG allows: 12, 12.5, .5,
# and any of those with an exponent. The old pattern listed only the commands
# it handled and matched `-?\d+\.?\d*`, so a leading-dot number was read as
# no number at all and any other command was not even tokenised -- its
# coordinates arrived as bare numbers and were dropped one at a time.
_TOK = re.compile(r'[A-Za-z]|[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?')


def pts(d):
    """Every point a path visits, in order.

    A reader for one writer's output, not a general SVG parser: Boxes.py emits
    absolute M/L/H/V with the occasional C, and those are what this handles.

    IT NOW REFUSES WHAT IT CANNOT READ. The fall-through was `i += 1`, which
    stepped silently past anything unexpected -- a relative lineto, an arc, a
    number the pattern did not match -- and returned a SHORTER point list than
    the path actually visits. Every measurement downstream is taken from that
    list: the sliver check, the bed check, the overlap check, the label
    placement. A quietly truncated outline is the one input none of them can
    survive, and none of them would have said so.
    """
    toks = _TOK.findall(d)
    P, i, cur = [], 0, None
    while i < len(toks):
        c = toks[i]
        if c in 'ML':
            cur = (float(toks[i+1]), float(toks[i+2])); P.append(cur); i += 3
        elif c in 'HV':
            if cur is None:
                raise ValueError(f'path starts with {c}, which has no point '
                                 f'to carry forward: {d[:60]!r}')
            cur = ((float(toks[i+1]), cur[1]) if c == 'H'
                   else (cur[0], float(toks[i+1])))
            P.append(cur); i += 2
        elif c == 'C':
            cur = (float(toks[i+5]), float(toks[i+6])); P.append(cur); i += 7
        elif c in 'Zz':
            i += 1
        elif c.isalpha():
            raise ValueError(
                f'path command {c!r} is one this reader does not handle, and '
                f'skipping it would drop the points it carries. Path: '
                f'{d[:60]!r}')
        else:
            raise ValueError(
                f'number {c!r} with no command in front of it, at token {i} '
                f'of {d[:60]!r}')
    return P
