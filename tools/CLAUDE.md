# CLAUDE.md

Rules for working on the bore toolchain. The reasoning, history and measurements
behind each one are in `NOTES.md`, under the heading given in brackets. Read that
section before changing the code it names.

## What lives here

- The bore toolchain: a walk through a lattice of blocks goes in, checked per-piece
  cut files come out. It ships no cut files of its own. It depends on nothing in
  `octomino-snakes`. [What this is]
- The writeup is `README.md` at the repository root. This file covers the code only.
- **Nothing is cut at two pitches.** Every design is on the 16mm block (10mm of air,
  3mm walls). Two-size machinery (`sizes.py`, `build_many`) works but has nothing to show.
- **A design naming a folder that is not there fails** in `regress.py`. Keep it
  that way: otherwise it silently gates geometry only.
- Scripts sit flat in `tools/`, at the depth `octomino-snakes/generator/` had, so
  relative paths match. `bore_split.py --write` with no path writes to
  `../parts/bore/concept`. [Where things are]

```
svgpath      <- bore_split, check
bore_split   <- bore_render, check, viewer, mcwalk, nest, hilbert, piece_render
bore_render  <- viewer, mcwalk
assemble     <- check
snakeboxvar  is a Boxes.py generator, driven by subprocess, not imported
```

## Boxes.py and licences

- `snakeboxvar.py` installs into a Boxes.py checkout (Florian Festi's,
  GPL-3.0-or-later, external, not copied). Set `SNAKEBOX_BOXES` to the checkout and
  `SNAKEBOX_PY` to its venv python. [The dependency graph]
- **`SnakeBoxVar` is the only generator.** The old `snakebox.py` is not part of this
  toolchain.
- `snakeboxvar.py`, `check.py` and `piece_render.py` are **GPL-3.0-or-later** and carry
  a header saying so. **Any new file that imports `boxes`, directly or through the Var,
  gets that header.** Everything else is CC0 (`LICENSE`); cut files stay CC0.
- `~/Software/boxes` is a shared checkout every design depends on: change it with care.

## Code rules

- **Switches must reach the gate.** `check.py` re-imports `bore_split`, so set every
  module switch on `B`, never on `globals()` of `__main__`.
  [The switches must reach the gate]
- **Outer ends are always plain** (`plain_ends()`). A rename orphans the old file and
  nothing deletes it. **A check count that rises after a rename is a warning**: look
  for orphans.
- `pin_width()` floors at the finger tooth; `pin_play()` reads `PLAY_BY_BORE` (0.025 per
  side at 10mm). Both are in `COMMON`. `--play` is for cutting coupons, not parts: when
  one fits, add the row and stop passing the flag.
- `COMMON` is built once at import. Any setter (`set_play()`, `set_blocksize()`) must
  rebuild it. Always use `bore_split.COMMON`, never a typed-out copy.
- `--bore` and `--blocksize` are one number (`blocksize = bore + 2t`); both together
  must agree.
- Gate a folder at the **pitch it was cut at**: `check.py --files` never checks pitch,
  so a wrong `--blocksize` still reports a clean run.
- `KERF` (0.15) is the full width; `BURN` is half of it, as Boxes.py means it. Don't swap
  them. `SHEET` and `THICKNESS` are both 3.0 but independent.
  [The ply is 3.0 and the kerf 0.15]
- `walk_text()` reads `<div class="walk">` with attributes. `square-rise3` keeps its walk
  **only** in its page, so it is the one place that route is tested.
  [A page turns back into cut files]

## Viewer

- `viewer.build_many()` is the only page builder; a gallery is more sets in the same
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
- **`check.py` does not consult `--refuse-elbows`**, so a green `regress.py` says nothing
  about whether a walk is bend-only. Probe through the command line, not the corpus.
- Don't "fix" elbows in the corpus by respelling walks: their names are their parameters.
- What a turn costs is set by every window of three consecutive terms (outer A,
  middle m, outer C). `bore_split.py` is the authority, not this table:

| A and C | case | m must be |
| --- | --- | ---: |
| same axis, same direction | step | >= 1 |
| same axis, opposite direction | hairpin | >= 2 |
| different axes | coil | >= 3 |

- **A walk that revisits a cell is refused.** Suspect the transcription first (wrong
  direction before wrong length); `mcwalk.py` lights up the revisited cells.
- A walk of absurd length (`N9999`) is **not** guarded: it hangs.

## What the gate cannot see

- **A passing gate means no check failed, not that the part is buildable.** Nothing
  compares a feature against its neighbours or checks joint clearance.
- *bore volume matches the walk* computes its expected value from the walk. **Never
  refactor it to call `bore_split.extent()`**: a check must not share a source with
  what it checks. [check.py, against artefacts it should reject]
- *the sheets are this walk's sections* checks the folder's `-NNofMM-` names against the
  walk. Keep it.
- A guard that did not fire is untested until you know your input reached it.
  [bore_split.py's guards]
- `doc-audit.py` and `flat-part-check.py` have not been tested against bad input yet.
- `old/` folders are gitignored and invisible to `repro.py`.
- `coil-10x10x30-1.5t` is **pinned by hash** in `as-built.sha256`: its sheets describe wood.
  Redrawing it should differ. Re-pin only with `repro.py --update`, deliberately.

## Before pushing

**`regress.py` must pass before anything is pushed** (about four minutes, Boxes.py
venv). A change that alters cut geometry and still passes has been proved *not
obviously wrong*, not right: say which. [Never regenerate what you cannot check]

```sh
~/Software/boxes/venv/bin/python regress.py
python3 bore_split.py "N3 U1 E3" --no-write                  # 2 elbows, exits 0
python3 bore_split.py "N3 U1 E3" --no-write --refuse-elbows  # refuses, exits 1
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
