# bore-stretched

Bores whose **straight blocks are longer than their turns**. The cross-section
stays square the whole way — 10 × 10mm of air — but a block that runs straight
is 30mm long, while a block that turns is a 16mm cube. The same walk that gives
352mm of centreline on a uniform 16mm block gives **548mm** here, without
changing a single term of it.

    N N1 W3 U2 E3 N3 D3 W2 U3 N1

This is a **test**, not an instrument: it has no mouthpiece and no bell, and the
walk was borrowed from
[trumpet-switchback](https://github.com/Gernreich/trumpet-switchback)
rather than designed for a stretched lattice.

<!-- readme-only -->
**[Read the writeup](https://gernreich.github.io/bore-stretched/)** — the same
text as this page, set for reading, with a table of contents.

**[Download everything as a ZIP](https://github.com/Gernreich/bore-stretched/archive/refs/heads/main.zip)**
— GitHub builds it from `main` on every push, so it is never out of date.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where
the cutting and the playing are shown.

**[The rest of the build files](https://gernreich.github.io/)** — every
instrument, generator and tool, indexed.

<div class="tw">
<table>
<tr>
<td align="center"><a href="coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg"><img src="coil-10x10x30-1.5t/previews/bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg" alt="The first of six sheets for the 1.5-turn coil: flat finger-jointed plates for one bend of a 10 × 10mm bore, each engraved with its number" width="620"></a></td>
</tr>
<tr>
<td align="center"><sub>bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg &middot; 325.2 &times; 70.2mm sheet</sub></td>
</tr>
</table>
</div>

## Why a turn has to be a cube

A block is 16mm outside and 10mm of air, a 3mm wall each side. Along the way the
bore travels it can be any length, and here it is 30mm — but **only if it runs
straight**. A block where the bore turns has two openings on two different
faces, and both have to sit square in the same 16mm frame, so a turning block is
a 16mm cube. Stretch it and one opening would end up longer than the other.

That is the whole design. Eight of the 22 blocks turn, so:

| | |
| --- | ---: |
| straight blocks, 30mm each | 14 |
| turning blocks, 16mm cubes | 8 |
| centreline | 548mm |
| the same walk on a uniform 16mm block | 352mm |
| envelope | 92 × 92 × 152mm |

**1.56x, not 3x.** Stretching 30/16 is 1.875 per straight block, and the turns
do not stretch at all.

## Four coils, one walk truncated

They are the same coil stopped at different points. `WUED` repeats, an `N` spacer
lands every three terms, and the walk can be cut at any spacer:

| folder | turns | blocks | centreline | sections | envelope |
| --- | ---: | ---: | ---: | ---: | --- |
| [`coil-10x10x30-0.75t`](coil-10x10x30-0.75t/) | 0.75 | 11 | 274mm | 3 | 92 × 62 × 76mm |
| [`coil-10x10x30-1.5t`](coil-10x10x30-1.5t/) | 1.5 | 22 | 548mm | 6 | 92 × 92 × 152mm |
| [`coil-10x10x30-2.25t`](coil-10x10x30-2.25t/) | 2.25 | 33 | 822mm | 9 | 122 × 92 × 228mm |
| [`coil-10x10x30-3t`](coil-10x10x30-3t/) | 3 | 44 | 1096mm | 12 | 122 × 122 × 304mm |

Each group costs exactly **11 blocks**, so shortening to the previous spacer is a
flat price — 11, 22, 33, 44. All four enter and leave north with flat outer faces, and none has
an elbow.

**[`coils.html`](coils.html) holds all four in one page**, with a control that
switches between them. Comparing lengths of one coil is the whole point of
having four, and four separate pages lose the camera every time you open
another. All four are drawn to **one locked scale**, so the ¾ is visibly the
smallest and the 3-turn fills the frame — fitting each to the canvas would draw
four coils the same size, which would defeat the page. Switching keeps your
camera, so they swap in place. It is not a second viewer — `tools/coils.py`
calls the same `viewer.build_many` the per-coil pages go through, with more than
one set in it.

Each folder also carries its own viewer:
[`coil-10x10x30-0.75t.html`](coil-10x10x30-0.75t/coil-10x10x30-0.75t.html),
[`coil-10x10x30-1.5t.html`](coil-10x10x30-1.5t/coil-10x10x30-1.5t.html),
[`coil-10x10x30-2.25t.html`](coil-10x10x30-2.25t/coil-10x10x30-2.25t.html) and
[`coil-10x10x30-3t.html`](coil-10x10x30-3t/coil-10x10x30-3t.html).

**Every sheet says which coil it belongs to and how many there are** — a name
runs `bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg`, so a file
that has left its folder can still be placed, and a set with a sheet missing
says so. The first section is the same shape in all four coils; only the last
differs, since that is where each one stops:
[`…-03of03-bend-RD-buttout-…`](coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-03of03-bend-RD-buttout-cut-files.svg)
at 0.75,
[`…-06of06-bend-RD-buttout-…`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-06of06-bend-RD-buttout-cut-files.svg)
at 1.5,
[`…-09of09-bend-LD-buttout-…`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-09of09-bend-LD-buttout-cut-files.svg)
at 2.25 and
[`…-12of12-bend-LD-buttout-…`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-12of12-bend-LD-buttout-cut-files.svg)
at 3. Cut from one folder at a time.

**The 1.5t is [trumpet-switchback](https://github.com/Gernreich/trumpet-switchback)'s
walk exactly.** It arrived here as a borrowed test before anyone noticed it was a
coil — which is why its folder was called `bore/` until the other two existed.

### Why four groups make three turns

Two rhythms run through this walk at once, and they are deliberately out of step:

    term       1    2    3    4    5    6    7    8    9   10   11   12
    letter     W    U    E    D    W    U    E    D    W    U    E    D
               <-- turn 1 -->      <-- turn 2 -->      <-- turn 3 -->
    N after              N              N              N              N
              [ group 1 ]    [ group 2 ]    [ group 3 ]    [ group 4 ]

A **turn** is four terms — one full `WUED` circuit. A **group** is three — as
many as fit between `N` spacers. Group boundaries fall after terms 3, 6, 9 and
12; turn boundaries after 4, 8 and 12. The only place both land together is
**term 12**, the lowest common multiple of 3 and 4.

So the walk is **three turns and four groups at the same time**, and neither
number is wrong. The longest coil is named `3t` and has four folders' worth of
groups in it, which is the thing most likely to trip you up.

Two consequences follow, and both show up in the table above:

- **Only multiples of three quarters of a turn are reachable.** One group is
  ¾ of a turn, so the cut points are 0.75, 1.5, 2.25 and 3 — all four built.
  Two turns would be eight terms, and no `N` lands on eight.
- **Only the last group completes the pattern.** The first three stop
  mid-circuit — on `E`, `U` and `W` — which is why the cross-section does not
  close until the fourth.

The group count runs one ahead of the turn count here only because 3 and 4 are
one apart; a spacer every two terms or every five would not behave that way. If
you ever want one group to *be* one turn, put the `N` every four terms — but
that is a different coil, not a shorter one, and it would need checking against
the elbow rule.

### 0.75 turns — `coil-10x10x30-0.75t/`

| # | file | sheet | parts |
|---|---|---|---:|
| 1 | [`coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-01of03-bend-DL-buttin-cut-files.svg`](coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-01of03-bend-DL-buttin-cut-files.svg) | 325x70mm | 6 |
| 2 | [`coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-02of03-bend-LUUR-cut-files.svg`](coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-02of03-bend-LUUR-cut-files.svg) | 456x86mm | 8 |
| 3 | [`coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-03of03-bend-RD-buttout-cut-files.svg`](coil-10x10x30-0.75t/bore10-coil-10x10x30-0.75t-03of03-bend-RD-buttout-cut-files.svg) | 337x70mm | 6 |

### 1.5 turns — `coil-10x10x30-1.5t/`

| # | file | sheet | parts |
|---|---|---|---:|
| 1 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-01of06-bend-DL-buttin-cut-files.svg) | 325x70mm | 6 |
| 2 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-02of06-bend-LUUR-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-02of06-bend-LUUR-cut-files.svg) | 456x86mm | 8 |
| 3 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-03of06-bend-RD-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-03of06-bend-RD-cut-files.svg) | 337x70mm | 6 |
| 4 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-04of06-bend-DL-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-04of06-bend-DL-cut-files.svg) | 331x73mm | 6 |
| 5 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-05of06-bend-DLLU-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-05of06-bend-DLLU-cut-files.svg) | 482x73mm | 8 |
| 6 | [`coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-06of06-bend-RD-buttout-cut-files.svg`](coil-10x10x30-1.5t/bore10-coil-10x10x30-1.5t-06of06-bend-RD-buttout-cut-files.svg) | 337x70mm | 6 |

