// Verify an octagonal torus cut file: geometry, joint phase, and nesting clearances.
//
// Named for what it knows. It understands one object -- an octagonal torus with plates
// at R 90, a hole at apothem 58.149 and a five-colour cut order -- and its build sheet,
// its three boxes.py runs and the trumpet that shares the plate. Given anything else it
// prints the palette, inventory and sheet bounds, which are measured and correct, then
// says so and stops rather than reporting nonsense with stars on it.
//
//   node verify_torus.js BuildA1_90_25.svg
//   node verify_torus.js BuildA1_90_25.svg RunA2_R59Point693.svg    # 2nd arg = disc the panels key to
//
// Transform-aware: composes every <g transform> on the ancestor chain. A tool that skips them
// reports parts at pre-transform coordinates and will confidently mislocate a correct part.
var fs = require('fs');
var COS = Math.cos(Math.PI / 8), SEC = 1 / COS, TAN = Math.tan(Math.PI / 8);

function mul(A, B) {
  return [A[0]*B[0]+A[2]*B[1], A[1]*B[0]+A[3]*B[1], A[0]*B[2]+A[2]*B[3],
          A[1]*B[2]+A[3]*B[3], A[0]*B[4]+A[2]*B[5]+A[4], A[1]*B[4]+A[3]*B[5]+A[5]];
}
function apply(M, p) { return [M[0]*p[0]+M[2]*p[1]+M[4], M[1]*p[0]+M[3]*p[1]+M[5]]; }
function parseT(s) {
  var M = [1,0,0,1,0,0], re = /(translate|rotate|scale|matrix)\s*\(([^)]*)\)/g, m;
  while ((m = re.exec(s))) {
    var n = m[2].trim().split(/[\s,]+/).map(Number), T;
    if (m[1] === 'translate') T = [1,0,0,1,n[0],n[1]||0];
    else if (m[1] === 'scale') T = [n[0],0,0,n.length>1?n[1]:n[0],0,0];
    else if (m[1] === 'matrix') T = n;
    else { var a=n[0]*Math.PI/180,c=Math.cos(a),s2=Math.sin(a); T=[c,s2,-s2,c,0,0];
           if (n.length===3) T = mul([1,0,0,1,n[1],n[2]], mul(T,[1,0,0,1,-n[1],-n[2]])); }
    M = mul(M, T);
  }
  return M;
}
// Sample an elliptical arc, rather than jumping to its far end.
//
// pts_() used to take only an arc's endpoint. A circle drawn as four arcs therefore
// reduced to four points at the cardinal positions -- a diamond -- and every measurement
// built on it was confidently wrong: eleven concentric mouthpiece rings were reported as
// twelve squares at 1/root2 of their true diameter, a reading consistent enough to be
// believed and documented. Arcs are geometry, not travel.
//
// Endpoint to centre parameterisation, SVG 1.1 F.6.5. Sampled about every degree, so the
// sagitta error is r*(1-cos(0.5deg)) -- 0.0023mm on a 60mm radius.
function arcPts(x1, y1, rx, ry, rot, large, sweep, x2, y2) {
  if (!rx || !ry) return [[x2, y2]];
  rx = Math.abs(rx); ry = Math.abs(ry);
  var phi = rot * Math.PI / 180, cp = Math.cos(phi), sp = Math.sin(phi);
  var dx2 = (x1 - x2) / 2, dy2 = (y1 - y2) / 2;
  var x1p =  cp * dx2 + sp * dy2, y1p = -sp * dx2 + cp * dy2;
  var lam = (x1p*x1p)/(rx*rx) + (y1p*y1p)/(ry*ry);
  if (lam > 1) { var k = Math.sqrt(lam); rx *= k; ry *= k; }
  var num = rx*rx*ry*ry - rx*rx*y1p*y1p - ry*ry*x1p*x1p;
  var den = rx*rx*y1p*y1p + ry*ry*x1p*x1p;
  var co = den ? Math.sqrt(Math.max(0, num / den)) : 0;
  if (large === sweep) co = -co;
  var cxp = co * rx * y1p / ry, cyp = -co * ry * x1p / rx;
  var cx = cp*cxp - sp*cyp + (x1 + x2)/2, cy = sp*cxp + cp*cyp + (y1 + y2)/2;
  function ang(ux, uy, vx, vy) {
    var d = Math.sqrt((ux*ux+uy*uy)*(vx*vx+vy*vy));
    var t = d ? Math.max(-1, Math.min(1, (ux*vx + uy*vy) / d)) : 1;
    var a = Math.acos(t);
    return (ux*vy - uy*vx < 0) ? -a : a;
  }
  var ux = (x1p - cxp)/rx, uy = (y1p - cyp)/ry;
  var vx = (-x1p - cxp)/rx, vy = (-y1p - cyp)/ry;
  var th1 = ang(1, 0, ux, uy), dth = ang(ux, uy, vx, vy);
  if (!sweep && dth > 0) dth -= 2*Math.PI;
  else if (sweep && dth < 0) dth += 2*Math.PI;
  var n = Math.max(4, Math.ceil(Math.abs(dth) / (Math.PI/180)));
  var pts = [];
  for (var i = 1; i <= n; i++) {
    var th = th1 + dth * (i / n);
    var ct = Math.cos(th), st = Math.sin(th);
    pts.push([cx + cp*rx*ct - sp*ry*st, cy + sp*rx*ct + cp*ry*st]);
  }
  return pts;
}
function pts_(d) {
  var toks = d.match(/[MmLlHhVvCcSsQqAaZz]|-?\d*\.?\d+(?:[eE]-?\d+)?/g) || [];
  var out=[], i=0, cmd=null, x=0, y=0, sx=0, sy=0;
  function num(){ return parseFloat(toks[i++]); }
  while (i < toks.length) {
    if (/^[A-Za-z]$/.test(toks[i])) cmd = toks[i++];
    if (!cmd) { i++; continue; }
    var rel = cmd === cmd.toLowerCase(), c = cmd.toUpperCase();
    if (c==='M'){ var nx=num(),ny=num(); x=rel?x+nx:nx; y=rel?y+ny:ny; sx=x; sy=y; out.push([x,y]); cmd=rel?'l':'L'; }
    else if (c==='L'){ var lx=num(),ly=num(); x=rel?x+lx:lx; y=rel?y+ly:ly; out.push([x,y]); }
    else if (c==='H'){ var v=num(); x=rel?x+v:v; out.push([x,y]); }
    else if (c==='V'){ var w=num(); y=rel?y+w:w; out.push([x,y]); }
    else if (c==='C'){ num();num();num();num(); var ex=num(),ey=num(); x=rel?x+ex:ex; y=rel?y+ey:ey; out.push([x,y]); }
    else if (c==='S'||c==='Q'){ num();num(); var qx=num(),qy=num(); x=rel?x+qx:qx; y=rel?y+qy:qy; out.push([x,y]); }
    else if (c==='A'){
      var arx=num(), ary=num(), arot=num(), alarge=num(), asweep=num();
      var ax=num(), ay=num(), ex=rel?x+ax:ax, ey=rel?y+ay:ay;
      arcPts(x, y, arx, ary, arot, alarge, asweep, ex, ey).forEach(function(q){ out.push(q); });
      x=ex; y=ey;
    }
    else if (c==='Z'){ x=sx; y=sy; }
    else i++;
  }
  return out;
}
function apo(dx, dy) {
  var m = -1e9;
  for (var k = 0; k < 8; k++) { var t=k*Math.PI/4, v=dx*Math.cos(t)+dy*Math.sin(t); if (v>m) m=v; }
  return m;
}
// Non-cut geometry. Widening this list is how a recolour silently loses parts —
// an earlier version skipped #0000ff too, and reported 14 contours for an intact
// file — so each entry has to earn its place.
//
//   #8000ff  the only skip colour. Lines carried in the file that this build does
//            not cut -- here, the trumpet lines that slice the torus into the
//            simple trumpet. Explicit, so "not cut" is a decision recorded in the
//            drawing rather than a colour someone forgot to map.
//
// Red and green were listed here until the trumpet lines were recoloured. They are
// gone deliberately: red means CUT in every repository alongside this one, and a
// verifier that quietly ignores it would pass a file whose parts never get cut.
var IGNORE = { '#8000ff': 'violet — skip, not cut in this build',
               '#0000ff': 'blue — engraving, marked before anything is cut' };
