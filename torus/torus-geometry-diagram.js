#!/usr/bin/env node
// Generates torus-geometry-diagram.svg — the verified geometry of the 25 × 25 octagonal torus.
//
//   node torus-geometry-diagram.js [outerR] [ring] [thickness]
//   defaults: 90 25 3
//
// Every coordinate is computed; nothing is typed by hand.

// All three or none. Filling a missing argument from the defaults would answer a question
// nobody asked: `... 200 60` would silently compute in 3mm and print a plausible, wrong
// R_inner of 131.809 instead of 128.562.
var given = process.argv.slice(2);
if (given.length !== 0 && given.length !== 3) {
  console.error('REFUSING: give all three numbers, or none for the ' +
                'defaults (90 25 3). Got ' + given.length + ': ' + given.join(' '));
  console.error('\n  node torus-geometry-diagram.js <outerR> <ring> <thickness>');
  console.error('                                  │        │      └─ material thickness');
  console.error('                                  │        └──────── ring, face to face');
  console.error('                                  └───────────────── outer radius, corner to centre');
  process.exit(1);
}

var Ro = Number(given[0] || 90);             // outer octagon, vertex radius (boxes.py `radius`)
var RING = Number(given[1] || 25);           // clear annulus, face to face
var T = Number(given[2] || 3);               // material thickness

var COS = Math.cos(Math.PI / 8), SEC = 1 / COS;
var Ri = Ro - (RING + T) * SEC;              // inner octagon = the bore

var aRim = Ro * COS;                         // plate rim / outer tube inner surface
var aOut = aRim + T;                         // outer tube outer surface
var aBore = Ri * COS;                        // bore
var aRingIn = aBore + T;                     // ring inner bound = inner tube wall, outer face
var Rhole = Ri - T * SEC;                    // run 3 — the pre-compensated hole cutter

// NaN fails every comparison, so a non-numeric argument would slip past the check below and
// produce an SVG full of NaN coordinates. Reject it here.
[['outer radius', Ro], ['ring', RING], ['thickness', T]].forEach(function (p) {
  if (!isFinite(p[1]) || p[1] <= 0) {
    console.error('REFUSING: ' + p[0] + ' must be a positive number, got "' + p[1] + '"');
    console.error('\n  node torus-geometry-diagram.js <outerR> <ring> <thickness>');
    process.exit(1);
  }
});

// The three inputs are not independent: the ring and both walls have to fit inside the outer
// octagon. Below the floor there is no inner tube, and every number after this is nonsense.
if (Rhole <= 0) {
  var floor = (RING + 2 * T) * SEC;
  console.error('REFUSING: these numbers do not describe a torus.\n');
  console.error('  outer radius   ' + Math.round(Ro * 1000) / 1000);
  console.error('  ring           ' + RING);
  console.error('  thickness      ' + T);
  console.error('  hole cutter    ' + Math.round(Rhole * 1000) / 1000 + '   ← must be positive\n');
  console.error('R_outer must exceed (ring + 2 x thickness) x sec(180/n)');
  console.error('  = (' + RING + ' + 2 x ' + T + ') x ' + (Math.round(SEC * 1000000) / 1000000) +
                ' = ' + (Math.round(floor * 1000) / 1000) + '\n');
  console.error('Raise the outer radius above ' + (Math.round(floor * 1000) / 1000) +
                ', or reduce the ring or the thickness.');
  process.exit(1);
}

function f(x) { return Math.round(x * 1000) / 1000; }
function pt(r, deg) { var a = deg * Math.PI / 180; return [f(r * Math.cos(a)), f(-r * Math.sin(a))]; }
function poly(apothem, cx, cy) {
  var out = [], k, R = apothem * SEC;
  for (k = 0; k < 8; k++) { var p = pt(R, 22.5 + 45 * k); out.push((p[0] + cx) + ',' + (p[1] + cy)); }
  return out.join(' ');
}
function ring(aOuter, aInner, cx, cy, fill, op) {
  return '<path d="M' + poly(aOuter, cx, cy).replace(/ /g, 'L') + 'Z M' +
    poly(aInner, cx, cy).split(' ').reverse().join('L') + 'Z" fill="' + fill +
    '" fill-rule="evenodd" opacity="' + op + '"/>';
}

var CX = 122, CY = 158;                       // plan view centre
var SX = 372, SY = 118, S = 2.2;              // section view centre and scale
var BLUE = '#1d4ed8', GREEN = '#047857', INK = '#0f172a', MUT = '#475569';

