#!/usr/bin/env python3
"""Generate a bell whose section morphs from the bore's square to a round rim.

    python3 bell-round.py                     # all four ring budgets
    python3 bell-round.py 20                  # one, at most 20 rings
    python3 bell-round.py 67 --morph=flare    # a different square-to-round schedule
    python3 bell-round.py 67 --law=width      # follow the half-width, not the area
    python3 bell-round.py --length=100        # a 100mm bell, same throat and rim
    python3 bell-round.py --length=100 --rim=80   # ... and a narrower rim to suit
    python3 bell-round.py --length=152 --mouth=80 --bore=10   # the 10mm trumpet's bell

The four bells bell.py makes are square end to end: a square bore opening into a square
rim. This one keeps the square only where it has to. Station one is the same 31mm square
that closes onto the bore, and the corners are rounded away up the horn until the rim is
a true circle. Same Bessel profile, gamma 0.7, same 3mm ply, same 1.5mm lap.

THE RINGS STACK; THEY DO NOT TELESCOPE. Each ring's outer contour is the NEXT ring's
aperture offset outward by LAP, and that is what fixes the seat. Offsetting a rounded
square outward by d gives another rounded square -- half-width h+d, corner radius c+d --
and the gap between the two is exactly d in every direction, corners included. So every
joint seats on 1.5mm per side however square or however round the two rings are. A ring's
own wall then works out to its radius gain plus the lap, exactly as in bell.py.

A ROUNDED SQUARE IS THE RIGHT SHAPE and a superellipse is not. Rounded squares offset
exactly, they draw in H, V and A alone, corner radius 0 is the bore's square exactly, and
corner radius = half-width is a circle exactly. Nothing is approximated at either end of
the morph, so station one really does close onto the bore and the rim really is round.

AREA, NOT WIDTH, is held to the profile. A circle inscribed in a square has 21% less
area, so rounding the corners at constant width would quietly choke the horn at the very
place it should be opening. Each station is widened instead to enclose the area the square
bell would have had -- by nothing at the throat, by 12.8% at the round rim. `--law=width`
turns that off and follows the half-width, at the cost of the area schedule.

The profile constants are restated here rather than imported: bell.py writes its four
sheets at import time, so importing it would cut a new set of files as a side effect.

Ring sizes and the rise are stated in the file, so bell-section.py and bell-view.py read
these sheets like any other. verify_bell.py does not -- it takes an arc as proof that it
is looking at the mouthpiece -- so the checks it would have run are run here instead,
before anything is written.
"""
import sys, math, pathlib, subprocess

RISE, LAP, MINWALL = 3.0, 3.0, 2.0
GAMMA, RT, RIM, L  = 0.7, 12.5, 61.5, 201.0     # 25mm square throat = the bore's channel
# 10mm has been the bore since 2026-09-05, when the 25mm was retired and
# bore_split.py's default followed. This one did not, so a bare run went on
# writing parts for a tube nothing in the repository cuts any more.
BORE     = 10.0      # the bore's air channel; the horn continues it, it does not step
PLATE    = 16.0      # the bore's outside. Ring 0 has to cover this whole face
OVERHANG = 3.0       # and stand proud of it, to glue against and to locate the joint

args  = [a for a in sys.argv[1:] if not a.startswith("--")]
opts  = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
for k in opts:
    if k not in ("morph", "law", "length", "rim", "mouth", "gamma", "lap", "overhang",
                 "bore", "numbers", "out"):
        sys.exit(f"unknown option --{k}: morph, law, length, rim, mouth, gamma, lap, "
                 f"overhang, bore, numbers or out")
# Numbering is part of writing the sheet, not a step to remember afterwards. Rings differ
# by a couple of millimetres and the sheet is laid out smallest-first, which is the order
# they stop being in the moment someone lifts them off the bed. --numbers=no writes it bare.
NUMBERS = opts.get("numbers", "yes")
if NUMBERS not in ("yes", "no"):
    sys.exit(f"--numbers: yes or no, not {NUMBERS!r}")
if "rim" in opts and "mouth" in opts:
    sys.exit("--rim and --mouth both set the same thing from different ends; pick one")
