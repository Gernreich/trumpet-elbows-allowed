#!/bin/bash
# Resolve both directories BEFORE moving, because $0 is relative to where the
# script was invoked from and stops resolving the moment we cd anywhere. That
# bug made R empty and the loop read /checks/*.txt off the filesystem root.
HERE="$(cd "$(dirname "$0")" && pwd)"
R="$(cd "$HERE/.." && pwd)"
cd "$HERE/../../../../../../../../tools"
# Resolve the interpreter before the loop and stop if it is missing. Captured
# with 2>&1, a missing interpreter does not fail the run: the shell's error is
# written into the transcript in place of the check, and gen_readme.js -- which
# tallies only transcripts carrying an "N checks, N failed" line -- then reports
# "0 checks across 0 spirals, 0 failed", which reads as a pass. It was hardcoded
# to ~/boxes and the checkout moved to ~/Software/boxes, so this was live.
# bore_split.py searches for the checkout for the same reason; do as it does.
PY="${BORE_PY:-}"
for c in ~/Software/boxes ~/boxes; do
  [ -z "$PY" ] && [ -x "$c/venv/bin/python" ] && PY="$c/venv/bin/python"
done
if [ ! -x "$PY" ]; then
  if [ -n "$BORE_PY" ]; then
    echo "run_checks: BORE_PY is set to $BORE_PY, which is not executable." >&2
  else
    echo "run_checks: no Boxes.py venv found. Looked in ~/Software/boxes, ~/boxes." >&2
    echo "  Set BORE_PY=/path/to/venv/bin/python." >&2
  fi
  exit 1
fi
mkdir -p $R/checks
for f in $R/walks/*.txt; do
  n=$(basename "$f" .txt)
  out=$("$PY" check.py "$(cat "$f")" 2>&1)
  echo "$out" > "$R/checks/$n.txt"
  line=$(echo "$out" | grep -E "[0-9]+ checks, [0-9]+ failed" | tail -1)
  fails=$(echo "$out" | grep -cE "^  .*(fail|FAIL)" || true)
  printf "%-18s %s   (failing rows: %s)\n" "$n" "${line:-NO SUMMARY}" "$fails"
done
