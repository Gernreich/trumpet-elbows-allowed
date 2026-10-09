# CLAUDE.md

Rules for this bore's cut files. The reasoning, history and measurements behind
each one are in `NOTES.md`, under the heading given in brackets. Read that section
before changing what it names.

## What lives here

- **No README in this folder.** The reading page is `../../../../../../switchback/`;
  the repository's writeup is at its root.
- **One 10mm bore**: six sections on a 16mm block, 22 blocks, 352mm of centreline. One
  pitch, so no `sizes.html`. [What this is]
- Everything here is **generated** by `../../../../../../tools` (see its `CLAUDE.md`).
  Nothing is hand-authored except this file and `NOTES.md`.
- **The bell and mouthpiece live in `../../../../../../..` (`parts/`), never here.**
  Only the tube belongs to an instrument. Do not move them back without deciding what
  changed about that argument.
- Filed as a **coil**, not a meander: it measures 450° about a north–south axis. The
  name lives in the folder, the walk file, the `regress.py` entry, the cut-file names
  and the page title: **change all five together or none**.
  [Why this is filed as a coil and not a meander]

## The walk

- The walk is the whole specification: `N1 W3 U2 E3 N3 D3 W2 U3 N1`. Blocks = 1 + the
  sum of the numbers. Axes match Minecraft (`U`/`D` ±Y, `N` −Z, `S` +Z, `E` +X, `W` −X).
  [The design is one line]
- **It is stored in `../../../../../../tools/walks/coil_fold2.txt`. Never transcribe it
  from memory; read the file.**
- There is no bare-letter lead-out any more: you leave facing the last term.
- **Every window sits exactly on its minimum.** Shortening any interior term strands
  turns, so any shorter proposal must add length elsewhere. Both hairpins are at m = 2.
  See the step / hairpin / coil table in `../../../../../../tools/CLAUDE.md`.
  [Every turn a bend — the rule that shapes the walk]
- The end sections hold one straight block either side of their turn, the minimum.
  They cannot be trimmed; if a socket must seat *into* a section, grow the walk.
  [The ends are as short as a section can be]

## Pitch and flags

- **The bore is the air (10mm); the block is air + two walls (16mm).** This design is
  gated at `--blocksize=16`. Omitting it fills `bore/` with stock-pitch parts under
  this set's names. [One block is 16mm, not 10]
- **Don't type the Boxes.py flags**: `bore_split.py` builds them from its own constants.
  `--thickness` is the sheet and `--burn` is half the kerf. Print them rather than
  trusting a written list.
- Sheet names carry the bore (`bore10-coil-fold2-01of06-…`), plus a `<title>` and `<desc>`.
- `--pin_width` is derived (0.48 × the sound square, floored at the finger tooth: 6mm
  here). Never pass 12.

## Sections and orphans

- Outer ends are plain (`buttin` / `buttout`), so all six sections are distinct.
  [Six sections, six shapes]
- **A rename orphans files and the gate counts them.** A rising check count after a
  rename is a warning. Check `bore/` for orphans after every regenerate.
- A walk that gains or loses a section renames **all** six sheets.

## Cut files belong to the author

The author edits SVGs in Inkscape during a session.

- **Stage by name.** Never `git add -A` or `git add .`.
- **Never regenerate a hand-edited cut file without asking.** Regenerating rewrites every
  SVG in the target folder.
- Cut order by colour: **blue engraves, then green → orange → cyan → black**; violet
  `#8000ff` means skip. These nets use blue (section number) then black.

## Running the generator and gate

- **Use the Boxes.py venv** (`~/Software/boxes/venv/bin/python`). Under the system
  python, `--write` writes all six files and **then** dies on shapely, leaving them
  ungated. Start `regress.py` with the same interpreter.
  [The gate does not run under the system python]
- `--files` never checks pitch: the wrong `--blocksize` still passes.
- `regress.py` only checks the sheets **when the folder exists**; renaming a folder
  makes it silently gate geometry alone.
- The gate does not look at the bell or the mouthpiece; their own generators check
  them before writing.
- `--refuse-elbows` is off by default in this repository. Rewrite this bore **with** it.
  `check.py` does not consult it.

```sh
G=../../../../../../../lasermade-tools
S=../../../../../../tools
```

Test a walk without writing anything (always do this before proposing a change):

```sh
cd $S && python3 bore_split.py --no-write "N1 W3 U2 E3 N3 D3 W2 U3 N1"
```

Regenerate the cut files (rewrites everything: **ask first**):

```sh
cd $S
W="$(cat walks/coil_fold2.txt)"
D=../parts/bore/concept/walk/coil/fold2
~/Software/boxes/venv/bin/python bore_split.py --blocksize=16 "$W" --write $D/bore
```

Gate alone, and the checks (`$W` and `$D` as above, run from `$S`):

```sh
~/Software/boxes/venv/bin/python check.py "$W" --blocksize=16 --files $D/bore/cut-files
python3 $G/svg-stroke-check.py --dir . --quiet   # stroke declared twice, disagreeing
cd $S && ~/Software/boxes/venv/bin/python regress.py      # every design in the library
```

The ends, from `parts/` (each rebuilds the shipped sheet byte for byte):

```sh
cd ../../../../../mouthpiece && python3 mouthpiece-round.py
cd ../bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80
```

## Publishing

- After editing this bore's page, regenerate and audit. **Read the audit before
  pushing.** `.doc-audit-ignore` lists `bore_split.py` and `regress.py`.

```sh
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ../../../../../../switchback
```

- Pages deploys from `main` with a per-sha concurrency group. `index.html` is
  committed, so a stale one publishes stale content. **Match the deploy to your SHA**,
  not to the latest run: [Publishing]

```sh
SHA=$(git rev-parse HEAD)
gh run list -L5 --json status,conclusion,headSha \
  -q ".[] | select(.headSha==\"$SHA\") | .status+\" \"+(.conclusion//\"-\")"
```
