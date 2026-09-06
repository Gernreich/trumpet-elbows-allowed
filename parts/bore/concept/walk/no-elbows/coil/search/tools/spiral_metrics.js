// Rotation metrics for a lattice bore walk.
//
// A coil turns about one axis while advancing along it. The lateral projection
// -- the walk with the advancing axis dropped -- is what actually rotates, and
// on a cubic lattice it can only ever turn in 90 degree steps, so the winding
// is counted in quarter turns and multiplied up. Steps along the coil axis
// project to nothing and are skipped; they are advance, not rotation.
// The block pitch in millimetres. 16 is a 10mm bore in 3mm stock, the default
// since 2026-09-05. It was 31 -- a 25mm bore -- and stayed 31 through the
// conversion, because that changed bore_split.py's default and nothing looked
// for a second copy of the number over here. Every mm figure in the scoring
// and the tables came from this line.
const MM = 16;
const V = {N:[0,0,-1], S:[0,0,1], E:[1,0,0], W:[-1,0,0], U:[0,1,0], D:[0,-1,0]};
const AXNAME = ['x (east/west)', 'y (up/down)', 'z (north/south)'];
const ALONG = [{p:'E', n:'W'}, {p:'U', n:'D'}, {p:'S', n:'N'}];  // +axis, -axis (north is -z)

// One direction per block: the way the bore travels through it. The walk's
// LAST term is the exit direction of the final block and adds no block of its
// own -- "N N5 U" is six blocks, the sixth of them turning up on the way out --
// so it is dropped here and the block count matches bore_split.py.
function blockDirs(walk){
  const terms = walk.trim().split(/\s+/);
  const out=[];
  for(const t of terms.slice(0, -1)){
    const m=t.match(/^([NSEWUD])(\d*)$/);
    if(!m) throw new Error('bad term: '+t);
    const n=m[2]?parseInt(m[2],10):1;
    for(let i=0;i<n;i++) out.push(m[1]);
  }
  return out;
}

// range, when given, is a 1-indexed inclusive block span: measure only that part
// of the bore. Used to leave the mouth and exit pieces out of every judgement.
function metrics(walk, range){
  let s = blockDirs(walk);                   // n directions, one per block
  if (range) s = s.slice(range[0] - 1, range[1]);
  let p=[0,0,0]; const cells=[p.slice()];
  for(let i=0;i+1<s.length;i++){             // n-1 moves between n blocks
    const v=V[s[i]]; p=[p[0]+v[0],p[1]+v[1],p[2]+v[2]]; cells.push(p.slice());
  }

  const lo=[1e9,1e9,1e9], hi=[-1e9,-1e9,-1e9];
  for(const c of cells) for(let i=0;i<3;i++){ if(c[i]<lo[i])lo[i]=c[i]; if(c[i]>hi[i])hi[i]=c[i]; }
  const size=[hi[0]-lo[0]+1, hi[1]-lo[1]+1, hi[2]-lo[2]+1];

  // the coil axis is the one the walk actually travels down
  const net = p;
  let ax = 0;
  for(let i=1;i<3;i++) if(Math.abs(net[i]) > Math.abs(net[ax])) ax = i;
  const lat = [0,1,2].filter(i=>i!==ax);

  // winding of the lateral projection, in quarter turns
  const proj=[];
  for(const d of s){ const v=V[d]; const a=v[lat[0]], b=v[lat[1]]; if(a||b) proj.push([a,b]); }
  let q=0;
  for(let i=0;i+1<proj.length;i++){
    const u=proj[i], w=proj[i+1];
    q += u[0]*w[1] - u[1]*w[0];        // +1 one way, -1 the other, 0 straight on
  }

  // 90 degree turns along the bore, and the longest run without one. Bend
  // density is what separates these coils acoustically: the tube length is the
  // same, so what differs is how often the air is asked to turn a corner.
  let turns90 = 0, run = 1, longest = 1;
  for (let i = 1; i < s.length; i++){
    if (s[i] === s[i-1]) { run++; if (run > longest) longest = run; }
    else { turns90++; run = 1; }
  }

  const deg   = q * 90;
  const turns = Math.abs(deg) / 360;
  const rise  = Math.abs(net[ax]);
  const blocks= cells.length;

  // Blocks that touch without being joined along the bore. bore_split warns
  // about these, and they are the version of "density" that has a consequence:
  // where two runs of the bore share a wall, that wall is all that separates
  // them. Raw fill density says the same thing as the bounding box at a fixed
  // tube length; this does not.
  const key = c => c.join(',');
  const at = new Map(cells.map((c, i) => [key(c), i]));
  const NB = [[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]];
  let touching = 0;
  for (let i = 0; i < cells.length; i++){
    for (const n of NB){
      const j = at.get(key([cells[i][0]+n[0], cells[i][1]+n[1], cells[i][2]+n[2]]));
      if (j === undefined) continue;
      if (j > i && j !== i + 1) touching++;      // count each pair once
    }
  }

  // The cross-section is the envelope with the coil axis taken out: how fat the
  // coil is, which is what decides whether it fits inside anything. Two coils
  // with the same box can be 3x3x47 or 5x8x17.
  const cross = lat.map(i => size[i]).sort((a, b) => a - b);

  return {
    blocks, mm: blocks*MM, size, vol: size[0]*size[1]*size[2],
    cross, crossMM: cross.map(c => c*MM),
    axisLen: size[ax], axisLenMM: size[ax]*MM,
    turns90, turnsPerMetre: turns90 / (blocks*MM/1000),
    touching, touchingPerBlock: touching / blocks,
    longestStraight: longest, longestStraightMM: longest*MM,
    axis: AXNAME[ax],
    axisDir: net[ax] > 0 ? ALONG[ax].p : ALONG[ax].n,
    // Handedness is a property of the helix, not of where you stand. The
    // lateral pair (lat0,lat1,ax) is a right-handed frame for ax = x and z but
    // a left-handed one for ax = y, and advancing down the negative axis flips
    // it again; fold both in so the answer means the same thing every time.
    hand: (() => {
      if (deg === 0) return 'none';
      const frame = (ax === 1) ? -1 : 1;
      const along = (net[ax] >= 0) ? 1 : -1;
      return (q * frame * along) > 0 ? 'right-handed' : 'left-handed';
    })(),
    degrees: Math.abs(deg),
    turns,
    blocksPer360: turns ? blocks / turns : null,
    risePer360:   turns ? rise  / turns : null,
    riseMMPer360: turns ? rise * MM / turns : null,
    degPerBlock:  Math.abs(deg) / blocks,
    rise, riseMM: rise*MM
  };
}
// The repeating unit: how many terms, and how many blocks. Worth having beside
// the rhythm, because rhythm = period terms x the share of turns that leave the
// plane, and with only the rhythm you cannot tell those two apart -- a rhythm of
// 12 could be a long period turning gently or a short one leaving the plane at
// every chance.
function period(walk){
  const t = walk.trim().split(/\s+/).slice(2, -2);
  for (let p = 1; p <= t.length - p; p++){
    let ok = true;
    for (let i = 0; i + p < t.length; i++) if (t[i] !== t[i+p]){ ok = false; break; }
    if (ok) return { terms: p,
                     blocks: t.slice(0,p).reduce((a,x) => a + (+x.slice(1) || 1), 0) };
  }
  return { terms: null, blocks: null };
}

module.exports = { metrics, blockDirs, period, MM };

if (require.main === module) {
  const m = metrics(process.argv[2]);
  console.log(JSON.stringify(m, null, 2));
}
