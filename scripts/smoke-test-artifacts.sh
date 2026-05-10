#!/bin/sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"

found=0
for checksum in "$ARTIFACT_DIR"/*.sha256; do
  [ -f "$checksum" ] || continue
  found=1
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$ARTIFACT_DIR" && sha256sum -c "$(basename "$checksum")")
  else
    while IFS='  ' read -r expected filename || [ -n "$expected" ]; do
      [ -z "$expected" ] && continue
      actual="$(sha256 -q "$ARTIFACT_DIR/$filename")"
      if [ "$actual" != "$expected" ]; then
        echo "Checksum mismatch for $filename" >&2
        exit 1
      fi
    done < "$checksum"
  fi
done

if [ "$found" -eq 0 ]; then
  echo "No checksum files found in $ARTIFACT_DIR" >&2
  exit 1
fi
