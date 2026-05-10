#!/bin/sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

required_files="
freebsd/release/release.conf
distribution/packages/base.txt
distribution/config/etc/rc.conf
distribution/config/etc/loader.conf
distribution/config/etc/sysctl.conf
distribution/config/etc/fstab
distribution/config/usr/local/etc/lightdm.conf
distribution/ui/openbox/autostart.sh
distribution/ui/tint2/tint2rc
distribution/ui/skel/.xinitrc
scripts/build-freebsd-image.sh
scripts/configure-ui.sh
scripts/package-artifacts.sh
scripts/smoke-test-artifacts.sh
"

for relpath in $required_files; do
  full="$REPO_ROOT/$relpath"
  if [ ! -f "$full" ]; then
    echo "Missing required file: $relpath" >&2
    exit 1
  fi
done

for script in build-freebsd-image.sh configure-ui.sh package-artifacts.sh smoke-test-artifacts.sh validate-layout.sh; do
  if [ ! -x "$REPO_ROOT/scripts/$script" ]; then
    echo "Script is not executable: scripts/$script" >&2
    exit 1
  fi
  sh -n "$REPO_ROOT/scripts/$script"
done

echo "Layout validation passed."
