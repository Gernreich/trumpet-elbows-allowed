#!/usr/bin/env python3
"""Generate a whole mouthpiece: backbore, throat, entrance and cup, in one sheet.

    python3 mouthpiece-round.py [OUT.svg]
    python3 mouthpiece-round.py --taper=2.0     # finer steps down the bore-side run
    python3 mouthpiece-round.py --seat=1.25     # demand more seat, round more slowly
    python3 mouthpiece-round.py --rim=17.0      # a wider rim
    python3 mouthpiece-round.py --bowl=5        # a 15mm cup instead of 12mm
    python3 mouthpiece-round.py --layout=trumpet   # real proportions, for a new one

`mouthpiece.py` puts a 25mm ROUND aperture in its 31mm square plate, so the joint that
meets the bore throws away the bore's corners: a 25mm square channel opens into a 25mm
circle and 21% of the area steps out of the airway at station one. This one starts as the
bore actually is -- a sharp 25mm square inside a sharp 31mm plate -- and rounds the corners
away going up, exactly as bell-round.py does going out. Same rounded squares, same exact
offsets, opposite direction.

THE 4mm TAPER HAD TO GO. A ring's outer is its aperture offset by WALL, so the seat the
next ring lands on is WALL minus how much the two apertures differ in that direction. On
the flats that is half the aperture step: the old 4mm steps left 3 - 2 = 1.00mm. Through
the corners of a SHARP square the same step costs step x root2 = 2.83mm, leaving 0.17mm --
and rounding a corner pulls the diagonal in further, which takes it negative. So a square
mouthpiece cannot be built on a 4mm taper at all. At 2.5mm the corner seat is 1.23mm with
room to round. The run is the same 25 -> 5mm cone either way, sampled finer: three more
rings, 9mm more mouthpiece, and no change to the profile the air sees.

ROUNDNESS IS NOT SCHEDULED, IT IS AS FAST AS THE SEAT ALLOWS. Each station takes the
largest corner radius that still leaves MINSEAT in both directions, in closed form. The
part therefore rounds as quickly as its own geometry permits and no quicker -- on the
default profile it reaches a true circle at the 7.5mm station, 24mm up from the bore, and
the throat and the whole cup are round.

THE TWO LAYOUTS. Both are 30 rings and 90mm; they differ in where the length goes.

    --layout=trumpet   75mm backbore, 12mm cup                   (default)
    --layout=legacy    27mm backbore, 51mm entrance, 12mm cup

A real mouthpiece spends its length on the backbore and keeps the cup short: the throat
opens into the cup almost at once, and the long gradual run is on the other side of it.
`legacy` has that close to inverted -- a short steep backbore and 51mm of near-cylindrical
entrance on the LIP side of the throat, which is a 48mm-deep cup by any honest reading. It
exists because a mouthpiece was built to that profile before the trumpet layout did, and
its sheet is kept as a record of that part. It was the default until 2026-09-03 and called
`asbuilt`, which said when it was made rather than whether to cut it. Do not cut it new.

`--backbore` sets the backbore rings and `--power` how it opens: 1 is a straight cone,
higher keeps it near-cylindrical off the throat and opens it later, as a real one does.

THE WALL IS NOT A CONSTANT, and it cannot be. A ring's outer is its aperture plus two
walls, so the seat above it is the wall of whichever ring is narrow there, less half the
aperture step. A real cup turns far too fast for 3mm: straight off the throat it goes ø3.66
to ø11.25 in one ring, which wants 4.80mm of wall to have anything to sit on. So each ring
takes the wall its own step demands -- 3mm nearly everywhere, 4.80mm on the throat ring of
a trumpet cup, and nowhere else. Holding the wall at 3mm would have forced a 36mm cup, which
is not a cup.

THE CUP IS A BOWL, NOT A CONE. This used to stop at ø10.06 opening 0.40mm a ring -- a 3.8
degree half-angle, a tube, against the 16 to 17mm a trumpet rim is across inside. The bowl
is an ellipse arc whose slope is zero at the rim, so the wall stands parallel to the axis
where the lip sits and turns toward the throat going down; its first ring opens 3.21mm and
its rim ring 0.33mm. A cone would meet the rim at an angle and feel like a funnel.

The three runs are named for the anatomy, which `mouthpiece.py` gets backwards:

    BACKBORE   25 -> 5mm, the end that closes onto the bore, square becoming round
    ENTRANCE   the 3.66mm throat, then 0.40mm a ring out to 10.06 -- 48mm of it, which is
               long for a mouthpiece and is where to look first if it plays stuffy
    BOWL       10.06 -> 16.5mm over 12mm, the cup proper, ending at the rim

A sheet from here is the same part as mouthpiece-bore25-legacy-parts-cut-files.svg plus mouthpiece-cup.py's
four rings, ring for ring, so both routes number identically. `mouthpiece-cup.py` stays for
retrofitting a mouthpiece already glued without a bowl.

The bowl is a staircase of 3mm ply. Sand or fill the steps and round the rim over before
playing it; as cut the rim edge is a square corner and your lip will say so.
"""
import sys, math, pathlib, subprocess

