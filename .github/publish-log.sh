#!/usr/bin/env bash
# Turn a cargo log into check-run annotations and a step summary.
# Annotations are readable through the public API without signing in to
# GitHub; raw job logs and artifacts are not.
# Usage: publish-log.sh <logfile> <title> <exit-status>
set -u
log="$1"; title="$2"; status="$3"

# Strip ANSI colour codes first so the filters below see plain text.
sed 's/\[[0-9;]*[A-Za-z]//g' "$log" > plain.txt
grep -v -E '^\s*(Compiling|Checking|Downloaded|Downloading|Updating|Locking|Adding|Finished|Running|Blocking|Fresh|Documenting|Installing|Installed|Unpacking|Removing)' plain.txt > filtered.txt || true

encode() { sed 's/%/%25/g' "$1" | tr -d '\r' | sed ':a;N;$!ba;s/\n/%0A/g'; }

{ echo "### $title"; echo '```'; tail -n 200 filtered.txt; echo '```'; } >> "${GITHUB_STEP_SUMMARY:-/dev/null}"

if [ "$status" != "0" ]; then
  # Keep the lines that carry diagnostics, then chunk them into annotations.
  grep -E '^(error|warning)|^\s+-->|^\s*[0-9]+ \||^\s*\||^\s*=|^\s*test .*(FAILED|panicked)|^thread|panicked|assertion|^Diff in|^\+|^-' filtered.txt | head -c 24000 > diag.txt || true
  [ -s diag.txt ] || head -c 24000 filtered.txt > diag.txt
  rm -f chunk_*; split -b 3000 -d diag.txt chunk_
  n=0
  for c in chunk_*; do
    n=$((n+1)); [ "$n" -gt 8 ] && break
    echo "::error title=$title (part $n)::$(encode "$c")"
  done
else
  tail -n 30 filtered.txt | head -c 3000 > ok.txt
  echo "::notice title=$title::$(encode ok.txt)"
fi
