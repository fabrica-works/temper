# forgejo-17: later

What the design names, or v17 offers, that this plan does not build.

| Item | Design | When |
|---|---|---|
| A failure the runner caused run again through the API (`Write::Rerun`, `reruns`), before a repair | `draft/ci-on-forgejo-17.md`, section 2; `domain/forge.md`, 20.2 | the next plan of CI, once the draft is adopted (its section 7: re-runs first) |
| A stall's reasons: each pending job's state, read when the stall passes | `draft/ci-on-forgejo-17.md`, section 3 | after re-runs, once people run their own runners (`setup.md`) |
| The agent's `job` tool taking a step and a search (`q`, `qi`) | `draft/ci-on-forgejo-17.md`, section 4 | with the next change to temper's host tools |
| Webhooks, hinting CI by v17's `workflow_job_*` deliveries | `forge.md`, section 5 | with the forge's setup (`setup.md`, section 14), whose reconcile makes the webhooks |
| The webhook facts of section 13 ("later, the webhook headers and payloads"); `unchecked webhook-deliveries` stays | `forge.md`, section 13 | with webhooks |
| The fact that a job waiting for a runner has no steps (`waiting-job-has-no-steps`), the conformance holding its runner | `forge.md`, section 13 | with a stall's reasons, which read pending jobs (`draft/ci-on-forgejo-17.md`, section 3) |
| The fake forge's Actions answers compared with v17's recordings, as `testing/temper-fake-git` compares git's (development.md, 5.3: a fake's scenarios run against the real side) | `forge.md`, section 9; `testing.md`, 4.2 | the next plan that changes the fake forge's Actions |
| A later patch of v17.0 (or v17.1) pinned: the constants, the transcripts and the facts | `forge.md`, section 10 | when one is released; no design change |
