#!/usr/bin/env sh
set -eu

EXE="${1:-./target/release/qorx}"
BIND="${QORX_BIND:-127.0.0.1:47188}"
TIMEOUT_SECONDS="${QORX_SMOKE_TIMEOUT_SECONDS:-15}"
BASE="http://$BIND"

case "$BIND" in
  0.0.0.0:*) BASE="http://127.0.0.1:${BIND##*:}" ;;
esac

HOME_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qorx-smoke.XXXXXX")"
LOG_FILE="$HOME_DIR/daemon.log"
PID=""

cleanup() {
  if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$HOME_DIR"
}
trap cleanup EXIT INT TERM

QORX_HOME="$HOME_DIR" QORX_BIND="$BIND" "$EXE" daemon >"$LOG_FILE" 2>&1 &
PID="$!"

i=0
while [ "$i" -lt "$TIMEOUT_SECONDS" ]; do
  if curl -fsS "$BASE/health" >/dev/null 2>&1; then
    break
  fi
  i=$((i + 1))
  sleep 1
done

if ! curl -fsS "$BASE/health" >/dev/null; then
  echo "qorx gateway did not become healthy at $BASE" >&2
  cat "$LOG_FILE" >&2
  exit 1
fi

curl -fsS "$BASE/stats" >/dev/null
QORX_HOME="$HOME_DIR" QORX_BIND="$BIND" "$EXE" doctor --json | grep '"gateway_healthy": true' >/dev/null

printf '{"ok":true,"bind":"%s","base":"%s","data_dir":"%s"}\n' "$BIND" "$BASE" "$HOME_DIR"
