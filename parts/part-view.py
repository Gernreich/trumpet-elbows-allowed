#!/usr/bin/env python3
"""A bell or a mouthpiece you can turn: rings stacked, drag to rotate.

    python3 part-view.py bell/cut-files/bell-round10-153mm-17rings-x3-rim86-cut-files.svg
    python3 part-view.py mouthpiece/cut-files/mouthpiece-bore10-trumpet-parts-cut-files.svg

bell-view.py and mouthpiece-view.py draw one fixed isometric each, which is the
right thing for a page and no use for looking at the object. This is the same
geometry as a solid you can turn, in the family of the bore viewers in
bore-ribbon.

It reads the ring sizes with bell-view.py's own sections(), executed out of
that file rather than copied, so the two cannot come to disagree about what is
on a sheet. DISPLAY ONLY. Not a cut file.
"""
import json
import math
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

# bell-view.py is a script: its helpers sit above the line that reads argv.
# Take that prefix rather than a second copy of sections() and outline().
_bv = open(os.path.join(HERE, 'bell', 'bell-view.py')).read()
exec(_bv[:_bv.index('src = sys.argv[1]')])


def prof(h, c, n):
    """A rounded square of half-width h and corner radius c, as exactly n
    points, evenly spaced along its own perimeter.

    Always n, whatever the corner: outline() gives four points for a square and
    4*(per+1) for anything rounded, and a stack that mixes the two cannot be
    zipped into quads. Resampling by arc length also keeps the facets even, so
    a circle does not bunch its points at the corners it does not have.
    """
    p = outline(h, c, per=24)
    p = p + [p[0]]
    run = [0.0]
    for a, b in zip(p, p[1:]):
        run.append(run[-1] + math.hypot(b[0] - a[0], b[1] - a[1]))
    total, out, j = run[-1], [], 0
    for k in range(n):
        t = total * k / n
        while j + 1 < len(run) - 1 and run[j + 1] < t:
            j += 1
        span = run[j + 1] - run[j]
        u = 0.0 if span < 1e-12 else (t - run[j]) / span
        out.append((p[j][0] + (p[j+1][0] - p[j][0]) * u,
                    p[j][1] + (p[j+1][1] - p[j][1]) * u))
    return out


def rings_of(path):
    """[(aperture h, aperture c, outer h, outer c)], bottom of the stack first,
    plus the rise per ring."""
    s = open(path).read()

    def grp(colour):
        m = re.search(r'<g[^>]*stroke="%s"[^>]*>(.*?)</g>' % colour, s, re.S)
        if not m:
            return []
        out = []
        for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', m.group(1)):
            w = sections(d)
            out.append(w[0] if w else None)
        return out

    aps, outs = grp('#ff8000'), grp('#000000')
    if aps:
        # by index, in file order: the generators write both groups from one
        # list, so ring i is aps[i] with outs[i], and that is assembly order
        assert len(aps) == len(outs), f'{len(aps)} apertures, {len(outs)} outlines'
        pairs = [(a[0], a[1], o[0], o[1]) for a, o in zip(aps, outs) if a and o]
    else:                                  # a file from before the colour split
        pairs = []
        for d in re.findall(r'<path\b[^>]*\bd="([^"]+)"', s):
            w = sections(d)
            if len(w) == 2:
                (ah, ac), (oh, oc) = sorted(w)
                pairs.append((ah, ac, oh, oc))
    m = re.search(r'([\d.]+)mm of rise', s)
    return pairs, float(m.group(1)) if m else 3.0


def solid(pairs, rise, n):
    """Vertices and quads for the stack.

    Per ring: the outside band, the bore band, and the top annulus. The ring
    above overdraws whatever of that annulus it covers, so there is no need to
    work out which part shows - the painter's order does it.
    """
    V, Q, base = [], [], 0
    for i, (aw, ac, ow, oc) in enumerate(pairs):
        z0, z1 = i * rise, (i + 1) * rise
        A = prof(aw / 2.0, ac, n)
        O = prof(ow / 2.0, oc, n)
        for z in (z0, z1):
            V += [[p[0], p[1], z] for p in A]
            V += [[p[0], p[1], z] for p in O]
        a0, o0 = base, base + n            # at z0
        a1, o1 = base + 2 * n, base + 3 * n  # at z1
        for k in range(n):
            j = (k + 1) % n
            Q.append({'v': [o0 + k, o0 + j, o1 + j, o1 + k], 'f': 1, 's': i})
            Q.append({'v': [a1 + k, a1 + j, a0 + j, a0 + k], 'f': 2, 's': i})
            Q.append({'v': [a1 + k, o1 + k, o1 + j, a1 + j], 'f': 0, 's': i})
            if i == 0:
                Q.append({'v': [a0 + j, o0 + j, o0 + k, a0 + k], 'f': 3, 's': i})
        base += 4 * n
    return V, Q


