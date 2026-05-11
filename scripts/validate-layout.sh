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
distribution/ui/applications/pamplemouche-control-center.desktop
distribution/ui/applications/pamplemouche-launcher.desktop
distribution/ui/bin/pamplemouche-control-center
distribution/ui/bin/pamplemouche-launcher
distribution/ui/bin/pamplemouche-session-init
distribution/ui/lib/pamplemouche-desktop.sh
distribution/ui/openbox/autostart.sh
distribution/ui/openbox/menu.xml
distribution/ui/openbox/rc.xml
distribution/ui/pamplemouche/defaults.conf
distribution/ui/tint2/tint2rc
distribution/ui/skel/.xinitrc
scripts/common.sh
scripts/build-freebsd-image.sh
scripts/configure-ui.sh
scripts/package-artifacts.sh
scripts/smoke-test-artifacts.sh
tests/verify-ui-profile.sh
"

for relpath in $required_files; do
  full="$REPO_ROOT/$relpath"
  if [ ! -f "$full" ]; then
    echo "Missing required file: $relpath" >&2
    exit 1
  fi
done

for script in "$REPO_ROOT"/scripts/*.sh "$REPO_ROOT"/tests/*.sh; do
  if [ ! -x "$script" ]; then
    echo "Script is not executable: ${script#"$REPO_ROOT/"}" >&2
    exit 1
  fi
  sh -n "$script"
done

echo "Layout validation passed."
