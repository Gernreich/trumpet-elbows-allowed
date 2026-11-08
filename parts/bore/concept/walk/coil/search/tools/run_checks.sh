#!/bin/bash
# Resolve both directories BEFORE moving, because $0 is relative to where the
# script was invoked from and stops resolving the moment we cd anywhere. That
# bug made R empty and the loop read /checks/*.txt off the filesystem root.
HERE="$(cd "$(dirname "$0")" && pwd)"
R="$(cd "$HERE/.." && pwd)"
cd "$HERE/../../../../../../../tools"
# The checker is the trumpet crate's `check` (check.py until 2026-11-08, which
# needed the Boxes.py venv). Resolve it before the loop and stop if it is
# missing: captured with 2>&1, a missing program does not fail the run -- the
# shell's error is written into the transcript in place of the check, and
# gen_readme.rs, which tallies only transcripts carrying an "N checks, N failed"
# line, then reports "0 checks across 0 spirals, 0 failed", which reads as a pass.
CHECK="$(pwd)/../target/release/check"
if [ ! -x "$CHECK" ]; then
  echo "run_checks: $CHECK is not built. Run cargo build --release in the repository root." >&2
  exit 1
fi
mkdir -p $R/checks
# Every walk's check at once, then the summary in walk order. Each run reads
# its own walk and writes only its own transcript, so they need not wait: one
# after another they were 130 of all-gates.sh's 708 seconds. `wait` holds the
# summary until every transcript is complete.
for f in $R/walks/*.txt; do
  n=$(basename "$f" .txt)
  ( out=$("$CHECK" "$(cat "$f")" 2>&1); echo "$out" > "$R/checks/$n.txt" ) &
done
wait
for f in $R/walks/*.txt; do
  n=$(basename "$f" .txt)
  out=$(cat "$R/checks/$n.txt")
  line=$(echo "$out" | grep -E "[0-9]+ checks, [0-9]+ failed" | tail -1)
  fails=$(echo "$out" | grep -cE "^  .*(fail|FAIL)" || true)
  printf "%-18s %s   (failing rows: %s)\n" "$n" "${line:-NO SUMMARY}" "$fails"
done
