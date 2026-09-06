#!/usr/bin/env node
// Put every coil in the same orientation, so two coils differ only where they
// really differ.
//
//   forward   north for all of them, and the walk opens on a north term
//   rotation  counter-clockwise seen looking along the bore from the mouth
//   length    whole periods only, the count that lands nearest a common target
//   ends      one block in, one block out, and no partial period between them
//
// A bore opens at both ends, so the first and last piece cannot be got rid of:
// they are bounded by the mouth and the exit rather than by a neighbour, and the
// notation says as much -- the first term is only the way you came in. What can
// be done is to make them the same on every walk, which is what this does, and
// the judged metrics ignore them anyway.
//
// Usage: node tools/standardise.js [--write]
const fs = require('fs'), cp = require('child_process'), path = require('path');
const root = path.join(__dirname, '..');
// The bore toolchain, six levels up since the 2026-09-05 restructure. Built
// from fragments, which is why three path sweeps walked straight past it:
// nothing in this file ever contains the string '../tools' to match on.
const GEN  = path.join(root, '..', '..', '..', '..', '..', '..', '..', 'tools');
const PY   = process.env.BORE_PY || (process.env.HOME + '/boxes/venv/bin/python');
const WRITE = process.argv.includes('--write');

const MAIN = require.main === module;
const say = (...a) => { if (MAIN) console.log(...a); };

const V = {N:[0,0,-1], S:[0,0,1], E:[1,0,0], W:[-1,0,0], U:[0,1,0], D:[0,-1,0]};
const NAME = {}; for (const [k,v] of Object.entries(V)) NAME[v.join(',')] = k;
const FWD = V.N;                                   // forward is north

// the 48 signed axis permutations: rotations and mirrors both, because some
// coils are the wrong way round and only a reflection fixes handedness
const TRANSFORMS = [];
for (const p of [[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]])
  for (const s of [[1,1,1],[1,1,-1],[1,-1,1],[1,-1,-1],[-1,1,1],[-1,1,-1],[-1,-1,1],[-1,-1,-1]])
    TRANSFORMS.push(v => [s[0]*v[p[0]], s[1]*v[p[1]], s[2]*v[p[2]]]);

const parse = w => w.trim().split(/\s+/).map(t => ({ d:t[0], n:t.length>1?+t.slice(1):1 }));
const netOf = t => t.reduce((a,m) => { const v = V[m.d];
  return [a[0]+v[0]*m.n, a[1]+v[1]*m.n, a[2]+v[2]*m.n]; }, [0,0,0]);
const blocksOf = t => t.reduce((a,m) => a + m.n, 0);

// Counter-clockwise seen looking ALONG forward. Forward is -z, so the viewer
// looks down -z with +x right and +y up, and CCW is +x turning toward +y.
function windingXY(t){
  const proj = [];
  for (const m of t){ const v = V[m.d]; if (v[0] || v[1]) proj.push([v[0], v[1]]); }
  let q = 0;
  for (let i = 0; i < proj.length; i++){
    const u = proj[i], w = proj[(i+1) % proj.length];
    q += u[0]*w[1] - u[1]*w[0];
  }
  return q;
}
// Fixing forward and the sense of rotation still leaves freedom: the four
// rotations about the forward axis all satisfy both, and so does every rotation
// of the cycle that opens on a north term. Picking the first that fits is not
// canonical -- it left coils that were the same walk looking different, four of
// them in one case. So enumerate every representation and take the
// lexicographically smallest, which makes equal coils identical strings.
function orient(period){
  const cands = [];
  for (const T of TRANSFORMS){
    const t = period.map(m => ({ d: NAME[T(V[m.d]).join(',')], n: m.n }));
    if (t.some(m => !m.d)) continue;
    const net = netOf(t);
    if (net[0] !== 0 || net[1] !== 0 || net[2] >= 0) continue;   // must travel north
    if (windingXY(t) <= 0) continue;                            // must be CCW
    for (let i = 0; i < t.length; i++){
      if (t[i].d !== 'N') continue;                             // must open on north
      cands.push([...t.slice(i), ...t.slice(0, i)].map(m => m.d + m.n).join(' '));
    }
  }
  if (!cands.length) return null;
  cands.sort();
  return cands[0].split(' ').map(x => ({ d: x[0], n: +x.slice(1) }));
}