WALL    = 3.0        # ring width, and the ply is 3mm so a ring is as thick as it is wide
PLATE   = 31.0       # station one, square, matching the bore's closing face
BORE    = 25.0       # the square bore this meets, corners and all
NECK    = 5.0        # where the backbore run ends and the throat begins
THROAT  = 3.66       # a #27 drill, the standard trumpet mouthpiece throat
TAPER   = 2.5        # aperture step down the backbore run; 4.0 cannot be made square
RIM     = 16.5       # the rim your lip sits on; a trumpet is 16 to 17mm inside
BOWL    = 4          # rings of cup: 4 x 3mm = a 12mm bowl
LAYOUT  = "trumpet"  # trumpet | legacy -- see THE TWO LAYOUTS below
NBB     = 25         # trumpet layout: rings of backbore, 25 x 3mm = 75mm
BACK_P  = 2.0        # trumpet layout: backbore power. 1 is a cone, higher opens later
MINSEAT = 1.0        # every joint, flats and corners alike
GAP     = 2.0
MARGIN  = 3.0
DIAG    = math.sqrt(2.0)

opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in opts:
    if k not in ("taper", "seat", "rim", "bowl", "layout", "backbore", "power", "bore",
                 "numbers"):
        sys.exit(f"unknown option --{k}: taper, seat, rim, bowl, layout, backbore, "
                 f"power, bore or numbers")
TAPER   = float(opts.get("taper", TAPER))
MINSEAT = float(opts.get("seat", MINSEAT))
RIM     = float(opts.get("rim", RIM))
BOWL    = int(opts.get("bowl", BOWL))
LAYOUT  = opts.get("layout", LAYOUT)
NBB     = int(opts.get("backbore", NBB))
BACK_P  = float(opts.get("power", BACK_P))
# --bore is the square air channel this closes onto; the plate around it is that plus a
# 3mm wall each side, exactly as the tube is. It exists for the 10mm trumpet. The throat
# does NOT follow it: 3.66mm is a #27 drill and a real trumpet throat, and a mouthpiece
# is sized by the lip at one end and the drill at the other, not by the tube it feeds.
BORE    = float(opts.get("bore", BORE))
PLATE   = BORE + 6.0
# Numbering is part of writing the sheet, not a step to remember afterwards. Thirty rings
# that pass through the same diameters twice are thirty anonymous discs the moment they
# leave the bed, and a sheet cut without numbers cannot be numbered later -- the parts are
# already loose. --numbers=no writes it bare.
NUMBERS = opts.get("numbers", "yes")
if NUMBERS not in ("yes", "no"):
    sys.exit(f"--numbers: yes or no, not {NUMBERS!r}")
if BORE <= THROAT:
    sys.exit(f"--bore must be wider than the ø{THROAT:g}mm throat")
if LAYOUT == "asbuilt":
    sys.exit("--layout=asbuilt is now --layout=legacy, and it is no longer the default.\n"
             "  It names a profile built before the trumpet layout existed; trumpet is the\n"
             "  one to cut new.")
