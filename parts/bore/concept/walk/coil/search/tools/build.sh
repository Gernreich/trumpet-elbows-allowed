#!/bin/bash
# NOTE, 2026-09-10: README.md and index.html were deleted on 2026-09-05 along with
# every other README in the trumpet repository, pending one new writeup for the trumpet as a
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
MD2HTML="${MD2HTML:-../../../../../../../lasermade-tools/md2html.py}"

# The tools are Rust since 2026-11-02 (they were Node). Built first, every time, so
# what runs is the code as it stands and never a stale binary. cargo is looked for on
# PATH and then where rustup puts it, since a non-interactive shell may not have it.
CARGO="${CARGO:-$(command -v cargo || echo "$HOME/.cargo/bin/cargo")}"
"$CARGO" build -q --release --locked

target/release/parts > /dev/null     # piece counts and distinct shapes
target/release/gen_scoring           # SCORING.md
target/release/gen_readme            # README.md
python3 "$MD2HTML" README.md  index.html
python3 "$MD2HTML" SCORING.md SCORING.html
echo "built: README.md SCORING.md index.html SCORING.html"