var palette = {};   // every stroke colour seen -> { n, ignored }

// Blue is here for a different reason from violet, and the distinction is the
// point of the warning above. Violet is geometry this build declines to cut.
// Blue never cuts anywhere in these repositories -- it is the engrave stage,
// and it carries the piece numbers written by number_pieces.js. Counted as cut
// geometry it turns 20 contours into 34, puts 49 marks in the inventory and
// makes the tool tell you to give blue a cutting operation, which would burn
// the numbers straight through the ply.
//
// The earlier version this comment warns about skipped blue when nothing in
// the file was blue and parts were: that was a recolour losing parts, not this.
// The guard against repeating it is SKIP_IS_MARKING below -- blue is only
// ignorable while every blue contour is small enough to be a mark.
var ENGRAVE = '#0000ff', MARK_MAX = 20;   // mm; a glyph, not a part

// An explicit #000000 and no stroke at all are the same operation on the machine,
// so they must collapse to one key or the cut order sees an unknown colour.
//
// An element may declare its stroke twice — once in style="", once as a presentation
// attribute stroke="". The CSS cascade says style wins, and browsers and Inkscape both
// agree, so that is what this reads. Not every laser importer applies the cascade, and
// one that takes the attribute instead reads a different cut stage for the same part.
// Anywhere the two disagree is recorded and reported rather than silently resolved.
var strokeClash = [];
function strokeOf(attrs) {
  var sm = /style="[^"]*?stroke:\s*(#[0-9a-fA-F]{6})/.exec(attrs);
  var am = /(?:^|\s)stroke="(#[0-9a-fA-F]{6})"/.exec(attrs);
  if (sm && am && sm[1].toLowerCase() !== am[1].toLowerCase()) {
    var id = /\bid="([^"]+)"/.exec(attrs);
    strokeClash.push({ id: id ? id[1] : '(no id)',
                       style: sm[1].toLowerCase(), attr: am[1].toLowerCase() });
  }
  var m = sm || /stroke:\s*(#[0-9a-fA-F]{6})/.exec(attrs) || am;
  if (!m) return 'black';
  var c = m[1].toLowerCase();
  return c === '#000000' ? 'black' : c;
}
// ─── geometry from the shapes that are not paths ──────────────────────────────
// collect() read <g> and <path> only. Everything else -- a bore drawn as <circle>, an
// outline as <rect>, a fold as <line> -- was not skipped with a warning, it was never
// seen: absent from the contour count, the inventory, the palette and therefore the cut
// order, the nesting checks and the sheet bounds. The tool reported success on a partial
// reading, which is the worst way for a checker to be wrong.
//
// Circles and ellipses have no vertices, so they are sampled. 360 segments puts the
// radial error at r*(1-cos(0.5°)) -- 0.0023mm on a 60mm radius, far below anything
// measured here.
var ARC = 360;
function attrNum(attrs, name, dflt) {
  var m = new RegExp('\\b' + name + '="([^"]*)"').exec(attrs);
  return m ? (parseFloat(m[1]) || 0) : dflt;
}
function shapePts(tag, attrs) {
  var i, t, out = [];
  if (tag === 'rect') {
    var x = attrNum(attrs,'x',0), y = attrNum(attrs,'y',0);
    var w = attrNum(attrs,'width',0), h = attrNum(attrs,'height',0);
    if (!(w > 0 && h > 0)) return [];
    var rx = Math.min(attrNum(attrs,'rx',0), w/2), ry = Math.min(attrNum(attrs,'ry',0) || attrNum(attrs,'rx',0), h/2);
    if (rx > 0 && ry > 0) {                       // rounded corners, sampled
      var corners = [[x+w-rx, y+ry, 0], [x+w-rx, y+h-ry, 1], [x+rx, y+h-ry, 2], [x+rx, y+ry, 3]];
      corners.forEach(function (c) {
        for (i = 0; i <= 16; i++) {
          t = (-Math.PI/2) + c[2]*Math.PI/2 + (i/16)*Math.PI/2;
          out.push([c[0] + rx*Math.cos(t), c[1] + ry*Math.sin(t)]);
        }
      });
      return out;
    }
    return [[x,y],[x+w,y],[x+w,y+h],[x,y+h]];
  }
  if (tag === 'circle' || tag === 'ellipse') {
    var cx = attrNum(attrs,'cx',0), cy = attrNum(attrs,'cy',0);
    var ax = tag === 'circle' ? attrNum(attrs,'r',0) : attrNum(attrs,'rx',0);
    var ay = tag === 'circle' ? ax : attrNum(attrs,'ry',0);
    if (!(ax > 0 && ay > 0)) return [];
    for (i = 0; i < ARC; i++) { t = i/ARC*2*Math.PI; out.push([cx + ax*Math.cos(t), cy + ay*Math.sin(t)]); }
    return out;
  }
  if (tag === 'line') {
    return [[attrNum(attrs,'x1',0), attrNum(attrs,'y1',0)],
            [attrNum(attrs,'x2',0), attrNum(attrs,'y2',0)]];
  }
  if (tag === 'polyline' || tag === 'polygon') {
    var pm = /\bpoints="([^"]*)"/.exec(attrs);
    if (!pm) return [];
    var n = pm[1].trim().split(/[\s,]+/).map(Number);
    for (i = 0; i + 1 < n.length; i += 2) out.push([n[i], n[i+1]]);
    return out;
  }
  return [];
}
// A path in these files always comes from a stroked context, so no stroke means black --
// that is what strokeOf does and it stays. A bare shape is different: BuildA1_90_25.svg
// carries a 1029.79 x 880.71mm <rect> with no stroke and no style, an Inkscape canvas
// artifact larger than the sheet. Counting it would have added a phantom contour and
// broken the bounds check. So a non-path shape must declare a stroke, its own or an
// ancestor's, before it is treated as cut geometry.
function hasStroke(attrs) { return /stroke\s*[:=]\s*"?#?[0-9a-zA-Z]/.test(attrs) && !/stroke\s*[:=]\s*"?none/.test(attrs); }

