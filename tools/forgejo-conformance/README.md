# Throwaway Forgejo conformance

The Rust tool starts only the checked Forgejo v16.0.5 binary named by
`TEMPER_FORGEJO`, with fresh SQLite state on loopback. It creates the
fixture and removes its instance and scratch directory when it ends.

```sh
TEMPER_FORGEJO=<absolute path to Forgejo v16.0.5> heavy --gate \
  cargo run -p forgejo-conformance -- \
  --scratch target/forgejo-conformance --out tools/forgejo-conformance
```

Scratch must be empty or absent, outside `/tmp`. The binary's first version
line and SHA-256 are kept in `transcripts/binary.txt`.

Each case is `transcripts/<case>/{request,response}`: raw HTTP/1.1 bytes.
Authorization is `token MASKED`. Responses are unframed, with chunking
removed and Content-Length recomputed. JSON key order, whitespace and types
remain as received. Cases replace their own directory only.

One map masks object ids and RFC3339 times by first appearance across the
run, preserving id width and time offset. Ports become 3000; Date and
Last-Modified become 2020-01-01. Numeric resource ids remain stable because
provisioning has a fixed order. Every output is checked for all fixture
secrets before it is written, and response bodies are also checked before
masking. A leak reports the case and secret kind without printing its value.

The same run writes `facts.txt`, with provenance followed by `holds`,
`fails` or `unchecked` lines. Actions execution follows in 05.3b; webhooks
are left unchecked until webhooks are built. The committed facts retain the
full observed Forgejo/runner capture; the runner implementation follows.
