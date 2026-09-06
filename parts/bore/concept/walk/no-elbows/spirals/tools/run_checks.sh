#!/bin/bash
# Resolve both directories BEFORE moving, because $0 is relative to where the
# script was invoked from and stops resolving the moment we cd anywhere. That
# bug made R empty and the loop read /checks/*.txt off the filesystem root.
HERE="$(cd "$(dirname "$0")" && pwd)"
R="$(cd "$HERE/.." && pwd)"
cd "$HERE/../../../../../../../tools"
mkdir -p $R/checks
for f in $R/walks/*.txt; do
  n=$(basename "$f" .txt)
  out=$(~/boxes/venv/bin/python check.py "$(cat "$f")" 2>&1)
  echo "$out" > "$R/checks/$n.txt"
  line=$(echo "$out" | grep -E "[0-9]+ checks, [0-9]+ failed" | tail -1)
  fails=$(echo "$out" | grep -cE "^  .*(fail|FAIL)" || true)
  printf "%-18s %s   (failing rows: %s)\n" "$n" "${line:-NO SUMMARY}" "$fails"
done