function collect(file) {
  var src = fs.readFileSync(file, 'utf8'), stack = [[1,0,0,1,0,0]], parts = [];
  var re = /<(\/?)(g|path|rect|circle|ellipse|line|polyline|polygon)\b([^>]*?)(\/?)>/g, m;
  while ((m = re.exec(src))) {
    var close=m[1], tag=m[2], attrs=m[3], self=m[4];
    if (tag === 'g') {
      if (close) { stack.pop(); continue; }
      var tm = /transform="([^"]+)"/.exec(attrs);
      stack.push(tm ? mul(stack[stack.length-1], parseT(tm[1])) : stack[stack.length-1]);
      if (self) stack.pop();
      continue;
    }
    if (close) continue;
    var dm = /(?:^|\s)d="([^"]+)"/.exec(attrs);
    var raw;
    if (tag === 'path') {
      if (!dm) continue;
      raw = pts_(dm[1]);
    } else {
      if (!hasStroke(attrs)) continue;      // an unstroked shape is not a cut
      raw = shapePts(tag, attrs);
    }
    if (!raw.length) continue;
    var col = strokeOf(attrs);
    if (col === 'black' && /fill:\s*#00ff00/i.test(attrs)) col = '#00ff00';
    var ign = Object.prototype.hasOwnProperty.call(IGNORE, col);
    if (!palette[col]) palette[col] = { n: 0, ignored: ign };
    palette[col].n++;
    if (ign) continue;
    var pm = /transform="([^"]+)"/.exec(attrs);
    var M = pm ? mul(stack[stack.length-1], parseT(pm[1])) : stack[stack.length-1];
    var p = raw.map(function (q) { return apply(M, q); });
    // A closed rectangle is four points. The threshold was 8, to skip stray fragments,
    // and it silently dropped whole parts: a square face drawn "M V H V Z" never reached
    // the inventory, so a four-piece sheet was reported as three. Fragments are excluded
    // by being open two-point lines, not by being simple.
    if (p.length < 4) continue;
    var xs=p.map(function(q){return q[0];}), ys=p.map(function(q){return q[1];});
    var x0=Math.min.apply(null,xs), x1=Math.max.apply(null,xs);
    var y0=Math.min.apply(null,ys), y1=Math.max.apply(null,ys);
    var idm = /\bid="([^"]+)"/.exec(attrs);
    parts.push({ pts:p, w:x1-x0, h:y1-y0, cx:(x0+x1)/2, cy:(y0+y1)/2, col:col,
                 id: idm ? idm[1] : '(no id)' });
  }
  return parts;
}
function f(x){ return Math.round(x*1000)/1000; }

var KERF = 0.1, A_HOLE_IN = 55.149, A_HOLE_OUT = 58.149, A_RIM_OUT = 86.149;

// Convex hull, and the minimum-area rectangle that contains it. A rotated rectangle's
// axis-aligned bounding box is NOT its size: a 73.326 x 31.2 panel turned 45° measures
// 66.527 square, which matches no panel and reads as unidentified geometry. Four were
// reported missing from a sheet that had all sixteen because of exactly that. Measure
// the shape, never the box around it.
function hull(pts) {
  var p = pts.slice().sort(function (a, b) { return a[0] - b[0] || a[1] - b[1]; });
  function cross(o, a, b) {
    return (a[0]-o[0])*(b[1]-o[1]) - (a[1]-o[1])*(b[0]-o[0]);
  }
  var lo = [], hi = [], i;
  for (i = 0; i < p.length; i++) {
    while (lo.length >= 2 && cross(lo[lo.length-2], lo[lo.length-1], p[i]) <= 0) lo.pop();
    lo.push(p[i]);
  }
  for (i = p.length - 1; i >= 0; i--) {
    while (hi.length >= 2 && cross(hi[hi.length-2], hi[hi.length-1], p[i]) <= 0) hi.pop();
    hi.push(p[i]);
  }
  return lo.slice(0, -1).concat(hi.slice(0, -1));
}
function minRect(pts) {
  var h = hull(pts);
  if (h.length < 3) return null;
  var best = null;
  for (var i = 0; i < h.length; i++) {
    var a = h[i], b = h[(i + 1) % h.length];
    var dx = b[0]-a[0], dy = b[1]-a[1], L = Math.hypot(dx, dy);
    if (L < 1e-9) continue;
    var ux = dx/L, uy = dy/L, e0 = 1e9, e1 = -1e9, n0 = 1e9, n1 = -1e9;
    h.forEach(function (q) {
      var e = (q[0]-a[0])*ux + (q[1]-a[1])*uy, n = -(q[0]-a[0])*uy + (q[1]-a[1])*ux;
      if (e < e0) e0 = e; if (e > e1) e1 = e;
      if (n < n0) n0 = n; if (n > n1) n1 = n;
    });
    var w = e1-e0, hh = n1-n0, area = w*hh;
    if (!best || area < best.area) {
      best = { w: Math.max(w, hh), h: Math.min(w, hh), area: area,
               ang: ((Math.atan2(dy, dx)*180/Math.PI % 90) + 90) % 90 };
    }
  }
  return best;
}
// A circle has no minimum-area rectangle worth the name -- every orientation gives the
// same area, so the one the caliper happens to land on is noise. Reporting "@68.5°" on a
// round part is the kind of confident, meaningless label that gets believed, so roundness
// is detected and the angle suppressed.
// Test the convex hull, not every point: a ring holds an outer circle and an aperture at
// two different radii, so the whole set is never equidistant from anything. The hull of a
// ring is its outer boundary, which is the shape the size describes.
function isRound(p) {
  var h = hull(p.pts), i;
  if (h.length < 16) return false;
  var cx = 0, cy = 0;
  for (i = 0; i < h.length; i++) { cx += h[i][0]; cy += h[i][1]; }
  cx /= h.length; cy /= h.length;
  var lo = 1e9, hi = -1e9;
  for (i = 0; i < h.length; i++) {
    var r = Math.hypot(h[i][0]-cx, h[i][1]-cy);
    if (r < lo) lo = r; if (r > hi) hi = r;
  }
  return hi > 0 && (hi - lo) / hi < 0.02;
}
function sizeOf(p) {
  if (!p._sz) {
    p._sz = minRect(p.pts) || { w: Math.max(p.w,p.h), h: Math.min(p.w,p.h), ang: 0 };
    if (isRound(p)) { p._sz.ang = 0; p._sz.round = true; }
  }
  return p._sz;
}

