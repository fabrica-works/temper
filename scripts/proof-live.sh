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
store=
if [ "${1-}" = forge-land ]; then
    store=${TEMPER_LIVE_STORE:-$HOME/src/rust/tmp/live-store/forge-land}
    # Reserve the store in the test, atomically, before any box starts. Refuse
    # here too so a rerun needs neither a build nor a new scratch.
    if [ -e "$store" ] || [ -L "$store" ]; then
        printf 'kept store %s exists: reruns on a kept sign-in are not built yet; move it aside to sign in afresh\n' "$store" >&2
        exit 1
    fi
    store_parent=$(dirname -- "$store")
    mkdir -p -- "$store_parent"
    store=$(readlink -f -- "$store")
    export TEMPER_LIVE_STORE=$store
fi
export NEXTEST_DOUBLE_SPAWN=0
heavy cargo nextest run -p temper --profile live --no-run
scratch=$(readlink -f "$(mktemp -d "${TMPDIR:-/tmp}/temper-live.XXXXXX")")
# Every process whose cwd is in this newly created private scratch was started
# by this sitting. As in jig's live-sign-in.sh, stop those owned descendants
# before keeping or erasing their stores, including after an interruption.
stop_owned() {
    [ -d "$scratch" ] || return 0
    owned=
    for process in /proc/[0-9]*; do
        case $(readlink "$process/cwd" 2>/dev/null || true) in
            "$scratch"/*) owned="$owned ${process#/proc/}" ;;
            *)
                if [ -n "$store" ]; then
                    case $(readlink "$process/cwd" 2>/dev/null || true) in
                        "$store"|"$store"/*) owned="$owned ${process#/proc/}" ;;
                    esac
                fi
                ;;
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
}
notice=$scratch/notice
: > "$notice"
chmod 600 "$notice"
export TEMPER_LIVE_NOTICE=$notice
runner=
follower=
finish() {
    status=$?
    trap - EXIT INT TERM HUP
    set +e
    if [ -n "$runner" ]; then
        kill -TERM "$runner" 2>/dev/null
        wait "$runner" 2>/dev/null
    fi
    if [ -n "$follower" ]; then
        kill "$follower" 2>/dev/null
        wait "$follower" 2>/dev/null
    fi
    stop_owned
    printf '%s\n' 'Last live notices:'
    tail -n 30 "$notice"
    keep=${TEMPER_LIVE_KEEP:-$HOME/src/rust/tmp/live-evidence}
    tag=$(sed -n 's/^TAG \([^; ]*\).*$/\1/p' "$notice" | head -n 1 | tr -cd 'A-Za-z0-9_.:-')
    [ -n "$tag" ] || tag=$(date -u +%Y-%m-%dT%H:%M:%SZ)-$$
    if mkdir -p -- "$keep" && chmod 700 "$keep"; then
        # The kept notice never carries the transient CLIENT bearer.
        kept=$keep/$tag.notice
        if sed 's/secret=[^&[:space:]]*/secret=…/g' "$notice" > "$kept" && chmod 600 "$kept"; then
            printf 'notice %s\n' "$kept"
        else
            printf 'could not retain notice in %s\n' "$keep" >&2
            status=1
        fi
    else
        printf 'could not create private evidence directory %s\n' "$keep" >&2
        status=1
    fi
    if [ "$status" -eq 0 ]; then
        rm -rf -- "$scratch"
    else
        # Retain scratch evidence, and redact the transient link there too.
        sed 's/secret=[^&[:space:]]*/secret=…/g' "$notice" > "$scratch/notice.redacted"
        mv -- "$scratch/notice.redacted" "$notice"
        printf 'kept %s; erase it with rm -rf -- %s once read\n' "$scratch" "$scratch"
    fi
    [ -z "$store" ] || printf 'kept store %s\n' "$store"
    exit "$status"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
printf '%s\n' 'Approve the printed ChatGPT address in a browser that reaches this machine.' \
    'For a remote browser: ssh -L 1455:127.0.0.1:1455 -L <client port>:127.0.0.1:<client port> HOST'
TMPDIR=$scratch heavy cargo nextest run -p temper --profile live --run-ignored only --no-capture \
    -E "test(=$test_name)" &
runner=$!
tail -n +1 -f --sleep-interval=0.2 --pid="$runner" "$notice" &
follower=$!
status=0
wait "$runner" || status=$?
runner=
# Let tail flush notices appended just before heavy returned.
wait "$follower" || true
follower=
exit "$status"
