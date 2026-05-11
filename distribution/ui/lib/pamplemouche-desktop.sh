#!/bin/sh
set -eu

PAMPLEMOUCHE_DEFAULTS_FILE="${PAMPLEMOUCHE_DEFAULTS_FILE:-/usr/local/etc/pamplemouche-desktop/defaults.conf}"

pamplemouche_settings_dir() {
  printf '%s\n' "${XDG_CONFIG_HOME:-$HOME/.config}/pamplemouche-desktop"
}

pamplemouche_runtime_dir() {
  printf '%s\n' "$(pamplemouche_settings_dir)/runtime"
}

pamplemouche_settings_file() {
  printf '%s\n' "$(pamplemouche_settings_dir)/settings.conf"
}

pamplemouche_set_defaults() {
  DESKTOP_PRESET="ventura-night"
  DESKTOP_ACCENT="blue"
  DESKTOP_DENSITY="comfortable"
  DESKTOP_PANEL_MODE="frosted"
}

pamplemouche_apply_value() {
  key="$1"
  value="$2"

  case "$key" in
    DESKTOP_PRESET)
      case "$value" in
        ventura-night|ventura-light|ventura-sunset) DESKTOP_PRESET="$value" ;;
      esac
      ;;
    DESKTOP_ACCENT)
      case "$value" in
        blue|violet|mint|graphite) DESKTOP_ACCENT="$value" ;;
      esac
      ;;
    DESKTOP_DENSITY)
      case "$value" in
        comfortable|compact) DESKTOP_DENSITY="$value" ;;
      esac
      ;;
    DESKTOP_PANEL_MODE)
      case "$value" in
        frosted|solid) DESKTOP_PANEL_MODE="$value" ;;
      esac
      ;;
  esac
}

pamplemouche_load_file() {
  file_path="$1"
  [ -f "$file_path" ] || return 0

  while IFS='=' read -r raw_key raw_value || [ -n "$raw_key$raw_value" ]; do
    case "$raw_key" in
      ""|\#*) continue ;;
    esac
    key=$(printf '%s' "$raw_key" | tr -d '[:space:]')
    value=$(printf '%s' "$raw_value" | tr -d '[:space:]')
    pamplemouche_apply_value "$key" "$value"
  done < "$file_path"
}

pamplemouche_ensure_settings_file() {
  settings_dir="$(pamplemouche_settings_dir)"
  settings_file="$(pamplemouche_settings_file)"

  mkdir -p "$settings_dir"

  if [ ! -f "$settings_file" ]; then
    cat > "$settings_file" <<EOF
DESKTOP_PRESET=$DESKTOP_PRESET
DESKTOP_ACCENT=$DESKTOP_ACCENT
DESKTOP_DENSITY=$DESKTOP_DENSITY
DESKTOP_PANEL_MODE=$DESKTOP_PANEL_MODE
EOF
  fi
}

pamplemouche_load_settings() {
  pamplemouche_set_defaults
  pamplemouche_load_file "$PAMPLEMOUCHE_DEFAULTS_FILE"
  pamplemouche_ensure_settings_file
  pamplemouche_load_file "$(pamplemouche_settings_file)"
}

pamplemouche_write_settings() {
  settings_dir="$(pamplemouche_settings_dir)"
  settings_file="$(pamplemouche_settings_file)"

  mkdir -p "$settings_dir"
  cat > "$settings_file" <<EOF
DESKTOP_PRESET=$DESKTOP_PRESET
DESKTOP_ACCENT=$DESKTOP_ACCENT
DESKTOP_DENSITY=$DESKTOP_DENSITY
DESKTOP_PANEL_MODE=$DESKTOP_PANEL_MODE
EOF
}

pamplemouche_apply_palette() {
  case "$DESKTOP_PRESET" in
    ventura-light)
      DESKTOP_BACKGROUND_HEX="#dfe8f7"
      DESKTOP_PANEL_HEX="#f8fbff"
      DESKTOP_TEXT_HEX="#1f2937"
      DESKTOP_SUBTLE_HEX="#5b6678"
      ;;
    ventura-sunset)
      DESKTOP_BACKGROUND_HEX="#241827"
      DESKTOP_PANEL_HEX="#2f1f33"
      DESKTOP_TEXT_HEX="#fff4ff"
      DESKTOP_SUBTLE_HEX="#d9b6dc"
      ;;
    *)
      DESKTOP_BACKGROUND_HEX="#0f111a"
      DESKTOP_PANEL_HEX="#161821"
      DESKTOP_TEXT_HEX="#f4f7fb"
      DESKTOP_SUBTLE_HEX="#a3adbd"
      ;;
  esac

  case "$DESKTOP_ACCENT" in
    violet) DESKTOP_ACCENT_HEX="#c084fc" ;;
    mint) DESKTOP_ACCENT_HEX="#5eead4" ;;
    graphite) DESKTOP_ACCENT_HEX="#94a3b8" ;;
    *) DESKTOP_ACCENT_HEX="#7aa2f7" ;;
  esac

  case "$DESKTOP_DENSITY" in
    compact)
      PANEL_HEIGHT="30"
      PANEL_PADDING="8 5 8"
      CLOCK_PADDING="10 4"
      TASK_SIZE="160 20"
      ;;
    *)
      PANEL_HEIGHT="34"
      PANEL_PADDING="10 6 10"
      CLOCK_PADDING="12 4"
      TASK_SIZE="180 24"
      ;;
  esac

  case "$DESKTOP_PANEL_MODE" in
    solid) DESKTOP_PANEL_OPACITY="94" ;;
    *) DESKTOP_PANEL_OPACITY="78" ;;
  esac
}