// A panel is a ~31.2mm-deep rectangle 40–90mm long, at whatever angle it was nested.
function isPanel(p) {
  var s = sizeOf(p);
  return s.h > 28 && s.h < 34 && s.w > 40 && s.w < 90;
}

// Apothem range of a contour measured about a given centre — the octagon's own metric,
// which is what separates "at the hole boundary" from "adrift in the waste inside it".
function apoRange(p, cx, cy) {
  var lo = 1e9, hi = -1e9;
  p.pts.forEach(function (q) { var a = apo(q[0] - cx, q[1] - cy); if (a < lo) lo = a; if (a > hi) hi = a; });
  return { lo: lo, hi: hi };
}

// The hole boundary sits AT the hole apothem, whether drawn as one loop or as 8 segments,
// so every point of it is out at 55.149 or beyond. A contour that reaches further in than
// that is not the hole: it is a piece being harvested from the waste disc the hole frees.
// Until patches were added to the waste this distinction did not exist and anything inside
// the rim counted as hole geometry — which quietly reported 6 holes across 4 plates.
// The hole boundary sits AT the hole apothem, so its innermost point IS that apothem --
// not merely at or beyond it. The looser test accepted anything nested further out: two
// 12mm parts added inside the plate wall at apothem 67-79 were unioned into "the hole",
// which threw the eccentricity to 13.842mm and emptied the joint-phase pattern, so the
// tool announced "NOT COMPLEMENTARY -- will not assemble" about a hole that was exactly
// concentric. A segmented hole still passes: every segment lies on the same apothem.
function pointIn(pts, x, y) {
  var n = pts.length, c = false;
  for (var i = 0, j = n - 1; i < n; j = i++) {
    var a = pts[i], b = pts[j];
    if ((a[1] > y) !== (b[1] > y) &&
        x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]) c = !c;
  }
  return c;
}
function isHolePartOf(plate, p) {
  if (p === plate || isPanel(p)) return false;
  var r = apoRange(p, plate.cx, plate.cy);
  return r.hi <= 83.0 && Math.abs(r.lo - A_HOLE_IN) <= 0.5;
}
function holePartsFor(plate, all) {
  return all.filter(function (p) { return isHolePartOf(plate, p); });
}

var file = process.argv[2];
var P = collect(file);
console.log('\n════ ' + file.split('/').pop() + '   contours: ' + P.length);

// Print the palette before anything else. Stroke colour decides what gets counted,
// so a recolour must be visible here rather than showing up as a mystery contour count.
var cols = Object.keys(palette).sort(function (a, b) { return palette[b].n - palette[a].n; });
console.log('\n  colours (stroke)');
cols.forEach(function (c) {
  console.log('    ' + c.padEnd(12) + 'x' + String(palette[c].n).padEnd(4) +
              (palette[c].ignored ? 'IGNORED — ' + IGNORE[c] : 'counted as cut geometry'));
});
if (cols.filter(function (c) { return !palette[c].ignored; }).length > 2) {
  console.log('    note: cut contours span several colours. All are counted; make sure your');
  console.log('          laser software assigns every one of them a cutting operation.');
}
if (strokeClash.length) {
  var byPair = {};
  strokeClash.forEach(function (s) { var k = s.style + ' / ' + s.attr; byPair[k] = (byPair[k] || 0) + 1; });
  console.log('\n    *** ' + strokeClash.length + ' path(s) declare their stroke twice, with different ' +
              'colours.\n        style="" wins in a browser and in Inkscape, and is what is counted above.\n' +
              '        A laser importer that reads the presentation attribute instead puts these\n' +
              '        parts in a different cut stage:');
  Object.keys(byPair).sort().forEach(function (k) {
    var pair = k.split(' / ');
    console.log('          x' + String(byPair[k]).padEnd(3) + ' style ' + pair[0] +
                '  vs  attribute ' + pair[1] + '   (counted as ' + pair[0] + ')');
  });
  console.log('        Deleting the redundant stroke="" attribute removes the ambiguity.');
}

var agg = {};
P.forEach(function (p) {
  var sz = sizeOf(p);
  var k = (sz.round ? 'ø ' + f(sz.w)
                    : f(sz.w) + ' x ' + f(sz.h) + (sz.ang > 0.5 ? '  @' + f(sz.ang) + '°' : ''));
  agg[k] = (agg[k]||0) + 1;
});
console.log('\n  inventory');
Object.keys(agg).sort(function(a,b){return parseFloat(b)-parseFloat(a);}).forEach(function (k) {
  var w0=parseFloat(k), h0=parseFloat(k.split('x')[1]), ex='';
  var W=Math.max(w0,h0), H=Math.min(w0,h0);   // a rotated panel is the same panel
  if (W>40 && W<80 && H>28 && H<34) {
    var aA=(W-4.443)/(2*TAN), aB=(W-2.685)/(2*TAN);
    ex = '   -> panel for R ' + f(aA*SEC) + ' | ' + f(aB*SEC);
  }
  console.log('    ' + k + '   x' + agg[k] + ex);
});

var plates = P.filter(function(p){ return p.w>160 && Math.abs(p.w-p.h)<1; });

// ─── is this a file this tool knows anything about? ───────────────────────────
// Everything below understands exactly one object: an octagonal torus with plates at
// R 90, a hole at apothem 58.149 and a five-colour cut order. Pointed anywhere else it
// still parses correctly -- piece counts and sizes are right -- but its plate, hole and
// cut-order logic is meaningless, and it said so in stars: a buzz disc was reported as a
// plate with a missing hole, a geometry diagram's slate and blue as cut colours out of
// sequence. A checker whose complaints have to be mentally discarded trains you to
// discard the real ones too.
//
// The signature is a regular octagon at the rim apothem. Nothing else here has one.
// Two signatures, because the family has two shapes of file. The build sheet and the
// R 90 run carry an octagon at the rim apothem -- an octagon's corners project further
// than its flats, so only its outer edge is diagnostic, and that edge is the rim apothem
// plus half the drawn stroke. The inner-disc runs (R 59.693, R 56.446) have no such
// octagon at all, and are recognised by their panels instead: four or more at one of the
// four widths this torus uses. Matching on "looks like a panel" alone was not enough --
// trumpet-curved's 75mm walls are 75.14 x 31.14 and pass that shape test while belonging
// to a different instrument.
var nPanels = P.filter(isPanel).length;
var nDiscs  = P.filter(function (p) { return sizeOf(p).w > 90 && Math.abs(p.w - p.h) < 1; }).length;
var hasRim  = plates.some(function (pl) {
  return Math.abs(apoRange(pl, pl.cx, pl.cy).hi - (A_RIM_OUT + 0.1)) < 0.5;
});
// Structure, not magic numbers. A disc run is two square discs and eight panels, and the
// R 56.446 run's panels are a different width from the build's -- listing the four build
// widths gated it, which is wrong: it is a torus file. Two discs plus four panels catches
// all three runs and nothing else here. trumpet-curved's 75mm walls pass "looks like a
// panel" but come with no discs.
var isTorusFamily = hasRim || (nDiscs >= 2 && nPanels >= 4);

