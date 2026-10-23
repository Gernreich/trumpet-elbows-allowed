"""Write a bore as a self-contained page you can turn around with the mouse.

The still renders answer "what shape is it"; this answers "where does the bore
go", which on a spiral needs you to move it. One HTML file, no libraries, no
network: the walk is baked in as data and drawn on a canvas.

    python3 viewer.py "U3 N2 W2 S4 E4 U2 N6 W6 S8 E8 U2 N10 W10 S12 E12 U3"

Writes bore.html in the working directory; --out and --title override that. It
named a page under a walk/ folder until 2026-09-08, which had been
deleted, so the one line here that could go stale was the one nothing needed.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bore_split                                           # noqa: E402
from bore_split import specs_for, DIRS, walk_text           # noqa: E402
from bore_render import DIRCOL, DIRNAME, wheel              # noqa: E402

HTML = r'''<title>__TITLE__</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans+Condensed:wght@400;500;600;700&display=swap">
<style>
:root {
  --ground: #f2ede4;        /* birch, the stock this is cut from */
  --panel: #fbf9f5;
  --panel-edge: #ded5c6;
  --ink: #1f2528;
  --ink-soft: #5d6a6d;
  --ink-faint: #8b968f;
  --accent: #b4661f;        /* the char an edge takes coming off the laser */
  --accent-ink: #fdfaf4;
  --shadow: 0 1px 2px rgba(31,37,40,.08), 0 8px 24px rgba(31,37,40,.10);
  --edge: rgba(31,37,40,.55);
}
:root:not([data-theme="light"]) {
  color-scheme: light;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --ground: #14181b;
    --panel: #1c2226;
    --panel-edge: #2e373c;
    --ink: #e8eeea;
    --ink-soft: #9dabab;
    --ink-faint: #6d7c7c;
    --accent: #d9a441;
    --accent-ink: #16191c;
    --shadow: 0 1px 2px rgba(0,0,0,.4), 0 10px 30px rgba(0,0,0,.35);
    --edge: rgba(8,10,11,.7);
    color-scheme: dark;
  }
}
:root[data-theme="dark"] {
  --ground: #14181b;
  --panel: #1c2226;
  --panel-edge: #2e373c;
  --ink: #e8eeea;
  --ink-soft: #9dabab;
  --ink-faint: #6d7c7c;
  --accent: #d9a441;
  --accent-ink: #16191c;
  --shadow: 0 1px 2px rgba(0,0,0,.4), 0 10px 30px rgba(0,0,0,.35);
  --edge: rgba(8,10,11,.7);
  color-scheme: dark;
}

* { box-sizing: border-box; }
body {
  margin: 0;
  min-height: 100vh;
  background: var(--ground);
  color: var(--ink);
  font-family: "IBM Plex Sans Condensed", ui-sans-serif, system-ui, sans-serif;
  overflow: hidden;
}
#stage { position: fixed; inset: 0; }
canvas { display: block; width: 100%; height: 100%; touch-action: none; cursor: grab; }
canvas.dragging { cursor: grabbing; }

.rail {
  position: fixed;
  background: color-mix(in srgb, var(--panel) 92%, transparent);
  border: 1px solid var(--panel-edge);
  border-radius: 3px;
  box-shadow: var(--shadow);
  padding: 14px 16px;
  backdrop-filter: blur(8px);
  max-width: min(340px, calc(100vw - 32px));
}
#title-rail { top: 16px; left: 16px; }
#controls { bottom: 16px; left: 16px; display: grid; gap: 13px; }
#legend { bottom: 16px; right: 16px; }
#blocks { top: 16px; right: 16px; display: none; max-height: calc(100vh - 220px);
          overflow-y: auto; max-width: 260px; }
#blocks.on { display: block; }
#blocks table { border-collapse: collapse; font-family: "IBM Plex Mono", ui-monospace, Menlo, monospace;
                font-size: 11.5px; font-variant-numeric: tabular-nums; }
#blocks td { padding: 1px 6px 1px 0; white-space: nowrap; color: var(--ink); }
#blocks td.n { color: var(--ink-faint); text-align: right; }
#blocks td.t { color: var(--accent); font-weight: 600; }
#blocks .sw { display: inline-block; width: 11px; height: 11px; border-radius: 2px;
              border: 1px solid var(--edge); vertical-align: -1px; }
#blocks tr.turn td { border-top: 1px solid var(--panel-edge); }

h1 {
  margin: 0 0 2px;
  font-size: 17px; font-weight: 700;
  letter-spacing: .01em; text-wrap: balance;
}
.walk {
  font-family: "IBM Plex Mono", ui-monospace, Menlo, monospace;
  font-size: 12px; line-height: 1.55; color: var(--accent);
  word-spacing: .1em; margin: 6px 0 8px;
}
.stat {
  font-family: "IBM Plex Mono", ui-monospace, Menlo, monospace;
  font-size: 11.5px; color: var(--ink-soft);
  font-variant-numeric: tabular-nums;
}
.label {
  font-size: 10px; font-weight: 600;
  letter-spacing: .13em; text-transform: uppercase;
  color: var(--ink-faint);
  margin-bottom: 6px;
}

.seg { display: flex; gap: 0; border: 1px solid var(--panel-edge); border-radius: 3px; overflow: hidden; }
.seg button {
  flex: 1; appearance: none; border: 0; background: transparent;
  color: var(--ink-soft); cursor: pointer;
  font-family: inherit; font-size: 12px; font-weight: 600;
  letter-spacing: .04em; padding: 7px 12px;
}
.seg button + button { border-left: 1px solid var(--panel-edge); }
.seg button[aria-pressed="true"] { background: var(--accent); color: var(--accent-ink); }
.seg button:not([aria-pressed="true"]):hover { color: var(--ink); }

.row { display: flex; align-items: center; gap: 10px; }
input[type="range"] {
  flex: 1; appearance: none; height: 3px; border-radius: 2px;
  background: var(--panel-edge); outline-offset: 4px;
}
input[type="range"]::-webkit-slider-thumb {
  appearance: none; width: 13px; height: 13px; border-radius: 50%;
  background: var(--accent); border: 0; cursor: pointer;
}
input[type="range"]::-moz-range-thumb {
  width: 13px; height: 13px; border-radius: 50%;
  background: var(--accent); border: 0; cursor: pointer;
}
.count {
  font-family: "IBM Plex Mono", ui-monospace, Menlo, monospace;
  font-size: 11.5px; font-variant-numeric: tabular-nums;
  color: var(--ink-soft); min-width: 62px; text-align: right;
}
.ghost {
  appearance: none; background: transparent; cursor: pointer;
  border: 1px solid var(--panel-edge); border-radius: 3px;
  color: var(--ink-soft); font-family: inherit;
  font-size: 11px; font-weight: 600; letter-spacing: .08em;
  text-transform: uppercase; padding: 6px 11px;
}
.ghost:hover { color: var(--ink); border-color: var(--ink-faint); }

#keys { display: grid; gap: 1px; }
.key {
  display: grid; grid-template-columns: 15px 1fr auto;
  align-items: center; gap: 9px;
  padding: 3px 5px; margin: 0 -5px; border-radius: 2px;
  background: transparent; border: 0; width: calc(100% + 10px);
  font-family: inherit; color: var(--ink); cursor: pointer; text-align: left;
}
.key:hover { background: color-mix(in srgb, var(--ink) 7%, transparent); }
.key[aria-pressed="true"] { background: color-mix(in srgb, var(--accent) 22%, transparent); }
.key .sw { width: 13px; height: 13px; border-radius: 2px; border: 1px solid var(--edge); }
.key .nm { font-size: 12px; }
.key .ct {
  font-family: "IBM Plex Mono", ui-monospace, Menlo, monospace;
  font-size: 11px; color: var(--ink-soft); font-variant-numeric: tabular-nums;
}
.hint { margin-top: 10px; font-size: 11px; color: var(--ink-faint); line-height: 1.5; }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
@media (max-width: 720px) {
  #legend { display: none; }
  .rail { padding: 11px 13px; }
}
/* ?embed: framed on a writeup page, which carries the title and the walk itself */
.embed .rail { display: none !important; }
</style>
<script>
/* Follow the theme the site's pages store, so a framed viewer matches the page around it. */
(function () {
  var r = document.documentElement;
  function apply() { var t = null; try { t = localStorage.getItem('ctg-theme'); } catch (e) {}
    if (t === 'light' || t === 'dark') r.setAttribute('data-theme', t); else r.removeAttribute('data-theme'); }
  apply(); addEventListener('storage', function (e) { if (e.key === 'ctg-theme') apply(); });
  if (/[?&]embed\b/.test(location.search)) r.classList.add('embed');
})();
</script>

<div id="stage"><canvas id="c"></canvas></div>

<div class="rail" id="title-rail">
  <h1>__TITLE__</h1>
  <div class="walk" id="walk">__WALK__</div>
  <div class="stat" id="stat"></div>
</div>

<div class="rail" id="controls">
  <div id="pick-wrap" style="display:none">
    <div class="label">Turns</div>
    <div class="seg" role="group" aria-label="Which coil" id="pick"></div>
  </div>
  <div>
    <div class="label">Colour by</div>
    <div class="seg" role="group" aria-label="Colour by">
      <button id="mode-dir" aria-pressed="true">Direction</button>
      <button id="mode-sec" aria-pressed="false">Section</button>
    </div>
  </div>
  <div>
    <div class="label">Follow the bore</div>
    <div class="row">
      <input type="range" id="reveal" min="1" max="__N__" value="__N__"
             aria-label="Blocks shown, from the mouth">
      <span class="count" id="reveal-count"></span>
    </div>
  </div>
  <div class="row">
    <button class="ghost" id="reset">Reset view</button>
    <button class="ghost" id="spin" aria-pressed="false">Spin</button>
    <button class="ghost" id="list" aria-pressed="false">Blocks</button>
    <button class="ghost" id="ends" aria-pressed="true">Bell &amp; mouthpiece</button>
  </div>
</div>

<div class="rail" id="blocks">
  <div class="label">Every block, from the mouth</div>
  <table id="blocks-table"></table>
</div>

<div class="rail" id="legend">
  <div class="label" id="legend-label">Direction of travel</div>
  <div id="keys"></div>
  <div class="hint">Drag to turn &middot; right-drag or shift-drag to pan &middot; scroll or pinch to zoom &middot; double-click a block to turn round it</div>
</div>

<script>
const SETS = __SETS__;
let D = SETS[0].d;
/* The mouthpiece and the bell, as ring stacks in their own frame: z up the
   stack from the face that meets the bore. Read from their cut files by
   parts/part-view.py when the page was written. */
const ENDS = __ENDS__;
let showEnds = true;

const cv = document.getElementById('c');
const ctx = cv.getContext('2d');
const NRM = [[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]];
const CORN = [
  [[1,0,0],[1,0,1],[1,1,1],[1,1,0]],
  [[0,0,0],[0,1,0],[0,1,1],[0,0,1]],
  [[0,1,0],[1,1,0],[1,1,1],[0,1,1]],
  [[0,0,0],[0,0,1],[1,0,1],[1,0,0]],
  [[0,0,1],[0,1,1],[1,1,1],[1,0,1]],
  [[0,0,0],[1,0,0],[1,1,0],[0,1,0]]
];

let yaw = 0.62, pitch = 0.72, zoom = 1, panX = 0, panY = 0;
let pivot = null, drawn = [], lastP = null;   /* the point turned round, once a part is picked; what was drawn, for picking */
let mode = 'dir', reveal = D.cells.length, isolate = null, spinning = false;

/* A cell is a box in mm, c.a to c.b. Blocks differ in length, so a face is
   hidden only where a neighbour covers the whole of it - measured, because
   there is no lattice index to compare any more. */
function covers(c, o, f) {
  const n = NRM[f], ax = n[0] ? 0 : (n[1] ? 1 : 2);
  const touch = n[ax] > 0 ? Math.abs(o.a[ax] - c.b[ax]) : Math.abs(o.b[ax] - c.a[ax]);
  if (touch > 1e-6) return false;
  for (let i = 0; i < 3; i++) {
    if (i === ax) continue;
    const lap = Math.min(c.b[i], o.b[i]) - Math.max(c.a[i], o.a[i]);
    if (lap < (c.b[i] - c.a[i]) - 1e-6) return false;
  }
  return true;
}

/* The mouthpiece seats on the IN face and the bell on the OUT face: each stack's
   first ring flat on the block's end, its axis along the face's outward normal.
   The map is a rotation, never a reflection. Cached per set. */
function endsOf(d) {
  if (d._ends) return d._ends;
  const out = [];
  for (const [key, which] of Object.entries(d.openings)) {
    const [ci, fi] = key.split(':').map(Number);
    const c = d.cells[ci], n = NRM[fi], part = which === 'IN' ? ENDS.mouthpiece : ENDS.bell;
    const o = [0, 1, 2].map(i => n[i] > 0 ? c.b[i] : n[i] < 0 ? c.a[i] : (c.a[i] + c.b[i]) / 2);
    const ex = (n[2] === 0) ? [0, 0, 1] : [1, 0, 0];
    const ey = [n[1]*ex[2] - n[2]*ex[1], n[2]*ex[0] - n[0]*ex[2], n[0]*ex[1] - n[1]*ex[0]];
    const V = part.V.map(p => [0, 1, 2].map(i => o[i] + p[0]*ex[i] + p[1]*ey[i] + p[2]*n[i]));
    const lo = [Infinity, Infinity, Infinity], hi = [-Infinity, -Infinity, -Infinity];
    for (const v of V) for (let i = 0; i < 3; i++) { lo[i] = Math.min(lo[i], v[i]); hi[i] = Math.max(hi[i], v[i]); }
    out.push({ which, V, Q: part.Q, col: part.col, box: { a: lo, b: hi } });
  }
  return (d._ends = out);
}
/* what to fit and centre on: the blocks, and the ends when they are shown */
/* a colour scaled toward black, so the stack's four kinds of face read apart */
const dim = (hex, k) => '#' + [16, 8, 0].map(b =>
  Math.round(((parseInt(hex.slice(1), 16) >> b) & 255) * k).toString(16).padStart(2, '0')).join('');
const extent = (d) => showEnds ? d.cells.concat(endsOf(d).map(e => e.box)) : d.cells;

/* centre on the whole bore, so revealing blocks does not shift the model */
let centre = null;
function recentre() { centre = (() => {
  const lo = [Infinity, Infinity, Infinity], hi = [-Infinity, -Infinity, -Infinity];
  for (const c of extent(D)) for (let i = 0; i < 3; i++) {
    lo[i] = Math.min(lo[i], c.a[i]); hi[i] = Math.max(hi[i], c.b[i]);
  }
  return { c: lo.map((v, i) => (v + hi[i]) / 2), lo: lo, hi: hi };
})(); }
recentre();

function faces() {
  const shown = D.cells.slice(0, reveal);
  const out = [];
  for (let i = 0; i < shown.length; i++) {
    const c = shown[i];
    if (isolate !== null && (mode === 'dir' ? c.d : c.s) !== isolate) continue;
    for (let f = 0; f < 6; f++) {
      if (shown.some(o => o !== c && covers(c, o, f))) continue;
      out.push({ cell: c, i: i, f: f });
    }
  }
  return out;
}

function rotC(p, c) {
  const cy = Math.cos(yaw), sy = Math.sin(yaw);
  const cp = Math.cos(pitch), sp = Math.sin(pitch);
  let x = p[0] - c[0], y = p[1] - c[1], z = p[2] - c[2];
  let x2 = x * cy + z * sy, z2 = -x * sy + z * cy;
  let y2 = y * cp - z2 * sp, z3 = y * sp + z2 * cp;
  return [x2, y2, z3];
}
function rot(p) { return rotC(p, centre.c); }

/* What the boxes project to at this angle, in mm, as [x0, x1, y0, y1]. */
function fit(cells, c) {
  let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
  for (const q of cells) {
    for (let i = 0; i < 8; i++) {
      const r = rotC([(i & 1) ? q.b[0] : q.a[0],
                      ((i >> 1) & 1) ? q.b[1] : q.a[1],
                      ((i >> 2) & 1) ? q.b[2] : q.a[2]], c);
      x0 = Math.min(x0, r[0]); x1 = Math.max(x1, r[0]);
      y0 = Math.min(y0, r[1]); y1 = Math.max(y1, r[1]);
    }
  }
  return [x0, x1, y0, y1];
}

/* The scale is locked to the biggest set, not fitted to whichever is shown.
   Fitting each one to the canvas draws four coils the same size, which is the
   one thing a page comparing lengths must not do: the longest coil is four
   times the shortest and has to look it. Each set still centres on itself, so the shown one
   stays in the middle rather than drifting off with its own origin. */
const bboxOf = (cells) => {
  const lo = [Infinity, Infinity, Infinity], hi = [-Infinity, -Infinity, -Infinity];
  for (const q of cells) for (let i = 0; i < 3; i++) {
    lo[i] = Math.min(lo[i], q.a[i]); hi[i] = Math.max(hi[i], q.b[i]);
  }
  return [lo, hi];
};
const REF = SETS.reduce((best, s) => {
  const sp = (d) => { const [lo, hi] = bboxOf(d.cells);
    return Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]); };
  return sp(s.d) > sp(best.d) ? s : best;
}, SETS[0]).d;
let refCentre = null;
function reref() { refCentre = (() => {
  const [lo, hi] = bboxOf(extent(REF));
  return lo.map((v, i) => (v + hi[i]) / 2);
})(); }
reref();

function shade(hex, n) {
  const L = [-0.32, 0.86, 0.4], m = Math.hypot(...L);
  const d = (n[0]*L[0] + n[1]*L[1] + n[2]*L[2]) / m;
  const f = 0.66 + 0.34 * (d + 1) / 2 + (n[1] > 0 ? 0.07 : 0);
  const v = parseInt(hex.slice(1), 16);
  const c = [(v >> 16) & 255, (v >> 8) & 255, v & 255]
    .map(q => Math.min(255, Math.round(q * f)));
  return 'rgb(' + c.join(',') + ')';
}

function draw() {
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const w = cv.clientWidth, h = cv.clientHeight;
  if (cv.width !== w * dpr || cv.height !== h * dpr) {
    cv.width = w * dpr; cv.height = h * dpr;
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);

  // Fit what the blocks actually project to at this angle. Not a figure that
  // has to hold at every angle - a cube-shaped walk projects near its diagonal
  // and a long thin one nowhere near it, so one fixed number either clips the
  // first or strands the second in the middle of the canvas. And not the
  // bounding box either: a walk that climbs as it runs fills a diagonal slab
  // of a mostly empty box, and the box's corners are nowhere near the shape.
  // Position from the set on screen, scale from the reference set.
  const [px0, px1, py0, py1] = fit(extent(D), centre.c);
  const [rx0, rx1, ry0, ry1] = fit(extent(REF), refCentre);
  const M = 56;
  const S = Math.min((w - M) / Math.max(rx1 - rx0, 0.001),
                     (h - M) / Math.max(ry1 - ry0, 0.001)) * zoom;
  /* the middle: the shape's own, or the part picked to turn round */
  const [mx, my] = pivot ? rot(pivot) : [(px0 + px1) / 2, (py0 + py1) / 2];
  const ox = w / 2 - mx * S + panX;
  const oy = h / 2 + my * S + panY;
  const P = (p) => { const r = rot(p); return [ox + r[0] * S, oy - r[1] * S, r[2]]; };

  const list = [];
  for (const q of faces()) {
    const n = NRM[q.f], rn = rot([n[0] + centre.c[0], n[1] + centre.c[1], n[2] + centre.c[2]]);
    if (rn[2] <= 0.001) continue;                 /* facing away */
    const pts = CORN[q.f].map(v => P([v[0] ? q.cell.b[0] : q.cell.a[0],
                                      v[1] ? q.cell.b[1] : q.cell.a[1],
                                      v[2] ? q.cell.b[2] : q.cell.a[2]]));
    const z = pts.reduce((a, p) => a + p[2], 0) / 4;
    list.push({ pts, z, q, rn });
  }
  /* The ends: every face of each stack, the far ones too, drawn in depth order
     with the blocks. The mouthpiece needs the first block and the bell the
     last, so each shows only when its block does, and neither while one
     direction or section is picked out. */
  if (showEnds && isolate === null) for (const e of endsOf(D)) {
    if (e.which === 'OUT' && reveal < D.cells.length) continue;
    for (const q of e.Q) {
      const w = q.slice(0, 4).map(i => e.V[i]);
      const u = [0, 1, 2].map(i => w[1][i] - w[0][i]), v = [0, 1, 2].map(i => w[3][i] - w[0][i]);
      let n = [u[1]*v[2] - u[2]*v[1], u[2]*v[0] - u[0]*v[2], u[0]*v[1] - u[1]*v[0]];
      const m = Math.hypot(...n) || 1; n = n.map(x => x / m);
      let rn = rot([n[0] + centre.c[0], n[1] + centre.c[1], n[2] + centre.c[2]]);
      if (rn[2] < 0) rn = rn.map(x => -x);
      const pts = w.map(P);
      list.push({ pts, z: pts.reduce((a, p) => a + p[2], 0) / 4, end: e, rn, k: q[4], c3: w });
    }
  }
  list.sort((a, b) => a.z - b.z);
  drawn = list; lastP = P;

  const secCol = D.sections.map(s => s.col);
  for (const it of list) {
    if (it.end) {
      ctx.beginPath();
      ctx.moveTo(it.pts[0][0], it.pts[0][1]);
      for (let i = 1; i < 4; i++) ctx.lineTo(it.pts[i][0], it.pts[i][1]);
      ctx.closePath();
      ctx.fillStyle = shade(dim(it.end.col, ENDS.shade[it.k]), it.rn);
      ctx.fill();
      ctx.strokeStyle = 'rgba(0,0,0,.18)';
      ctx.lineWidth = 0.5;
      ctx.stroke();
      continue;
    }
    const c = it.q.cell;
    const open = D.openings[c.i0 + ':' + it.q.f];
    let fill;
    if (open) fill = open === 'IN' ? D.inCol : D.outCol;
    else fill = shade(mode === 'dir' ? D.dircol[c.d] : secCol[c.s], it.rn);
    ctx.beginPath();
    ctx.moveTo(it.pts[0][0], it.pts[0][1]);
    for (let i = 1; i < 4; i++) ctx.lineTo(it.pts[i][0], it.pts[i][1]);
    ctx.closePath();
    ctx.fillStyle = fill;
    ctx.fill();
    ctx.lineJoin = 'round';
    ctx.strokeStyle = 'rgba(0,0,0,.34)';
    ctx.lineWidth = Math.max(0.5, S * 0.022);
    ctx.stroke();
    if (open) {
      const cx = it.pts.reduce((a, p) => a + p[0], 0) / 4;
      const cy = it.pts.reduce((a, p) => a + p[1], 0) / 4;
      ctx.fillStyle = '#fff';
      ctx.font = '600 ' + Math.max(9, S * 0.34) + 'px "IBM Plex Sans Condensed", sans-serif';
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(open, cx, cy);
    }
  }
  document.getElementById('stat').textContent =
    D.blocks + ' blocks · ' + D.sections.length
    + (D.sections.length === 1 ? ' section · ' : ' sections · ')
    + D.mm + 'mm of centreline';
}

/* ---- pointer ---- */
/* turn, pan, zoom: drag turns; right-drag, shift-drag or two fingers pan;
   scroll or pinch zooms about the pointer; double-click a part to turn round it */
const pts = new Map(); let drag = null, pinch = null;
const at = e => { const b = cv.getBoundingClientRect(); return [e.clientX - b.left, e.clientY - b.top]; };
const two = () => { const [a, b] = [...pts.values()];
  return { h: Math.hypot(a[0] - b[0], a[1] - b[1]), x: (a[0] + b[0]) / 2, y: (a[1] + b[1]) / 2 }; };
function zoomBy(f, x, y) {   /* about canvas point (x, y), which stays where it is */
  const z0 = zoom; zoom = Math.max(0.25, Math.min(6, zoom * f));
  const u = x - cv.clientWidth / 2, v = y - cv.clientHeight / 2, k = zoom / z0;
  panX = u - (u - panX) * k; panY = v - (v - panY) * k;
}
const mid3 = it => it.end ? [0, 1, 2].map(i => it.c3.reduce((a, p) => a + p[i], 0) / 4)
                  : [0, 1, 2].map(i => (it.q.cell.a[i] + it.q.cell.b[i]) / 2);
function pick(x, y) {   /* the part drawn on top at (x, y) */
  for (let i = drawn.length - 1; i >= 0; i--) {
    const p = drawn[i].pts; let inside = false;
    for (let j = 0, k = p.length - 1; j < p.length; k = j++)
      if ((p[j][1] > y) !== (p[k][1] > y)
          && x < (p[k][0] - p[j][0]) * (y - p[j][1]) / (p[k][1] - p[j][1]) + p[j][0]) inside = !inside;
    if (inside) return mid3(drawn[i]);
  }
  return null;
}
cv.addEventListener('pointerdown', e => {
  cv.setPointerCapture(e.pointerId); pts.set(e.pointerId, at(e)); cv.classList.add('dragging');
  if (pts.size === 1) drag = { pan: e.button === 1 || e.button === 2 || e.shiftKey };
  else { drag = null; pinch = two(); }
});
cv.addEventListener('pointermove', e => {
  if (!pts.has(e.pointerId)) return;
  const [x0, y0] = pts.get(e.pointerId), [x, y] = at(e); pts.set(e.pointerId, [x, y]);
  if (pinch) {   /* two fingers: move together to pan, pinch to zoom */
    const m = two(); panX += m.x - pinch.x; panY += m.y - pinch.y;
    if (pinch.h && m.h) zoomBy(m.h / pinch.h, m.x, m.y); pinch = m;
  } else if (drag && drag.pan) { panX += x - x0; panY += y - y0; }
  else if (drag) {
    yaw += (x - x0) * 0.008;
    pitch = Math.max(-1.5, Math.min(1.5, pitch + (y - y0) * 0.008));
  }
  draw();
});
const lift = e => { pts.delete(e.pointerId); if (pts.size < 2) pinch = null;
  if (!pts.size) { drag = null; cv.classList.remove('dragging'); } };
cv.addEventListener('pointerup', lift);
cv.addEventListener('pointercancel', lift);
cv.addEventListener('contextmenu', e => e.preventDefault());
cv.addEventListener('wheel', e => {
  e.preventDefault(); const [x, y] = at(e);
  zoomBy(Math.exp(-e.deltaY * 0.0016), x, y); draw();
}, { passive: false });
cv.addEventListener('dblclick', e => {
  const [x, y] = at(e), p = pick(x, y);
  if (!p) {  return; }
  /* p holds the middle from now on: start it where it is, then glide it in */
  const s = lastP(p); pivot = p;
  const x1 = panX = s[0] - cv.clientWidth / 2, y1 = panY = s[1] - cv.clientHeight / 2, t0 = performance.now();
  const glide = t => { const k = Math.min(1, (t - t0) / 300), f = 1 - k * (2 - k);
    panX = x1 * f; panY = y1 * f; draw(); if (k < 1) requestAnimationFrame(glide); };
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) { panX = panY = 0; draw(); }
  else requestAnimationFrame(glide);
});

/* ---- controls ---- */
const $ = (id) => document.getElementById(id);
function setMode(m) {
  mode = m; isolate = null;
  $('mode-dir').setAttribute('aria-pressed', m === 'dir');
  $('mode-sec').setAttribute('aria-pressed', m === 'sec');
  $('legend-label').textContent = m === 'dir' ? 'Direction of travel' : 'Sections, in order';
  buildKeys(); draw();
}
$('mode-dir').onclick = () => setMode('dir');
$('mode-sec').onclick = () => setMode('sec');

$('reveal').addEventListener('input', e => {
  reveal = +e.target.value;
  $('reveal-count').textContent = reveal + ' / ' + D.cells.length;
  draw();
});
$('reset').onclick = () => {
  yaw = 0.62; pitch = 0.72; zoom = 1; panX = panY = 0; pivot = null;
  reveal = D.cells.length; $('reveal').value = reveal;
  $('reveal-count').textContent = reveal + ' / ' + D.cells.length;
  isolate = null; buildKeys(); draw();
};
const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
let spinTimer = null;
$('spin').onclick = () => {
  spinning = !spinning;
  $('spin').setAttribute('aria-pressed', spinning);
  if (spinning) {
    const step = () => { yaw += 0.004; draw(); spinTimer = requestAnimationFrame(step); };
    spinTimer = requestAnimationFrame(step);
  } else cancelAnimationFrame(spinTimer);
};
if (reduced) $('spin').style.display = 'none';
$('ends').onclick = () => {
  showEnds = !showEnds;
  $('ends').setAttribute('aria-pressed', showEnds);
  recentre(); reref(); draw();
};

const DIRNAME = {N: 'north', S: 'south', E: 'east', W: 'west',
                 U: 'up', D: 'down'};
function buildList() {
  const t = $('blocks-table');
  t.textContent = '';
  D.cells.forEach((c, i) => {
    const turn = i > 0 && D.cells[i - 1].d !== c.d;
    const tr = document.createElement('tr');
    if (turn) tr.className = 'turn';
    tr.innerHTML = '<td class="n"></td><td></td><td></td><td class="t"></td>';
    tr.children[0].textContent = i + 1;
    tr.children[1].innerHTML = '<span class="sw" style="background:'
      + D.dircol[c.d] + '"></span> ' + DIRNAME[c.d];
    /* the block's low corner in mm; there is no lattice index to show once
       straights and turns are different lengths */
    tr.children[2].textContent = '(' + c.a.map(v => Math.round(v)).join(',') + ')';
    tr.children[3].textContent = turn ? 'turn' : '';
    t.appendChild(tr);
  });
}
$('list').onclick = () => {
  const on = !$('blocks').classList.contains('on');
  $('blocks').classList.toggle('on', on);
  $('list').setAttribute('aria-pressed', on);
};

function buildKeys() {
  const box = $('keys');
  box.textContent = '';
  const rows = mode === 'dir'
    ? D.dirs.map(d => ({ id: d.k, col: D.dircol[d.k], nm: d.name, ct: d.n + ' blk' }))
    : D.sections.map((s, i) => ({ id: i, col: s.col, nm: (i + 1) + '. ' + s.kind,
                                  ct: s.n + ' blk' }));
  for (const r of rows) {
    const b = document.createElement('button');
    b.className = 'key';
    b.setAttribute('aria-pressed', isolate === r.id);
    b.innerHTML = '<span class="sw" style="background:' + r.col + '"></span>'
      + '<span class="nm"></span><span class="ct"></span>';
    b.querySelector('.nm').textContent = r.nm;
    b.querySelector('.ct').textContent = r.ct;
    b.onclick = () => { isolate = (isolate === r.id) ? null : r.id; buildKeys(); draw(); };
    box.appendChild(b);
  }
}

/* Several coils in one page: same walk truncated at different points, so the
   only thing that changes is which set of cells is drawn. Reset the view with
   it -- they differ in size, and keeping a zoom from a longer one strands a
   shorter one off-screen. */
function pickCoil(i) {
  D = SETS[i].d;
  recentre();
  reveal = D.cells.length;
  $('reveal').max = reveal; $('reveal').value = reveal;
  $('reveal-count').textContent = reveal + ' / ' + reveal;
  document.getElementById('walk').textContent = SETS[i].walk;
  // Keep the camera. It used to reset here, because each coil fitted itself and
  // a zoom held from the 3-turn threw the 3/4 off the canvas. The scale is
  // locked to the longest now, so every coil draws inside that frame -- and
  // holding the angle is the whole point of a switch: you are comparing
  // lengths, which you cannot do if the view jumps each time you swap. Reset
  // is still a button.
  isolate = null;
  for (const b of $('pick').children)
    b.setAttribute('aria-pressed', +b.dataset.i === i);
  buildKeys(); buildList(); draw();
}
if (SETS.length > 1) {
  $('pick-wrap').style.display = '';
  SETS.forEach((s, i) => {
    const b = document.createElement('button');
    b.textContent = s.label; b.dataset.i = i;
    b.setAttribute('aria-pressed', i === 0);
    b.onclick = () => pickCoil(i);
    $('pick').appendChild(b);
  });
}

window.addEventListener('resize', draw);
$('reveal-count').textContent = D.cells.length + ' / ' + D.cells.length;
buildKeys();
buildList();
draw();
</script>
'''


def data_for(text):
    """The one dict the page draws from. Split out of build() so several
    walks can share a page: a gallery is the same viewer with more than one
    set, not a second viewer."""
    rec, groups, plans, plan, unfilled = specs_for(text)
    sec_of = {}
    for i, g in enumerate(groups):
        for j in g:
            sec_of[j] = i
    pal = wheel(len(groups))
    # Real boxes in mm, not lattice cells: a straight block may be longer than
    # a turn, and drawing every cell as a unit cube then shows a bore nobody
    # is cutting. For a cubic cell a..b is exactly the old unit cube scaled.
    boxes = bore_split.block_boxes(rec)
    cells = [{'a': list(boxes[i][0]), 'b': list(boxes[i][1]),
              'd': r['out'], 's': sec_of[i], 'i0': i}
             for i, r in enumerate(rec)]

    # the two ends of the bore, as (cell index : face index)
    order = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
    def fidx(d):
        return order.index(tuple(d))
    openings = {
        f"0:{fidx(tuple(-x for x in DIRS[rec[0]['in']]))}": 'IN',
        f"{len(rec)-1}:{fidx(DIRS[rec[-1]['out']])}": 'OUT',
    }

    data = {
        'cells': cells,
        'blocks': len(rec),
        'mm': round(sum(bore_split.extent(r, bore_split.AXIS[r['out']])
                        for r in rec)),
        'dircol': DIRCOL,
        'dirs': [{'k': d, 'name': DIRNAME[d],
                  'n': sum(1 for r in rec if r['out'] == d)}
                 for d in 'NSEWUD' if any(r['out'] == d for r in rec)],
        'sections': [{'col': pal[i % len(pal)], 'kind': p['kind'], 'n': len(g)}
                     for i, (g, p) in enumerate(zip(groups, plan))],
        'openings': openings,
        'inCol': '#1c1c20',
        'outCol': '#e0457b',
    }
    return data


# The mouthpiece and the bell, drawn on every page at IN and OUT (2026-10-14):
# their ring stacks, read from their own cut files by ../parts/part-view.py,
# 24 facets round. The page places them; this only says what they are.
PARTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'parts')
END_PARTS = (('bell', 'bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg', '#d8b445'),
             ('mouthpiece', 'mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg', '#8a5a2b'))


def ends_json():
    """part-view.py executes bell-view.py up to the line that reads argv, and
    that prefix checks argv is long enough, so it is given two names."""
    import importlib.util
    argv, sys.argv = sys.argv, ['part-view.py', '-', '-']
    try:
        spec = importlib.util.spec_from_file_location(
            'part_view', os.path.join(PARTS, 'part-view.py'))
        m = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(m)
    finally:
        sys.argv = argv
    out = {}
    for name, svg, col in END_PARTS:
        d = m.data_for(os.path.join(PARTS, svg), 24)
        out[name] = {'V': [[round(c, 2) for c in v] for v in d['V']],
                     'Q': [q['v'] + [q['f']] for q in d['Q']], 'col': col}
    # part-view's four faces -- ring face, outside, the bore, underside --
    # each scaled toward black so they read apart
    out['shade'] = [1.0, 0.92, 0.55, 0.7]
    return json.dumps(out, separators=(',', ':'))


def build(text, title):
    return build_many([(None, text)], title)


def build_many(items, title):
    """items is [(label, walk)]. One item hides the selector, so a single
    page is what it always was."""
    return build_sets([(lab, txt, data_for(txt)) for lab, txt in items], title)


def build_sets(items, title):
    """items is [(label, walk, data)], with the data ALREADY computed.

    sizes.py needs this and build_many cannot serve it: sizes.py changes the
    block pitch between sets, and data_for() reads that pitch when it is called.
    Computing lazily in here would give every set the LAST pitch -- two sets both
    at 16mm with one of them labelled 682mm regardless, which is the bug
    sizes.py carries a comment about.

    The name existed until 4cbf437 collapsed the forked toolchain into tools/,
    where build_many replaced it and this was not kept. sizes.py went on calling
    it and has raised AttributeError on every run since 2026-09-05 -- truncating
    its own output file first, because open(out, 'w') runs before the call. No
    gate ran sizes.py, so nothing said so for five days.
    """
    sets = [{'label': lab or '', 'walk': txt.strip(), 'd': d}
            for lab, txt, d in items]
    first = sets[0]
    return (HTML.replace('__SETS__', json.dumps(sets, separators=(',', ':')))
                .replace('__ENDS__', ends_json())
                .replace('__TITLE__', title)
                .replace('__WALK__', first['walk'])
                .replace('__N__', str(first['d']['blocks'])))


def main(argv):
    out, title = 'bore.html', 'Bore Viewer'
    words = []
    i = 0
    while i < len(argv):
        if argv[i] == '--out':
            out = argv[i + 1]; i += 2
        elif argv[i] == '--title':
            title = argv[i + 1]; i += 2
        else:
            words.append(argv[i]); i += 1
    text = walk_text(' '.join(words))
    html = build(text, title)
    open(out, 'w').write(html)
    print(f'  wrote {out}  ({len(html)/1024:.0f} kB, no external assets '
          f'but the fonts)')


if __name__ == '__main__':
    try:
        main(sys.argv[1:])
    except ValueError as e:
        print(f'error: {e}')
        sys.exit(1)