### 2.25 turns — `coil-10x10x30-2.25t/`

| # | file | sheet | parts |
|---|---|---|---:|
| 1 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-01of09-bend-DL-buttin-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-01of09-bend-DL-buttin-cut-files.svg) | 325x70mm | 6 |
| 2 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-02of09-bend-LUUR-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-02of09-bend-LUUR-cut-files.svg) | 456x86mm | 8 |
| 3 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-03of09-bend-RD-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-03of09-bend-RD-cut-files.svg) | 337x70mm | 6 |
| 4 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-04of09-bend-DL-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-04of09-bend-DL-cut-files.svg) | 331x73mm | 6 |
| 5 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-05of09-bend-DLLU-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-05of09-bend-DLLU-cut-files.svg) | 482x73mm | 8 |
| 6 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-06of09-bend-RD-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-06of09-bend-RD-cut-files.svg) | 337x70mm | 6 |
| 7 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-07of09-bend-DR-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-07of09-bend-DR-cut-files.svg) | 331x73mm | 6 |
| 8 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-08of09-bend-RDDL-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-08of09-bend-RDDL-cut-files.svg) | 456x86mm | 8 |
| 9 | [`coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-09of09-bend-LD-buttout-cut-files.svg`](coil-10x10x30-2.25t/bore10-coil-10x10x30-2.25t-09of09-bend-LD-buttout-cut-files.svg) | 337x70mm | 6 |