// A figure is not a sheet. torus-geometry-diagram.svg draws this torus at true size, so
// it has the rim octagon and passes every structural test -- and then its slate, blue and
// green get reported as cut colours out of sequence. What separates it is the ink: a cut
// sheet's colours all belong to the cut order or the skip list, a drawing's mostly do not.
var KNOWN_INK = { 'black':1, '#00ff00':1, '#ff8000':1, '#00ffff':1, '#0000ff':1, '#ff0000':1, '#8000ff':1 };
var inks = Object.keys(palette);
var strangers = inks.filter(function (c) { return !KNOWN_INK[c]; }).length;
if (strangers > inks.length / 2) isTorusFamily = false;
if (!isTorusFamily) {
  console.log('\n  NOT A TORUS-FAMILY SHEET');
  console.log('    No octagonal plate at rim apothem ' + A_RIM_OUT + ' — this file is not one');
  console.log('    this tool was written for. The palette, inventory and sheet bounds above');
  console.log('    are measured and correct. Everything below them would be nonsense, so it');
  console.log('    is skipped: plate and hole geometry, joint phase, nesting clearances,');
  console.log('    cut order, skip lines.');
  console.log('    For a sheet like this use svg-stroke-check.py, and read the sizes above.\n');
  process.exit(0);
}
var holeCount = plates.reduce(function (t, pl) { return t + holePartsFor(pl, P).length; }, 0);
// boxes.py run 1 emits two solid discs; having no hole is what they are, and the hole
// cutter is run 3. No plate having a hole is therefore a kind of file, not a fault --
// but SOME plates having one and others not is a fault, and still reads as one.
var anyHole = holeCount > 0;
console.log('\n  plates: ' + plates.length + '   hole contours: ' + holeCount +
            (plates.length ? '  (' + (holeCount / plates.length) + ' per plate — 8 if segmented, 1 if stitched)' : ''));

plates.forEach(function (PL, i) {
  var near = holePartsFor(PL, P);
  var all = []; near.forEach(function(s){ all = all.concat(s.pts); });
  if (!all.length) {
    console.log('\n  PLATE ' + i + '  centre (' + f(PL.cx) + ', ' + f(PL.cy) + ')');
    console.log(anyHole
      ? '     *** NO HOLE FOUND IN THIS PLATE — it is solid, or the hole is not positioned in it ***'
      : '     solid disc — no plate here carries a hole, so this is a disc run, not a build sheet');
    return;
  }
  var xs=all.map(function(q){return q[0];}), ys=all.map(function(q){return q[1];});
  var hx=(Math.min.apply(null,xs)+Math.max.apply(null,xs))/2;
  var hy=(Math.min.apply(null,ys)+Math.max.apply(null,ys))/2;
  console.log('\n  PLATE ' + i + '  centre (' + f(PL.cx) + ', ' + f(PL.cy) + ')   segments ' + near.length +
              '   hole eccentricity ' + f(Math.hypot(hx-PL.cx, hy-PL.cy)));
  [['rim', PL.pts, PL.cx, PL.cy], ['hole', all, hx, hy]].forEach(function (S) {
    var vals = S[1].map(function(q){ return apo(q[0]-S[2], q[1]-S[3]); }).sort(function(a,b){return a-b;});
    var cl=[]; vals.forEach(function(v){ var l=cl[cl.length-1]; if(l&&v-l.hi<0.06){l.hi=v;l.n++;} else cl.push({lo:v,hi:v,n:1}); });
    cl.forEach(function (c) { if (c.n<8) return;
      console.log('     ' + S[0].padEnd(5) + ' line ' + f(c.lo-0.1) + '   R ' + f((c.lo-0.1)*SEC) +
                  '   flats ' + f(2*(c.lo-0.1)) + '   n ' + c.n); });
  });
});


// ─── joint phase ──────────────────────────────────────────────────────────────
// Which of the two boundary lines each point of a face sits on, compressed to runs.
// Counts are NOT usable for this: files re-saved through Inkscape carry duplicate nodes,
// so identical parts can report opposite majorities. Intervals depend only on geometry.
function facePattern(pts, cx, cy, aIn, aOut) {
  var half = aOut * TAN, seg = [];
  pts.forEach(function (q) {
    var dx = q[0] - cx, dy = cy - q[1];
    if (Math.abs(dx) > half - 0.5) return;
    var w = Math.abs(dy - aIn) < 0.15 ? 'in ' : (Math.abs(dy - aOut) < 0.15 ? 'OUT' : null);
    if (w) seg.push({ x: +dx.toFixed(2), w: w });
  });
  seg.sort(function (a, b) { return a.x - b.x; });
  var runs = [], cur = null;
  seg.forEach(function (s) {
    if (!cur || cur.w !== s.w) { cur = { w: s.w, lo: s.x, hi: s.x }; runs.push(cur); } else cur.hi = s.x;
  });
  return runs.filter(function (r) { return r.hi - r.lo > 0.3; });
}
function fmtPattern(runs) {
  return runs.map(function (r) { return r.w + '[' + r.lo.toFixed(1) + '…' + r.hi.toFixed(1) + ']'; }).join(' ');
}
// Complementary = the same runs along the face, but on opposite boundary lines.
// Compare run midpoints with tolerance: the hole is kerf-offset from the disc, so the
// interval ends legitimately differ by ~0.1mm and an exact match would never fire.
function complementary(a, b, tol) {
  if (a.length !== b.length || !a.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i].w === b[i].w) return false;                                  // same line = same phase
    if (Math.abs((a[i].lo + a[i].hi) / 2 - (b[i].lo + b[i].hi) / 2) > tol) return false;
  }
  return true;
}

// KERF and the apothem constants are declared once, above isPanel.

