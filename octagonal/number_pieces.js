// Engrave a number on every piece in BuildA1_90_25.svg.
//
//   node number_pieces.js --dry-run          say what it would engrave
//   node number_pieces.js                    write the numbers in
//   node number_pieces.js --clear            take them back out
//
// The sheet holds eighteen pieces whose differences are small and
// consequential: the outer panels are 73.326 and 71.568mm wide and the inner
// ones 50.130 and 48.372mm, so two parts belonging at opposite ends of the
// torus differ by 1.76mm. Off the bed they are a pile of near-identical
// rectangles.
//
// BuildA1_90_25.svg is authored, not generated, so this only ever ADDS one
// group of blue polylines and never reads back into a cut path. The group
// carries an id and a re-run replaces it, so numbers cannot stack.
//
// It borrows verify_torus.js's reader rather than parsing paths again. That
// reader is transform-aware and samples arcs; a second, simpler one would
// disagree with the verifier about where the parts are, and the disagreement
// would show up as a number engraved off the edge of a piece.
var fs = require('fs');
var SRC = fs.readFileSync(__dirname + '/verify_torus.js', 'utf8');
eval(SRC.slice(0, SRC.indexOf('var file = process.argv[2];')));

var GID = 'piece-numbers';
var ENGRAVE = '#0000ff';     // blue engraves; it cuts in no repository here
var H = 6.0;                 // glyph height, mm

// The hex glyphs the bore sections and the bell rings use, so one number reads
// the same way wherever it appears on the instrument. Unit box, y up.
var GLYPH = {
 '0':[[[0.5,1],[0.85,0.8],[0.85,0.2],[0.5,0],[0.15,0.2],[0.15,0.8],[0.5,1]]],
 '1':[[[0.3,0.78],[0.52,1],[0.52,0]],[[0.28,0],[0.78,0]]],
 '2':[[[0.1,0.78],[0.3,1],[0.7,1],[0.9,0.78],[0.9,0.6],[0.1,0],[0.9,0]]],
 '3':[[[0.1,1],[0.9,1],[0.45,0.55]],[[0.45,0.55],[0.9,0.55],[0.9,0.16],[0.72,0],[0.28,0],[0.1,0.16]]],
 '4':[[[0.7,0],[0.7,1],[0.12,0.32],[0.92,0.32]]],
 '5':[[[0.85,1],[0.2,1],[0.15,0.55],[0.5,0.62],[0.8,0.5],[0.88,0.28],[0.75,0.06],[0.4,0],[0.15,0.12]]],
 '6':[[[0.82,0.92],[0.55,1],[0.25,0.85],[0.15,0.45],[0.15,0.18],[0.35,0],[0.62,0],[0.85,0.18],[0.85,0.38],[0.62,0.55],[0.3,0.55],[0.15,0.45]]],
 '7':[[[0.12,1],[0.9,1],[0.42,0]]],
 '8':[[[0.5,0.55],[0.22,0.68],[0.22,0.87],[0.5,1],[0.78,0.87],[0.78,0.68],[0.5,0.55],[0.18,0.4],[0.18,0.14],[0.5,0],[0.82,0.14],[0.82,0.4],[0.5,0.55]]],
 '9':[[[0.18,0.08],[0.45,0],[0.75,0.15],[0.85,0.55],[0.85,0.82],[0.65,1],[0.38,1],[0.15,0.82],[0.15,0.62],[0.38,0.45],[0.7,0.45],[0.85,0.55]]],
 'A':[[[0.1,0],[0.5,1],[0.9,0]],[[0.26,0.4],[0.74,0.4]]],
 'B':[[[0.15,0],[0.15,1],[0.68,1],[0.88,0.83],[0.88,0.68],[0.68,0.55],[0.15,0.55]],[[0.15,0.55],[0.72,0.55],[0.9,0.4],[0.9,0.16],[0.7,0],[0.15,0]]],
 'C':[[[0.9,0.8],[0.7,1],[0.3,1],[0.1,0.8],[0.1,0.2],[0.3,0],[0.7,0],[0.9,0.2]]],
 'D':[[[0.15,0],[0.15,1],[0.58,1],[0.88,0.74],[0.88,0.26],[0.58,0],[0.15,0]]],
 'E':[[[0.9,1],[0.15,1],[0.15,0],[0.9,0]],[[0.15,0.5],[0.68,0.5]]],
 'F':[[[0.88,1],[0.15,1],[0.15,0]],[[0.15,0.52],[0.68,0.52]]]
};