if LAYOUT not in ("trumpet", "legacy"):
    sys.exit(f"--layout: trumpet or legacy, not {LAYOUT!r}")
# Both parameters that make one sheet a different part from another go in the name:
# the bore it closes onto and the layout it spends its length on. Naming only the
# non-default one left the 25mm legacy sheet called mouthpiece-round-parts.svg and the
# 25mm trumpet sheet with no way to say it was not that -- two different parts, and
# nothing in either name to tell them apart.
_dflt = f"mouthpiece-bore{BORE:g}-{LAYOUT}-parts-cut-files.svg"
out_path = next((a for a in sys.argv[1:] if not a.startswith("--")), _dflt)


def support(h, c, s):
    """How far a rounded square of half-width h and corner radius c reaches in a direction,
    s = |cos| + |sin| of it: 1 straight out through a flat, root2 out through a corner."""
    return (h - c)*s + c


# ── the profile, in assembly order from the bore ─────────────────────────────
def ellipse_bowl(from_dia):
    """The cup, as an ellipse arc through the (depth, radius) plane, depth measured DOWN
    from the rim. Its slope is -k d / r, zero at d = 0, so the wall stands parallel to the
    axis where the lip sits and turns toward the throat going down. A cone would meet the
    rim at an angle and feel like a funnel."""
    if RIM <= from_dia:
        sys.exit(f"--rim must open past the ø{from_dia:g}mm the cup starts from")
    r0, R, D = from_dia/2.0, RIM/2.0, BOWL*WALL
    return [2.0*math.sqrt(R*R - (R*R - r0*r0)*((D - WALL*(k+1))/D)**2) for k in range(BOWL)]


if LAYOUT == "trumpet":
    # A real mouthpiece spends its length on the BACKBORE and keeps the cup short. The
    # backbore is nearly cylindrical off the throat and opens faster toward the shank,
    # which is what BACK_P does; the cup then runs straight from the throat to the rim.
    backbore = [THROAT + (BORE - THROAT)*(((NBB - k)/NBB)**BACK_P) for k in range(NBB + 1)]
    entrance = []
    bowl     = ellipse_bowl(THROAT)
else:
    # As built: a 27mm backbore and a 48mm entrance, which is close to inverted. Kept
    # because a mouthpiece exists to this profile and its rings are numbered for it.
    steps    = int(round((BORE - NECK)/TAPER))
    backbore = [BORE - TAPER*i for i in range(steps + 1)]      # 25 .. 5, the bore end
    entrance = [round(THROAT + 0.40*i, 3) for i in range(17)]  # the throat, then .. 10.06
    bowl     = ellipse_bowl(entrance[-1])

profile  = backbore + entrance + bowl
hs       = [a/2.0 for a in profile]

# ── the wall each ring needs ─────────────────────────────────────────────────
# A ring's outer is its aperture plus two walls, so the seat the joint above it lands on is
# the wall of whichever ring is the NARROW one there, less half the aperture step. A real
# cup turns too fast for a 3mm wall -- ø3.66 to ø11.25 in one ring wants 4.80mm -- so the
# wall is not a constant. It thickens exactly where the profile turns, and nowhere else.
Ws = [WALL]*len(profile)
for i in range(1, len(profile)):
    step = profile[i] - profile[i-1]
    if step > 0:
        Ws[i-1] = max(Ws[i-1], MINSEAT + step/2.0)
    else:
        Ws[i] = max(Ws[i], MINSEAT - step/2.0)

# ── roundness: the largest corner radius each station can afford ─────────────
# seat(s) = WALL - |support_i(s) - support_{i-1}(s)|, so keeping MINSEAT bounds how far
# this station's diagonal reach may move from the one below. Within that band we want the
# reach as SMALL as possible, because smaller reach is a rounder corner.
cs = [0.0]                                                     # station one is the bore
for i in range(1, len(profile)):
    # the wall that carries this joint is the one on the NARROW side of it, the same
    # rule that set Ws above — the ring below when the airway opens, this one when it
    # narrows. Taking the smaller of the two would refuse a cup the walls can hold.
    h = hs[i]
    room = (Ws[i-1] if hs[i] > hs[i-1] else Ws[i]) - MINSEAT
    prev = support(hs[i-1], cs[i-1], DIAG)
    lo, hi = max(h, prev - room), min(h*DIAG, prev + room)
    if lo > hi + 1e-9:
        sys.exit(f"station {i+1} cannot hold {MINSEAT}mm of seat at any corner radius — "
                 f"try --taper below {TAPER:g} or --seat below {MINSEAT:g}")
    cs.append((h*DIAG - lo)/(DIAG - 1.0))

