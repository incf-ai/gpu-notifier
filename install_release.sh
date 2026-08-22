#!/usr/bin/env bash
set -euo pipefail

# install_release.sh
# Build the release binary, install it into ~/.local/bin, create a default
# config file if missing, and set up a user-level systemd unit file.
# This script can be run multiple times and will overwrite the installed binary
# and the generated systemd unit file.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BINARY_NAME="gpu-notifier"
INSTALL_DIR="$HOME/.local/bin"
INSTALL_PATH="$INSTALL_DIR/$BINARY_NAME"
CONFIG_DIR="$HOME/.config/gpu-notifier"
CONFIG_FILE="$CONFIG_DIR/config.ron"
NOTIFY_AUDIO_SRC="$SCRIPT_DIR/notify.mp3"
NOTIFY_AUDIO_DST="$CONFIG_DIR/notify.mp3"
SYSTEMD_DIR="$HOME/.config/systemd/user"
UNIT_FILE="$SYSTEMD_DIR/$BINARY_NAME.service"
EXAMPLE_CONFIG="config.example.ron"

echo "Building release..."
cargo build --release

if [[ ! -f "target/release/$BINARY_NAME" ]]; then
  echo "Error: release binary not found at target/release/$BINARY_NAME" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
install -Dm755 "target/release/$BINARY_NAME" "$INSTALL_PATH"
echo "Installed $BINARY_NAME to $INSTALL_PATH"

mkdir -p "$CONFIG_DIR"
if [[ -f "$NOTIFY_AUDIO_DST" ]]; then
  echo "Notify audio already exists at $NOTIFY_AUDIO_DST; leaving it intact."
elif [[ -f "$NOTIFY_AUDIO_SRC" ]]; then
  install -Dm644 "$NOTIFY_AUDIO_SRC" "$NOTIFY_AUDIO_DST"
  echo "Copied notify audio to $NOTIFY_AUDIO_DST"
else
  echo "Warning: notify.mp3 not found at $NOTIFY_AUDIO_SRC; default notify command will point to $NOTIFY_AUDIO_DST"
fi

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "Creating default configuration at $CONFIG_FILE"
  if [[ -f "$EXAMPLE_CONFIG" ]]; then
    install -Dm644 "$EXAMPLE_CONFIG" "$CONFIG_FILE"
  else
    cat > "$CONFIG_FILE" <<EOF
GpuNotifier(
    gpus: [
        PerGpuConfig(
            gpu_id: 0,
            enabled: true,
            parser: AmdSmi,
            monitor_interval: Seconds(60.0),
            monitor_command: "amd-smi metric --json",
            monitor_threshold: PowerBelow(50.0),
            monitor_grace: Some(1),
            notify_grace: Some(1),
            notify_command: "ffplay -v 0 -nodisp -autoexit $NOTIFY_AUDIO_DST",
        ),
    ],
)
EOF
  fi
else
  echo "Configuration file already exists at $CONFIG_FILE; leaving it intact."
fi

mkdir -p "$SYSTEMD_DIR"
cat > "$UNIT_FILE" <<EOF
[Unit]
Description=GPU Notifier
After=default.target

[Service]
Type=simple
ExecStart=$INSTALL_PATH --config $CONFIG_FILE
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF
chmod 644 "$UNIT_FILE"
echo "Wrote systemd user unit to $UNIT_FILE"

if command -v systemctl >/dev/null 2>&1; then
  echo "Reloading systemd user daemon..."
  systemctl --user daemon-reload
  echo "Enabling and starting $BINARY_NAME service..."
  systemctl --user enable --now "$BINARY_NAME.service"
  systemctl --user restart "$BINARY_NAME.service"
  echo "Service enabled and started."
else
  echo "Warning: systemctl not found. Unit file created, but could not reload or enable the service."
fi

echo "Installation complete."