// One glyph, top-left at (x, y). SVG y runs down, the table's y runs up.
function digit(ch, x, y, w, h) {
  return GLYPH[ch].map(function (st) {
    return 'M ' + st.map(function (p) {
      return (x + p[0]*w).toFixed(3) + ',' + (y + (1 - p[1])*h).toFixed(3);
    }).join(' L ');
  });
}

// The glyphs for `text` centred on (cx, cy), plus the baseline tick that says
// which way up the number is -- 6 and 9 are one shape turned over.
function marks(text, cx, cy, h) {
  var w = h * 0.62, gap = h * 0.18;
  var total = text.length * w + (text.length - 1) * gap;
  var x = cx - total / 2, out = [];
  for (var i = 0; i < text.length; i++) {
    out = out.concat(digit(text[i], x, cy - h / 2, w, h));
    x += w + gap;
  }
  var tg = h * 0.16, tl = h * 0.22, right = cx + total / 2, base = cy + h / 2;
  out.push('M ' + (right + tg).toFixed(3) + ',' + base.toFixed(3) +
           ' L ' + (right + tg + tl).toFixed(3) + ',' + base.toFixed(3));
  return out;
}

// Where the crossings of a horizontal line fall inside a closed contour.
function crossings(pts, y) {
  var xs = [], n = pts.length;
  for (var i = 0; i < n; i++) {
    var a = pts[i], b = pts[(i + 1) % n];
    if ((a[1] <= y && y < b[1]) || (b[1] <= y && y < a[1]))
      xs.push(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]));
  }
  return xs.sort(function (p, q) { return p - q; });
}

// Is the point inside this closed contour? Even-odd, which is what a single
// ring answers correctly whether it winds one way or the other.
function inside(pts, x, y) {
  var n = pts.length, c = false;
  for (var i = 0, j = n - 1; i < n; j = i++) {
    var a = pts[i], b = pts[j];
    if ((a[1] > y) !== (b[1] > y) &&
        x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]) c = !c;
  }
  return c;
}

// Distance from a point to the nearest edge of a contour.
function clearance(pts, x, y) {
  var n = pts.length, best = Infinity;
  for (var i = 0; i < n; i++) {
    var a = pts[i], b = pts[(i + 1) % n];
    var dx = b[0] - a[0], dy = b[1] - a[1], L = dx*dx + dy*dy;
    var t = L ? Math.max(0, Math.min(1, ((x-a[0])*dx + (y-a[1])*dy) / L)) : 0;
    var d = Math.hypot(x - (a[0] + t*dx), y - (a[1] + t*dy));
    if (d < best) best = d;
  }
  return best;
}

// Where to engrave a piece's number: the point on its material furthest from
// any edge.
//
// The bounding-box centre and the centroid both fail on the same part. A plate
// is a ring; both land in its hole, and its hole has eight panels nested in it,
// so a number placed there would be engraved on waste that gets cut away and
// on parts it does not belong to. So: sample the piece, keep the points that
// are inside it and inside nothing that sits within it, and take the one with
// the most room around it.
function anchor(pts, obstacles) {
  var xs = pts.map(function (p) { return p[0]; });
  var ys = pts.map(function (p) { return p[1]; });
  var x0 = Math.min.apply(null, xs), x1 = Math.max.apply(null, xs);
  var y0 = Math.min.apply(null, ys), y1 = Math.max.apply(null, ys);
  var near = obstacles.filter(function (o) {
    return o !== pts && o.some(function (q) { return inside(pts, q[0], q[1]); });
  });
  var best = null, N = 90;
  for (var i = 1; i < N; i++) {
    for (var j = 1; j < N; j++) {
      var x = x0 + (x1 - x0) * i / N, y = y0 + (y1 - y0) * j / N;
      if (!inside(pts, x, y)) continue;
      if (near.some(function (o) { return inside(o, x, y); })) continue;
      var room = clearance(pts, x, y);
      for (var k = 0; k < near.length; k++)
        room = Math.min(room, clearance(near[k], x, y));
      if (!best || room > best.room) best = { x: x, y: y, room: room };
    }
  }
  return best;
}

