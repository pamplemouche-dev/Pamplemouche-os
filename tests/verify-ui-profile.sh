#!/bin/sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

while IFS= read -r relpath || [ -n "$relpath" ]; do
  [ -z "$relpath" ] && continue
  if [ ! -f "$REPO_ROOT/$relpath" ]; then
    echo "Missing UI profile file: $relpath" >&2
    exit 1
  fi
done < "$REPO_ROOT/tests/ui-profile-required-files.txt"

echo "UI profile files are present."