def wheel(n):
    out = []
    for i in range(max(n, 1)):
        h = (i * 320.0 / max(n, 1) + 20) % 360
        c, x = 0.55, 0.55 * (1 - abs((h / 60) % 2 - 1))
        r, g, b = [(c, x, 0), (x, c, 0), (0, c, x),
                   (0, x, c), (x, 0, c), (c, 0, x)][int(h // 60) % 6]
        m = 0.34
        out.append('#%02x%02x%02x' % (int((r+m)*255), int((g+m)*255), int((b+m)*255)))
    return out


def data_for(path, n):
    pairs, rise = rings_of(path)
    assert pairs, f'no rings in {path}'
    V, Q = solid(pairs, rise, n)
    ap0, rim = pairs[0][0], pairs[-1][2]
    rnd = [p for p in pairs if p[3] > 0.4 * p[2]]
    return {
        'V': [[round(v, 3) for v in p] for p in V], 'Q': Q,
        'rings': len(pairs), 'rise': rise,
        'height': round(len(pairs) * rise, 1),
        'throat': round(ap0, 2), 'rim': round(rim, 2),
        'shape': ('square throat to round rim' if len(rnd) and pairs[0][1] < 0.1
                  else 'square throat to square rim' if pairs[0][1] < 0.1
                  else 'round'),
        'pal': wheel(len(pairs)),
        'faces': ['ring face', 'outside', 'the bore', 'underside'],
        'facecol': ['#d8b445', '#9c7c1c', '#4a3a0c', '#7a6216'],
        'keycol': ['#e8c95f', '#a8871f', '#3a2d08', '#7a6216'],
    }


HTML = r'''<title>__TITLE__</title>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root{color-scheme:dark;--bg:#14161a;--ink:#e8eaed;--dim:#9aa3ad;--line:#2a2f36;
      --panel:#1b1e24;--accent:#e0457b;--edge:rgba(0,0,0,.42)}
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
button{font:inherit;color:var(--ink);background:#232830;border:1px solid var(--line);
       border-radius:6px;padding:5px 9px;cursor:pointer;margin:0 4px 4px 0}
button[aria-pressed=true]{background:var(--accent);border-color:var(--accent);color:#fff}
button:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
input[type=range]{width:100%;accent-color:var(--accent)}
.key{display:flex;align-items:center;gap:7px;margin:3px 0;font-size:12.5px}
.sw{width:12px;height:12px;border-radius:3px;flex:0 0 auto;
    box-shadow:inset 0 0 0 1px rgba(255,255,255,.35)}
.hint{color:var(--dim);font-size:12px;margin-top:10px}
dl{margin:0;display:grid;grid-template-columns:auto 1fr;gap:2px 10px;font-size:12.5px}
dt{color:var(--dim)}dd{margin:0;text-align:right;font-variant-numeric:tabular-nums}
@media (max-width:720px){main{grid-template-columns:1fr;height:auto}
  #wrap{height:56vh}aside{border-left:0;border-top:1px solid var(--line)}}
</style>
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
    <div class="hint">Drag to turn. Scroll to zoom. Double-click to reset.</div>
  </aside>
</main>
<script>
const D = __DATA__;
const cv = document.getElementById('c'), cx = cv.getContext('2d');
let yaw = -0.5, pitch = -0.62, zoom = 1, mode = 'face', reveal = D.rings;
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
  const P=p=>{const r=rot(p); return [w/2+(r[0]-mx)*s, h/2-(r[1]-my)*s, r[2]];};
  const items=[];
  for (const q of D.Q){
    if (q.s >= reveal) continue;
    const pts=q.v.map(i=>P(D.V[i]));
    let area=0;
    for(let i=0;i<4;i++){const a=pts[i],b=pts[(i+1)%4]; area+=a[0]*b[1]-b[0]*a[1];}
    items.push({q,pts,z:(pts[0][2]+pts[1][2]+pts[2][2]+pts[3][2])/4,front:area<0});
  }
  items.sort((a,b)=>a.z-b.z);
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
let drag=null;
cv.addEventListener('pointerdown',e=>{drag=[e.clientX,e.clientY];
  cv.classList.add('drag'); cv.setPointerCapture(e.pointerId);});
cv.addEventListener('pointermove',e=>{ if(!drag) return;
  yaw += (e.clientX-drag[0])*0.008;
  pitch = Math.max(-1.55, Math.min(1.55, pitch + (e.clientY-drag[1])*0.008));
  drag=[e.clientX,e.clientY]; draw();});
for (const ev of ['pointerup','pointercancel'])
  cv.addEventListener(ev,()=>{drag=null; cv.classList.remove('drag');});
cv.addEventListener('wheel',e=>{e.preventDefault();
  zoom=Math.max(0.35,Math.min(6,zoom*(e.deltaY<0?1.1:1/1.1))); draw();},{passive:false});
cv.addEventListener('dblclick',()=>{yaw=-0.5;pitch=-0.62;zoom=1;draw();});
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
'''


def main():
    a = sys.argv[1:]
    files = [x for x in a if not x.startswith('-')]
    if not files:
        print(__doc__.strip().splitlines()[2].strip())
        return 1
    n = 28
    for x in a:
        if x.startswith('--facets='):
            n = int(x.split('=', 1)[1])
    src = files[0]
    d = data_for(src, n)
    base = os.path.basename(src).replace('-cut-files.svg', '')
    kind = 'Mouthpiece' if 'mouthpiece' in base else 'Bell'
    title = f'{kind}: {base}'
    out = files[1] if len(files) > 1 else os.path.join(
        os.path.dirname(os.path.abspath(src)), base + '-turn.html')
    open(out, 'w').write(HTML.replace('__TITLE__', title)
                             .replace('__DATA__', json.dumps(d, separators=(',', ':'))))
    print(f'  {os.path.basename(out):<58}{d["rings"]:>3} rings, '
          f'{len(d["Q"])} quads, drag to turn')
    return 0


if __name__ == '__main__':
    sys.exit(main())
