#!/bin/sh
# Borrowed login, 25 completions at most, 15 minutes at most.
set -eu
cd "$(dirname "$0")/.."
: "${TEMPER_RUN_LOGS:?the run's log directory}"
umask 077
mkdir -p "$TEMPER_RUN_LOGS"
trap 'rm -f "$TEMPER_RUN_LOGS/live-account.json"' EXIT
export NEXTEST_DOUBLE_SPAWN=0
cargo build -p temper --bin temper -p live-login --bin live-login
set --
[ -n "${TEMPER_LIVE_MODEL:-}" ] || set -- --models "$HOME/src/rust/smith/benchmarks/agents/models.toml"
target/debug/live-login "$HOME/.codex/auth.json" "$TEMPER_RUN_LOGS" "$@"
started=$(date +%s)
status=0
timeout --signal=TERM --kill-after=10s 900s \
    cargo nextest run -p temper --profile live --success-output immediate || status=$?
echo "proof-live: wall=$(( $(date +%s) - started ))s exit=$status"
exit "$status"
