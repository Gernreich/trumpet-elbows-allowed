// Tightest ELBOW-FREE spiral. Corpus rule, stated in bore-designs/README:
// consecutive terms are always on different axes (a reversal is refused), so a
// window of three terms names three axes exactly when the outer two differ ---
// and then the middle term must be >= 3. If the outer two share an axis it is a
// fold, and a fold can be as tight as you like.
const AX = {N:2,S:2,E:0,W:0,U:1,D:1};
const V = {N:[0,0,-1], S:[0,0,1], E:[1,0,0], W:[-1,0,0], U:[0,1,0], D:[0,-1,0]};
const DIRS = Object.keys(V);
const TARGET = 196;

function cells(terms, reps, T, ax){
  let p=[0,0,0]; const out=[p];
  for(let r=0;r<reps;r++) for(const [d,L] of terms){
    const v=V[d];
    for(let i=0;i<L;i++){ p=[p[0]+v[0],p[1]+v[1],p[2]+v[2]]; out.push(p); }
  }
  return out;
}
function ok3(terms){                       // the >=3 rule, applied cyclically
  const k=terms.length;
  for(let i=0;i<k;i++){
    const a=terms[(i+k-1)%k], b=terms[i], c=terms[(i+1)%k];
    const need = AX[a[0]]!==AX[c[0]] ? 3      // coil: leaves the plane
               : a[0]===c[0]           ? 1      // step: free
               :                         2;     // hairpin: doubles back
    if(b[1]<need) return false;
  }
  return true;
}
function quarter(terms, ax){
  const lat=[0,1,2].filter(i=>i!==ax); const proj=[];
  for(const [d] of terms){ const v=V[d]; const a=v[lat[0]],b=v[lat[1]]; if(a||b) proj.push([a,b]); }
  let q=0;
  for(let i=0;i<proj.length;i++){ const u=proj[i], w=proj[(i+1)%proj.length];
    q+=u[0]*w[1]-u[1]*w[0]; }
  return q;
}
const res=[];
const MAXK=8, MAXL=4;
function dfs(terms, pos, occ){
  const k=terms.length;
  if(k>=4 && ok3(terms)){
    for(let ax=0;ax<3;ax++){
      const lat=[0,1,2].filter(i=>i!==ax);
      if(pos[lat[0]]===0 && pos[lat[1]]===0 && pos[ax]!==0){
        const q=quarter(terms,ax);
        if(q!==0 && q%4===0){
          const per=terms.reduce((a,t)=>a+t[1],0);
          const reps=Math.max(2,Math.ceil(TARGET/per));
          const c=cells(terms,reps);
          const seen=new Set(); let bad=false;
          for(const q2 of c){ const key=q2.join(','); if(seen.has(key)){bad=true;break;} seen.add(key); }
          if(!bad){
            const use=c.slice(0,TARGET);
            const lo=[1e9,1e9,1e9],hi=[-1e9,-1e9,-1e9];
            for(const q2 of use) for(let i=0;i<3;i++){ if(q2[i]<lo[i])lo[i]=q2[i]; if(q2[i]>hi[i])hi[i]=q2[i]; }
            const sz=[hi[0]-lo[0]+1,hi[1]-lo[1]+1,hi[2]-lo[2]+1];
            const vol=sz[0]*sz[1]*sz[2];
            res.push({terms:terms.map(t=>t[0]+t[1]).join(' '), per, rise:Math.abs(pos[ax]),
                      turns:q/4, size:sz.slice().sort((a,b)=>a-b).join('x'), vol,
                      dens:TARGET/vol});
          }
        }
      }
    }
  }
  if(k===MAXK) return;
  for(const d of (k===0?['N']:DIRS)){
    if(k>0 && AX[d]===AX[terms[k-1][0]]) continue;   // no repeat, no reversal
    for(let L=1;L<=MAXL;L++){
      const v=V[d]; let p=pos.slice(); const added=[]; let clash=false;
      for(let i=0;i<L;i++){
        p=[p[0]+v[0],p[1]+v[1],p[2]+v[2]];
        const key=p.join(',');
        if(occ.has(key)){clash=true;break;}
        occ.add(key); added.push(key);
      }
      if(!clash){ terms.push([d,L]); dfs(terms,p,occ); terms.pop(); }
      for(const key of added) occ.delete(key);
    }
  }
}
dfs([],[0,0,0],new Set(['0,0,0']));
const seen=new Map();
for(const r of res){ const k=r.size+'|'+r.vol+'|'+r.per; if(!seen.has(k)) seen.set(k,r); }
const list=[...seen.values()].sort((a,b)=>a.vol-b.vol);
require('fs').writeFileSync('cands.json', JSON.stringify(list));
console.log('elbow-free helices found:', res.length, ' distinct:', list.length);
console.log('\nenvelope   vol   fill   per  rise  turns   terms');
for(const r of list.slice(0,12))
  console.log(r.size.padStart(9), String(r.vol).padStart(5), (r.dens*100).toFixed(0).padStart(5)+'%',
    String(r.per).padStart(5), String(r.rise).padStart(5), String(r.turns).padStart(6), '  ', r.terms);
