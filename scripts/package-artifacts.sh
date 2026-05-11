#!/bin/sh
set -eu

TAG="${1:-dev}"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"

. "$REPO_ROOT/scripts/common.sh"

SAFE_TAG="$(artifact_tag "$TAG")"

ISO="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.iso"
IMG="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.img"
SUM="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.sha256"

for file in "$ISO" "$IMG" "$SUM"; do
  if [ ! -f "$file" ]; then
    echo "Missing expected artifact: $file" >&2
    exit 1
  fi
done

tar -C "$ARTIFACT_DIR" -czf "$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}-artifacts.tar.gz" \
  "$(basename "$ISO")" "$(basename "$IMG")" "$(basename "$SUM")"

echo "Packaged artifacts at $ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}-artifacts.tar.gz"