if (plates.length) {
  var pl0 = plates[0];
  var sg0 = holePartsFor(pl0, P);
  var ha = []; sg0.forEach(function (s) { ha = ha.concat(s.pts); });
  if (!ha.length) {
    console.log('\n  JOINT PHASE');
    console.log(anyHole
      ? '    *** cannot check — no hole geometry found in plate 0 ***'
      : '    not applicable — a disc run has no hole to phase against');
  } else {
    var hxs = ha.map(function (q) { return q[0]; }), hys = ha.map(function (q) { return q[1]; });
    var hcx = (Math.min.apply(null, hxs) + Math.max.apply(null, hxs)) / 2;
    var hcy = (Math.min.apply(null, hys) + Math.max.apply(null, hys)) / 2;
    var hp = facePattern(ha, hcx, hcy, A_HOLE_IN + KERF, A_HOLE_OUT + KERF);
    console.log('\n  JOINT PHASE  (interval pattern along one face)');
    console.log('    plate hole  : ' + fmtPattern(hp));
    if (process.argv[3]) {
      var ref = collect(require('path').resolve(process.argv[2], '..', process.argv[3]))
                  .filter(function (p) { return Math.abs(p.w - 116.499) < 0.05; })[0];
      if (ref) {
        var rp = facePattern(ref.pts, ref.cx, ref.cy, A_HOLE_IN + KERF, A_HOLE_OUT + KERF);
        console.log('    ref disc    : ' + fmtPattern(rp));
        console.log('    -> ' + (complementary(hp, rp, 0.3)
                     ? 'COMPLEMENTARY ✓  the plate\'s tabs land in the panel\'s notches'
                     : '*** NOT COMPLEMENTARY — will not assemble ***'));
      } else {
        console.log('    (no R 59.693 disc found in ' + process.argv[3] + ')');
      }
    } else {
      console.log('    (pass the R 59.693 run file as a 2nd argument to check complementarity)');
    }
  }
}

// ─── nesting clearances ───────────────────────────────────────────────────────
// Bounding boxes are useless here: the plates have a 110mm hole and panels are legitimately
// nested in that waste. Classify each panel by the octagon support function instead.
var panels = P.filter(isPanel);   // strictly ~31.2mm tall rectangles — not hole loops
if (plates.length && panels.length) {
  var bad = 0, tight = 1e9, tightWho = '';
  panels.forEach(function (pn) {
    plates.forEach(function (pl, pi) {
      var lo = 1e9, hi = -1e9;
      pn.pts.forEach(function (q) { var a = apo(q[0] - pl.cx, q[1] - pl.cy); if (a < lo) lo = a; if (a > hi) hi = a; });
      var margin = null;
      if (hi <= A_HOLE_IN) margin = A_HOLE_IN - hi;
      else if (lo >= A_RIM_OUT) margin = lo - A_RIM_OUT;
      else { bad++; console.log('    *** CONFLICT: panel ' + f(sizeOf(pn).w) + ' @(' + f(pn.cx) + ',' + f(pn.cy) +
                                ') crosses plate ' + pi + ' material, spans a ' + f(lo) + '…' + f(hi)); }
      if (margin !== null && margin < tight) { tight = margin; tightWho = 'panel ' + f(sizeOf(pn).w) + ' vs plate ' + pi; }
    });
  });
  // Distance between the two outlines themselves. Bounding boxes were used here until
  // panels were nested at 45°, where the box covers a great deal of sheet the part does
  // not: it reported a 0.668mm gap between two panels that are nowhere near that close.
  function segDist(p0, p1, q) {
    var dx = p1[0]-p0[0], dy = p1[1]-p0[1], L2 = dx*dx + dy*dy;
    var t = L2 ? Math.max(0, Math.min(1, ((q[0]-p0[0])*dx + (q[1]-p0[1])*dy) / L2)) : 0;
    return Math.hypot(p0[0] + t*dx - q[0], p0[1] + t*dy - q[1]);
  }
  function polyDist(A, B) {
    var best = 1e9;
    [[A,B],[B,A]].forEach(function (pair) {
      var X = pair[0], Y = pair[1];
      for (var i = 0; i < X.length; i++) {
        var a = X[i], b = X[(i+1) % X.length];
        for (var j = 0; j < Y.length; j++) {
          var d = segDist(a, b, Y[j]);
          if (d < best) best = d;
        }
      }
    });
    return best;
  }
  function inside(poly, q) {
    var c = false;
    for (var i = 0, j = poly.length-1; i < poly.length; j = i++) {
      if ((poly[i][1] > q[1]) !== (poly[j][1] > q[1]) &&
          q[0] < (poly[j][0]-poly[i][0]) * (q[1]-poly[i][1]) / (poly[j][1]-poly[i][1]) + poly[i][0]) c = !c;
    }
    return c;
  }
  var ov = 0, mg = 1e9, mgWho = '';
  var hulls = panels.map(function (p) { return hull(p.pts); });
  for (var i = 0; i < panels.length; i++) for (var j = i + 1; j < panels.length; j++) {
    var a = panels[i], b = panels[j];
    if (Math.hypot(a.cx-b.cx, a.cy-b.cy) > 200) continue;
    var overlap = hulls[i].some(function (q) { return inside(hulls[j], q); }) ||
                  hulls[j].some(function (q) { return inside(hulls[i], q); });
    if (overlap) {
      ov++;
      console.log('    *** PANEL OVERLAP: ' + f(sizeOf(a).w) + ' and ' + f(sizeOf(b).w));
      continue;
    }
    var g = polyDist(hulls[i], hulls[j]);
    if (g < mg) { mg = g; mgWho = f(sizeOf(a).w) + ' ↔ ' + f(sizeOf(b).w); }
  }
  console.log('\n  NESTING');
  console.log('    panels crossing plate material : ' + bad + (bad ? '  ✗' : '  ✓'));
  console.log('    panel-to-panel overlaps        : ' + ov + (ov ? '  ✗' : '  ✓'));
  console.log('    tightest panel↔plate margin    : ' + f(tight) + 'mm   (' + tightWho + ')');
  console.log('    tightest panel↔panel gap       : ' + f(mg) + 'mm   (' + mgWho + ')');
}

// ─── cut order ────────────────────────────────────────────────────────────────
// Colour is the cut sequence. The rule a laser job must respect is that a contour
// is cut while its material is still held: anything nested inside a piece of waste
// goes before the cut that frees that waste. Here that means the panels sitting in
// the plate holes precede the holes, and the holes precede the rims.
// green -> orange -> cyan -> black, and blue is not here at all: blue means ENGRAVE
// across these repositories and never cuts. Green goes first because it carries both
// the panels nested in the plate holes and the patch lines, and each has to be cut
// while its material is still held — the panels before the orange hole drops the
// waste they sit in, the patch lines before the black rim frees the plate.
var CUT_ORDER = ['#00ff00', '#ff8000', '#00ffff', 'black'];
var CUT_NAME = { '#00ff00': 'green', '#ff8000': 'orange', '#00ffff': 'cyan', 'black': 'black' };