morph = opts.get("morph", "linear")
law   = opts.get("law", "area")
if morph not in ("linear", "flare", "early"): sys.exit(f"--morph: linear, flare or early, not {morph!r}")
if law   not in ("area", "width"):            sys.exit(f"--law: area or width, not {law!r}")

# The throat is 31mm square because the bore is, and a ring rises 3mm because the ply does.
# The RISE is still not ours to scale -- the ply is 3mm whatever the horn does -- but the
# throat is, if the bore itself changed: --bore is the air channel this closes onto, and
# the plate around it is that plus a 3mm wall each side, exactly as the tube is. It exists
# for the 10mm trumpet; leave it alone for the 25mm one and nothing moves.
# --rim is the bore's width AT the rim, before the wall is added.
BORE  = float(opts.get("bore", BORE))
PLATE = BORE + 6.0
RT    = BORE / 2.0
L     = float(opts.get("length", L))
RIM   = float(opts.get("rim", RIM*2)) / 2.0
GAMMA    = float(opts.get("gamma", GAMMA))
LAP      = float(opts.get("lap", LAP))
OVERHANG = float(opts.get("overhang", OVERHANG))

# --mouth is the airway at the rim: the hole, the number a rule across the bell's mouth
# reads. --rim is the SQUARE bell's width there, which the area law then opens out by
# 2/sqrt(pi) to enclose the same area once the section is a circle -- so --rim=80 lands a
# ø90.3 hole, and asking for --mouth=80 is how you get 80. Inverted here in closed form;
# every bell still prints the aperture it actually reached, so a wall floor biting at the
# rim would show up rather than pass as the number you asked for.
if "mouth" in opts:
    RIM = float(opts["mouth"]) / 2.0 * (math.sqrt(math.pi)/2.0 if law == "area" else 1.0)
if L < 2*RISE:  sys.exit(f"--length must be at least two rings, {2*RISE:g}mm")
if RIM <= RT:   sys.exit(f"--rim must open past the {2*RT:.0f}mm throat")

U0 = L / ((RIM/RT)**(1/GAMMA) - 1)
B  = RT*(L+U0)**GAMMA
rad = lambda z: B*((L-z)+U0)**(-GAMMA)

# A non-default profile carries its length in the filename, so a short bell that happens to
# land on the same ring count never overwrites one of the four standard sheets.
# The name is built in the loop below, not here: it carries the rim diameter and the
# height as built, and neither is known until the ring count is chosen.

DIAG = math.sqrt(2.0)                           # across the corners, against 1 across the flats


def support(h, c, s):
    """How far a rounded square reaches in a direction, s = |cos| + |sin| of it.

    s = 1 is straight out through a flat, s = sqrt(2) out through a corner. Two rounded
    squares nest if and only if this holds at both ends, because it is linear in s."""
    return (h - c)*s + c


def roundness(z, morph):
    """0 at the bore's square, 1 at the round rim. Read off the ideal profile rather than
    the built one, so how round a station is never depends on the wall floors below it."""
    z = min(z, L)
    if morph == "flare":                            # rounds in step with the flare itself
        return (rad(z) - RT)/(RIM - RT)
    if morph == "early":                            # circular by mid-horn
        return min(1.0, 2.0*z/L)
    return z/L                                      # linear: evenly along the length