var s = [];
s.push('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 300" width="1000" height="600" ' +
  'role="img" aria-label="Verified geometry of the 25 by 25 millimetre octagonal torus: a plan ' +
  'section through a plate, and a radial cross-section of the cavity">');
s.push('<rect width="500" height="300" fill="#fbfcfd"/>');
s.push('<g font-family="ui-sans-serif,system-ui,Helvetica,Arial,sans-serif">');

// ── titles ────────────────────────────────────────────────────────────────────
s.push('<text x="18" y="22" font-size="9" font-weight="700" fill="' + INK + '">' +
  f(RING) + ' × ' + f(RING) + ' octagonal torus — verified geometry</text>');
s.push('<text x="18" y="32" font-size="5.4" fill="' + MUT + '">R_inner = R_outer − (ring + t) × sec(22.5°) = ' +
  f(Ro) + ' − ' + f(RING + T) + ' × 1.082392 = ' + f(Ri) +
  '   ·   nominal values, before the 0.1mm kerf outset</text>');

// ── plan view ─────────────────────────────────────────────────────────────────
s.push(ring(aOut, aRim, CX, CY, '#bfdbfe', '0.95'));
s.push(ring(aRim, aRingIn, CX, CY, '#e2e8f0', '0.9'));
s.push(ring(aRingIn, aBore, CX, CY, '#bbf7d0', '0.95'));

s.push('<g stroke="#94a3b8" stroke-width="0.35" stroke-dasharray="3 2.5">');
s.push('<line x1="' + (CX - aOut - 10) + '" y1="' + CY + '" x2="' + (CX + aOut + 10) + '" y2="' + CY + '"/>');
s.push('<line x1="' + CX + '" y1="' + (CY - aOut - 10) + '" x2="' + CX + '" y2="' + (CY + aOut + 10) + '"/>');
s.push('</g>');

[[aOut, BLUE], [aRim, BLUE], [aRingIn, GREEN], [aBore, GREEN]].forEach(function (L) {
  s.push('<polygon points="' + poly(L[0], CX, CY) + '" fill="none" stroke="' + L[1] +
    '" stroke-width="1.4" stroke-linejoin="round"/>');
});

var DX = CX + aOut + 16;                      // dimension line, clear of the octagon
function ext(y) {                             // extension line from the face out to DX
  return '<line x1="' + f(CX + 6) + '" y1="' + f(y) + '" x2="' + (DX + 3) + '" y2="' + f(y) +
    '" stroke="#cbd5e1" stroke-width="0.35"/>';
}
function vd(y1, y2) {
  return '<line x1="' + DX + '" y1="' + f(y1) + '" x2="' + DX + '" y2="' + f(y2) + '"/>' +
    '<line x1="' + (DX - 2.6) + '" y1="' + f(y1) + '" x2="' + (DX + 2.6) + '" y2="' + f(y1) + '"/>' +
    '<line x1="' + (DX - 2.6) + '" y1="' + f(y2) + '" x2="' + (DX + 2.6) + '" y2="' + f(y2) + '"/>';
}
s.push(ext(CY - aOut) + ext(CY - aRim) + ext(CY - aRingIn) + ext(CY - aBore));
s.push('<g stroke="' + INK + '" stroke-width="0.55" fill="none">');
s.push(vd(CY - aOut, CY - aRim) + vd(CY - aRim, CY - aRingIn) + vd(CY - aRingIn, CY - aBore));
s.push('</g>');
s.push('<text x="' + (DX + 5) + '" y="' + f(CY - (aOut + aRim) / 2) + '" font-size="5" fill="' + BLUE + '" dominant-baseline="middle">' + f(T) + ' outer wall</text>');
s.push('<text x="' + (DX + 5) + '" y="' + f(CY - (aRim + aRingIn) / 2 - 2.8) + '" font-size="9" font-weight="700" fill="' + INK + '" dominant-baseline="middle">' + f(RING) + '</text>');
s.push('<text x="' + (DX + 5) + '" y="' + f(CY - (aRim + aRingIn) / 2 + 4.6) + '" font-size="4.8" fill="' + MUT + '" dominant-baseline="middle">ring, face to face</text>');
s.push('<text x="' + (DX + 5) + '" y="' + f(CY - (aRingIn + aBore) / 2) + '" font-size="5" fill="' + GREEN + '" dominant-baseline="middle">' + f(T) + ' inner wall</text>');

