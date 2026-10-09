# CLAUDE.md

Rules for the long-straight coils. The reasoning, history and measurements behind
each one are in `NOTES.md`, under the heading given in brackets. Read that section
before changing what it names.

## What lives here

- **No README in this folder.** The reading page for the built coil is
  `../../../../../../three-turn/`; the writeup is at the repository root.
- Straight blocks are 30mm, turning blocks 16mm cubes, the airway 10 x 10mm throughout.
  Four designs that are **one coil truncated at its `N` spacers**, each with its walk in
  `tools/walks/`. [What this is]
- A turn is four terms and a group three: lengths come in whole groups of 11 blocks.
  Folder names are **measured turns** (0.75, 1.5, 2.25, 3).
- **`spiral_metrics.js` reads a quarter turn low on a walk that closes.** Don't take a
  turn count from it without checking whether the walk closes.

| folder | blocks | status |
| --- | ---: | --- |
| `coil-10x10x30-0.75t/` | 11 | uncut |
| `coil-10x10x30-1.5t/` | 22 | **cut, pinned**: same walk file as `../fold2` |
| `coil-10x10x30-2.25t/` | 33 | uncut |
| `coil-10x10x30-3t/` | 44 | built, lives in `../../../../../../built/coil-fold2-long-straight-3t` |

- The bell and mouthpiece are in `../../../../../bell` and `../../../../../mouthpiece`.
  **Name them, never copy them.**

## The pinned folder

**`coil-10x10x30-1.5t/` records wood that was cut**, at a 0.1mm kerf where the generator
now assumes 0.15. **Do not cut from it as it stands**: its slots come out about 0.05mm
wide. Regenerate it first (switches in `tools/regress.py`), knowing that overwrites the
record. Its six sheets are pinned by hash in `tools/as-built.sha256`, and `tools/repro.py`
fails with *this file describes wood that was cut* if one changes. Re-pin only with
`repro.py --update`, as a deliberate act. **Ask before any rebuild that changes it.**

## Toolchain rules

- **One generator, `../../../../../../tools`. Never introduce a copy.** It installs into
  Boxes.py as `SnakeBoxVar` beside `SnakeBox`; `~/Software/boxes` is a shared checkout.
  [One toolchain, not a fork]
- Only straight blocks stretch (`--straight`); a turn stays a cube. `extent()` is the
  single place that decides. [A turn is a cube because it has to be]
- No single number maps a lattice index to mm. Positions come from `block_boxes()`; its
  cubic-cell test is the first thing to rerun if geometry looks wrong. Within one piece a
  straight and a turn may not share a column. [The lattice is not a grid any more]
- **The 1mm spurs on five of six plates are known and benign. Do not "fix" them**: the
  cause is in Boxes.py. [The 1mm spurs are known…]
- **After touching Boxes.py, check a shipped file still reproduces**, writing to a temp
  tree that mirrors the real folder stack (sheet names come from the path):

```sh
cd ../../../../../../tools
W="$(cat walks/coil_fold2.txt)"
rm -rf /tmp/f && mkdir -p /tmp/f/coil/fold2
~/Software/boxes/venv/bin/python bore_split.py --blocksize=16 "$W" \
    --write /tmp/f/coil/fold2/bore
B=bore10-coil-fold2-02of06-bend-LUUR-cut-files.svg
cmp /tmp/f/coil/fold2/bore/cut-files/$B \
    ../parts/bore/concept/walk/coil/fold2/bore/cut-files/$B
```

## Joint fit (the author's standing decisions)

- **One tab size and one notch size across the whole bore.**
- **Adjust by narrowing the notch, never by widening the tab.** Tune with `PLAY_BY_BORE`,
  which `pin_play()` reads; this design does not use `--notch`.
- The current fit is 0.05mm clearance (tab 6.0, notch 6.05), confirmed on the bench.
- **Before changing it, ask what is already cut.** [Commands]

## Gate and render

- **Don't trust a passing gate.** The meaningful number is the voxelised bore volume
  matching the walk's (55520 mm3 on the 1.5t). Re-derive it by hand after changing
  geometry. [Do not trust a passing gate]
- **`regress.py` cannot see a broken `extent()`**: measured and expected volume move
  together. Check a change to `extent()` by hand.
  [`regress.py` gates these designs, and what it cannot see]
- `check.py` needs **the same `--bore=10 --straight=30`** as the writer; `--files` never
  checks pitch.
- Use the Boxes.py venv: the system python writes every file, then dies on shapely
  ungated.
- `check_page` catches a stale page but not a wrong picture. **Look at the render
  after changing geometry.**
- `coils.py` writes `coils.html`: four coils in one `build_many` viewer, scale locked
  to the longest, camera kept on switch. [One viewer, one or several coils]

## Commands

**`--title` is part of the command**: without it a shipped page is silently retitled.

```sh
cd tools
# the 1.5t's walk is named for its design, because ../fold2 is cut from the same file
for t in 0.75:coil-0.75t:"¾ Turn" 1.5:coil_fold2:"1½ Turns" \
         2.25:coil-2.25t:"2¼ Turns" 3:coil-3t:"3 Turns"; do
  n=${t%%:*}; rest=${t#*:}; w=${rest%%:*}; lab=${rest#*:}
  W="$(cat walks/$w.txt)"
  ~/Software/boxes/venv/bin/python bore_split.py --bore=10 --straight=30 \
      --title="10x10x30 Coil, $lab" "$W" \
      --write ../parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-${n}t
done
python3 coils.py                    # coils.html here, all four in one viewer
```

```sh
~/Software/boxes/venv/bin/python check.py "$(cat walks/coil_fold2.txt)" \
    --bore=10 --straight=30 \
    --files ../parts/bore/concept/walk/coil/fold2-long-straight/coil-10x10x30-1.5t/cut-files
cd ../../../../../../tools && ~/Software/boxes/venv/bin/python regress.py        # every design
cd ../../../../../../tools && ~/Software/boxes/venv/bin/python regress.py coil   # the coils
```

After editing this family's page, check strokes here, regenerate and audit:

```sh
G=../../../../../../../lasermade-tools
python3 $G/svg-stroke-check.py --dir . --quiet
~/LaserMadeMusic/GIT/lasermade-tools/rebuild-page.sh ../../../../../../three-turn
```

## Cut files belong to the author

- The author edits SVGs in Inkscape during a session. **Stage by name**, never
  `git add -A`.
- The generator never deletes what it stops writing: check for orphans after a regenerate.
- Cut order by colour: **blue engraves, then green → orange → cyan → black**; violet
  `#8000ff` means skip.
