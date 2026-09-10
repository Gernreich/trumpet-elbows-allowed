#!/usr/bin/env node
// Piece counts, distinct piece shapes and plate sizes, read off bore_split.py and
// cached in parts.json so table.js does not have to shell out on every run.
//
// The plate is the bounding box the piece is cut from, which is what goes on the
// sheet and what the operator handles. Its average is the laser-cutting number:
// fewer, larger parts is less weeding, less sorting and fewer fingers to align.
// Note it is a bounding box, not the cut outline -- an L-shaped piece leaves the
// corner behind, so this overstates material for those and is a size measure
// rather than an area-used one.
// Usage: node tools/parts.js            (rebuild the cache)
const fs = require('fs'), cp = require('child_process'), path = require('path');
const root = path.join(__dirname, '..');
// The bore toolchain, seven levels up since the 2026-09-05 restructure. Built
// from fragments, which is why three path sweeps walked straight past it:
// nothing in this file ever contains the string '../tools' to match on.
const GEN  = path.join(root, '..', '..', '..', '..', '..', '..', '..', 'tools');
const CANDIDATES = ['~/Software/boxes', '~/boxes'];   // as bore_split.py searches
function py() {
  const found = process.env.BORE_PY ||
    CANDIDATES.map(c => c.replace('~', process.env.HOME) + '/venv/bin/python')
              .find(p => fs.existsSync(p));
  // BORE_PY is checked too: taken on trust, a wrong one fails later and deeper,
  // as a spawn error against a walk rather than a bad setting.
  if (!found || !fs.existsSync(found)) {
    console.error(found ? 'BORE_PY is set to ' + found + ', which does not exist.'
                        : 'no Boxes.py venv found. Looked in ' + CANDIDATES.join(', ') +
                          '.\n  Set BORE_PY=/path/to/venv/bin/python.');
    process.exit(1);
  }
  return found;
}

// The walk this whole search started from is no longer in walks/: it won a category and
// was promoted out to a sibling directory of its own. Every "against the walk this
// started from" figure on the page still compares to it, so it is measured here too,
// from wherever it now lives. gen_readme.js used to fall back to the first row when it
// could not find it, and so named a different coil in the opening sentence and down a
// whole column of the winners table without saying anything.
// Found rather than written down: derived.txt says which coil came from staircase_coil,
// and the promotion convention is the same identifier with coil_ dropped and the
// underscores hyphenated.
const origin = (() => {
  const rows = fs.readFileSync(path.join(root, 'derived.txt'), 'utf8').split('\n')
    .map(l => l.trim()).filter(l => l && !l.startsWith('#')).map(l => l.split(/\s+/));
  const row = rows.find(([, ...src]) => src.includes('staircase_coil'));
  if (!row) return null;
  const [name] = row;
  if (fs.existsSync(path.join(root, 'walks', name + '.txt'))) return null;   // still in the set
  const dir = name.replace(/^coil_/, '').replace(/_/g, '-');
  const family = path.join(root, '..');
  for (const group of fs.readdirSync(family)) {
    const f = path.join(family, group, dir, dir + '.txt');
    if (fs.existsSync(f)) return { name, walk: fs.readFileSync(f, 'utf8').trim(),
                                   promotedTo: path.posix.join('..', group, dir, dir + '.html') };
  }
  console.error(`derived.txt says the search started from ${name}, which is neither in
walks/ nor anywhere under ${path.resolve(family)}. Nothing can be measured against it,
and guessing a substitute is what this replaced.`);
  process.exit(1);
})();

const sources = fs.readdirSync(path.join(root, 'walks')).filter(f => f.endsWith('.txt')).sort()
  .map(f => ({ name: f.replace(/\.txt$/, ''),
               walk: fs.readFileSync(path.join(root, 'walks', f), 'utf8').trim() }));
if (origin) sources.push(origin);