if (plates.length) {
  console.log('\n  CUT ORDER');
  // Check every colour the file uses, not just the closed contours. The patch lines
  // are open paths, so a check that walked contours alone reported a clean order
  // while twenty cut-coloured segments had no place in it.
  Object.keys(palette).forEach(function (c) {
    if (palette[c].ignored || CUT_ORDER.indexOf(c) >= 0) return;
    console.log('    *** ' + c + ' (x' + palette[c].n + ') is a cut colour with no place in the ' +
                'cut order — sequence unknown for it');
  });
  var rank = {}; CUT_ORDER.forEach(function (c, i) { rank[c] = i; });

  // Which contours sit inside a plate's hole, and therefore drop out with the waste?
  // Not only panels: patches harvested from the waste disc are nested in exactly the same
  // sense and carry exactly the same ordering obligation. Restricting this to panels let
  // two 76mm patches be added inside the ring holes and cut a stage AFTER the hole that
  // frees the disc they are drawn on, with the check still printing a tick.
  var nested = [];
  plates.forEach(function (pl, pi) {
    P.forEach(function (p) {
      if (p === pl || isHolePartOf(pl, p)) return;
      var r = apoRange(p, pl.cx, pl.cy);
      if (r.hi <= A_HOLE_IN) {
        nested.push({ part: p, plate: pi, kind: isPanel(p) ? 'panel' : 'patch',
                      web: A_HOLE_IN + 0.1 - r.hi });
      }
    });
  });
  // How much material is left between a nested piece and the hole cut that surrounds it.
  // The NESTING block above measures panels only, so a patch tucked hard against the hole
  // does not show up there.
  var thin = nested.length
    ? nested.slice().sort(function (a, b) { return a.web - b.web; })[0] : null;

  CUT_ORDER.forEach(function (c, i) {
    var g = P.filter(function (p) { return p.col === c; });
    // A stage may be open paths only — the patch lines are — in which case there is
    // no contour to describe, but the stage still has to appear in the sequence.
    if (!g.length) {
      if (palette[c] && !palette[c].ignored) {
        console.log('    ' + (i + 1) + '. ' + CUT_NAME[c].padEnd(7) + 'x' +
                    String(palette[c].n).padEnd(4) +
                    'open cut lines — patch cuts, no closed contour of their own');
      }
      return;
    }
    var nPan = g.filter(function (p) { return isPanel(p); }).length;
    var nRim = g.filter(function (p) { return p.w > 160 && Math.abs(p.w - p.h) < 1; }).length;
    var nPatch = g.filter(function (p) {
      return nested.some(function (nn) { return nn.part === p && nn.kind === 'patch'; });
    }).length;
    var nHole = g.length - nPan - nRim - nPatch;
    var role;
    if (nPatch === g.length) role = 'patches harvested from the waste — cut before the waste is freed';
    else if (nPan === g.length) {
      role = g.every(function (p) { return nested.some(function (nn) { return nn.part === p; }); })
        ? 'panels nested in the plate holes — cut before the waste is freed'
        : (nested.some(function (nn) { return nn.part.col === c; })
            ? 'panels, some nested in the holes and some on the open sheet'
            : 'panels on the open sheet');
    } else if (nRim === g.length) role = 'plate rims — frees the plates';
    else if (nHole === g.length) role = 'plate holes';
    else role = 'mixed — ' + nPan + ' panel(s), ' + nPatch + ' patch(es), ' +
                nHole + ' hole(s), ' + nRim + ' rim(s)';
    console.log('    ' + (i + 1) + '. ' + CUT_NAME[c].padEnd(7) + 'x' + String(g.length).padEnd(4) + role);
  });

  if (thin) {
    console.log('    tightest nested piece to its hole: ' + f(thin.web) + 'mm  (' +
                f(thin.part.w) + ' x ' + f(thin.part.h) + ' ' + thin.kind +
                ' in plate ' + thin.plate + ') — kerf comes off both sides of that');
  }
  var viol = 0;
  var holeCol = null, rimCol = null;
  P.forEach(function (p) {
    if (p.w > 160 && Math.abs(p.w - p.h) < 1) rimCol = p.col;
    else if (plates.some(function (pl) { return isHolePartOf(pl, p); })) holeCol = p.col;
  });
  nested.forEach(function (nn) {
    if (holeCol !== null && rank[nn.part.col] > rank[holeCol]) {
      viol++;
      console.log('    *** ' + f(nn.part.w) + 'mm ' + nn.kind + ' ' + nn.part.id +
                  ' sits inside plate ' + nn.plate + "'s hole but is cut " + CUT_NAME[nn.part.col] +
                  ', AFTER the ' + CUT_NAME[holeCol] + ' hole — the waste it is drawn on is ' +
                  'already loose by then');
    }
  });
  if (holeCol !== null && rimCol !== null && rank[holeCol] > rank[rimCol]) {
    viol++;
    console.log('    *** holes (' + CUT_NAME[holeCol] + ') are cut after rims (' + CUT_NAME[rimCol] +
                ') — the plate is loose before its hole is made');
  }
  var nP = nested.filter(function (n) { return n.kind === 'panel'; }).length;
  console.log('    ' + (viol ? '*** ' + viol + ' ordering problem(s) ***'
                             : nP + ' nested panels and ' + (nested.length - nP) +
                               ' patches cut before their hole ✓   holes before rims ✓'));
}

// ─── the engraving ────────────────────────────────────────────────────────────
// Blue is ignored as cut geometry, so like the skip lines it has to be re-read to be
// described. This is also the guard the IGNORE comment promises: blue is only safely
// ignorable while every blue contour is small enough to be a mark. A part recoloured
// blue would be silently dropped otherwise -- which is the exact failure that comment
// records from an earlier version.
(function () {
  var src = fs.readFileSync(file, 'utf8'), stack = [[1,0,0,1,0,0]], ink = [];
  var re = /<(\/?)(g|path|rect|circle|ellipse|line|polyline|polygon)\b([^>]*?)(\/?)>/g, m;
  while ((m = re.exec(src))) {
    var close = m[1], tag = m[2], attrs = m[3], self = m[4];
    if (tag === 'g') {
      if (close) { stack.pop(); continue; }
      var tm = /transform="([^"]+)"/.exec(attrs);
      stack.push(tm ? mul(stack[stack.length-1], parseT(tm[1])) : stack[stack.length-1]);
      if (self) stack.pop();
      continue;
    }
    if (close) continue;
    if (strokeOf(attrs) !== ENGRAVE) continue;
    var dm = /(?:^|\s)d="([^"]+)"/.exec(attrs);
    var base;
    if (tag === 'path') { if (!dm) continue; base = pts_(dm[1]); }
    else { if (!hasStroke(attrs)) continue; base = shapePts(tag, attrs); }
    if (!base.length) continue;
    var pm = /transform="([^"]+)"/.exec(attrs);
    var M = pm ? mul(stack[stack.length-1], parseT(pm[1])) : stack[stack.length-1];
    ink.push(base.map(function (t) { return apply(M, t); }));
  }
  if (!ink.length) return;

  console.log('\n  ENGRAVING  (blue — marked, never cut)');
  var big = 0, on = 0;
  ink.forEach(function (q) {
    var xs = q.map(function (t) { return t[0]; }), ys = q.map(function (t) { return t[1]; });
    var w = Math.max.apply(null, xs) - Math.min.apply(null, xs);
    var h = Math.max.apply(null, ys) - Math.min.apply(null, ys);
    if (Math.max(w, h) > MARK_MAX) big++;
    var cx = (Math.max.apply(null, xs) + Math.min.apply(null, xs)) / 2;
    var cy = (Math.max.apply(null, ys) + Math.min.apply(null, ys)) / 2;
    if (P.some(function (pc) { return pointIn(pc.pts, cx, cy); })) on++;
  });
  console.log('    ' + ink.length + ' strokes, ' + on + ' of them inside a cut piece');
  if (big) {
    console.log('    !! ' + big + ' blue contour(s) larger than ' + MARK_MAX +
                'mm — too big to be a mark.');
    console.log('       Blue is ignored as cut geometry. If a PART has been recoloured');
    console.log('       blue it is being dropped from every check above. Fix the colour.');
  } else {
    console.log('    every blue contour fits in ' + MARK_MAX +
                'mm \u2014 marks, not parts \u2713');
  }
})();

