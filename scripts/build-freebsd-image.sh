#!/bin/sh
set -eu

TAG="${1:-dev}"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"
ROOTFS_DIR="$ARTIFACT_DIR/rootfs"
RELEASE_DIR="$ARTIFACT_DIR/release"

mkdir -p "$ROOTFS_DIR" "$RELEASE_DIR"

if [ "$(uname -s)" != "FreeBSD" ]; then
  echo "FreeBSD host required to build bootable images." >&2
  echo "Current host: $(uname -s)." >&2
  echo "Run this script in CI with a FreeBSD runner/vm." >&2
  exit 1
fi

if [ ! -d /usr/src/release ]; then
  echo "Missing /usr/src/release; install FreeBSD src tree first." >&2
  exit 1
fi

echo "==> Installing package set"
while IFS= read -r pkg || [ -n "$pkg" ]; do
  [ -z "$pkg" ] && continue
  pkg install -y "$pkg"
done < "$REPO_ROOT/distribution/packages/base.txt"

echo "==> Staging distribution rootfs"
install -d "$ROOTFS_DIR/etc" "$ROOTFS_DIR/boot" "$ROOTFS_DIR/usr/local/etc"
cp "$REPO_ROOT/distribution/config/etc/rc.conf" "$ROOTFS_DIR/etc/rc.conf"
cp "$REPO_ROOT/distribution/config/etc/loader.conf" "$ROOTFS_DIR/boot/loader.conf"
cp "$REPO_ROOT/distribution/config/etc/sysctl.conf" "$ROOTFS_DIR/etc/sysctl.conf"
cp "$REPO_ROOT/distribution/config/etc/fstab" "$ROOTFS_DIR/etc/fstab"
cp "$REPO_ROOT/distribution/config/usr/local/etc/lightdm.conf" "$ROOTFS_DIR/usr/local/etc/lightdm.conf"

TARGET_ROOT="$ROOTFS_DIR" REPO_ROOT="$REPO_ROOT" "$REPO_ROOT/scripts/configure-ui.sh"

echo "==> Building image files"
ROOTFS_IMAGE="$RELEASE_DIR/pamplemouche-rootfs-${TAG}.ufs"
IMG="$ARTIFACT_DIR/pamplemouche-os-${TAG}.img"
ISO="$ARTIFACT_DIR/pamplemouche-os-${TAG}.iso"
SUM="$ARTIFACT_DIR/pamplemouche-os-${TAG}.sha256"
rm -f "$ROOTFS_IMAGE" "$IMG" "$ISO" "$SUM"

makefs -t ffs -s 2g "$ROOTFS_IMAGE" "$ROOTFS_DIR"
mkimg -s gpt -p freebsd-ufs:="$ROOTFS_IMAGE" -o "$IMG"
makefs -t cd9660 -o rockridge "$ISO" "$ROOTFS_DIR"

echo "==> Writing checksums"
for file in "$IMG" "$ISO"; do
  sha256 -q "$file" | awk -v f="$(basename "$file")" '{print $1 "  " f}' >> "$SUM"
done
