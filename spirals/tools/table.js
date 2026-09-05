#!/usr/bin/env node
// The metrics tables. Usage: node tools/table.js [--md] [--shape|--rot|--all]
// Piece counts come from parts.json; rebuild it with `node tools/parts.js`.
const fs = require('fs'), path = require('path');
const { metrics, period } = require('./spiral_metrics.js');
const root = path.join(__dirname, '..');
const md   = process.argv.includes('--md');
const want = process.argv.includes('--shape') ? 'shape'
           : process.argv.includes('--rot')   ? 'rot' : 'all';

const UNSCORED = new Set(
  (fs.existsSync(path.join(root, 'unscored.txt'))
    ? fs.readFileSync(path.join(root, 'unscored.txt'), 'utf8') : '')
  .split('\n').map(l => l.replace(/#.*/, '').trim()).filter(Boolean));
let parts = {};
try { parts = JSON.parse(fs.readFileSync(path.join(root, 'parts.json'), 'utf8')); } catch (e) {}

const rows = fs.readdirSync(path.join(root, 'walks'))
  .filter(f => f.endsWith('.txt'))
  .map(f => {
    const name = f.replace(/\.txt$/, '');
    const walk = fs.readFileSync(path.join(root, 'walks', f), 'utf8').trim();
    const p = parts[name] || {};
    // Judged columns are measured over the interior; blocks and mm describe the
    // whole bore, ends included, because that is what the bore is.
    return { name, p, full: metrics(walk), per: period(walk),
             m: metrics(walk, p.interiorBlocks) };
  })
  .sort((a, b) => a.m.vol - b.m.vol);
const scored   = rows.filter(r => !UNSCORED.has(r.name));
const unscored = rows.filter(r =>  UNSCORED.has(r.name));

const link = r => md ? `[\`${r.name}\`](pages/${r.name}.html)` : r.name;

const TABLES = {
  shape: {
    head: ['spiral', 'blocks', 'mm', 'envelope', 'box', 'cross-section', 'along axis',
           'pieces', 'distinct', 'period', 'rhythm', 'mean plate mm2', 'touching'],
    align:['---', '---:', '---:', '---', '---:', '---', '---:', '---:', '---:', '---:', '---:', '---:', '---:'],
    row: r => [link(r), r.full.blocks, r.full.mm, r.m.size.join(' x '), r.m.vol,
               r.m.cross.join(' x ') + ' blk / ' + r.m.crossMM.join(' x ') + ' mm',
               r.m.axisLen + ' / ' + r.m.axisLenMM + ' mm',
               r.p.innerPieces ?? '-', r.p.interiorDistinct ?? '-',
               r.per.terms ? r.per.terms + ' tm / ' + r.per.blocks + ' blk' : '-',
               r.p.rhythm ? r.p.rhythm + ' x' + r.p.repeats.toFixed(1) : '-',
               r.p.meanPlate ? Math.round(r.p.meanPlate) : '-', r.m.touching]
  },
  rot: {
    head: ['spiral', 'axis', 'rotation', 'turns', 'blocks/360', 'rise/360 blk / mm',
           'deg/block', '90deg turns', 'turns/m', 'longest straight'],
    align:['---', ':-:', '---:', '---:', '---:', '---:', '---:', '---:', '---:', '---:'],
    row: r => [link(r), r.m.axisDir, r.m.degrees, r.m.turns.toFixed(2),
               r.m.blocksPer360.toFixed(1),
               r.m.risePer360.toFixed(2) + ' / ' + Math.round(r.m.riseMMPer360),
               r.m.degPerBlock.toFixed(1), r.m.turns90,
               r.m.turnsPerMetre.toFixed(1),
               r.m.longestStraight + ' blk / ' + r.m.longestStraightMM + ' mm']
  }
};

function emit(key, set){
  const t = TABLES[key];
  const body = (set || scored).map(r => t.row(r).map(String));
  if (md) {
    console.log('| ' + t.head.join(' | ') + ' |');
    console.log('| ' + t.align.join(' | ') + ' |');
    for (const b of body) console.log('| ' + b.join(' | ') + ' |');
  } else {
    const w = t.head.map((h, i) => Math.max(h.length, ...body.map(b => b[i].length)));
    console.log(t.head.map((h, i) => h.padEnd(w[i])).join('  '));
    for (const b of body) console.log(b.map((v, i) => v.padEnd(w[i])).join('  '));
  }
}
if (want === 'all'){ emit('shape'); console.log(); emit('rot'); } else emit(want);
if (unscored.length && want !== 'rot') {
  console.log(md ? '\nNot scored (see `unscored.txt`):\n' : '\nnot scored:');
  emit('shape', unscored);
}
if (unscored.length && want === 'rot') {
  console.log(md ? '\nNot scored (see `unscored.txt`):\n' : '\nnot scored:');
  emit('rot', unscored);
}