// ─── the skip lines ───────────────────────────────────────────────────────────
// collect() drops ignored colours before they ever become contours, which is right for
// counting cut geometry and useless for describing what is skipped. So re-read the file
// for them here.
//
// This exists because the writeup twice described these lines wrongly -- once as 42
// trumpet slices plus 22 patch lines, when they are one family of 16 per octagon -- and
// nothing could contradict it. A count printed from the file can.
(function () {
  var src = fs.readFileSync(file, 'utf8'), stack = [[1,0,0,1,0,0]], skip = [];
  var re = /<(\/?)(g|path|rect|circle|ellipse|line|polyline|polygon)\b([^>]*?)(\/?)>/g, m;
  while ((m = re.exec(src))) {
    var close = m[1], tag = m[2], attrs = m[3], self = m[4];
    if (tag === 'g') {
      if (close) { stack.pop(); continue; }
      var tm = /transform="([^"]+)"/.exec(attrs);
      stack.push(tm ? mul(stack[stack.length-1], parseT(tm[1])) : stack[stack.length-1]);
      if (self) stack.pop();
      continue;
    }
    if (close) continue;
    // violet only. IGNORE also holds blue since the piece numbers arrived, and
    // reading blue here would report 49 glyph strokes as skip lines.
    if (strokeOf(attrs) !== '#8000ff') continue;
    var dm = /(?:^|\s)d="([^"]+)"/.exec(attrs);
    var base;
    if (tag === 'path') { if (!dm) continue; base = pts_(dm[1]); }
    else { if (!hasStroke(attrs)) continue; base = shapePts(tag, attrs); }
    if (!base.length) continue;
    var pm = /transform="([^"]+)"/.exec(attrs);
    var M = pm ? mul(stack[stack.length-1], parseT(pm[1])) : stack[stack.length-1];
    var q = base.map(function (t) { return apply(M, t); });
    if (q.length) skip.push({ pts: q, n: q.length });
  }
  if (!skip.length || !plates.length) return;

  console.log('\n  SKIP LINES  (violet — carried, not cut)');
  var per = plates.map(function () { return []; });
  var loose = 0;
  skip.forEach(function (s) {
    var best = -1, bd = 1e9;
    plates.forEach(function (pl, i) {
      var hi = apoRange(s, pl.cx, pl.cy).hi;
      if (hi < bd) { bd = hi; best = i; }
    });
    if (bd > 110) { loose++; return; }
    per[best].push(apoRange(s, plates[best].cx, plates[best].cy));
  });
  // How far out the octagon itself is drawn, measured rather than assumed: the plates
  // carry finger joints and the rings do not, so their outer edges are not the same
  // number and neither is the constant a skip line should stop at.
  var edge = plates.map(function (pl) { return apoRange(pl, pl.cx, pl.cy).hi; });

  var counts = per.map(function (g) { return g.length; }), over = 0;
  per.forEach(function (g, i) {
    if (!g.length) { console.log('    plate ' + i + ': none'); return; }
    var lo = Math.min.apply(null, g.map(function (r) { return r.lo; }));
    var hi = Math.max.apply(null, g.map(function (r) { return r.hi; }));
    console.log('    plate ' + i + ': ' + g.length + ' line(s), apothem ' +
                f(lo) + ' … ' + f(hi) + '   edge at ' + f(edge[i]));
    // A skip line is a division of the wall, so it belongs between the hole and the
    // edge. One running past the edge sticks out into the waste, where it marks a cut
    // through nothing -- two did, by 0.414mm, and only a reading of the picture caught
    // them. Half the drawn stroke is the tolerance; anything more is real geometry.
    g.forEach(function (r) {
      if (r.hi > edge[i] + 0.15) {
        over++;
        console.log('      *** one reaches ' + f(r.hi) + ', ' + f(r.hi - edge[i]) +
                    'mm past the edge — it overshoots into the waste');
      }
    });
  });
  if (!over) console.log('    none reaches past its octagon\'s outer edge ✓');
  if (loose) console.log('    ' + loose + ' not associated with any plate');
  var same = counts.every(function (c) { return c === counts[0]; });
  console.log('    ' + (same ? 'every plate carries the same ' + counts[0] + ' ✓'
                              : '*** plates carry different counts: ' + counts.join(', ') +
                                ' — deliberate, or a line missed on one of them?'));
})();

// ─── sheet bounds ─────────────────────────────────────────────────────────────
var allx = [], ally = [];
P.forEach(function (p) { p.pts.forEach(function (q) { allx.push(q[0]); ally.push(q[1]); }); });
var x0 = Math.min.apply(null, allx), x1 = Math.max.apply(null, allx);
var y0 = Math.min.apply(null, ally), y1 = Math.max.apply(null, ally);
var vbm = /viewBox="([^"]+)"/.exec(fs.readFileSync(file, 'utf8'));
console.log('\n  SHEET');
console.log('    content : x ' + f(x0) + ' … ' + f(x1) + '   y ' + f(y0) + ' … ' + f(y1) +
            '   (' + f(x1 - x0) + ' × ' + f(y1 - y0) + 'mm)');
if (vbm) {
  var vb = vbm[1].trim().split(/[\s,]+/).map(Number);
  var outside = x0 < vb[0] - 0.01 || x1 > vb[0] + vb[2] + 0.01 || y0 < vb[1] - 0.01 || y1 > vb[1] + vb[3] + 0.01;
  console.log('    viewBox : x ' + vb[0] + ' … ' + f(vb[0] + vb[2]) + '   y ' + vb[1] + ' … ' + f(vb[1] + vb[3]));
  console.log('    ' + (outside ? '*** content extends outside the viewBox ***' : 'all content inside the viewBox ✓'));
}
console.log('');
