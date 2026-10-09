# CLAUDE.md

**Nothing in this folder has been cut.** It is gated, and a passing gate means no
check failed, not that the part is buildable — see
`../../../../../../tools/CLAUDE.md` for what the gate cannot see. Say "gated" and
not "built" until one exists.

## What this is

A flat meander closed into a loop, with two mouths.

    N3 W3 N3 E3 N3 W3 N3 E3 N3 W5 S15 E4        52 blocks, 832mm, 2 sections

- **It is S15, not S14.** S14 ends diagonal to the start and strands block 1 as a stub.
  `bore_split.py` reporting the pair `1-52` touching is the loop, not a fault.
- `--mouth-at=31,50`: the middles of the W5 and E4 runs, cut through **opposite face
  plates** (mouthpiece in one face, bell out the other). In section 2's numbering they
  are cells 28 and 47, hence `~m28f47m` in the file name.
- **Mouths are 7 x 14, not 10 x 10.** A finger-jointed plate has no solid around a
  bore-square hole; 7mm leaves 1.5mm a side, exactly `MIN_FEATURE`.
- **Use a mouth, not `--ports`**: a port replaces the rim opening and would cut the loop
  in two.
- **Two air paths between the mouths are the design**, as on the 13-facet ring and the
  scallop in `../../../swept-curve/`.
