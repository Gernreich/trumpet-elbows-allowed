# Octagonal trumpet

The trumpet form of the **[octagonal torus](https://gernreich.github.io/torus-octagonal/)**:
the same 25 × 25mm square channel and the same R 90 octagonal plate, opened out into a
curve instead of closed into a ring. Cut from 3mm Baltic birch plywood,
millimetre-true at `1 user unit = 1mm`, so it prints and cuts at real size.

The torus writeup describes the cut that turns one into the other. This is the result as
its own sheet.

The **bell** and the **mouthpiece** live in
**[trumpet-parts](https://github.com/Gernreich/trumpet-parts)**, shared with the
[coiled trumpet](https://github.com/Gernreich/trumpet-coiled) and built on this same
25 × 25mm channel. Neither is touched by the way a bore turns, so only the tube belongs to
an instrument.

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and the playing are shown.

<!-- readme-only -->
**[Read the writeup](https://gernreich.github.io/trumpet-octagonal/)** — the same text as
this page, set for reading, with a table of contents.

**[Download everything as a ZIP](https://github.com/Gernreich/trumpet-octagonal/archive/refs/heads/main.zip)**
— the cut file and the page it regenerates from. GitHub builds it from `main` on every
push, so it is never out of date.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
generator and tool, indexed.

Released under [CC0 1.0](LICENSE) — do what you like with it, no attribution needed.

## The sheet

Click it to download the cut file. It is a display rendering — the cut file draws a
hairline on no background, which a browser shows almost invisibly, so this is thickened
and painted onto a light ground. Geometry and sheet position are untouched. The three
lightest cut-order inks — green, orange and cyan — are darkened here. At full strength
they fall below the contrast a light background can carry, so the cut order could not be
read off them. Hue and sequence are unchanged, and the cut file keeps the exact values.

<div class="tw">
<table>
<tr>
<td align="center"><a href="octagonal-trumpet.svg"><img src="previews/octagonal-trumpet.svg" alt="The cut sheet: one finger-jointed octagonal plate with a central hole and sixteen violet lines across its wall, beside a long curved finger-jointed band that forms the curved bore" width="620"></a></td>
</tr>
<tr>
<td align="center"><sub>octagonal-trumpet.svg · 444.1 × 484.6mm sheet</sub></td>
</tr>
</table>
</div>

## What is on it

Measured out of the file itself:

| Part | Count | Size |
|---|---|---|
| Octagonal plate, finger-jointed rim | 1 | 172.298mm across flats — R 90 |
| Its central hole | 1 | 116.298mm across flats — R 62.94 |
| Curved bore band, finger-jointed on both edges | 1 | 507.368 × 306.074mm |

The sheet is 444.077 × 484.599mm and every part sits inside it, with no geometry hanging
off the page.

**The plate is the torus's plate.** Rim at apothem 86.149, hole at 58.149, hole concentric
with the rim to within 0.001mm, and the same joint phase along every face. Checked against
the torus's own R 59.693 run with
[its verifier](https://github.com/Gernreich/torus-octagonal/blob/main/verify_torus.js), the result
is **COMPLEMENTARY ✓** — the plate's tabs land in the panel's notches — so the side panels cut for the torus mate with this plate
too, and a dry-fit done for one is a dry-fit done for both.

## The traced bore

**[`octagonal-trumpet-trace.svg`](octagonal-trumpet-trace.svg)** — the bore
band's cheek, traced by hand, with the centreline taken from it: **1107.9mm
over twelve facets of 45°** at 25 × 25mm. **Display only, not a cut file.**

The curve behind this sheet is not written down as parameters anywhere, so it
was measured rather than generated. The trace confirmed as much as it supplied:
the strip came out **31.2mm wide** — 25mm of airway with a 3mm wall each side —
the turns came out at **±45°**, the same facet angle as the torus, and the total
turning came out at **0°**, which is what a bore between a mouthpiece and a bell
must do. An opening faces *out* of the tube, so the mouth faces back along the
run and the bell faces forward, and the two are opposed — you blow towards the
instrument and it speaks away from you — exactly when the turns cancel. The width scatters 30.7 to 35.5mm about that nominal, so treat the
length as ±1%.

The numbers live in
[`bore-ribbon/traces/octagonal-trumpet.json`](https://github.com/Gernreich/bore-ribbon/blob/main/traces/octagonal-trumpet.json)
with how they were taken, and the bore is drawn, turnable, at
[bore-ribbon](https://gernreich.github.io/bore-ribbon/).

**It is kept out of the cut file deliberately.** Red is a cut colour in every
repository here, so a red path on a sheet is a path the laser tries to cut —
[the torus's verifier](https://github.com/Gernreich/torus-octagonal/blob/main/verify_torus.js)
says so in as many words: *#ff0000 is a cut colour with no place in the cut
order*. Drawing it there also grew the canvas from
444 × 485mm to **798 × 492mm**, wider than the 600mm bed.

## Colour is the cut order

**Blue engraves, then green → orange → cyan → black**, with black always the cut that frees
the part and violet always skip. That sequence is the same in every LaserMadeMusic
repository, and a file uses only the stages it needs. This one needs two, and has nothing
to engrave.

| | Colour | What | Why then |
|---|---|---|---|
| 1 | **orange `#ff8000`** | the plate's central hole | cut while the plate is still held by the sheet |
| 2 | **black `#000000`** | the plate rim and the bore band | frees them, so they go last |

Holes before rims is the whole of the rule here: once the black rim is through, the plate
is loose, and anything still to be cut inside it will move.

Give both colours an explicit operation. A per-colour job **silently skips any colour you
leave unmapped** — leave orange out and you get a plate with no bore.

### The violet lines are not cuts

The plate carries **16 violet `#8000ff` lines**: one at the middle of every flat and one at
every corner, each a single straight line from the hole edge out to the rim. Together they
divide the wall into sixteen segments.

They are the optional cuts. Take them and the plate comes apart into segments; leave them
and it stays whole. Marking them explicitly is the point — "not cut" is then a decision
recorded in the drawing rather than a colour someone forgot to map. **Turn one green to
cut it**, and leave the rest violet.

## Before you cut

**Cut this in 3mm Baltic birch plywood.** That is what it is built in, and the void-free
core earns its place at the finger joints: a void landing in a tooth that has to carry the
curve is a break waiting to happen.

**Grain direction is a real choice here**, because the band curves. Run the face grain
along its length and it bends more willingly; run it across and the band resists and holds
its shape harder. Neither is wrong — they give different curves, and it is worth cutting
one of each before deciding.

**Dry-fit before committing a sheet.** The plate's tabs around the hole should drop into
the band's notches without forcing. If they are tight, that is material thickness and kerf
rather than the drawing: nominal 3mm ply is commonly 2.7–3.2mm, and the joint is cut for
exactly 3.000.

## Files

| | |
|---|---|
| `octagonal-trumpet.svg` | the cut-ready sheet |
| `previews/` | display rendering — **not** a cut file |
| `README.md` · `index.html` | the page; the markdown is the source |