### 3 turns — `coil-10x10x30-3t/`

The longest, and the first walk laid out for a stretched lattice rather than
borrowed from a cubic one. **`WUED` repeated**, an `N` spacer every three terms,
and a north buffer at each end:

    N N1 W3 U2 E3 N3 D3 W2 U3 N3 E3 D2 W3 N3 U3 E2 D3 N1

Each `W U E D` returns to the same place in cross-section, so the bore walks a
square circuit while the `N` terms push it north — a square coil. The three
terms in each circuit read `3 2 3`, and the middle ones taken in order spell
`UWDE`.

| | |
| --- | ---: |
| blocks | 44 — 28 straights at 30mm, 16 cubic turns |
| centreline | **1096mm** |
| envelope | 122 × 122 × 304mm |
| sections | 12, 80 flat parts |

**It enters and leaves north, and both outer faces are flat**, so a mouthpiece
and a bell seat straight onto them.

**44 is the shortest this walk gets without an elbow.** Drop the lead-in and
section 1 becomes an elbow — the mouth block would turn with nothing to fold
into. Drop the lead-out and it is 43 blocks but leaves pointing down. Every
buffer length from 1 to 3 at either end is elbow-free, so the ends are free;
these are just the smallest that keep both openings facing north.

**There is no slack anywhere else.** All fifteen windows of the elbow rule sit
exactly on their minimum, so shortening any middle term costs an elbow at once.

