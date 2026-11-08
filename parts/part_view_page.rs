//! The turn page part-view writes: its title and data are filled in at
//! __TITLE__ and __DATA__. Browser code, kept as it was written.
pub const HTML: &str = r##"<title>__TITLE__</title>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root{color-scheme:dark;--bg:#14161a;--ink:#e8eaed;--dim:#9aa3ad;--line:#2a2f36;
      --panel:#1b1e24;--accent:#e0457b;--edge:rgba(0,0,0,.42);
      --btn:#232830;--swring:rgba(255,255,255,.35)}
:root[data-theme=light]{color-scheme:light;--bg:#f7f5f1;--ink:#1b1e24;--dim:#5f6873;
      --line:#dcd7cd;--panel:#fffdf9;--btn:#efebe4;--swring:rgba(0,0,0,.28)}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);
     font:14px/1.5 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif}
header{padding:14px 18px 10px;border-bottom:1px solid var(--line)}
h1{margin:0;font-size:17px;font-weight:650;letter-spacing:-.01em}
.sub{color:var(--dim);font-size:13px;margin-top:3px}
.sub b{color:var(--ink);font-weight:600}
main{display:grid;grid-template-columns:1fr 220px;height:calc(100vh - 62px);min-height:420px}
#wrap{position:relative;overflow:hidden}
canvas{display:block;width:100%;height:100%;cursor:grab;touch-action:none}
canvas.drag{cursor:grabbing}
aside{border-left:1px solid var(--line);padding:14px;overflow:auto;background:var(--panel)}
h2{margin:0 0 8px;font-size:11px;letter-spacing:.09em;text-transform:uppercase;
   color:var(--dim);font-weight:600}
.grp{margin-bottom:18px}
button{font:inherit;color:var(--ink);background:var(--btn);border:1px solid var(--line);
       border-radius:6px;padding:5px 9px;cursor:pointer;margin:0 4px 4px 0}