pamplemouche_prepare_rofi_theme() {
  runtime_dir="$(pamplemouche_runtime_dir)"
  theme_file="$runtime_dir/rofi.rasi"
  mkdir -p "$runtime_dir"
  pamplemouche_render_rofi_theme > "$theme_file"
  printf '%s\n' "$theme_file"
}

pamplemouche_render_tint2() {
  cat <<EOF
panel_items = E:TSC
panel_position = top center horizontal
panel_size = 100% $PANEL_HEIGHT
panel_margin = 0 0
panel_padding = $PANEL_PADDING
panel_background_id = 1
wm_menu = 1

rounded = 12
border_width = 0
background_color = $DESKTOP_PANEL_HEX $DESKTOP_PANEL_OPACITY
border_color = $DESKTOP_PANEL_HEX 0

rounded = 10
border_width = 0
background_color = #ffffff 10
border_color = $DESKTOP_ACCENT_HEX 35

rounded = 10
border_width = 0
background_color = $DESKTOP_ACCENT_HEX 32
border_color = $DESKTOP_ACCENT_HEX 55

font_color = $DESKTOP_TEXT_HEX 100

execp = new
execp_command = printf 'Pamplemouche'
execp_interval = 0
execp_has_icon = 0
execp_cache_icon = 0
execp_continuous = 0
execp_markup = 0
execp_font = Sans 10
execp_font_color = $DESKTOP_TEXT_HEX 100
execp_background_id = 0
execp_padding = 12 4
execp_lclick_command = /usr/local/bin/pamplemouche-launcher

taskbar_mode = multi_desktop
taskbar_padding = 4 0 4
taskbar_background_id = 0
taskbar_active_background_id = 0

task_icon = 1
task_text = 1
task_centered = 1
task_maximum_size = $TASK_SIZE
task_padding = 12 4 12
task_background_id = 2
task_active_background_id = 3
task_font_color = $DESKTOP_SUBTLE_HEX 100
task_active_font_color = $DESKTOP_TEXT_HEX 100
task_icon_asb = 100 0 0

systray_padding = 8 4 8
systray_background_id = 0

time1_format = %a %d %b   %H:%M
time1_font = Sans 10
time1_background_id = 0
time1_font_color = $DESKTOP_TEXT_HEX 100
clock_padding = $CLOCK_PADDING
clock_tooltip = %A %d %B %Y
EOF
}

pamplemouche_render_rofi_theme() {
  cat <<EOF
* {
  bg: $DESKTOP_PANEL_HEX;
  bg-alt: $DESKTOP_BACKGROUND_HEX;
  fg: $DESKTOP_TEXT_HEX;
  accent: $DESKTOP_ACCENT_HEX;
  muted: $DESKTOP_SUBTLE_HEX;
}
window {
  width: 34em;
  padding: 18px;
  border: 2px;
  border-color: @accent;
  border-radius: 18px;
  background-color: @bg;
}
mainbox {
  spacing: 10px;
  children: [ inputbar, listview ];
}
inputbar {
  padding: 10px 12px;
  border: 0;
  border-radius: 12px;
  background-color: @bg-alt;
  text-color: @fg;
}
prompt {
  text-color: @accent;
}
listview {
  lines: 6;
  columns: 1;
  fixed-height: false;
  border: 0;
  spacing: 8px;
  background-color: transparent;
}
element {
  padding: 10px 12px;
  border-radius: 12px;
  background-color: transparent;
  text-color: @fg;
}
element selected {
  background-color: @accent;
  text-color: #101214;
}
element-text {
  vertical-align: 0.5;
}
EOF
}

pamplemouche_preset_label() {
  case "$1" in
    ventura-light) printf '%s\n' "Ventura Light" ;;
    ventura-sunset) printf '%s\n' "Ventura Sunset" ;;
    *) printf '%s\n' "Ventura Night" ;;
  esac
}

pamplemouche_accent_label() {
  case "$1" in
    violet) printf '%s\n' "Violet" ;;
    mint) printf '%s\n' "Menthe" ;;
    graphite) printf '%s\n' "Graphite" ;;
    *) printf '%s\n' "Bleu" ;;
  esac
}

pamplemouche_density_label() {
  case "$1" in
    compact) printf '%s\n' "Compact" ;;
    *) printf '%s\n' "Confort" ;;
  esac
}

pamplemouche_panel_mode_label() {
  case "$1" in
    solid) printf '%s\n' "Opaque" ;;
    *) printf '%s\n' "Effet givré" ;;
  esac
}

pamplemouche_stop_pidfile() {
  pid_file="$1"
  [ -f "$pid_file" ] || return 0
  pid=$(cat "$pid_file")
  case "$pid" in
    ''|*[!0-9]*) return 0 ;;
  esac
  kill "$pid" >/dev/null 2>&1 || true
  rm -f "$pid_file"
}

pamplemouche_reload_session() {
  runtime_dir="$(pamplemouche_runtime_dir)"
  mkdir -p "$runtime_dir"
  pamplemouche_stop_pidfile "$runtime_dir/tint2.pid"
  pamplemouche_stop_pidfile "$runtime_dir/plank.pid"
  /usr/local/bin/pamplemouche-session-init >/dev/null 2>&1 &
}
