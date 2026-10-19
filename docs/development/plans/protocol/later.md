# protocol: later

What the design names that this plan does not build, and when it should
be done. Most rows are `STATUS.md`'s "Not planned"; the rest are what
the planner's audit found (`findings/README.md`).

| Item | Design | When |
|---|---|---|
| The forge's webhooks: the listener, signatures, hints | `forge.md`, section 5; `configuration.md`, section 3 (`webhooks` refused) | when a deployment's forge can reach the engine; standalone polls |
| Hosts apart: TLS on the engine's listeners, a worker-only process, contained process trees for agents and git | `protocol.md`, section 12, "After standalone"; `git.md`, section 10 | after skein's TLS server side and contained process trees |
| The forge as people's sign-in provider | `credentials.md`, section 6; `domain/people.md`, section 14 | a deployment with more than one person |
| GitHub | `forge.md`, section 11 | a project on GitHub |
| The forge's client documents and panels; the form choosing among configured forges and saying whether the project has a home; an adopted repository named by owner and name | `client.md`, sections 4.2, 11; `web/ux/forge.md` | a plan of its own after this one |
| The browser | `web/architecture.md`; `client.md`, section 11 | jig's browser shell |
| A run's live text in the client | jig's `protocol/channel.md`, 6.4; `llm.md`, section 7 | smith's channel carrying it |
| A failed preparation's words on the task's page | `domain/engine.md`, section 8 | jig's core keeping an attempt's failure detail |
| A resource pattern's words (a grant's) in the forge's view | `client.md`, 4.1 | jig's view giving grant patterns a slot (it prints them itself at temper's pin) |
| jig's charter and tools translations in place of `temper-engine-smith` | `llm.md`, section 7 | the root's tools reconciled with jig's core tool events |
| A calendar for periods (one period per project at the first start today) | `configuration.md`, section 11 | a calendar child designed |
| Splitting the files past 800 lines: `forge_route.rs` (adoption, workspace, change procedure, outputs, effects), the forge's `domain.rs` (adoption, change, release, projection, topics, restart), `root_routes.rs` and `forge_routes.rs` (their harness into the world's `src/`, their sections into files) | code-organization.md, section 8; `findings/temper-engine.md`, section 7 | a plan that changes them next; this plan adds at most 40 lines to each |
| `forge::Role::Adopted`, a role the form never offers | `domain/forge.md`, section 4 | the next change to the forge's roles |
| `Event::Forge` taking any forge event, where only answers and hints come from below | `forge.md`, section 2 | the next change to the root's boundary |
| `PushDiagnostic` defined twice (checkout and worker) | `channel.md`, section 5 | the next change to either |
| The worker creating a missing base branch; a push with no expected head | `domain/worker.md`, sections 11 and 13 | not planned (`STATUS.md`) |
| A capped growing buffer and `ByteRing` in skein's lib | `performance.md`, section 3 | when skein's lib gains them; until then modules of the components |
| `jig-shell-native` saying why it refused a client's directory | `client.md`, 6.1 | a jig change; `temper open` prints the refusal it knows |
| Documents of deleted code: `engine-domain.md`, `agent-domain.md`, `worker-domain.md` | `STATUS.md`, "Not planned" | once the code citing them is repointed |
| temper's `domain/` as an application of jig's | `STATUS.md`, "Not planned" | once jig's boundary settles |
| Performance's measurements: the allocator, the byte budget | `performance.md`, sections 5 and 6 | once `temper serve` runs a real load |
| Repinning jig, smith and skein | `STATUS.md`, "Not planned" | after jig's test-budgets and smith's reliability plans end |
| An adopted repository listed on the project's resources | `client.md`, 4.3 and 11; `domain/testing.md`, 2.6 | after jig's core lists adopted resources, owned included, and temper repins jig (decisions.md, "For the user") |
