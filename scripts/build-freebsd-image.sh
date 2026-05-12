#!/bin/sh
set -eu

TAG="${1:-dev}"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$REPO_ROOT/artifacts}"
ROOTFS_DIR="$ARTIFACT_DIR/rootfs"
RELEASE_DIR="$ARTIFACT_DIR/release"
DISTSETS_DIR="$RELEASE_DIR/distsets"

. "$REPO_ROOT/scripts/common.sh"

SAFE_TAG="$(artifact_tag "$TAG")"

mkdir -p "$RELEASE_DIR" "$DISTSETS_DIR"
rm -rf "$ROOTFS_DIR"
mkdir -p "$ROOTFS_DIR"

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

release_tag="$(freebsd-version -k 2>/dev/null || uname -r)"
release_tag="${release_tag%%-p*}"

case "$release_tag" in
  *-RELEASE) ;;
  *)
    release_tag="$(printf '%s' "$release_tag" | sed 's/-.*//')-RELEASE"
    ;;
esac

arch="$(uname -m)"
dist_base_url="https://download.freebsd.org/releases/$arch/$release_tag"

case "$arch" in
  amd64|i386)
    bios_boot_arch="i386"
    ;;
  *)
    echo "Unsupported architecture for BIOS bootable ISO generation: $arch" >&2
    exit 1
    ;;
esac

fetch_distset() {
  set_name="$1"
  local_set="$DISTSETS_DIR/$set_name"

  if [ -f "$local_set" ] && [ -s "$local_set" ]; then
    return 0
  fi

  if [ -f "/usr/freebsd-dist/$set_name" ] && [ -s "/usr/freebsd-dist/$set_name" ]; then
    cp "/usr/freebsd-dist/$set_name" "$local_set"
    return 0
  fi

  echo "==> Downloading missing $set_name from $dist_base_url"
  fetch -o "$local_set" "$dist_base_url/$set_name" || {
    echo "Unable to retrieve $set_name from local dist sets or FreeBSD mirrors." >&2
    exit 1
  }
}

fetch_distset base.txz
fetch_distset kernel.txz

echo "==> Extracting FreeBSD base system into rootfs"
tar -xpf "$DISTSETS_DIR/base.txz" -C "$ROOTFS_DIR"
tar -xpf "$DISTSETS_DIR/kernel.txz" -C "$ROOTFS_DIR"

echo "==> Staging distribution rootfs"
install -d "$ROOTFS_DIR/etc" "$ROOTFS_DIR/boot" "$ROOTFS_DIR/usr/local/etc"
cp "$REPO_ROOT/distribution/config/etc/rc.conf" "$ROOTFS_DIR/etc/rc.conf"
cp "$REPO_ROOT/distribution/config/etc/loader.conf" "$ROOTFS_DIR/boot/loader.conf"
cp "$REPO_ROOT/distribution/config/etc/sysctl.conf" "$ROOTFS_DIR/etc/sysctl.conf"
cp "$REPO_ROOT/distribution/config/etc/fstab" "$ROOTFS_DIR/etc/fstab"
cp "$REPO_ROOT/distribution/config/usr/local/etc/lightdm.conf" "$ROOTFS_DIR/usr/local/etc/lightdm.conf"

TARGET_ROOT="$ROOTFS_DIR" REPO_ROOT="$REPO_ROOT" "$REPO_ROOT/scripts/configure-ui.sh"

echo "==> Installing package set into rootfs"
ASSUME_ALWAYS_YES=yes pkg -r "$ROOTFS_DIR" bootstrap -f
while IFS= read -r pkg_name || [ -n "$pkg_name" ]; do
  [ -z "$pkg_name" ] && continue
  pkg -r "$ROOTFS_DIR" install -y "$pkg_name" || {
    echo "Failed to install package into rootfs: $pkg_name" >&2
    exit 1
  }
done < "$REPO_ROOT/distribution/packages/base.txt"

echo "==> Building image files"
ROOTFS_IMAGE="$RELEASE_DIR/${ARTIFACT_PREFIX}-rootfs-${SAFE_TAG}.ufs"
IMG="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.img"
ISO="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.iso"
SUM="$ARTIFACT_DIR/${ARTIFACT_PREFIX}-${SAFE_TAG}.sha256"
rm -f "$ROOTFS_IMAGE" "$IMG" "$ISO" "$SUM"

for boot_file in pmbr gptboot cdboot; do
  if [ ! -f "$ROOTFS_DIR/boot/$boot_file" ]; then
    echo "Missing required boot loader file in rootfs: /boot/$boot_file" >&2
    exit 1
  fi
done

makefs -t ffs -s 2g "$ROOTFS_IMAGE" "$ROOTFS_DIR"
mkimg -s gpt \
  -b "$ROOTFS_DIR/boot/pmbr" \
  -p freebsd-boot:="$ROOTFS_DIR/boot/gptboot" \
  -p freebsd-ufs:="$ROOTFS_IMAGE" \
  -o "$IMG"
makefs -t cd9660 \
  -o "rockridge,bootimage=$bios_boot_arch;$ROOTFS_DIR/boot/cdboot,no-emul-boot" \
  "$ISO" "$ROOTFS_DIR"

echo "==> Writing checksums"
TAB="$(printf '\t')"
for file in "$IMG" "$ISO"; do
  printf '%s%s%s\n' "$(sha256 -q "$file")" "$TAB" "$(basename "$file")" >> "$SUM"
done
