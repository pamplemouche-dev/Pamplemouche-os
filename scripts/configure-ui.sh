#!/bin/sh
set -eu

REPO_ROOT="${REPO_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}"
TARGET_ROOT="${TARGET_ROOT:-/}"

install -d -m 0755 "$TARGET_ROOT/usr/local/bin"
install -d -m 0755 "$TARGET_ROOT/usr/local/libexec"
install -d -m 0755 "$TARGET_ROOT/usr/local/etc/xdg/openbox"
install -d -m 0755 "$TARGET_ROOT/usr/local/etc/xdg/tint2"
install -d -m 0755 "$TARGET_ROOT/usr/local/etc/pamplemouche-desktop"
install -d -m 0755 "$TARGET_ROOT/usr/local/share/applications"
install -d -m 0755 "$TARGET_ROOT/usr/share/skel"

cat > "$TARGET_ROOT/usr/local/bin/start-pamplemouche-session" <<'SESSION'
#!/bin/sh
set -eu

if [ -x /usr/local/bin/openbox-session ]; then
  exec /usr/local/bin/openbox-session
fi

if command -v xterm >/dev/null 2>&1; then
  exec xterm
fi

exec /bin/sh
SESSION

chmod 0755 "$TARGET_ROOT/usr/local/bin/start-pamplemouche-session"
cp "$REPO_ROOT/distribution/ui/bin/pamplemouche-session-init" "$TARGET_ROOT/usr/local/bin/pamplemouche-session-init"
cp "$REPO_ROOT/distribution/ui/bin/pamplemouche-launcher" "$TARGET_ROOT/usr/local/bin/pamplemouche-launcher"
cp "$REPO_ROOT/distribution/ui/bin/pamplemouche-control-center" "$TARGET_ROOT/usr/local/bin/pamplemouche-control-center"
chmod 0755 \
  "$TARGET_ROOT/usr/local/bin/pamplemouche-session-init" \
  "$TARGET_ROOT/usr/local/bin/pamplemouche-launcher" \
  "$TARGET_ROOT/usr/local/bin/pamplemouche-control-center"
cp "$REPO_ROOT/distribution/ui/lib/pamplemouche-desktop.sh" "$TARGET_ROOT/usr/local/libexec/pamplemouche-desktop.sh"
chmod 0755 "$TARGET_ROOT/usr/local/libexec/pamplemouche-desktop.sh"
cp "$REPO_ROOT/distribution/ui/pamplemouche/defaults.conf" "$TARGET_ROOT/usr/local/etc/pamplemouche-desktop/defaults.conf"
cp "$REPO_ROOT/distribution/ui/applications/pamplemouche-control-center.desktop" \
  "$TARGET_ROOT/usr/local/share/applications/pamplemouche-control-center.desktop"
cp "$REPO_ROOT/distribution/ui/applications/pamplemouche-launcher.desktop" \
  "$TARGET_ROOT/usr/local/share/applications/pamplemouche-launcher.desktop"
cp "$REPO_ROOT/distribution/ui/openbox/autostart.sh" "$TARGET_ROOT/usr/local/etc/xdg/openbox/autostart"
cp "$REPO_ROOT/distribution/ui/openbox/rc.xml" "$TARGET_ROOT/usr/local/etc/xdg/openbox/rc.xml"
cp "$REPO_ROOT/distribution/ui/openbox/menu.xml" "$TARGET_ROOT/usr/local/etc/xdg/openbox/menu.xml"
cp "$REPO_ROOT/distribution/ui/tint2/tint2rc" "$TARGET_ROOT/usr/local/etc/xdg/tint2/tint2rc"
cp "$REPO_ROOT/distribution/ui/skel/.xinitrc" "$TARGET_ROOT/usr/share/skel/.xinitrc"
