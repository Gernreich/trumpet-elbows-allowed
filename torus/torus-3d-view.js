#!/usr/bin/env node
'use strict';
// =====================================================================
// octagonal torus -> three-quarter view SVG
// =====================================================================
//
//   node torus-3d-view.js [R_outer] [S] [t] [out.svg]
//   node torus-3d-view.js 90 25 3 torus-3quarter-view.svg
//
// THIS IS A DRAWING, NOT A PHOTOGRAPH and not a cut file. It shows what the
// finished object looks like, computed from the same three numbers Part 11
// takes, so it cannot drift from the geometry the rest of the repo describes.
//
// WHAT IT DRAWS
//   The FINISHED surfaces, not the nominal octagons the generator is fed:
//
//     outside apothem = R_outer*cos(pi/n) + t     the outer wall's outer face
//     bore apothem    = R_inner*cos(pi/n)         the inner wall's inner face
//     R_inner         = R_outer - (S + t)*sec(pi/n)
//     height          = S + 2t                    channel plus both plates
//
//   At 90 / 25 / 3 that is 172.298 across the outside flats and 110.298 across
//   the bore, which is what Part 11's script prints, and an external section of
//   31 x 31 -- the 25 channel plus a 3mm plate top and bottom.
//
//   Assembled, so no joinery is drawn: finger joints are inside the corners and
//   invisible from outside. The object is a closed box-section ring.
//
// HOW IT DRAWS
//   Solid faces, back-face culled, painter-sorted far to near. No ray tracing
//   and no anti-aliasing tricks -- flat Lambert shading per face, which is
//   enough to read the form and keeps the file small and inspectable.
//
const fs = require('fs');

const num = (v, d) => (v === undefined ? d : Number(v));
const argv = process.argv.slice(2);
if (argv.some(a => a === '--help' || a === '-h')) {
  console.log(`
torus-3d-view.js -- three-quarter view of the finished torus

  node torus-3d-view.js [R_outer] [S] [t] [out.svg]

    R_outer  outer octagon radius, corner to centre   (default 90)
    S        the square channel, radial and tall      (default 25)
    t        material thickness                       (default 3)
    out.svg  output path                              (default torus-3quarter-view.svg)

  Draws the finished object from the same three numbers Part 11 takes.
  It is a drawing, not a photograph, and not a cut file.
`);
  process.exit(0);
}

const R_outer = num(argv[0], 90);
const S       = num(argv[1], 25);
const t       = num(argv[2], 3);
const OUT     = argv[3] || 'torus-3quarter-view.svg';
const n       = 8;                       // octagons only, as everywhere else here

const sec = 1 / Math.cos(Math.PI / n);
const cos = Math.cos(Math.PI / n);

const R_inner = R_outer - (S + t) * sec;
if (R_inner <= 0) {
  console.error(`No inner tube: R_inner would be ${R_inner.toFixed(3)}mm. ` +
                `Raise R_outer above ${((S + t) * sec).toFixed(3)}.`);
  process.exit(1);
}
const apOut  = R_outer * cos + t;        // outside surface, apothem
const apBore = R_inner * cos;            // bore surface, apothem
const H      = S + 2 * t;                // external height
const Rout   = apOut * sec;              // outside surface, corner to centre
const Rbore  = apBore * sec;             // bore surface, corner to centre

// ---------------------------------------------------------------- view
const AZ = 34 * Math.PI / 180;           // yaw: enough to show two outer faces
const EL = 26 * Math.PI / 180;           // tilt: enough to see into the bore
const ca = Math.cos(AZ), sa = Math.sin(AZ), ce = Math.cos(EL), se = Math.sin(EL);
const dir   = [ce * ca, ce * sa, se];                 // camera -> origin
const right = [-sa, ca, 0];
const up    = [-ca * se, -sa * se, ce];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const project = p => [dot(p, right), -dot(p, up)];    // SVG y grows downward
const depth   = p => dot(p, dir);                     // larger = nearer

// ---------------------------------------------------------------- solid
// Vertex k of a flat-front octagon. Half-step so a face, not a corner, faces us.
const ring = (R, z) => Array.from({ length: n }, (_, k) => {
  const a = (k + 0.5) * 2 * Math.PI / n;
  return [R * Math.cos(a), R * Math.sin(a), z];
});
const oTop = ring(Rout, H),  oBot = ring(Rout, 0);
const iTop = ring(Rbore, H), iBot = ring(Rbore, 0);