def stations(plies, morph, law):
    """The n+1 apertures, from the bore's square at station 0 to the round rim at n.

    Ring k spans station k to station k+1, so there is always one more station than there
    are rings, and the top ring's outer contour is the rim opening plus the lap.

    The 2mm minimum wall has to be enforced in BOTH directions here. Rounding a corner
    pulls the diagonal in, so a station that is rounder than the one below reaches less
    far into its corners than its extra half-width suggests, and a wall that measures
    2.6mm across a flat can measure 1.97mm through a corner. Both floors are closed
    forms, so the station is placed once rather than searched for."""
    step = RISE*plies
    n = int(math.ceil(L/step))
    r, hs, cs = RT, [RT], [0.0]
    for k in range(1, n + 1):
        z0, z1 = min((k-1)*step, L), min(k*step, L)
        r += max(rad(z1) - rad(z0), MINWALL - LAP)  # the square bell's own half-width
        t = roundness(k*step, morph)
        h = r if law == "width" else r*2.0/math.sqrt(4.0 - (4.0 - math.pi)*t*t)
        h = max(h,
                hs[-1] + MINWALL - LAP,             # 2mm of wall across the flats
                (MINWALL - LAP + support(hs[-1], cs[-1], DIAG))
                / ((1.0 - t)*DIAG + t))             # and 2mm through the corners
        hs.append(h); cs.append(t*h)

    rings = []
    for k in range(n):
        rise = min((k+1)*step, L) - min(k*step, L)  # the top ring is usually a short one
        rings.append({
            "ah": hs[k],          "ac": cs[k],                  # aperture
            "oh": hs[k+1] + LAP,  "oc": cs[k+1] + LAP,          # outer: next aperture + lap
            "gain": hs[k+1] - hs[k], "rise": rise,
        })

    # Ring 0 is the flange onto the bore, and the one ring whose outer is not the next
    # station offset. The bore ends in a square annulus of ply 3mm wide -- ø25 inside, ø31
    # out -- and the flange has to cover all of it and stand proud, or there is nothing to
    # glue to but the outside of the tube. It stays a SHARP square for the same reason: the
    # face it lands on is square.
    rings[0]["oh"] = max(rings[0]["oh"], PLATE/2.0 + OVERHANG)
    rings[0]["oc"] = 0.0
    return rings, step


def check(rings):
    """Everything that has to be true before this is worth cutting."""
    notes, fails = [], []

    # Measured between the real contours, not assumed from the offset identity: ring 0 is
    # a flange and breaks that identity, and a seat is the one number a gap shows up in.
    seats = [support(a["oh"], a["oc"], s) - support(b["ah"], b["ac"], s)
             for a, b in zip(rings, rings[1:]) for s in (1.0, DIAG)]
    if min(seats) < LAP - 1e-6:
        fails.append(f"a joint seats on {min(seats):.2f}mm, under the {LAP:g}mm lap")
    notes.append(f"seat        {min(seats):.2f}-{max(seats):.2f}mm per side, "
                 f"flats and corners alike")

    # The flange has to cover the bore's end face completely. This is the joint that was
    # wrong: a ø31 aperture sat entirely outside the ø25-ø31 face and touched nothing.
    proud = rings[0]["oh"] - PLATE/2.0
    if rings[0]["ah"] > BORE/2.0 + 1e-9:
        fails.append(f"the flange aperture is ø{2*rings[0]['ah']:.1f}, wider than the "
                     f"ø{BORE:g} bore — it would not cover the end face")
    if proud < 0:
        fails.append(f"the flange is ø{2*rings[0]['oh']:.1f}, inside the ø{PLATE:g} bore "
                     f"outside — it would not cover the end face")
    notes.append(f"flange      ø{2*rings[0]['ah']:.0f} aperture in a ø{2*rings[0]['oh']:.0f} "
                 f"square: covers the bore's {BORE:g}-{PLATE:g} face, {proud:.1f}mm proud")

    walls = [(support(r["oh"], r["oc"], s) - support(r["ah"], r["ac"], s), s)
             for r in rings for s in (1.0, DIAG)]
    thin = min(walls)
    if thin[0] < MINWALL - 1e-9:
        fails.append(f"wall falls to {thin[0]:.2f}mm, under the {MINWALL}mm minimum")
    notes.append(f"wall        {thin[0]:.2f}-{max(walls)[0]:.2f}mm "
                 f"(thinnest {'across a flat' if thin[1] == 1.0 else 'through a corner'})")

    # The bore must never step inward, or a ring leaves a lip for the air to catch on.
    # Rounding the corners faster than the horn widens is exactly how that happens, so
    # it is checked through the corners as well as across the flats.
    lips = 0
    for a, b in zip(rings, rings[1:]):
        for s in (1.0, DIAG):
            if support(b["ah"], b["ac"], s) < support(a["ah"], a["ac"], s) - 1e-9:
                lips += 1
    if lips:
        fails.append(f"the bore steps inward at {lips} place(s) — the corners round "
                     "faster than the horn opens")
    notes.append(f"bore        opens at every station, corners included"
                 if not lips else f"bore        {lips} inward step(s)")

    return notes, fails


