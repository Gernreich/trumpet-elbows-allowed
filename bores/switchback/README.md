# trumpet-switchback

A trumpet whose bore **folds back on itself twice**, which is what the
switchback in the name means. **This repository is the bore** — six laser-cut
sections that assemble into one sealed passage, at a **10mm bore on a 16mm
block**.

The **mouthpiece** and the **bell** live in
**[trumpet-parts](https://github.com/Gernreich/trumpet/tree/main/parts)**, because neither
is touched by the way the bore turns. Only the tube belongs to an instrument.
The 25mm pair that used to sit beside them, and fitted the
[coiled](https://github.com/Gernreich/trumpet/tree/main/bores/coiled) and
[octagonal](https://github.com/Gernreich/trumpet/tree/main/octagonal) trumpets equally, was
retired on 2026-09-03.

    N N1 W3 U2 E3 N3 D3 W2 U3 N1

The walk folds back twice inside a 4×4 cross-section and runs nine blocks
north overall. Every section is a **bend** — there are no elbows, so no
section needs a neighbour's plate flattened and butt-glued to it, which is
the difficulty of the whole build.

<p>
<img src="10mm/bore/minecraft_bore.png" alt="The walk built in Minecraft from translucent coloured glass, one colour per direction: long blue runs heading north to the left and right, a green box heading east, magenta where it runs west, red columns where it climbs and a grey column where it drops" width="620">
</p>

*The walk laid out block by block in Minecraft, coloured by direction — blue
north, green east, magenta west, red up, grey down. The same walk serves both
sizes; only the block it is drawn on changes.*

<!-- readme-only -->
**[Read the writeup](https://gernreich.github.io/trumpet/bores/switchback/)** — the
same text as this page, set for reading, with a table of contents.

**[The repository](https://github.com/Gernreich/trumpet/tree/main/bores/switchback)** — both
instruments and the page they regenerate from.

**[Download the whole repository as a ZIP](https://github.com/Gernreich/trumpet/archive/refs/heads/main.zip)**
— every trumpet, not this one alone; the six sections are under
`bores/switchback/`. GitHub builds it from `main` on every push, so it is never out of date.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and the playing are shown.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
generator and tool, indexed.

## What is in each folder

    10mm/
      bore/                    six cut files and the page they belong to

Six sheets each, cut in the order they are numbered. Glued mouthpiece to bore to
bell, with the two ends coming from `trumpet-parts` — see
[The mouthpiece and the bell](#the-mouthpiece-and-the-bell) for which ones. Every
sheet in a folder belongs to the same size, so there is no mixing to get wrong —
see [Do not mix the folders](#do-not-mix-the-folders) for the one thing that can
still bite.

## The two sizes

A block is the sound square plus a wall each side, so **the bore is the block
less 6mm** in 3mm stock. That one number is the whole difference between the
folders:

| | `10mm/` |
| --- | --- |
| bore | 10mm square |
| block | 16mm outside |
| centreline | 352mm |

Everything else is shared: the same walk, the same six sections in the same
order, the same shapes, the same in and out faces, no elbows in either.

The two bores use the same six filenames — see
[Do not mix the folders](#do-not-mix-the-folders).

The tab is the one thing that does not simply scale. SnakeBox's `--pin_width`
defaults to 12mm, chosen when 31 was the only block there was, and 12mm does
not fit inside the 10mm end frame at all — it raises rather than cuts.
`bore-generator` sets it to 0.48 of the sound square **but never below the
finger joint tooth**, which is `2 × thickness` and does not shrink with the
block: 12mm at 25, 6mm at 10. Without that floor the fraction alone gives
4.8mm at 10 — a seam tab narrower than the ordinary teeth beside it, which is
exactly backwards.

The notch it enters is opened by a clearance that **differs between the two
sizes, because both were measured**: the 25mm joint carried **0.0** and the 10mm
carries **0.05mm**. That is not an oversight. Zero assembles and fits well at 25mm — it
is what this trumpet was built with — and at 10mm the same zero will not go
together at all, while 0.3 rocked and 0.1 was still slightly loose. The tab
keeps its full width at both. SnakeBox leaves that at zero, on the grounds that finger joints have no
play either, but a finger joint is a dozen teeth sharing an edge and a section
seam is one tab in one notch. At zero the two are drawn the same width and the
sections will not go together.

### The cut files — `10mm/bore/`

| # | file | blocks | in | out | plate | shape | sheet |
|---|---|---|---|---|---|---|---|
| 1 | [`10mm/bore/bore10-trumpet-switchback-01of06-bend-DL-buttin-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-01of06-bend-DL-buttin-cut-files.svg) | 1-3 | N | W | 2×2 | BDL~a | 241×56mm |
| 2 | [`10mm/bore/bore10-trumpet-switchback-02of06-bend-LUUR-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-02of06-bend-LUUR-cut-files.svg) | 4-8 | W | E | 2×3 | BLUUR | 344×72mm |
| 3 | [`10mm/bore/bore10-trumpet-switchback-03of06-bend-RD-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-03of06-bend-RD-cut-files.svg) | 9-11 | E | N | 2×2 | BRD | 253×56mm |
| 4 | [`10mm/bore/bore10-trumpet-switchback-04of06-bend-DL-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-04of06-bend-DL-cut-files.svg) | 12-14 | N | D | 2×2 | BDL | 247×59mm |
| 5 | [`10mm/bore/bore10-trumpet-switchback-05of06-bend-DLLU-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-05of06-bend-DLLU-cut-files.svg) | 15-19 | D | U | 3×2 | BDLLU | 370×59mm |
| 6 | [`10mm/bore/bore10-trumpet-switchback-06of06-bend-RD-buttout-cut-files.svg`](10mm/bore/bore10-trumpet-switchback-06of06-bend-RD-buttout-cut-files.svg) | 20-22 | U | N | 2×2 | BRD~b | 253×56mm |

### Either size

**The two end sections are flat where they face outward.** Every seam inside
the bore is a tab entering a notch, but sections 1 and 6 have nothing beyond
them to couple to — what meets them is the mouthpiece at one end and the bell
at the other, and both present a flat plate that glues straight onto the end
face. A tab standing 3mm proud of that face would hold the plate off it. So
section 1's mouth end and section 6's bell end are plain edges, which is what
`~a` and `~b` mark in the shape column and `_buttin` / `_buttout` in the
filename.

Sections 1 and 4 were the same shape until then, and 3 and 6 with them; the
flat ends make all six distinct. They were always cut separately anyway, so
each carries its own engraved number and the assembly order stays readable on
the bench.

Both end sections are a single straight block either side of their turn, which
is the least a section can have.

## The mouthpiece and the bell

**Neither is here.** Both live in
**[trumpet-parts](https://github.com/Gernreich/trumpet/tree/main/parts)** with the rest of
the shared parts, because neither depends on how a bore turns. What this repository owns is the tube. These are the ones to cut with
it:

| | for `10mm/` |
| --- | --- |
| mouthpiece | [`mouthpiece-bore10-trumpet-parts-cut-files.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/mouthpiece/mouthpiece-bore10-trumpet-parts-cut-files.svg) |
| station one | 10mm square in a 16mm plate |
| bell | [`bell-round10-153mm-17rings-x3-rim86-cut-files.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/bell/bell-round10-153mm-17rings-x3-rim86-cut-files.svg) |

**The bell sheet draws every ring once and is cut more than once.** A ring is a
stack of 3mm laminations, three of them here, so the sheet goes through the
machine that many times and you glue the copies up into one ring. Cut it once
and you get a **51mm bell** instead of 153, which is
the single most expensive mistake in this build. The bore is not like this:
those six sheets are cut once each.

Both bells are 17 rings and both seat on the ring below over **3.00mm per side
at every joint** — see
[bell-round.py](https://github.com/Gernreich/trumpet/blob/main/parts/bell/bell-round.py).
Each mouthpiece is 30 rings of one lamination, so its sheet is cut once.

**Every ring on all four carries its own number, engraved in blue before the
cut** — hex, one character, 0 at the bore end. A mouthpiece narrows to the throat and
opens again, so its backbore and its cup pass through the same diameters: once the rings
are off the bed, nothing but the number says which half a given ring belongs to. The
generators engrave these themselves, so a regenerated sheet still has them.

**Sheets regenerated from now on also carry an orientation tick** — a short mark on the
baseline, right of the number. A ring is a circle and has no top, so a number alone is
ambiguous: turned over, a `3` reads as an `E` and a `6` reads as a `9`. The tick says which
way up the label was engraved. All four sheets carry it. The parts already glued up were
cut before it existed and have only their number, which costs nothing on a stack that is
already in order — but cut a fresh sheet and the ring you pick up tells you which way round
it goes.
The mouthpiece uses the `trumpet` layout, which spends its length on the
backbore and keeps a short cup, as a real one does.

Each also carries a section drawing — an axial slice showing the bore climbing
one staircase and the outside climbing another:
[`bell-round10-153mm-17rings-x3-rim86-section.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/bell/bell-round10-153mm-17rings-x3-rim86-section.svg)
and
[`mouthpiece-bore10-trumpet-parts-section.svg`](https://github.com/Gernreich/trumpet/blob/main/parts/mouthpiece/mouthpiece-bore10-trumpet-parts-section.svg).
**Those are display only — never cut one.**

**Both live in `trumpet-parts`, not here.** They were kept in this repository
until 2026-09-02 on the grounds that nothing else cut them — which was true of
what had been cut, and not true of what fits: the pair then kept here suited any
channel of their size, and holding them here hid two general parts inside one
instrument. The rule
`trumpet-coiled` already stated is the one that decides it: neither end is
touched by the way a bore turns, so **only the tube belongs to an instrument**.

The 10mm pair suit no other bore *today*, which is an accident of the current
lineup rather than a principle, so they go with the rest. There is still one copy
of each sheet and nothing to keep in step.

**Every ring is engraved with its hex index**, 0 on the smallest. Rings glued
in the wrong order is rings unglued, and consecutive rings differ by about two
millimetres — nothing you can judge by eye once the parts are off the bed.

## Do not mix the folders

The two bores use the **same six filenames**, because the shapes really are the
same and only the pitch differs. Nothing stops you cutting from the wrong
folder, and the parts will look plausible right up until they do not fit. The
sheet size is the tell — section 4, say, is **247×59mm on the small bore and
397×89mm on the large** — and the tables above carry every sheet at both sizes.

The bell and the mouthpiece are safe: their filenames differ between sizes.

## The pages

**[`10mm/bore/bore.html`](10mm/bore/bore.html)** is the bore, drawn and
turnable. Drag to turn, colour by direction or by section, and a slider follows
the bore from the mouth. It is self-contained apart from its fonts, so it opens
from a checkout as readily as from the published page.

`sizes.html`, which held both sizes in one page with a control to swap them,
went with the 25mm folder on 2026-09-03.

## Where it comes from

Generated by
**[bore-generator](https://github.com/Gernreich/trumpet/tree/main/tools)**, which holds
the walk as `walks/trumpet_switchback.txt` and gates these files in
`regress.py`. Its
[CLAUDE.md](https://github.com/Gernreich/trumpet/blob/main/tools/CLAUDE.md)
carries the conventions — the elbow rule, which switches reach the gate, and why
a regenerated folder is not a checked one. To rebuild and check:

```sh
cd ../../tools
W="$(cat walks/trumpet_switchback.txt)"
D=.
python3 bore_split.py --blocksize=16 --refuse-elbows "$W" --write $D/10mm/bore
~/boxes/venv/bin/python regress.py trumpet
```

`--refuse-elbows` is not optional here: it stops before writing anything if the
walk would cost an elbow. Neither is the folder on the end of each line — leave
`--blocksize=16` off the second command and it quietly fills `10mm/bore/` with
full-size parts under the small set's names, which no drawing shows.

The mouthpiece and the bell come from
**[trumpet-parts](https://github.com/Gernreich/trumpet/tree/main/parts)**, not from here:

```sh
cd ../../parts/mouthpiece
python3 mouthpiece-round.py --bore=10 --rim=17 --layout=trumpet mouthpiece-bore10-trumpet-parts-cut-files.svg
cd ../bell && python3 bell-round.py 17 --bore=10 --length=152 --mouth=80 \
    --out=bell-round10-153mm-17rings-x3-rim86-cut-files.svg
```

Both sizes gate at **194 checks, 0 failed**. Nothing here should be cut from a
file that has not passed it.

## Licence

CC0 1.0 Universal. See `LICENSE`.