const faces = [];
for (let k = 0; k < n; k++) {
  const j = (k + 1) % n;
  faces.push([oTop[k], oTop[j], iTop[j], iTop[k]]);   // top plate
  faces.push([iBot[k], iBot[j], oBot[j], oBot[k]]);   // bottom plate
  faces.push([oBot[k], oBot[j], oTop[j], oTop[k]]);   // outer wall
  faces.push([iBot[j], iBot[k], iTop[k], iTop[j]]);   // bore wall
}

// ---------------------------------------------------------------- shading
const sub  = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = v => { const m = Math.hypot(...v); return m ? v.map(c => c / m) : v; };
const LIGHT = norm([-0.45, -0.35, 0.82]);            // high, front-left

const GOLD = [0xc9, 0xa2, 0x27];
const shade = i => '#' + GOLD.map(c => {
  const v = Math.round(Math.min(255, Math.max(0, c * i)));
  return v.toString(16).padStart(2, '0');
}).join('');

const drawn = [];
for (const f of faces) {
  const nrm = norm(cross(sub(f[1], f[0]), sub(f[2], f[0])));
  if (dot(nrm, dir) <= 0.001) continue;              // back-face cull
  const i = 0.52 + 0.48 * Math.max(0, dot(nrm, LIGHT));
  drawn.push({ f, i, z: f.reduce((s, p) => s + depth(p), 0) / f.length });
}
drawn.sort((a, b) => a.z - b.z);                     // far first

// ---------------------------------------------------------------- emit
const pts = drawn.flatMap(d => d.f.map(project));
const xs = pts.map(p => p[0]), ys = pts.map(p => p[1]);
const PAD = 10;
const x0 = Math.min(...xs) - PAD, y0 = Math.min(...ys) - PAD;
const w  = Math.max(...xs) - Math.min(...xs) + 2 * PAD;
const h  = Math.max(...ys) - Math.min(...ys) + 2 * PAD;

// Each facet gets a slightly darker edge. Without it, two adjacent faces at a
// similar angle to the light merge into one plane and the octagon reads as a
// circle -- which is the one thing this drawing exists to show.
const body = drawn.map(d => {
  const p = d.f.map(project).map(q => `${(q[0] - x0).toFixed(2)},${(q[1] - y0).toFixed(2)}`).join(' ');
  return `    <polygon points="${p}" fill="${shade(d.i)}" stroke="${shade(d.i * 0.82)}" stroke-width="0.6"/>`;
}).join('\n');

// A soft shadow under the object, so it sits on a surface rather than floating.
// Purely presentational: it is the outer octagon flattened onto the ground plane.
const shadowPts = oBot.map(v => project([v[0], v[1], 0]))
  .map(q => `${(q[0] - x0).toFixed(2)},${(q[1] - y0 + 2.5).toFixed(2)}`).join(' ');
const shadow = `  <polygon points="${shadowPts}" fill="#000" opacity="0.09"/>`;

// md2html inlines an SVG figure and drops the markdown alt text with it, so the
// text alternative has to travel inside the file. role + aria-label is what the
// geometry diagram in this repo uses; title and desc are kept for anything that
// prefers them.
const alt = `Three-quarter view of the finished octagonal torus: a closed eight-sided ring of ` +
  `square section, ${(2 * apOut).toFixed(3)}mm across the outside flats and ${(2 * apBore).toFixed(3)}mm ` +
  `across the bore, standing ${H}mm tall. Drawn from the measurements, not photographed.`;

const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w.toFixed(2)} ${h.toFixed(2)}"
     width="${w.toFixed(2)}mm" height="${h.toFixed(2)}mm"
     role="img" aria-label="${alt}">
  <title>Octagonal torus, three-quarter view — R ${R_outer}, ${S}×${S} section, ${t}mm material</title>
  <desc>A computed drawing of the finished object, not a photograph and not a cut file.
Outside ${(2 * apOut).toFixed(3)}mm across the flats, bore ${(2 * apBore).toFixed(3)}mm, external
section ${H}×${H}mm. Generated by torus-3d-view.js from R_outer=${R_outer} S=${S} t=${t}.</desc>
  <rect x="0" y="0" width="${w.toFixed(2)}" height="${h.toFixed(2)}" fill="#faf7f0"/>
${shadow}
  <g stroke-linejoin="round">
${body}
  </g>
</svg>
`;
fs.writeFileSync(OUT, svg);

console.log(`outside flats  ${(2 * apOut).toFixed(3)}   bore flats ${(2 * apBore).toFixed(3)}`);
console.log(`section        ${H} x ${H}   (channel ${S} + 2 x ${t}mm)`);
console.log(`corner radii   outside ${Rout.toFixed(3)}   bore ${Rbore.toFixed(3)}`);
console.log(`faces drawn    ${drawn.length} of ${faces.length} (rest culled)`);
console.log(`wrote ${OUT}`);
