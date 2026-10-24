# S: the free NOTICE smoke

S runs temper's NOTICE story on skein's lab runner, with no driver or model.
The subject supplies `temper` and the same binary named `git`. The cluster is
forge → TLS fake LLM → engine → two remote workers; its documents equal
`../cluster/` byte for byte. The existing runtime LLM adapter keeps the root
chat open after its delegate settles.

The runner sends the engine's private stdout directly to the `sign-in` seed
command, once, without retaining the link or any sign-in stream. Public
readiness still captures all three engine listeners. The remaining seed adds
the fake model key, waits for usable accounts and the connected forge, and
adopts `fake/widget`. The story asks for NOTICE through `tools/ctl`, requires
the coding delegate's historical successful change result, and reads a
complete root tree with no held or live delegates. It has one 220 s deadline;
seed waits allow 200 s within the class's 240 s command bound.

After the story, the live `forge` collection retains observations and C/P
object responses; the file-only `end-state` check judges them. After all
process exits and empty trees are confirmed, stopped collections copy the
forge's observations, then the engine store. The stopped `promises` check
uses those copies and temper's independent decision records. See
[checks/README.md](checks/README.md).

With the owner's subject digest:

```sh
L=/path/to/skein/scripts/lab/lab
S=sha256:...
$L validate lab/chat-change-smoke --subject "$S"
$L run lab/chat-change-smoke --subject "$S" --seed 1
$L run lab/chat-change-smoke --subject "$S" --seed 2
$L run lab/chat-change-smoke --subject "$S" --seed 3
$L judge lab/chat-change-smoke --subject "$S"
```

The cost class stays `temper-m1-calibration`. Each changed experiment has a
new key; a pass needs three seeds. Records and raw evidence live under the
lab's `keys/chat-change-smoke/<key>/runs/<run>/`.
