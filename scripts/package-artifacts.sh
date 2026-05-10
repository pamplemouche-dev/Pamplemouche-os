#!/bin/sh
set -eu

TAG="${1:-dev}"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"

ISO="$ARTIFACT_DIR/pamplemouche-os-${TAG}.iso"
IMG="$ARTIFACT_DIR/pamplemouche-os-${TAG}.img"
SUM="$ARTIFACT_DIR/pamplemouche-os-${TAG}.sha256"

for file in "$ISO" "$IMG" "$SUM"; do
  if [ ! -f "$file" ]; then
    echo "Missing expected artifact: $file" >&2
    exit 1
  fi
done

tar -C "$ARTIFACT_DIR" -czf "$ARTIFACT_DIR/pamplemouche-os-${TAG}-artifacts.tar.gz" \
  "$(basename "$ISO")" "$(basename "$IMG")" "$(basename "$SUM")"

echo "Packaged artifacts at $ARTIFACT_DIR/pamplemouche-os-${TAG}-artifacts.tar.gz"
