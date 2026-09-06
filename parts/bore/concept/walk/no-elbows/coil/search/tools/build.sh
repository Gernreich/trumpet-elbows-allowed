#!/bin/bash
# NOTE, 2026-09-05: this script writes README.md and index.html, and both were
# deleted along with every other README under trumpet/, pending one new writeup
# for the trumpet as a whole. Running it resurrects the spirals pair. That is
# harmless if you want them back and a surprise if you do not, so it is said
# here rather than discovered.
#
# Regenerate everything, in dependency order. index.html and SCORING.html are
# committed rather than built on the server, so they go stale silently unless
# this is run after every edit -- which is the whole reason it is one script and
# not four commands to remember.
set -e
cd "$(dirname "$0")/.."
MD2HTML="${MD2HTML:-../../../../../../../../lasermade-tools/md2html.py}"

node tools/parts.js > /dev/null      # piece counts and distinct shapes
node tools/gen_scoring.js            # SCORING.md
node tools/gen_readme.js             # README.md
python3 "$MD2HTML" README.md  index.html
python3 "$MD2HTML" SCORING.md SCORING.html
echo "built: README.md SCORING.md index.html SCORING.html"