function main() {
  var args = process.argv.slice(2);
  var dry = args.indexOf('--dry-run') >= 0, clear = args.indexOf('--clear') >= 0;
  var file = args.filter(function (a) { return a[0] !== '-'; })[0]
             || __dirname + '/BuildA1_90_25.svg';
  var src = fs.readFileSync(file, 'utf8');

  // drop any group this script wrote before, so a re-run replaces the numbers
  var had = new RegExp('\\n?\\s*<g id="' + GID + '"[\\s\\S]*?</g>', 'g');
  var stripped = src.replace(had, '');
  if (clear) {
    fs.writeFileSync(file, stripped);
    console.log('  numbers removed from ' + file.split('/').pop());
    return 0;
  }

  // read the stripped text, so this script never numbers its own output
  var tmp = file + '.reading';
  fs.writeFileSync(tmp, stripped);
  var P;
  try { P = collect(tmp); } finally { fs.unlinkSync(tmp); }

  // A plate's hole is a contour, not a piece: it is the cut that makes the
  // plate a ring, and there is nothing to pick up and label. verify_torus.js
  // already knows how to tell one from a part, so ask it rather than guessing
  // from colour -- the hole is orange here, but colour is the cut order and
  // could be re-staged without the geometry moving.
  var plates = P.filter(function (p) { return p.w > 160 && Math.abs(p.w - p.h) < 1; });
  var holes = [];
  plates.forEach(function (pl) {
    holePartsFor(pl, P).forEach(function (h) {
      if (holes.indexOf(h) < 0) holes.push(h);
    });
  });
  if (plates.length !== 2 || holes.length !== 2) {
    console.log('  ! expected 2 plates and 2 hole contours, found ' +
                plates.length + ' and ' + holes.length + '. Nothing written.');
    return 1;
  }
  var all = P.map(function (p) { return p.pts; });
  P = P.filter(function (p) { return holes.indexOf(p) < 0; });

  if (P.length !== 18) {
    console.log('  ! expected 18 pieces, found ' + P.length + '. Nothing written.');
    console.log('    This script knows one sheet. If the drawing has changed,');
    console.log('    read it before trusting anything here.');
    return 1;
  }

  var place = P.map(function (p) {
    var near = all.filter(function (o) {
      return o !== p.pts && o.some(function (q) { return inside(p.pts, q[0], q[1]); });
    });
    var xs = p.pts.map(function (q) { return q[0]; });
    var ys = p.pts.map(function (q) { return q[1]; });
    return { col: p.col, id: p.id, w: p.w, h: p.h, pts: p.pts, near: near,
             x0: Math.min.apply(null, xs), y0: Math.min.apply(null, ys),
             a: anchor(p.pts, all) };
  });
  // Reading order across the sheet -- bands, then left to right, because that
  // is the order the parts come off the bed.
  //
  // Ordered by where the PIECE sits, not where its number sits. A plate is
  // 172mm tall and its number lands wherever the annulus has most room, so
  // sorting on the label put the two plates in whichever band that happened to
  // fall in -- a stable-looking order that would move if the anchor moved.
  var band = 40;
  place.sort(function (u, v) {
    var bu = Math.round(u.y0 / band), bv = Math.round(v.y0 / band);
    return bu !== bv ? bu - bv : u.x0 - v.x0;
  });

  var paths = [], checked = 0;
  console.log('  ' + place.length + ' pieces, numbered in reading order\n');
  console.log('   #   colour     size            room   anchor');
  for (var i = 0; i < place.length; i++) {
    var q = place[i], text = (i + 1).toString(16).toUpperCase();
    // room is the distance to the nearest edge, so the number has to fit
    // inside a circle of that radius, with a millimetre left over
    var wide = text.length * H * 0.62 + (text.length - 1) * H * 0.18 + H * 0.38;
    var need = Math.hypot(wide, H) / 2 + 1.0;
    if (q.a.room < need) {
      console.log('  ! piece ' + text + ' has ' + q.a.room.toFixed(1) +
                  'mm to the nearest edge and the number needs ' +
                  need.toFixed(1) + 'mm. Nothing written.');
      return 1;
    }
    console.log('  ' + text.padStart(2) + '   ' + q.col.padEnd(9) + ' ' +
                (q.w.toFixed(2) + ' x ' + q.h.toFixed(2)).padEnd(16) +
                q.a.room.toFixed(1).padStart(5) + 'mm  (' +
                q.a.x.toFixed(1) + ', ' + q.a.y.toFixed(1) + ')');
    // Every point of the number has to land on this piece's material. The
    // anchor being inside is not the same claim as the glyphs being inside,
    // and a number half off the edge is the exact failure this whole file
    // exists to prevent.
    var glyphs = marks(text, q.a.x, q.a.y, H);
    var pts = [];
    glyphs.forEach(function (d) {
      d.replace(/(-?[\d.]+),(-?[\d.]+)/g, function (_, a, b) {
        pts.push([parseFloat(a), parseFloat(b)]); return _;
      });
    });
    var off = pts.filter(function (t) {
      return !inside(q.pts, t[0], t[1]) ||
             q.near.some(function (o) { return inside(o, t[0], t[1]); });
    });
    if (off.length) {
      console.log('  ! ' + off.length + ' of ' + pts.length + ' points of ' +
                  'number ' + text + ' fall off the piece. Nothing written.');
      return 1;
    }
    checked += pts.length;
    glyphs.forEach(function (d) { paths.push(d); });
  }
  // A check that looked at nothing would print the same clean run as a check
  // that looked at everything, so say how much it looked at.
  console.log('\n  ' + checked + ' engraved points, all on their own piece');
  if (args.indexOf('--table') >= 0) {
    // the README's key, computed here so it cannot drift from the sheet
    var tally = {};
    console.log('\n| # | piece | size | cut stage |');
    console.log('|---|---|---|---|');
    place.forEach(function (q, i) {
      var stage = { '#00ff00': 'green — nested, cut first',
                    '#00ffff': 'cyan — on the open sheet',
                    'black': 'black — frees the plate' }[q.col] || q.col;
      // sizeOf, not the bounding box: two panels are nested at 45 degrees and
      // their bounding boxes are squares of 51.882 and 66.527mm, which are not
      // dimensions of any part on this sheet. minRect gives the part.
      var sz = sizeOf({ pts: q.pts }), w = sz.w, h = sz.h;
      var what = w > 160 ? 'plate'
               : (Math.abs(w - 73.326) < 0.1 ? 'outer panel, wide'
               : (Math.abs(w - 71.568) < 0.1 ? 'outer panel, narrow'
               : (Math.abs(w - 50.130) < 0.1 ? 'inner panel, wide'
               : (Math.abs(w - 48.372) < 0.1 ? 'inner panel, narrow' : 'panel'))));
      tally[what] = (tally[what] || 0) + 1;
      console.log('| `' + (i + 1).toString(16).toUpperCase() + '` | ' + what +
                  ' | ' + w.toFixed(3) + ' × ' + h.toFixed(3) + 'mm | ' +
                  stage + ' |');
    });
    // The README says 2 plates, 8 outer panels and 8 inner panels. A table
    // that quietly classified two of them as 'panel' would still print
    // eighteen rows and look complete, so count the kinds and say so.
    var want = { 'plate': 2, 'outer panel, wide': 4, 'outer panel, narrow': 4,
                 'inner panel, wide': 4, 'inner panel, narrow': 4 };
    var bad = Object.keys(want).filter(function (k) { return tally[k] !== want[k]; })
              .concat(Object.keys(tally).filter(function (k) { return !(k in want); }));
    console.log('\n  ' + Object.keys(tally).sort().map(function (k) {
      return k + ' x' + tally[k];
    }).join(', '));
    if (bad.length) {
      console.log('  !! wrong inventory: ' + bad.join('; '));
      return 1;
    }
    console.log('  2 plates, 8 outer panels, 8 inner panels \u2713');
    return 0;
  }
  if (dry) return 0;

  var g = '<g id="' + GID + '" style="fill:none;stroke:' + ENGRAVE +
          ';stroke-width:0.2">\n' +
          paths.map(function (d) {
            return '  <path style="fill:none;stroke:' + ENGRAVE +
                   ';stroke-width:0.2" d="' + d + '"/>';
          }).join('\n') + '\n</g>\n';
  var at = stripped.lastIndexOf('</svg>');
  fs.writeFileSync(file, stripped.slice(0, at) + g + stripped.slice(at));
  console.log('\n  wrote ' + file.split('/').pop() + '  (' + paths.length +
              ' engraved paths in one group)');
  return 0;
}

process.exit(main());
