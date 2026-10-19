# Who drafted each session file

Experiment 7: each session file's drafter, its reviewer and its
revisions. Codex is gpt-6-astra, run through the director's
`codex-helper.sh` from a prompt that stands alone (draft-common, the
session's own part, and for temper the temper part); Claude is the
planner (Opus 5.5); Sonnet drafts are Claude Sonnet 5.5 sub-agents, from
the same prompt, once the director held Codex to `high` for its budget. The reviews are Sonnet sub-agents, reading against
the design as answered, at skein 064c55f and temper dd12b8e (the code of
skein d120820 and temper 27995fb).

| File | Drafted by | Reviewed by | Revised by | Notes |
|---|---|---|---|---|
| skein/01-forge.md | Codex, xhigh, 10-19 | Sonnet | Codex, xhigh (review, decision 25, answers 7 to 12 and 35; then the re-check); Claude (answer 34 marked; 01.5 moved out, decision 28) | 01.5 became skein 10 |
| skein/02-client.md | Codex, xhigh, 10-19 | Sonnet | Codex, xhigh (review and answers 1, 14, 17; then answers 15 to 22; then the re-check); Claude (02.2 switches the plans' widths, found by temper 02's draft) | |
| skein/03-connection.md | Codex, xhigh, 10-19 | Sonnet, twice | Codex, xhigh (review and answers 1, 13, 14; then answers 2 to 6; then the re-check) | 03.4 and 03.5 moved to skein 08 (decision 23) |
| skein/04-setup.md | Codex, xhigh, 10-19 | Sonnet, twice | Codex, xhigh (review and answer 23, interrupted at capacity and finished by a second run; then answers 24 to 29; then the re-check) | |
| skein/05-fakes.md | Codex, xhigh, 10-19 | Sonnet, twice | Codex, xhigh (review, answers 32 and 33; then the re-check and decision 26); Claude (05.3's Size, tests counted; 05.4 split into 05.4 and 05.5 on the base host, decision 26 revised after the tools plan landed) | |
| skein/06-conformance.md | Codex, xhigh, 10-19 | Sonnet, twice | Codex, xhigh (review, answers 30 and 31; then the re-check, its Sizes left open); Claude (decision 16 aligned; decision 27 and the Sizes) | |
| skein/07-worlds.md | Codex, xhigh, 10-19 | Sonnet, twice | Codex, xhigh (review, decisions 22 and 23; then the re-check); Claude (answer 25 marked; 07.1's Needs on the measure) | the protocol part moved to skein 08 |
| skein/08-machine-world.md | Codex, xhigh, 10-19, from the first drafts' 03.4, 03.5 and 07's protocol part, as one file with 09 | Sonnet, twice | Claude (answer 36); Codex, high (the review; split into 08 and 09); Claude (the re-check's Close table) | decision 23 |
| skein/09-protocol-world.md | as skein 08, from its 08.3 to 08.5 | Sonnet, twice, with 08 | as skein 08; Claude (the re-check's three items) | decision 23 |
| skein/10-plans.md | Sonnet, 10-19, from skein 01's 01.5 (decision 28) | Sonnet | Claude (the log read-through ruled, row 7; the machine world's count; the review's nine items) | |
| temper/01-forge.md | Codex, xhigh, 10-19 | Sonnet | Claude (the review's nine items; the tools' READMEs from temper 06) | |
| temper/02-client.md | Sonnet, 10-19 | Sonnet | Claude (the open question closed by skein 02.2; the counts relative; the review's nine items, the plans' widths repaired in the repin commit) | 02.1 about 680 lines moved (callers rewritten in place count as moved) and 90 new: one increment |
| temper/03-connection.md | Sonnet, 10-19 | Sonnet | Claude (the JSON increments moved to temper 06, decision 27; the counts relative; the joint review's items: the component recording's Proof, the Host claim) | five drafted increments: 03.2 to 03.4 became temper 06.2 to 06.4, 03.5 became 03.2 |
| temper/04-setup.md | Codex, xhigh, 10-19 | Sonnet | Sonnet (the review's nine items, ruled); Claude (the gate counts relative) | three increments after the recount |
| temper/05-fakes.md | Sonnet, 10-19 | Sonnet | Claude (the review's five items; decision 15's simulated address; the counts relative; the tools plan's real loop as landed, decision 26) | |
| temper/06-throwaway.md | Sonnet, 10-19 | Sonnet | Claude (READMEs to temper 01.2; temper 07 after 06.1; the review's ten items; the joint review with temper 03: the recording's checks out of Done when, process Proofs, serde out of a test name) | gained 06.2 to 06.4 from temper 03's draft (decision 27) |
| temper/07-worlds.md | Sonnet, 10-19 | Sonnet | Claude (codecs' fixtures to skein 06.3; temper 05.2 in the README; the review's six items) | |
