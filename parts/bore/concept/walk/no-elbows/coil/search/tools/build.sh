#!/bin/bash
# NOTE, 2026-09-10: README.md and index.html were deleted on 2026-09-05 along with
# every other README under trumpet/, pending one new writeup for the trumpet as a
# whole, and running this script resurrected them unasked. Both are committed again
# as of today, so it no longer surprises anyone -- and index.html is tracked rather
# than left untracked on the floor by the next run.
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