# ── outers, and the checks, before anything is drawn ─────────────────────────
# Station one's outer is the sharp plate, not the offset: it has to cover the bore's face
# corner to corner. Everywhere else the outer is the aperture offset by WALL, which for a
# rounded square is another rounded square, exactly WALL away in every direction.
OH = [PLATE/2.0] + [h + Ws[i] for i, h in enumerate(hs) if i]
OC = [0.0]       + [c + Ws[i] for i, c in enumerate(cs) if i]

fails, seats = [], []
for i in range(1, len(profile)):
    for s in (1.0, DIAG):
        # both are linear in s, so min-of-two minus max-of-two is concave and its smallest
        # value on [1, root2] is at an end: checking the flats and the corners is enough
        room = (min(support(OH[i], OC[i], s), support(OH[i-1], OC[i-1], s))
                - max(support(hs[i], cs[i], s), support(hs[i-1], cs[i-1], s)))
        seats.append(room)
        if room <= 0:
            fails.append(f"station {i+1} falls through station {i} "
                         f"{'across the flats' if s == 1.0 else 'through the corners'}")
        elif room < MINSEAT - 1e-6:
            fails.append(f"station {i+1} seats on only {room:.2f}mm at station {i}")

def rounded(cx, cy, h, c):
    """c = 0 is a sharp square, c = h is a true circle, and everything between is both."""
    c = min(max(c, 0.0), h)
    if c <= 1e-9:
        return f"M {cx-h:.4f},{cy-h:.4f} H {cx+h:.4f} V {cy+h:.4f} H {cx-h:.4f} Z"
    s = h - c
    arc = lambda x, y: f"A {c:.4f},{c:.4f} 0 0 1 {x:.4f},{y:.4f}"
    run = lambda cmd, v: f"{cmd} {v:.4f} " if s > 1e-9 else ""
    return (f"M {cx-s:.4f},{cy-h:.4f} "
            + run("H", cx+s) + arc(cx+h, cy-s) + " "
            + run("V", cy+s) + arc(cx+s, cy+h) + " "
            + run("H", cx-s) + arc(cx-h, cy+s) + " "
            + run("V", cy-s) + arc(cx-s, cy-h) + " Z")


def layout(per):
    placed, y, W = [], MARGIN, 0.0
    for k in range(0, len(profile), per):
        idx = range(k, min(k + per, len(profile)))
        x, rowh = MARGIN, max(2*OH[i] for i in idx)
        for i in idx:
            placed.append((i, x + OH[i], y + rowh/2.0))
            x += 2*OH[i] + GAP
        W = max(W, x - GAP + MARGIN); y += rowh + GAP
    return placed, W, y - GAP + MARGIN


if fails:
    for f in fails:
        print(f"  x {f}")
    sys.exit(f"  {out_path} not written: {len(fails)} problem(s)")

per = min(range(1, len(profile) + 1),
          key=lambda p: (round(max(layout(p)[1:]), 6), round(layout(p)[1]*layout(p)[2], 6)))
placed, W, H = layout(per)
# aperture and outline as separate paths so each can take its own cut stage
holes = [f'  <path d="{rounded(cx, cy, hs[i], cs[i])}"/>' for i, cx, cy in placed]
parts = [f'  <path d="{rounded(cx, cy, OH[i], OC[i])}"/>' for i, cx, cy in placed]