button[aria-pressed=true]{background:var(--accent);border-color:var(--accent);color:#fff}
button:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
input[type=range]{width:100%;accent-color:var(--accent)}
.key{display:flex;align-items:center;gap:7px;margin:3px 0;font-size:12.5px}
.sw{width:12px;height:12px;border-radius:3px;flex:0 0 auto;
    box-shadow:inset 0 0 0 1px var(--swring)}
.hint{color:var(--dim);font-size:12px;margin-top:10px}
dl{margin:0;display:grid;grid-template-columns:auto 1fr;gap:2px 10px;font-size:12.5px}
dt{color:var(--dim)}dd{margin:0;text-align:right;font-variant-numeric:tabular-nums}
@media (max-width:720px){main{grid-template-columns:1fr;height:auto}
  #wrap{height:56vh}aside{border-left:0;border-top:1px solid var(--line)}}
/* ?embed: framed on a writeup page, which carries the title and the numbers itself */
.embed header,.embed aside{display:none}
.embed main{grid-template-columns:1fr;height:100vh;min-height:0}
.embed #wrap{height:100vh}
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
<header><h1>__TITLE__</h1><div class="sub" id="sub"></div></header>
<main>
  <div id="wrap"><canvas id="c"></canvas></div>
  <aside>
    <div class="grp"><h2>Colour</h2>
      <button id="m-face" aria-pressed="true">by face</button>
      <button id="m-ring" aria-pressed="false">by ring</button></div>
    <div class="grp"><h2>Stack</h2><input id="rev" type="range" min="1" max="1" value="1">
      <div class="hint" id="revn"></div></div>
    <div class="grp"><h2>Key</h2><div id="key"></div></div>
    <div class="grp"><h2>Numbers</h2><dl id="nums"></dl></div>
    <div class="hint">Drag to turn. Right-drag, shift-drag or two fingers to pan. Scroll or pinch to zoom. Double-click a part to turn round it, or empty space to reset.</div>
  </aside>
</main>
<script>
const D = __DATA__;
const cv = document.getElementById('c'), cx = cv.getContext('2d');
let yaw = -0.5, pitch = -0.62, zoom = 1, mode = 'face', reveal = D.rings;
let panX = 0, panY = 0, pivot = null, drawn = [], lastP = null;   /* pan; the point turned round, once a part is picked; what was drawn, for picking */
const C = (() => { let lo=[1e9,1e9,1e9], hi=[-1e9,-1e9,-1e9];
  for (const v of D.V) for (let i=0;i<3;i++){lo[i]=Math.min(lo[i],v[i]);hi[i]=Math.max(hi[i],v[i]);}
  return [(lo[0]+hi[0])/2,(lo[1]+hi[1])/2,(lo[2]+hi[2])/2]; })();
function rot(p){
  const x=p[0]-C[0], y=p[1]-C[1], z=p[2]-C[2];
  const cy=Math.cos(yaw), sy=Math.sin(yaw);
  const x1=x*cy - y*sy, y1=x*sy + y*cy;
  const cp=Math.cos(pitch), sp=Math.sin(pitch);
  return [x1, z*cp - y1*sp, z*sp + y1*cp];
}
function span(){
  let lo=[1e9,1e9], hi=[-1e9,-1e9];
  for (const v of D.V){ const r=rot(v);
    for (let i=0;i<2;i++){lo[i]=Math.min(lo[i],r[i]);hi[i]=Math.max(hi[i],r[i]);} }
  return [hi[0]-lo[0], hi[1]-lo[1], (lo[0]+hi[0])/2, (lo[1]+hi[1])/2];
}
function shade(hex,k){const n=parseInt(hex.slice(1),16);
  const f=v=>Math.max(0,Math.min(255,Math.round(v*k)));
  return `rgb(${f(n>>16&255)},${f(n>>8&255)},${f(n&255)})`;}
function draw(){
  const dpr=Math.min(devicePixelRatio||1,2), w=cv.clientWidth, h=cv.clientHeight;
  cv.width=w*dpr; cv.height=h*dpr; cx.setTransform(dpr,0,0,dpr,0,0); cx.clearRect(0,0,w,h);
  const [sw,sh,mx,my]=span();
  const s=Math.min(w/(sw||1), h/(sh||1))*0.86*zoom;
  const [ax,ay]=pivot?rot(pivot):[mx,my];   /* the shape's middle, or the part picked to turn round */
  const P=p=>{const r=rot(p); return [w/2+(r[0]-ax)*s+panX, h/2-(r[1]-ay)*s+panY, r[2]];};
  const items=[];
  for (const q of D.Q){
    if (q.s >= reveal) continue;
    const pts=q.v.map(i=>P(D.V[i]));
    let area=0;
    for(let i=0;i<4;i++){const a=pts[i],b=pts[(i+1)%4]; area+=a[0]*b[1]-b[0]*a[1];}
    items.push({q,pts,z:(pts[0][2]+pts[1][2]+pts[2][2]+pts[3][2])/4,front:area<0});
  }
  items.sort((a,b)=>a.z-b.z);
  drawn=items; lastP=P;
  for (const it of items){
    const q=it.q;
    const col = mode==='face' ? D.facecol[q.f] : D.pal[q.s % D.pal.length];
    const k = (it.front?1:0.62) * (q.f===0?1.05:q.f===1?0.88:q.f===2?0.60:0.74);
    cx.fillStyle=shade(col,k);
    cx.beginPath(); cx.moveTo(it.pts[0][0],it.pts[0][1]);
    for(let i=1;i<4;i++) cx.lineTo(it.pts[i][0],it.pts[i][1]);
    cx.closePath(); cx.fill();
  }
}
/* turn, pan, zoom: drag turns; right-drag, shift-drag or two fingers pan;
   scroll or pinch zooms about the pointer; double-click a part to turn round it */
const pts = new Map(); let drag = null, pinch = null;
const at = e => { const b = cv.getBoundingClientRect(); return [e.clientX - b.left, e.clientY - b.top]; };
const two = () => { const [a, b] = [...pts.values()];
  return { h: Math.hypot(a[0] - b[0], a[1] - b[1]), x: (a[0] + b[0]) / 2, y: (a[1] + b[1]) / 2 }; };
function zoomBy(f, x, y) {   /* about canvas point (x, y), which stays where it is */
  const z0 = zoom; zoom = Math.max(0.35, Math.min(6, zoom * f));
  const u = x - cv.clientWidth / 2, v = y - cv.clientHeight / 2, k = zoom / z0;
  panX = u - (u - panX) * k; panY = v - (v - panY) * k;
}
const mid3 = it => [0, 1, 2].map(i => it.q.v.reduce((a, j) => a + D.V[j][i], 0) / it.q.v.length);
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
  cv.setPointerCapture(e.pointerId); pts.set(e.pointerId, at(e)); cv.classList.add('drag');
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
    pitch = Math.max(-1.55, Math.min(1.55, pitch + (y - y0) * 0.008));
  }
  draw();
});
const lift = e => { pts.delete(e.pointerId); if (pts.size < 2) pinch = null;
  if (!pts.size) { drag = null; cv.classList.remove('drag'); } };
