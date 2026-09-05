#!/usr/bin/env node
// Iterated ranking: rank, cut the bottom half, re-rank the survivors, repeat.
//
// The question is whether the survivors keep their order when the losers leave.
// They do not, if the normalization is computed over the set -- min-max reads its
// lo and hi off whoever is present, so removing alternatives rescales every
// metric and reorders coils that did not change. That is an independence-of-
// irrelevant-alternatives failure, and it is why a cut should be made on a
// property decided in advance, not on composite score.
//
// The other end of it: make the normalization set-independent and give it a
// geometric mean, and the whole procedure is provably a no-op, because scaling
// metric i by c_i multiplies every score by the same (prod c_i)^(1/n).
//
// Usage: node tools/iterate.js [--md]
const { rows, METRICS, TOUCH_WEIGHT, EPS } = require('./score.js');

// mode 'minmax' is what SCORING.md ranks with: set-relative, floored, and with
// touching itself normalized against the set maximum.
// mode 'pure' is ratio-to-best with no floor and touching as a fixed factor --
// nothing in it depends on which other coils are present.
function score(set, mode, meanKind) {
  const lo = {}, hi = {};
  for (const M of METRICS) {
    const xs = set.map(r => r.v[M.k]);
    lo[M.k] = Math.min(...xs); hi[M.k] = Math.max(...xs);
  }
  return set.map(r => {
    const vals = [];
    for (const M of METRICS) {
      if (mode === 'minmax') {
        const span = hi[M.k] - lo[M.k];
        const t = span === 0 ? 1 : (r.v[M.k] - lo[M.k]) / span;
        const good = M.dir === 'lo' ? 1 - t : t;
        vals.push({ x: EPS + (1 - EPS) * good, w: 1 });
      } else {
        vals.push({ x: M.dir === 'lo' ? lo[M.k] / r.v[M.k] : r.v[M.k] / hi[M.k], w: 1 });
      }
    }
    // The touching term is the same in both modes and in score.js: 1/(1+t) is
    // absolute, so it is not the thing that moves when the set shrinks. What
    // moves is the min-max metrics above, which is exactly the point being made.
    vals.push({ x: 1 / (1 + r.v.touching), w: mode === 'minmax' ? TOUCH_WEIGHT : 1 });
    const W = vals.reduce((a, v) => a + v.w, 0);
    const s = meanKind === 'harm'
      ? W / vals.reduce((a, v) => a + v.w / v.x, 0)
      : Math.exp(vals.reduce((a, v) => a + v.w * Math.log(v.x), 0) / W);
    return { name: r.name, s };
  }).sort((a, b) => b.s - a.s);
}

function eliminate(mode, meanKind) {
  let set = rows.slice(), round = 0, first = null;
  const log = [];
  while (set.length > 1) {
    const ranked = score(set, mode, meanKind);
    if (round === 0) first = ranked.map(r => r.name);
    const keep = ranked.slice(0, Math.ceil(ranked.length / 2)).map(r => r.name);
    const before = ranked.map(r => r.name).filter(n => keep.includes(n));
    const after = score(set.filter(r => keep.includes(r.name)), mode, meanKind).map(r => r.name);
    log.push({ round, from: set.length, to: keep.length,
               moved: before.filter((n, i) => after[i] !== n).length, before, after });
    set = set.filter(r => keep.includes(r.name));
    round++;
  }
  return { log, winner: set[0].name, rankedFirst: first[0] };
}

const RUNS = [
  { mode: 'minmax', mean: 'harm', label: 'harmonic, min-max (what SCORING.md uses)' },
  { mode: 'minmax', mean: 'geo',  label: 'geometric, min-max' },
  { mode: 'pure',   mean: 'geo',  label: 'geometric, pure ratio-to-best' },
];
const results = RUNS.map(r => ({ ...r, ...eliminate(r.mode, r.mean) }));

if (require.main === module) {
  for (const r of results) {
    console.log('=== ' + r.label + ' ===');
    for (const l of r.log)
      console.log('  round ' + l.round + ': ' + String(l.from).padStart(2) + ' -> ' +
        String(l.to).padStart(2) + '   survivors reordered: ' + l.moved + '/' + l.to);
    console.log('  winner after elimination: ' + r.winner +
                '   |   ranking all ' + rows.length + ' once: ' + r.rankedFirst +
                '   ' + (r.winner === r.rankedFirst ? '(same)' : '(DIFFERENT)'));
    console.log();
  }
}
module.exports = { results };
