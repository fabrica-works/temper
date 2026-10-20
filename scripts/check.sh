#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then both test suites in full: the default one, then the fuzzy
# one. skein's `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

if grep -rnE '\b(task|watching|join|number|delegate|digest)\([0-9]+\)|calls\(\)\[[0-9]+\]' tests/real-loop/tests; then
    echo 'A setup written once, and rows carry data: name real-loop call positions.' >&2
    exit 1
fi

if grep -rnE 'assert_eq!\(\([a-z_]+\.exit, [a-z_]+\.exit|\.len\(\), [0-9]+[,)]' tests/real-loop/tests; then
    echo 'One behaviour per test: assert the act, and name real-loop sizes.' >&2
    exit 1
fi

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
# The browser's client crates (client/browser.md, 4.3).
cargo clippy --target wasm32-unknown-unknown \
    -p temper-client-domain -p temper-client-view \
    -p temper-client-protocol-wire -p temper-client-shell-web -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo nextest run --workspace
cargo nextest run --workspace --profile fuzzy
# The browser bundle, built once by the gate (client/browser.md, 4.3).
scripts/bundle.sh target/temper-bundle >/dev/null
# The browser tier, Chromium against temper serve on that bundle
# (client/browser.md, 4.3).
cargo build -p temper
TEMPER_TEST_BUNDLE="$(pwd)/target/temper-bundle"
export TEMPER_TEST_BUNDLE
cargo nextest run --workspace --profile browser
