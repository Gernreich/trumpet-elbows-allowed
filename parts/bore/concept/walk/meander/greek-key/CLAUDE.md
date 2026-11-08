# CLAUDE.md

**The bore has been cut and press-fit**, not glued, photographed in
`../../../../../../greek-spiral/`. No mouthpiece or bell is fitted and it has not
been blown, so it is a cut bore, not an instrument, and stays in `concept/`. A
passing gate means no check failed, not that the part is buildable — see
`../../../../../../tools/CLAUDE.md` for what the gate cannot see.

## What this is

The bore of a trumpet whose walk is a flat meander, the Greek key. Cut files only: the
generator is **`../../../../../../tools`**, the mouthpiece and bell are
**`../../../../..`**.

**There is no README in this folder.** The reading page is
`../../../../../../greek-spiral/`; the writeup is at the repository root.

- Walk: `walks/greek_spiral.txt`, gated by `../../../../../../tools/regress.rs` at
  `--bore=10`, pointing at `./bore`.

## One section is the whole design

- **No assembly order**: one section on two sheets; `01of01` is not a sequence.
- **Seam clearance is inert**: there is no section-to-section joint. Don't cite
  `coil/fold2`'s measured play as evidence about this bore.
- **Both ends are plain** on the one piece: `-buttin-buttout-`.
- If a change makes this split into more than one section, the walk changed: check it first.

## Regenerate with the built programs

`bore_split` and `check` are programs in the repository's `target/release/` (Python in
the Boxes.py venv until 2026-11-08); the README's rebuild block shows the command. 16 mm is
the default block; if you pass `--blocksize`, pass the same to **both** commands.

## Sheet 1 is 592 mm on a 600 mm bed

The largest sheet by area (592.0 x 284.4 mm), not the widest; width runs out first. The
two sheets within 1.5 mm of the bed width:

| sheet | width | margin | gated by |
|---|---|---|---|
| `double-volute` narrow panels | 598.85 mm | 1.15 mm | `ribbon_bore.rs` |
| `hilbert/open` section 14 | 598.60 mm | 1.40 mm | `check` |

**Two gates, one bed**: each sees only its own sheets, so read a change against both.
`sheet fits the bed` passes right up to 600.0 and the nester may re-split instead of
failing: **compare reported sheet sizes after any change**, and don't assume two sheets
stays two.

## Colour

Shared order: **blue engraves, then green → orange → cyan → black**; violet `#8000ff`
means skip. **This bore engraves nothing**: black is the only colour. A section number
would read `1` on every part, and the plate is the jig (all eleven wall lengths differ).

## Publishing

This folder publishes nothing. After editing the page's README, regenerate and audit
there, and read the audit before pushing:

```sh
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ../../../../../../greek-spiral
```