def rounded(cx, cy, h, c):
    """A square of half-width h with its corners rounded to radius c.

    c = 0 is the square that meets the bore, drawn in H and V like bell.py's. c = h is a
    true circle, drawn in four arcs like mouthpiece.py's. Everything between is both."""
    c = min(max(c, 0.0), h)
    if c <= 1e-9:
        return f"M {cx-h:.4f},{cy-h:.4f} H {cx+h:.4f} V {cy+h:.4f} H {cx-h:.4f} Z"
    s = h - c
    arc = lambda x, y: f"A {c:.4f},{c:.4f} 0 0 1 {x:.4f},{y:.4f}"
    run = lambda cmd, v: f"{cmd} {v:.4f} " if s > 1e-9 else ""   # a circle has no flats
    return (f"M {cx-s:.4f},{cy-h:.4f} "
            + run("H", cx+s) + arc(cx+h, cy-s) + " "
            + run("V", cy+s) + arc(cx+s, cy+h) + " "
            + run("H", cx-s) + arc(cx-h, cy+s) + " "
            + run("V", cy-s) + arc(cx-s, cy-h) + " Z")


def layout(rings, per):
    """Place the rings in rows of `per`, smallest first. Nothing is nested inside
    anything: these stack, and a sheet that reads in the order they stack is worth more
    than the material a nest would save. Returns the placements and the sheet."""
    GAP = M = 3.0
    placed, y, W = [], M, 0.0
    for k in range(0, len(rings), per):
        chunk = rings[k:k+per]; x = M; rowh = max(2*r["oh"] for r in chunk)
        for r in chunk:
            o = 2*r["oh"]
            placed.append((r, x + o/2.0, y + rowh/2))
            x += o + GAP
        W = max(W, x - GAP + M); y += rowh + GAP
    return placed, W, y - GAP + M


def emit(rings, plies, step, path, morph, law):
    # The bed, not the material, is what actually stops you, so the row split is chosen
    # to make the longest side as short as it will go, with area breaking ties.
    per = min(range(1, len(rings) + 1),
              key=lambda p: (round(max(layout(rings, p)[1:]), 6),
                             round(layout(rings, p)[1]*layout(rings, p)[2], 6)))
    placed, W, H = layout(rings, per)
    # Two stages, not one. A ring drawn as a single path has its aperture and its
    # outline in the same colour, so a per-colour job is free to cut the outline first
    # and free the part before its hole is in. Holes before rims.
    holes = [f'  <path d="{rounded(cx, cy, r["ah"], r["ac"])}"/>' for r, cx, cy in placed]
    lines = [f'  <path d="{rounded(cx, cy, r["oh"], r["oc"])}"/>' for r, cx, cy in placed]
    # throat is the aperture, rim is the outer edge — the same two the README's
    # "ø31 throat" and "Rim diameter" columns mean for the square bells
    ap, rim = 2*rings[0]["ah"], 2*rings[-1]["oh"]
    pathlib.Path(path).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.3f}mm" height="{H:.3f}mm"\n'
        f'     viewBox="0 0 {W:.3f} {H:.3f}">\n'
        f'  <title>Square-to-round trumpet bell - {len(rings)} rings of {plies} ply, '
        f'{len(rings)*step:g}mm built on a {L:g}mm profile, {ap:.0f}mm square throat '
        f'to ø{rim:.1f}mm round rim</title>\n'
        f'  <desc>1 user unit = 1mm. Bessel horn, gamma {GAMMA}, {ap:.0f}mm square throat '
        f'to ø{rim:.1f}mm round rim over {len(rings)*step:.0f}mm. The section morphs square '
        f'to round on the {morph} schedule, holding {law}. Each ring is {plies} '
        f'lamination(s) of {RISE:g}mm ply, {step:g}mm of rise, seating on the ring below '
        f'over {LAP}mm per side. Rings stack; they do not telescope.</desc>\n'
        + '  <g fill="none" stroke="#ff8000" stroke-width="0.1">\n'
        + "\n".join(holes) + "\n  </g>\n"
        + '  <g fill="none" stroke="#000000" stroke-width="0.1">\n'
        + "\n".join(lines) + "\n  </g>\n</svg>\n")
    return W, H