| # | file | sheet |
|---|---|---|
| 1 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-01of12-bend-DL-buttin-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-01of12-bend-DL-buttin-cut-files.svg) | 325x70mm |
| 2 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-02of12-bend-LUUR-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-02of12-bend-LUUR-cut-files.svg) | 456x86mm |
| 3 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-03of12-bend-RD-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-03of12-bend-RD-cut-files.svg) | 337x70mm |
| 4 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-04of12-bend-DL-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-04of12-bend-DL-cut-files.svg) | 331x73mm |
| 5 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-05of12-bend-DLLU-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-05of12-bend-DLLU-cut-files.svg) | 482x73mm |
| 6 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-06of12-bend-RD-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-06of12-bend-RD-cut-files.svg) | 337x70mm |
| 7 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-07of12-bend-DR-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-07of12-bend-DR-cut-files.svg) | 331x73mm |
| 8 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-08of12-bend-RDDL-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-08of12-bend-RDDL-cut-files.svg) | 456x86mm |
| 9 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-09of12-bend-LD-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-09of12-bend-LD-cut-files.svg) | 337x70mm |
| 10 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-10of12-bend-DR-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-10of12-bend-DR-cut-files.svg) | 331x73mm |
| 11 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-11of12-bend-URRD-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-11of12-bend-URRD-cut-files.svg) | 482x73mm |
| 12 | [`coil-10x10x30-3t/bore10-coil-10x10x30-3t-12of12-bend-LD-buttout-cut-files.svg`](coil-10x10x30-3t/bore10-coil-10x10x30-3t-12of12-bend-LD-buttout-cut-files.svg) | 337x70mm |

Sections 3 and 6 are the same shape, and 7 and 10; they are cut separately so
each carries its own engraved number. [`coil-10x10x30-3t/coil-10x10x30-3t.html`](coil-10x10x30-3t/coil-10x10x30-3t.html) is its
viewer, and [`tools/walks/coil-3t.txt`](tools/walks/coil-3t.txt) holds the walk.

    cd tools
    python3 bore_split.py --bore=10 --straight=30 --refuse-elbows \
        --title="10x10x30 Coil, 3 Turns" \
        "$(cat walks/coil-3t.txt)" --write ../coil-10x10x30-3t

`--title` is needed because the page title is otherwise built from the folder
name, and "Coil 10x10x30 3t Bore" is a filename read aloud. A folder has to
sort and survive a URL; a title has to read.

## What is missing: a mouthpiece and a bell

Everything here is bore. **A coil on its own does not play**, and nothing in this
repository closes either end.

