#!/bin/sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"

found=0
for checksum in "$ARTIFACT_DIR"/*.sha256; do
  [ -f "$checksum" ] || continue
  found=1
  while IFS="$(printf '\t')" read -r expected filename || [ -n "$expected" ]; do
    [ -z "$expected" ] && continue
    [ -n "$filename" ] || {
      echo "Malformed checksum line in $checksum" >&2
      exit 1
    }

    if command -v sha256sum >/dev/null 2>&1; then
      actual="$(sha256sum "$ARTIFACT_DIR/$filename" | awk '{print $1}')"
    else
      actual="$(sha256 -q "$ARTIFACT_DIR/$filename")"
    fi

    if [ "$actual" != "$expected" ]; then
      echo "Checksum mismatch for $filename" >&2
      exit 1
    fi
  done < "$checksum"
done

if [ "$found" -eq 0 ]; then
  echo "No checksum files found in $ARTIFACT_DIR" >&2
  exit 1
fi