round_at = next((i for i in range(len(profile)) if abs(cs[i] - hs[i]) < 1e-6), None)
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
       f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
       # The layout belongs here. Two 25mm mouthpieces exist -- legacy and trumpet --
       # with the same ring count, bore, throat and rim, and without the layout their
       # titles were identical while their profiles were not. A sheet has to say which
       # part it is.
       f'  <title>Square-to-round mouthpiece, {LAYOUT} layout - {len(profile)} rings, '
       f'{BORE:g}mm square bore to {THROAT:g}mm throat to ø{profile[-1]:g}mm lip</title>\n'
       f'  <desc>1 user unit = 1mm. Station one is a sharp {BORE:g}mm square aperture in a '
       f'sharp {PLATE:g}mm square plate, matching the bore corner for corner. The corners '
       f'round away going up and the section is a true circle from the '
       f'{profile[round_at]:g}mm station on, so the {THROAT:g}mm throat and the whole cup '
       f'are round. The {LAYOUT} layout: {len(backbore)} backbore rings, {len(entrance)} entrance '
       f'rings, then {len(bowl)} bowl rings to a ø{RIM:g}mm rim. Wall {min(Ws):g} to {max(Ws):.2f}mm, 3mm of rise a ring, stack '
       f'{len(profile)*WALL:g}mm. Every joint seats on at least {min(seats):.2f}mm per side, '
       f'flats and corners alike. Rings stack; they do not telescope.</desc>\n'
       + '  <g fill="none" stroke="#ff8000" stroke-width="0.1">\n'
       + "\n".join(holes) + "\n  </g>\n"
       + '  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
       + "\n".join(parts) + "\n  </g>\n</svg>\n")
open(out_path, "w").write(svg)

# --order=document, always: a mouthpiece narrows to the throat and opens again, so its
# backbore and its cup pass through the same diameters. Sorting those by size interleaves
# the two halves, which is the one thing a number on a ring must never do.
if NUMBERS == "yes":
    _tool = pathlib.Path(__file__).resolve().parent.parent / "bell" / "number_rings.py"
    _r = subprocess.run([sys.executable, str(_tool), out_path, "--order=document"],
                        capture_output=True, text=True)
    if _r.returncode:
        # Leave nothing cuttable behind. A sheet on disk gets cut, and an unnumbered one
        # is 30 discs nobody can order -- discovered after the machine, not before it.
        pathlib.Path(out_path).unlink(missing_ok=True)
        sys.exit(f"  {out_path} NOT written: the rings could not be numbered.\n"
                 f"{_r.stdout}{_r.stderr}"
                 f"  Fix that, or pass --numbers=no if you meant a bare sheet.")

print(f"  {out_path}: {len(profile)} rings, {W:.1f} x {H:.1f}mm, stack {len(profile)*WALL:g}mm")
print(f"    airway     {BORE:g}mm square -> {THROAT:g} throat -> ø{profile[-1]:g} lip")
print(f"    layout     {LAYOUT}")
if LAYOUT == "trumpet":
    print(f"    backbore   {len(backbore)} rings, {(len(backbore)-1)*WALL:g}mm, ø{BORE:g} "
          f"down to the ø{THROAT:g} throat, power {BACK_P:g}")
else:
    print(f"    backbore   {len(backbore)} rings in {TAPER:g}mm steps to the ø{NECK:g} neck")
    print(f"    entrance   {len(entrance)} rings, ø{THROAT:g} throat opening to "
          f"ø{entrance[-1]:g} over {len(entrance)*WALL:g}mm")
print(f"    bowl       {len(bowl)} rings, ø{profile[len(backbore)+len(entrance)-1]:g} to a "
      f"ø{RIM:g}mm rim over {BOWL*WALL:g}mm")
print(f"    wall       {min(Ws):.2f}-{max(Ws):.2f}mm, thickest where the bowl turns")
print(f"    round from station {round_at+1} (ø{profile[round_at]:g}mm), "
      f"{round_at*WALL:g}mm up from the bore")
print(f"    seat       {min(seats):.2f}-{max(seats):.2f}mm per side, flats and corners alike")
print(f"    numbers    " + ("each ring engraved with its hex index, 0 at the bore"
                            if NUMBERS == "yes" else
                            "NONE — --numbers=no; the rings carry nothing"))
print(f"    ok, written")
