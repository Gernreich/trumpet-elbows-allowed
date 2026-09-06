#!/bin/bash
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