function periodOf(t){
  const body = t.slice(2,-2).map(x => x.d + x.n);
  for (let p = 1; p <= body.length - p; p++){
    let ok = true;
    for (let i = 0; i + p < body.length; i++) if (body[i] !== body[i+p]){ ok = false; break; }
    if (ok) return body.slice(0,p).map(s => ({ d:s[0], n:+s.slice(1) }));
  }
  return null;
}
function split(walk){
  try {
    const out = cp.execSync(`${JSON.stringify(PY)} bore_split.py --no-write ${JSON.stringify(walk)}`,
      { cwd: GEN, encoding:'utf8', maxBuffer:1<<24, stdio:['pipe','pipe','pipe'] });
    const kinds = {};
    for (const l of out.split('\n')){ const m = l.match(/^\s+\d+\s+\d+-\d+\s+(\w+)\s/); if (m) kinds[m[1]] = (kinds[m[1]]||0)+1; }
    return { ok:true, elbows:kinds.elbow||0, pieces:+(out.match(/(\d+) pieces to assemble/)||[])[1],
             blocks:+(out.match(/(\d+) blocks/)||[])[1], oversize:/does not fit|too big/i.test(out) };
  } catch (e) { return { ok:false, err:(e.stderr||e.stdout||'').toString().split('\n')[0].slice(0,70) }; }
}

// Build a standard walk from a period: orient it, repeat it the number of times
// that lands nearest the target, one block in and one block out.
function buildWalk(period, target){
  const o = orient(period);
  if (!o) return null;
  const pb = blocksOf(o), k = Math.max(1, Math.round(target / pb));
  const body = Array.from({length:k}, () => o.map(m => m.d + m.n).join(' ')).join(' ');
  return { walk: 'N ' + body + ' ' + o[o.length-1].d, oriented: o, k, periodBlocks: pb };
}

// gather periods first, so the set decides the target
const src = [];
for (const f of fs.readdirSync(path.join(root,'walks')).filter(f => f.endsWith('.txt')).sort()){
  const name = f.replace(/\.txt$/,'');
  const walk = fs.readFileSync(path.join(root,'walks',f),'utf8').trim();
  const per = periodOf(parse(walk));
  src.push({ name, walk, per });
}
const known = src.filter(s => s.per);
// Whole periods of different lengths cannot all reach the same total, so the
// target is chosen rather than assumed: the one that makes the tube lengths as
// alike as possible while the bore stays a plausible size. Capping instead --
// "as many as fit under a limit" -- is worse, because it truncates the
// long-period coils hardest and every one of them lands below every short-period
// one. Nearest-to-target has no such lean.
const PERIODS = [...new Set(known.map(s => blocksOf(s.per)))];
const lengthsFor = L => PERIODS.map(p => Math.max(1, Math.round(L / p)) * p);
let TARGET = null, bestSpread = Infinity;
for (let L = 120; L <= 220; L++){                       // 3.7m to 6.8m of tube
  const len = lengthsFor(L);
  const spread = Math.max(...len) / Math.min(...len) - 1;
  if (spread < bestSpread - 1e-12){ bestSpread = spread; TARGET = L; }
}
const span = lengthsFor(TARGET);
say('target ' + TARGET + ' blocks, chosen to minimise the spread: lengths ' +
  Math.min(...span) + '-' + Math.max(...span) + ', ' + (bestSpread*100).toFixed(1) + '%\n');

const out = [];
for (const s of src){
  let per = s.per;
  if (!per){                        // too short to show its own period; borrow a twin's
    const twin = known.find(k => k.name !== s.name && s.walk.includes(k.per.map(m=>m.d+m.n).join(' ')));
    per = twin ? twin.per : null;
    if (!per){ say(s.name.padEnd(20) + 'no period; skipped'); continue; }
  }
  const o = orient(per);
  if (!o){ say(s.name.padEnd(20) + 'could not orient'); continue; }
  const built = buildWalk(per, TARGET);
  const pb = built.periodBlocks, k = built.k, walk = built.walk;
  const r = split(walk);
  say(s.name.padEnd(20) + 'period ' + String(pb).padStart(2) + ' x' + k +
    ' = ' + String(pb*k).padStart(3) + '  -> ' + String(r.blocks ?? '?').padStart(3) + ' blocks, ' +
    (r.ok ? r.pieces + ' pieces, elbows ' + r.elbows : 'REFUSED ' + r.err));
  if (r.ok && r.elbows === 0 && !r.oversize) out.push({ name:s.name, walk, period:o.map(m=>m.d+m.n).join(' '), k, r });
}
// standardising can reveal that two walks were the same coil all along
const byWalk = new Map();
for (const o of out){ if (!byWalk.has(o.walk)) byWalk.set(o.walk, []); byWalk.get(o.walk).push(o.name); }
say('\n' + out.length + ' standardised, ' + byWalk.size + ' distinct');
for (const [w, names] of byWalk) if (names.length > 1) say('  same coil: ' + names.join(', '));
if (WRITE) fs.writeFileSync(path.join(root,'standardised.json'),
  JSON.stringify([...byWalk].map(([walk,names]) => ({ walk, names })), null, 1));
module.exports = { out, byWalk, TARGET, orient, periodOf, blocksOf, buildWalk, split };
