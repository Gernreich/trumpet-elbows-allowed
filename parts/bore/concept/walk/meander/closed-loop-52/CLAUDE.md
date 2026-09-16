# CLAUDE.md

**Nothing in this folder has been cut.** It is gated, and a passing gate means no
check failed, not that the part is buildable — see
`../../../../../../tools/CLAUDE.md` for what the gate cannot see. Say "gated" and
not "built" until one exists.

## What this is

A flat meander closed into a loop, with two mouths.

    N3 W3 N3 E3 N3 W3 N3 E3 N3 W5 S15 E4        52 blocks, 832mm, 2 sections

**It is S15, not S14.** Written with S14 the walk ends one block north AND one
west of where it began — diagonal, not beside it — so it shuts onto block 2 and
strands block 1 as a stub. S15 ends beside block 1, and `bore_split.py` reports
the pair `1-52` touching: that warning is the loop, not a fault.

## The mouths

`--mouth-at=31,50`: block 31 is the middle of the W5 run, block 50 the middle of
the E4 run, and they are cut through **opposite face plates**, so the mouthpiece
enters one face and the bell leaves the other. In section 2's own numbering those
are cells 28 and 47, which is where the `~m28f47m` in the file name comes from.

**They are 7 x 14, not 10 x 10.** Every plate a walk is cut from is a bore-wide
body with finger teeth along its edges, and the teeth alternate, so they are not
material you can leave around a hole: 10mm of solid across a 10mm bore. A
bore-square hole severs the plate. 7mm leaves 1.5mm either side, MIN_FEATURE
exactly. The ribbon cheeks take a 10 x 10 because they are joined by tab-and-slot
and are solid between their mortices; this is joined by fingers.

A **port** would not do instead. `--ports` replaces a section's rim opening with
a face opening, which on a loop cuts it in two at that block. A mouth adds a hole
and leaves the section's ends alone.

## Two paths, on purpose

The loop gives the air both ways round between the mouths. That is the design,
as it is on the 13-facet ring and the scallop in `../../../swept-curve/`.