const out = {};
for (const { name, walk, promotedTo } of sources) {
  const res = cp.execSync(`${JSON.stringify(py())} bore_split.py --no-write ${JSON.stringify(walk)}`,
                          { cwd: GEN, encoding: 'utf8', maxBuffer: 1 << 24 });
  const pieces = +(res.match(/(\d+) pieces to assemble/) || [])[1];
  const plates = [];
  for (const m of res.matchAll(/^\s+\d+\s+(\d+)-(\d+)\s+\w+\s+\w+\s+\w+\s+\d+x\d+ bl (\d+)x(\d+)/gm))
    plates.push({ a: +m[1], b: +m[2], blocks: +m[2] - +m[1] + 1, mm2: +m[3] * +m[4] });
  // The bore's mouth and exit are forced -- every design has them and no design
  // chooses them -- so nothing here is judged on them. This is the block range
  // belonging to the pieces in between, and it is what the metrics are measured
  // over; blocks and mm still describe the whole bore.
  const interiorBlocks = plates.length > 2
    ? [plates[1].a, plates[plates.length - 2].b]
    : [1, plates.length ? plates[plates.length - 1].b : 0];
  const kinds = {};
  for (const l of res.split('\n')) {
    const m = l.match(/^\s+\d+\s+\d+-\d+\s+(\w+)\s/);
    if (m) kinds[m[1]] = (kinds[m[1]] || 0) + 1;
  }
    // A cut-list entry is <bore><n>-<design>-NNofMM-<kind>-<shape>-cut-files.svg
    // since 2026-09-03, when the design's name went into the file's name. It was
    // NN_<kind>_<shape>.svg before that, and this parsed only the old form -- so
    // it matched nothing and every shape count came out zero, silently. Hyphens
    // fold back to underscores so the strings match what parts.json has held.
    const seq = [...res.matchAll(/^\s+\d+\s+\S*?-\d+of\d+-(\S+?)-cut-files\.svg/gm)]
                  .map(m => m[1].replace(/-/g, '_'));
  const shapes = new Set(seq);
  // The rhythm: how many pieces you lay before the sequence starts over. The
  // first and last piece are the bore's mouth and exit and are one-offs, so the
  // repeat that matters is the interior one. Descriptive, not scored -- across a
  // set of periodic coils it barely varies, and a near-constant metric in a mean
  // only dilutes the ones that discriminate.
  const interior = seq.slice(1, -1);
  const period = (a) => {
    for (let p = 1; p <= a.length; p++) {
      let ok = true;
      for (let i = 0; i + p < a.length; i++) if (a[i] !== a[i + p]) { ok = false; break; }
      if (ok) return p;
    }
    return a.length;
  };
  const rhythm = interior.length ? period(interior) : 0;
  const inner = plates.slice(1, -1);
  const areas = (inner.length ? inner : plates).map(p => p.mm2);
  out[name] = { pieces, kinds, elbows: kinds.elbow || 0,
                ...(promotedTo ? { promotedTo, walk } : {}),
                interiorBlocks,
                innerPieces: Math.max(0, pieces - 2),
                distinct: shapes.size, shapes: [...shapes].sort(),
                rhythm, repeats: rhythm ? interior.length / rhythm : 0,
                interiorDistinct: new Set(interior).size,
                meanPlate: areas.reduce((a, x) => a + x, 0) / areas.length,
                maxPlate: Math.max(...areas), minPlate: Math.min(...areas),
                totalPlate: areas.reduce((a, x) => a + x, 0) };
  console.log(name.padEnd(18), pieces + ' pieces', String(shapes.size) + ' distinct',
              'rhythm ' + out[name].rhythm + ' x' + out[name].repeats.toFixed(1),
              'mean plate ' + Math.round(out[name].meanPlate) + ' mm2',
              'elbows ' + (kinds.elbow || 0));
}
fs.writeFileSync(path.join(root, 'parts.json'), JSON.stringify(out, null, 1));
console.log('wrote parts.json');