for want in ([int(args[0])] if args else [67, 20, 15, 10]):
    plies = 1
    while True:
        rings, step = stations(plies, morph, law)
        if len(rings) <= want: break
        plies += 1
    notes, fails = check(rings)
    # --out names the sheet outright. The generated name says how the bell was made
    # -- the generator, the length, the bore -- which is not what someone hunting for a
    # part needs to read. A repository that keeps one bell per size wants the size in
    # the name, and only the caller knows what it calls that size. One budget only:
    # naming four sheets with one string would write four bells over each other.
    # Everything a builder needs to tell one sheet from another, in the name:
    #   round25   square at the throat, circular by the rim -- bell.py's are square all
    #             the way up, and carry no arc on any ring
    #   204mm     the height as BUILT, rings x rise, not the --length asked for
    #   x4        how many times the sheet is cut; cutting once is the expensive mistake
    #   rim145    the outer rim it opens to
    built = len(rings) * step
    rimd = 2 * rings[-1]["oh"]
    if "out" in opts:
        if len(([int(args[0])] if args else [67, 20, 15, 10])) > 1:
            sys.exit("  --out names one sheet; give a ring budget too, "
                     "or drop --out and take the generated names")
        name = opts["out"]
    else:
        name = (f"bell-round{BORE:g}-{built:.0f}mm-{len(rings)}rings"
                f"-x{plies}-rim{rimd:.0f}-cut-files.svg")
    # throat is the aperture, rim is the outer edge — the same two the README's
    # "ø31 throat" and "Rim diameter" columns mean for the square bells
    ap, rim = 2*rings[0]["ah"], 2*rings[-1]["oh"]
    mouth = rim - 2*LAP                      # the hole, not the outside of the rim ring
    a0 = math.degrees(math.atan(rings[0]["gain"]/rings[0]["rise"]))
    a1 = math.degrees(math.atan(rings[-1]["gain"]/rings[-1]["rise"]))
    print(f"\n  {name}  {len(rings)} rings x {plies} ply ({step:g}mm rise), "
          f"{morph} morph, holding {law}")
    print(f"    section     {ap:.0f}mm square -> ø{rim:.1f}mm round over "
          f"{len(rings)*step:.0f}mm   (ø{mouth:.1f} of air at the mouth)")
    print(f"    flat angle  {a0:.1f}° -> {a1:.1f}°   (against the ring's own rise, "
          f"{rings[-1]['rise']:g}mm at the rim)")
    for n in notes:
        print(f"    {n}")
    for f in fails:
        print(f"    x {f}")
    if fails:
        sys.exit(f"  {name} not written: {len(fails)} problem(s)")
    W, H = emit(rings, plies, step, name, morph, law)
    # --order=document: a bell telescopes, so document order is already assembly order,
    # and saying so keeps one invocation for every sheet this repository writes.
    if NUMBERS == "yes":
        _tool = pathlib.Path(__file__).resolve().parent / "number_rings.py"
        _r = subprocess.run([sys.executable, str(_tool), name, "--order=document"],
                            capture_output=True, text=True)
        if _r.returncode:
            # Leave nothing cuttable behind. A sheet on disk gets cut, and an unnumbered
            # one is rings nobody can order -- discovered after the machine, not before.
            pathlib.Path(name).unlink(missing_ok=True)
            sys.exit(f"  {name} NOT written: the rings could not be numbered.\n"
                     f"{_r.stdout}{_r.stderr}"
                     f"  Fix that, or pass --numbers=no if you meant a bare sheet.")
    print(f"    sheet       {W:.0f} x {H:.0f}mm per pass, {plies} pass(es), "
          f"{len(rings)*plies} pieces, {W*H*plies/1e6:.2f} m2")
    # The leftover between a whole number of rings and L is a flat collar at the rim. On a
    # short bell it is a large fraction of the profile, so it is reported rather than left
    # to be discovered on the bed.
    over = len(rings)*step - L
    if over > 0.01:
        near = min((abs(L - m*step), m*step) for m in (len(rings) - 1, len(rings)) if m)[1]
        hint = "" if over <= step/2 else f", --length={near:g} divides evenly"
        print(f"    overshoot   {len(rings)*step:g}mm of rings on a {L:g}mm profile; the rim "
              f"ring is {step:g}mm tall and draws {step-over:g}mm of curve{hint}")
    print(f"    numbers     " + ("each ring engraved with its hex index, 0 at the bore"
                                   if NUMBERS == "yes" else
                                   "NONE — --numbers=no; the rings carry nothing"))
    print(f"    ok, written")