var pV = pt(Ro, 292.5), pA = pt(aRim, 270), pVi = pt(Ri, 292.5);
s.push('<g stroke="' + MUT + '" stroke-width="0.55" fill="none">');
s.push('<line x1="' + CX + '" y1="' + CY + '" x2="' + f(CX + pA[0]) + '" y2="' + f(CY + pA[1]) + '"/>');
s.push('<line x1="' + CX + '" y1="' + CY + '" x2="' + f(CX + pV[0]) + '" y2="' + f(CY + pV[1]) + '"/>');
var a1 = pt(24, 270), a2 = pt(24, 292.5);
s.push('<path d="M' + (CX + a1[0]) + ' ' + (CY + a1[1]) + ' A24 24 0 0 0 ' + (CX + a2[0]) + ' ' + (CY + a2[1]) + '"/>');
s.push('</g>');
s.push('<circle cx="' + f(CX + pVi[0]) + '" cy="' + f(CY + pVi[1]) + '" r="1.4" fill="' + GREEN + '"/>');
s.push('<text x="' + f(CX + pt(32, 281)[0]) + '" y="' + f(CY + pt(32, 281)[1]) + '" font-size="5" fill="' + MUT + '">22.5°</text>');
s.push('<text x="' + f(CX + pV[0] + 5) + '" y="' + f(CY + pV[1] + 8) + '" font-size="5.4" fill="' + BLUE + '">R ' + f(Ro) + '</text>');
s.push('<text x="' + f(CX + pVi[0] - 5) + '" y="' + f(CY + pVi[1] - 4) + '" font-size="5.4" fill="' + GREEN + '" text-anchor="end">R ' + f(Ri) + '</text>');
s.push('<text x="' + CX + '" y="' + f(CY + aOut + 20) + '" font-size="5" fill="' + MUT + '" text-anchor="middle">plan — section through a plate</text>');

// ── radial cross-section ──────────────────────────────────────────────────────
var hi = (RING / 2) * S, ho = (RING / 2 + T) * S, mid = (hi + ho) / 2;
s.push('<text x="' + SX + '" y="' + f(SY - ho - 16) + '" font-size="6" font-weight="700" fill="' + INK + '" text-anchor="middle">Radial cross-section</text>');
s.push('<text x="' + SX + '" y="' + f(SY - ho - 8.5) + '" font-size="4.6" fill="' + MUT + '" text-anchor="middle">drawn at ' + S + '× the plan</text>');
s.push('<path d="M' + (SX - ho) + ' ' + (SY - ho) + 'H' + (SX + ho) + 'V' + (SY + ho) + 'H' + (SX - ho) + 'Z ' +
  'M' + (SX - hi) + ' ' + (SY - hi) + 'V' + (SY + hi) + 'H' + (SX + hi) + 'V' + (SY - hi) + 'Z" ' +
  'fill="#e2e8f0" fill-rule="evenodd"/>');
s.push('<rect x="' + (SX - ho) + '" y="' + (SY - ho) + '" width="' + (2 * ho) + '" height="' + (2 * ho) + '" fill="none" stroke="#334155" stroke-width="1.1"/>');
s.push('<rect x="' + (SX - hi) + '" y="' + (SY - hi) + '" width="' + (2 * hi) + '" height="' + (2 * hi) + '" fill="none" stroke="#334155" stroke-width="1.1"/>');
s.push('<g font-size="4.4" fill="' + MUT + '" text-anchor="middle">');
s.push('<text x="' + SX + '" y="' + f(SY - mid + 1.6) + '">' + f(T) + '</text>');
s.push('<text x="' + SX + '" y="' + f(SY + mid + 1.6) + '">' + f(T) + '</text>');
s.push('<text x="' + f(SX - mid) + '" y="' + f(SY + 1.6) + '">' + f(T) + '</text>');
s.push('<text x="' + f(SX + mid) + '" y="' + f(SY + 1.6) + '">' + f(T) + '</text>');
s.push('</g>');
s.push('<text x="' + SX + '" y="' + f(SY + 2) + '" font-size="6.5" font-weight="700" fill="' + INK + '" text-anchor="middle">' + f(RING) + ' × ' + f(RING) + '</text>');
s.push('<g stroke="' + INK + '" stroke-width="0.5" fill="none">');
s.push('<line x1="' + f(SX - hi) + '" y1="' + f(SY + ho + 11) + '" x2="' + f(SX + hi) + '" y2="' + f(SY + ho + 11) + '"/>');
s.push('<line x1="' + f(SX - hi) + '" y1="' + f(SY + ho + 8.5) + '" x2="' + f(SX - hi) + '" y2="' + f(SY + ho + 13.5) + '"/>');
s.push('<line x1="' + f(SX + hi) + '" y1="' + f(SY + ho + 8.5) + '" x2="' + f(SX + hi) + '" y2="' + f(SY + ho + 13.5) + '"/>');
s.push('</g>');
s.push('<text x="' + SX + '" y="' + f(SY + ho + 19) + '" font-size="5.6" font-weight="700" fill="' + INK + '" text-anchor="middle">' + f(RING) + '</text>');
s.push('<text x="' + f(SX - ho - 5) + '" y="' + f(SY - ho - 2) + '" font-size="4.6" fill="' + GREEN + '" text-anchor="end">← bore</text>');
s.push('<text x="' + f(SX + ho + 5) + '" y="' + f(SY - ho - 2) + '" font-size="4.6" fill="' + BLUE + '">outside →</text>');
s.push('<text x="' + SX + '" y="' + f(SY - ho - 2) + '" font-size="4.6" fill="' + MUT + '" text-anchor="middle">plate</text>');
s.push('<text x="' + SX + '" y="' + f(SY + ho + 5.4) + '" font-size="4.6" fill="' + MUT + '" text-anchor="middle">plate</text>');

