#!/bin/sh
set -eu

REPO_ROOT="${REPO_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}"
TARGET_ROOT="${TARGET_ROOT:-/}"

install -d -m 0755 "$TARGET_ROOT/usr/local/bin"
install -d -m 0755 "$TARGET_ROOT/usr/local/etc/xdg/openbox"
install -d -m 0755 "$TARGET_ROOT/usr/local/etc/xdg/tint2"
install -d -m 0755 "$TARGET_ROOT/usr/share/skel"

cat > "$TARGET_ROOT/usr/local/bin/start-pamplemouche-session" <<'SESSION'
#!/bin/sh
set -eu

if [ -x /usr/local/bin/openbox-session ]; then
  exec /usr/local/bin/openbox-session
fi

exec xterm
SESSION

chmod 0755 "$TARGET_ROOT/usr/local/bin/start-pamplemouche-session"
cp "$REPO_ROOT/distribution/ui/openbox/autostart.sh" "$TARGET_ROOT/usr/local/etc/xdg/openbox/autostart"
cp "$REPO_ROOT/distribution/ui/tint2/tint2rc" "$TARGET_ROOT/usr/local/etc/xdg/tint2/tint2rc"
cp "$REPO_ROOT/distribution/ui/skel/.xinitrc" "$TARGET_ROOT/usr/share/skel/.xinitrc"