They already exist, though, and they fit. A coil's mouth is a 10mm square
opening in a 16mm square face, which is exactly what the 10mm pair made for
**[trumpet-switchback](https://github.com/Gernreich/trumpet-switchback)** seats
onto — that mouthpiece's station one is *a sharp 10mm square aperture in a sharp
16mm square plate*, and the bell's flange covers the same face. Nothing about
either depends on how long the bore behind it is, so one pair serves all four
coils:

| | file, in `trumpet-switchback/10mm/` |
| --- | --- |
| mouthpiece | `mouthpiece/mouthpiece-round-bore10-parts.svg` |
| bell | `bell/bell-round-152mm-bore10-17rings.svg` |

**They are named here rather than copied here.** Two copies of a cut file drift,
and that has already cost this project once. If you would rather have them in
the folder you are cutting from, say so — but then only one of the two places
can be the source.

The bell is 152mm to a ø80 mouth because that is what the switchback wanted. It
is not a constraint of the coils; `bell-round.py --bore=10 --length=… --mouth=…`
makes another.

## The joint between sections

Each section couples to the next with one tab entering one notch. The notch is
sized first and the tab follows:

| | |
| --- | ---: |
| tab | **6.0mm** wide, 3.0mm deep |
| notch | **6.05mm** wide, 3.2mm deep |
| clearance | 0.05mm across the width, 0.2mm at the bottom |
| shoulder | 2.0mm beside the tab, 1.975mm beside the notch |

**One tab size and one notch size, on every piece.** Every joint in the bore is
the same joint, so there is one number to check at the bench rather than a
different fit at each seam.

**The tab never moves; the notch closes onto it.** The tab is the finger-tooth
width and it is the load-bearing half — a notch is a hole. So the fit is tuned
by narrowing the notch, and `--pin_play` is the only number that changes.

**The size came from cutting it.** Four goes, three of them wrong:

| clearance | what happened |
| ---: | --- |
| 0.0mm | would not go together at all |
| 0.3mm | went together, perceptible rock |
| 0.1mm | very slightly loose |
| **0.05mm** | current, and confirmed: parts 1 and 2 fit |

**The clearance is the point.** SnakeBox leaves `--pin_play` at 0 on the grounds
that finger joints carry none either, and that does not follow: a finger joint
is a dozen teeth sharing an edge, where the errors average out, and a section
seam is one tab in one notch drawn to exactly the same width. At zero the two
will not go together.

**The tab is exactly the finger-tooth width**, 6mm — `2 x thickness`, and the
teeth do not shrink with the block. Sized as a fraction of the frame alone it
would come out at 4.8mm, narrower than the ordinary teeth beside it, which is
backwards for the one feature carrying a whole seam; the automatic width floors
it at the tooth for that reason, and that is the width used here.

Every seam runs SLOT to TAB, and the two outer ends carry neither.

## The toolchain is a fork, on purpose

`tools/` is a copy of
**[bore-generator](https://github.com/Gernreich/bore-generator)**, changed to
handle a lattice whose cells are not all the same length. That repository and
every repository it cuts for are **frozen**: they keep the scripts they were
gated against, and nothing here touches them.

Its generator installs into Boxes.py as **`SnakeBoxVar`**, beside the frozen
`SnakeBox` rather than over it, so both are available at once.

| file | what it does |
| --- | --- |
| `tools/bore_split.py` | turns a walk into pieces and drives the generator |
| `tools/snakeboxvar.py` | the Boxes.py generator that draws one flat piece |
| `tools/check.py` | the gate: every check that runs before anything is cut |
| `tools/assemble.py` | voxels the assembled bore, for the seal and volume checks |
| `tools/viewer.py` | writes the 3D page beside the cut files, one coil or several |
| `tools/coils.py` | writes `coils.html`, every coil in one page |
| `tools/bore_render.py` | the colours and direction names the viewer uses |
| `tools/svgpath.py` | reads and writes the path data |
| `tools/regress.py` | runs the gate over both designs at once |
| `tools/walks/coil-0.75t.txt` | the 0.75-turn coil |
| `tools/walks/coil-1.5t.txt` | the 1.5-turn coil |
| `tools/walks/coil-2.25t.txt` | the 2.25-turn coil |
| `tools/walks/coil-3t.txt` | the 3-turn coil |

```sh
cd tools
python3 bore_split.py --bore=10 --straight=30 --refuse-elbows \
    --title="10x10x30 Coil, 1½ Turns" \
    "$(cat walks/coil-1.5t.txt)" --write ../coil-10x10x30-1.5t
```

`--bore` is the airway, square, rather than the block outside. `--straight` is
how long a straight block runs; leave it out and every cell is a cube, which is
exactly what the frozen toolchain does. `--notch` would size the coupling from
the female side and move the tab to suit; **this design does not use it**, so
the tab keeps the finger-tooth width and the fit is set by `PIN_PLAY` alone.

## What had to change, and what it cost

**The integer lattice had to go.** A block is long only along the axis it runs
straight on, so the same column index wants 30mm in one part of the bore and
16mm in another — there is no single number taking a lattice index to a
millimetre. Positions are carried in mm instead, each block butting its entry
face onto the last block's exit face. For a cubic cell that reproduces
`index x blocksize` exactly.

Three things moved with it:

- **The plate outline.** A run's length now sums the widths of the columns its
  boundary crosses, rather than multiplying steps by one pitch.
- **The voxel model** behind the seal and volume checks, which takes real boxes
  and decides face contact geometrically.
- **The bed-fit test**, which summed `blocksize x blocks` and so under-reported
  this design by 13mm a side.

**Not every walk will take it.** A straight and a turn sharing a column want two
widths at once, and the generator raises, naming the column, rather than drawing
it. This walk satisfies the constraint; how many others would is not something
that has been measured.
One-cell pieces are refused outright: every straight being one file, and four
rotations of a turn sharing one, both rest on the cell being a cube.

## What the gate checks

**194 checks, 0 failed.** The number worth trusting is not that, though — it is
that the voxelised bore comes to **55520 mm3** and the independent arithmetic for
14 straights, 8 turns and their corner voids also comes to **55520 mm3**, exact
against a 0.5% tolerance, with the passage coming back as one region and no leak.

## Colour is the cut order

**Blue engraves, then green -> orange -> cyan -> black**; black frees the part,
violet `#8000ff` means skip. These nets use two stages: `#0000ff` engraves the
section number, `#000000` cuts.

## Licence

CC0 1.0 Universal. See `LICENSE`.