// ── legend ────────────────────────────────────────────────────────────────────
var LX = 268, LY = SY + ho + 42;
s.push('<text x="' + LX + '" y="' + LY + '" font-size="5.6" font-weight="700" fill="' + INK + '">The four boundaries, outward to inward</text>');
[[aOut, BLUE, 'outer tube, outer face'],
 [aRim, BLUE, 'plate rim = outer octagon, R ' + f(Ro)],
 [aRingIn, GREEN, 'ring inner bound = inner tube, outer face'],
 [aBore, GREEN, 'bore = inner octagon, R ' + f(Ri)]].forEach(function (L, i) {
  var y = LY + 11 + i * 9.6;
  s.push('<rect x="' + LX + '" y="' + f(y - 4) + '" width="9" height="2.4" fill="' + L[1] + '"/>');
  s.push('<text x="' + (LX + 13) + '" y="' + f(y - 1) + '" font-size="5" font-weight="700" fill="' + INK + '">a ' + f(L[0]) + '</text>');
  s.push('<text x="' + (LX + 42) + '" y="' + f(y - 1) + '" font-size="5" fill="' + MUT + '">' + L[2] + '</text>');
  s.push('<text x="' + (LX + 208) + '" y="' + f(y - 1) + '" font-size="5" fill="' + MUT + '" text-anchor="end">flats ' + f(2 * L[0]) + '</text>');
});
s.push('<text x="' + LX + '" y="' + f(LY + 11 + 4 * 9.6 + 3) + '" font-size="4.8" fill="' + MUT + '">' +
  'Only the inner tube\'s wall stands in the ring — the outer tube\'s grows outward, away from it.</text>');
s.push('<text x="' + LX + '" y="' + f(LY + 11 + 4 * 9.6 + 11) + '" font-size="4.8" fill="' + MUT + '">' +
  'So the outline-to-outline gap is ' + f(RING + T) + ', and the clear ring is ' + f(RING) + '.</text>');

s.push('</g></svg>');

require('fs').writeFileSync(__dirname + '/torus-geometry-diagram.svg', s.join('\n') + '\n');

// Pad the three columns to a common width. "R 90" against "R 59.693" is the common
// case and reads badly ragged, and the widths are what the writeup quotes verbatim.
function col(v, w) { var t = String(f(v)); return t + Array(Math.max(1, w - t.length + 1)).join(' '); }
var wR = Math.max(String(f(Ro)).length, String(f(Ri)).length, String(f(Rhole)).length);
var wA = Math.max(String(f(aRim)).length, String(f(aBore)).length);
var wW = Math.max(String(f(aOut)).length, String(f(aRingIn)).length);

console.log('outer octagon  R ' + col(Ro, wR) + '   apothem ' + col(aRim, wA) + '   wall out to ' + col(aOut, wW) + '   (run 1)');
console.log('inner octagon  R ' + col(Ri, wR) + '   apothem ' + col(aBore, wA) + '   wall out to ' + col(aRingIn, wW) + '   (run 2)');
console.log('hole cutter    R ' + col(Rhole, wR) + '   (run 3 — invert this disc)');
console.log('ring           ' + f(aRim) + ' − ' + f(aRingIn) + ' = ' + f(aRim - aRingIn));
console.log('outside flats  ' + f(2 * aOut) + '   bore flats ' + f(2 * aBore));
console.log('wrote torus-geometry-diagram.svg');
