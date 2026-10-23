#!/bin/sh
# A person's sitting (domain/testing.md, 2.7): fresh ChatGPT sign-in and real stories.
set -eu
umask 077
cd "$(dirname "$0")/.."
case ${1-} in
    '') test_name=a_real_model_answers_a_chat_and_pushes_a_small_change ;;
    forge-land) test_name=a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge ;;
    *) printf '%s\n' 'usage: scripts/proof-live.sh [forge-land]' >&2; exit 2 ;;
esac
[ "$#" -le 1 ] || { printf '%s\n' 'usage: scripts/proof-live.sh [forge-land]' >&2; exit 2; }
export NEXTEST_DOUBLE_SPAWN=0
heavy cargo nextest run -p temper --profile live --no-run
scratch=$(readlink -f "$(mktemp -d "${TMPDIR:-/tmp}/temper-live.XXXXXX")")
# Every process whose cwd is in this newly created private scratch was started
# by this sitting. As in jig's live-sign-in.sh, stop those owned descendants
# before erasing their stores, including after nextest or its test is interrupted.
erase() {
    [ -d "$scratch" ] || return 0
    owned=
    for process in /proc/[0-9]*; do
        case $(readlink "$process/cwd" 2>/dev/null || true) in
            "$scratch"/*) owned="$owned ${process#/proc/}" ;;
        esac
    done
    for process in $owned; do kill -KILL "$process" 2>/dev/null || true; done
    attempt=0
    while [ "$attempt" -lt 100 ]; do
        left=
        for process in $owned; do
            if kill -0 "$process" 2>/dev/null; then left="$left $process"; fi
        done
        [ -n "$left" ] || break
        owned=$left
        attempt=$((attempt + 1))
        sleep 0.1
    done
    rm -rf -- "$scratch"
}
trap erase EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
printf '%s\n' 'Approve the printed ChatGPT address in a browser that reaches this machine.' \
    'For a remote browser: ssh -L 1455:127.0.0.1:1455 -L <client port>:127.0.0.1:<client port> HOST'
status=0
TMPDIR=$scratch heavy cargo nextest run -p temper --profile live --run-ignored only --no-capture \
    -E "test(=$test_name)" || status=$?
exit "$status"
