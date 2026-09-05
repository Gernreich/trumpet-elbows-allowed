#!/usr/bin/env node
// Can this walk be made shorter without introducing an elbow?
//
// A term's floor is set by the window of three around it -- outer A, middle m,
// outer C. Consecutive terms are always on different axes, so:
//
//   A and C same axis, same direction      a step     m >= 1
//   A and C same axis, opposite direction  a hairpin  m >= 2
//   A and C different axes                 a coil     m >= 3
//
// A walk where every term already sits on its floor cannot be shortened at all
// by this rule: every block left in it is load-bearing.
//
// Removing the slack is not guaranteed safe: the rule is local, and a shorter
// walk can run into itself or start touching. Treat what this reports as
// candidates and put the result back through bore_split.py.
//
// Usage: node tools/minimal.js [--terms] [walk-or-file ...]   (default: every walk)
const fs = require('fs'), path = require('path');
const root = path.join(__dirname, '..');
const AX = { N:2, S:2, E:0, W:0, U:1, D:1 };

function terms(walk){
  return walk.trim().split(/\s+/).map(t => ({
    d: t[0], n: t.length > 1 ? parseInt(t.slice(1), 10) : 1, raw: t }));
}
function floors(walk){
  const t = terms(walk);
  const out = [];
  // the first and last term are the mouth and the exit; they have no window of
  // their own and are not the design's to choose, so they are left alone
  for (let i = 1; i < t.length - 1; i++){
    const a = t[i-1], m = t[i], c = t[i+1];
    const kind = AX[a.d] !== AX[c.d] ? 'coil'
               : a.d === c.d          ? 'step'
               :                        'hairpin';
    const floor = kind === 'coil' ? 3 : kind === 'step' ? 1 : 2;
    out.push({ i, term: m.raw, n: m.n, kind, floor, slack: m.n - floor });
  }
  return out;
}

function report(name, walk){
  const f = floors(walk);
  const slack = f.filter(x => x.slack > 0);
  const saving = slack.reduce((a, x) => a + x.slack, 0);
  const byKind = {};
  for (const x of f) byKind[x.kind] = (byKind[x.kind] || 0) + 1;
  console.log(name.padEnd(18) +
    String(f.length).padStart(4) + ' terms   ' +
    Object.entries(byKind).sort().map(([k,v]) => `${v} ${k}`).join(', ').padEnd(32) +
    (saving === 0 ? 'MINIMAL' : `${saving} blocks removable in ${slack.length} terms`));
  return { name, saving, slack };
}

const args = process.argv.slice(2).filter(a => a !== '--terms');
const VERBOSE = process.argv.includes('--terms');
const targets = args.length
  ? args.map(a => fs.existsSync(a)
      ? { name: path.basename(a, '.txt'), walk: fs.readFileSync(a, 'utf8').trim() }
      : { name: '(argument)', walk: a })
  : fs.readdirSync(path.join(root, 'walks')).filter(f => f.endsWith('.txt')).sort()
      .map(f => ({ name: f.replace(/\.txt$/, ''),
                   walk: fs.readFileSync(path.join(root, 'walks', f), 'utf8').trim() }));

const QUIET = require.main !== module;
const _log = console.log;
if (QUIET) console.log = () => {};
const results = targets.map(t => ({ ...report(t.name, t.walk), floors: floors(t.walk) }));
console.log('\n' + results.filter(r => r.saving === 0).length + ' of ' + results.length +
            ' are minimal: nothing can be shortened without an elbow.');
if (QUIET) console.log = _log;
const loose = results.filter(r => r.saving > 0);
if (loose.length && VERBOSE){
  console.log('\nterms with slack:');
  for (const r of loose)
    for (const x of r.slack)
      console.log('  ' + r.name.padEnd(18) + 'term ' + String(x.i + 1).padStart(3) +
        '  ' + x.term.padEnd(4) + x.kind.padEnd(9) + 'floor ' + x.floor +
        '  -> could drop ' + x.slack);
}

module.exports = { floors, results };