cv.addEventListener('pointerup', lift);
cv.addEventListener('pointercancel', lift);
cv.addEventListener('contextmenu', e => e.preventDefault());
cv.addEventListener('wheel', e => {
  e.preventDefault(); const [x, y] = at(e);
  zoomBy((e.deltaY < 0 ? 1.1 : 1 / 1.1), x, y); draw();
}, { passive: false });
cv.addEventListener('dblclick', e => {
  const [x, y] = at(e), p = pick(x, y);
  if (!p) { yaw = -0.5; pitch = -0.62; zoom = 1; panX = panY = 0; pivot = null; draw(); return; }
  /* p holds the middle from now on: start it where it is, then glide it in */
  const s = lastP(p); pivot = p;
  const x1 = panX = s[0] - cv.clientWidth / 2, y1 = panY = s[1] - cv.clientHeight / 2, t0 = performance.now();
  const glide = t => { const k = Math.min(1, (t - t0) / 300), f = 1 - k * (2 - k);
    panX = x1 * f; panY = y1 * f; draw(); if (k < 1) requestAnimationFrame(glide); };
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) { panX = panY = 0; draw(); }
  else requestAnimationFrame(glide);
});
const $=id=>document.getElementById(id);
function setMode(m){mode=m; $('m-face').setAttribute('aria-pressed',m==='face');
  $('m-ring').setAttribute('aria-pressed',m==='ring'); key(); draw();}
$('m-face').onclick=()=>setMode('face'); $('m-ring').onclick=()=>setMode('ring');
const rev=$('rev'); rev.max=D.rings; rev.value=D.rings;
rev.oninput=()=>{reveal=+rev.value; revn(); draw();};
function revn(){$('revn').textContent = reveal+' of '+D.rings+' rings'
  + (reveal===D.rings?'  (all)':'');}
function key(){
  const k=$('key'); k.innerHTML='';
  const rows = mode==='face' ? D.faces.map((n,i)=>[D.keycol[i],n])
    : [[D.pal[0],'ring 1 (throat)'],
       [D.pal[Math.floor(D.rings/2)%D.pal.length],'ring '+(Math.floor(D.rings/2)+1)],
       [D.pal[(D.rings-1)%D.pal.length],'ring '+D.rings+' (rim)']];
  for (const [c,n] of rows){const d=document.createElement('div'); d.className='key';
    d.innerHTML=`<span class="sw" style="background:${c}"></span><span>${n}</span>`;
    k.appendChild(d);}
}
$('sub').innerHTML = `<b>${D.rings} rings</b> of ${D.rise}mm, <b>${D.height}mm</b> tall`
  + ` — ${D.shape}, ø${D.throat} throat to ø${D.rim} rim`;
$('nums').innerHTML = [['rings',D.rings],['rise per ring',D.rise+'mm'],
  ['height',D.height+'mm'],['throat',D.throat+'mm'],['rim',D.rim+'mm'],
  ['section',D.shape]].map(([a,b])=>`<dt>${a}</dt><dd>${b}</dd>`).join('');
key(); revn(); addEventListener('resize',draw); draw();
</script>
"##;
