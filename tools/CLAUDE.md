# CLAUDE.md

Rules for working on the bore toolchain. The reasoning, history and measurements
behind each one are in `NOTES.md`, under the heading given in brackets. Read that
section before changing the code it names.

## What lives here

- The bore toolchain: a walk through a lattice of blocks goes in, checked per-piece
  cut files come out. It ships no cut files of its own. It depends on nothing in
  `octomino-snakes`. [What this is]
- The writeup is `README.md` at the repository root. This file covers the code only.
- **Nothing is cut at two pitches.** Every design is on the 16 mm block (10 mm of air,
  3 mm walls). Two-size machinery (`sizes`, `build_sets`) works but has nothing to show.
- **A design naming a folder that is not there fails** in `regress`. Keep it
  that way: otherwise it silently gates geometry only.
- **Every tool is Rust since 2026-11-08** (Python until then, byte-identical in what it
  writes). The sources sit flat in `tools/`; `cargo build --release` at the repository
  root puts the programs in `../target/release/`. `bore_split` given neither
  `--write DIR` nor `--no-write` writes to `../../test`, beside the repository, not into
  it. [Where things are]

```
lib.rs        the trumpet crate: every module below, shared by every program
bore.rs       the walk, the split, the sheets      <- every program
snakebox.rs   SnakeBoxVar and the Boxes.py it drew through   <- bore.rs, check.rs
geom.rs       polygon work (geo)                   <- bore.rs, check.rs
assemble.rs   the voxel model                      <- check.rs
check.rs      the gate                             <- bore.rs (--write), check_main.rs
viewer.rs     the page data; viewer_page.rs the page  <- bore.rs (--write), coils, sizes
render.rs     stills                               <- bore_render_main.rs, mcwalk.rs
designs.rs    the regress table                    <- regress.rs, repro.rs
xml.rs, pyrandom.rs   ElementTree and random.Random, as far as the tools used them
```

Programs: `bore_split_main.rs`, `check_main.rs`, `viewer_main.rs`,
`bore_render_main.rs`, `snakebox_main.rs`, and `regress.rs`, `repro.rs`, `coils.rs`,
`sizes.rs`, `hilbert.rs`, `mcwalk.rs` (with `mcwalk_page.rs`), `nest.rs`,
`bed_sheets.rs`, `piece_render.rs`. `glyphs.rs` is the engraving font.

## Boxes.py and licences

- `snakebox.rs` is `snakeboxvar.py` and the parts of Boxes.py (Florian Festi's,
  GPL-3.0-or-later) it drew through, ported to the byte: no Boxes.py checkout is needed.
  One deliberate departure: short plain plate borders are drawn straight, not as
  Boxes.py's 0.93 mm stub. [The dependency graph] [The stub Boxes.py drew]
- **`SnakeBoxVar` is the only generator.** The old `snakebox.py` is not part of this
  toolchain.
- `snakebox.rs`, `check.rs`, `check_main.rs` and `piece_render.rs` are
  **GPL-3.0-or-later** and carry a header saying so. **Any new file that ports Boxes.py
  code gets that header.** A program that links `snakebox.rs` is a GPL work as built,
  while the CC0 sources in it (`bore.rs` and the rest) stay CC0 on their own.
  Everything else is CC0 (`LICENSE`); cut files stay CC0.

## Code rules

- **Switches must reach the gate.** Every switch lives on the one `Design` the writer
  holds, and `--write` hands that same `Design` to the gate; never build a fresh
  `Design::default()` on the way. [The switches must reach the gate]
- **Outer ends are always plain** (`plain_ends()`). A rename orphans the old file and
  nothing deletes it. **A check count that rises after a rename is a warning**: look
  for orphans.
- `pin_width()` floors at the finger tooth; `pin_play()` reads `PLAY_BY_BORE` (0.025 per
  side at 10 mm). Both are in `Design.common`. `--play` is for cutting coupons, not parts: when
  one fits, add the row and stop passing the flag.
- `Design.common` is the section generator's argument list. Any setter (`set_play()`,
  `set_blocksize()`) must rebuild it (`make_common()`). Always use it, never a typed-out
  copy.
- `--bore` and `--blocksize` are one number (`blocksize = bore + 2t`); both together
  must agree.
- Gate a folder at the **pitch it was cut at**: `check --files` never checks pitch,
  so a wrong `--blocksize` still reports a clean run.
