#!/bin/sh
# The tiers the gate leaves out that main must keep green: skein's `land`
# runs this after each code landing, through heavy --gate, and a failure
# goes to the director (skein's owners.md, 4.5). Today the process tier.
set -eu
exec "$(dirname "$0")/proof-one-process.sh"