- `KERF` (0.15) is the full width; `BURN` is half of it, as Boxes.py means it. Don't swap
  them. `SHEET` and `THICKNESS` are both 3.0 but independent.
  [The ply is 3.0 and the kerf 0.15]
- `walk_text()` reads `<div class="walk">` with attributes. `square-rise3` keeps its walk
  **only** in its page, so it is the one place that route is tested.
  [A page turns back into cut files]

## Viewer

- `viewer PAGE --solidcraft FILE [switches]` (solidcraft_design.rs) writes a bore page's
  SolidCraft design: every plate and wall as snakebox cuts it (`Snake::render_local`,
  burn 0), placed in 3D, points fitted as sums of the parameters by nudging each one.
  It refuses unless its blocks are the page's. Each page's
  `solidcraft/` and `step/` are redrawn by lane B of lasermade-tools' all-gates.sh.

- `viewer::build_sets()` is the only page builder; a gallery is more sets in the same
  viewer, never a second viewer. Rebuild everything derived from `D` when the set
  changes; the switch keeps the camera. [One viewer, one bore or several]
- Scale is locked to the biggest set and cells are drawn in millimetres (`u` per step).
- **`rot()` scales by millimetres: never pass it a direction.** Use
  `rotC(p, centre.c, 1)` for normals.
- **Screenshot the page after any change to drawing code.** No gate sees the picture.

## Elbows and the walk

- Every turn folds into a bend wherever it can (`FOLD_TURNS`, always on). **Elbows are
  allowed by default**; `--refuse-elbows` refuses them before writing and exits 1. The
  piece kind is `elbow` everywhere. [Standing decisions]
- **`check` does not consult `--refuse-elbows`**, so a green `regress` says nothing
  about whether a walk is bend-only. Probe through the command line, not the corpus.
- Don't "fix" elbows in the corpus by respelling walks: their names are their parameters.
- What a turn costs is set by every window of three consecutive terms (outer A,
  middle m, outer C). `bore_split` is the authority, not this table:

| A and C | case | m must be |
| --- | --- | ---: |
| same axis, same direction | step | >= 1 |
| same axis, opposite direction | hairpin | >= 2 |
| different axes | coil | >= 3 |

- **A walk that revisits a cell is refused.** Suspect the transcription first (wrong
  direction before wrong length); `mcwalk` lights up the revisited cells.
- A walk of absurd length (`N9999`) is **not** guarded: it hangs.

## What the gate cannot see

- **A passing gate means no check failed, not that the part is buildable.** Nothing
  compares a feature against its neighbours or checks joint clearance.
- *bore volume matches the walk* computes its expected value from the walk. **Never
  refactor it to call `bore_split.extent()`**: a check must not share a source with
  what it checks. [check, against artefacts it should reject]
- *the sheets are this walk's sections* checks the folder's `-NNofMM-` names against the
  walk. Keep it.
- A guard that did not fire is untested until you know your input reached it.
  [bore_split's guards]
- `doc-audit` and `flat-part-check` have not been tested against bad input yet.
- `old/` folders are gitignored and invisible to `repro`.
- `coil-10x10x30-1.5t/as-cut/` is **pinned by hash** in `as-built.sha256`: its sheets describe
  wood; its `cut-files/` is an ordinary redraw. A redraw should differ from `as-cut/`. Re-pin only with `repro --update`, deliberately.

## Before pushing

**`regress` must pass before anything is pushed** (about three seconds on ten cores). A change that alters cut geometry and still passes has been proved *not
obviously wrong*, not right: say which. [Never regenerate what you cannot check]

```sh
../target/release/regress
../target/release/bore_split "N3 U1 E3" --no-write                  # 2 elbows, exits 0
../target/release/bore_split "N3 U1 E3" --no-write --refuse-elbows  # refuses, exits 1
```

## Cut files and publishing

- Output lands in other repositories. `--write` rewrites **every** file in its target
  directory: ask before pointing it at one with cut files, and never at hand-nested or
  hand-edited work. [Cut files belong to the author]
- Cut order by colour: **blue engraves, then green → orange → cyan → black**; violet
  `#8000ff` means skip. Bore nets use blue (section number) then black.
- Pages deploys from `main`. `index.html` is committed, so regenerate it after editing
  the README and read the audit before pushing:

```sh
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ..          # the root README and its page
```
