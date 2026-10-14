# LLM calls

Provisional, 2026-10-03, revised 2026-10-14. Where temper's LLM calls
are made, and what temper adds around them: none of the calls, since
temper's agents are smith's processes on the worker, which make their
own; the endpoints, models and accounts, which are jig's configuration on
the engine and smith's on the worker; the host tools the engine serves,
whose declarations, decoding and rendering are temper's; and the charter
and a run's result, which `temper-engine-smith` translates. LLM accounts
are credentials.md's (section 3). What is open is listed in section 7.

## 1. In one page

- **temper's engine makes no LLM call.** Its runs are on workers, jig's
  third shape (jig's `protocol/README.md`, section 4), each in an agent
  process: `temper agent`, smith's agent shell, which calls its providers
  over skein's LLM client (smith's `protocol/llm.md`; skein's `llm.md`).
  jig's calls in a host's own process (jig's `protocol/llm.md`, sections
  1 to 4) are not used.
- **Providers are smith's and skein's.** temper keeps no provider's
  dialect, identity or request shape, and no fake provider of its own:
  the tiers use skein's fake LLM peer.
- **Endpoints, models and accounts are configuration,** jig's `llm`
  member on the engine and smith's agent configuration on the worker, by
  the same names (section 3).
- **Accounts are jig's,** granted to runs with their assignments
  (credentials.md, section 3).
- **Host tools are temper's:** the engine's tools and the forge's reads
  and effects, declared, decoded and answered by `temper-engine-smith`,
  which the engine's routing calls after the fleet has fenced each call
  (jig's `protocol/channel.md`, 6.3). Nothing is parsed in a domain.
- **The charter and a run's result are temper's translation:**
  `temper-engine-smith` writes smith's charter from the root's assignment,
  and reads a run's result into the root's ending of its task (section
  5).
- **One translation.** The engine's protocol layer is the only place where
  temper's charter, tools and messages become smith's, and smith's calls
  become the engine's (`domain/agent.md`, section 1).

## 2. Where LLM calls are made

```
engine   no LLM call: accounts and grants, host tools, charters, results
worker   no LLM call: grants put in smith's form for smith's host half
agent    every LLM call: smith's agent shell over skein's LLM client
```

- **The agent process** is spawned by the worker's protocol layer for
  each run: temper's binary as `temper agent AGENT.json`
  (configuration.md, section 5), with an environment holding no
  credential. It reads smith's agent configuration, dials the endpoints
  it declares, and takes its run's grants over smith's channel from
  smith's host half (smith's `protocol/hosts.md`, section 2).
- **smith decides what a call does:** its session retries a failed
  completion, and its run holds the budget, reserving each completion's
  most before making it (jig's `protocol/llm.md`, section 1).
- **Unauthorized and exhausted** come back from the agent as notices
  that reach jig's accounts child, fenced by attempt (credentials.md,
  section 3).
- **Agents in a host's own process** (`smith-inline-agent`), in the
  engine or on a worker, are not used (section 7).

## 3. Endpoints, models and accounts

- **On the engine, jig's `engine.llm`** (jig's `protocol/configuration.md`,
  section 3; jig's `protocol/llm.md`, sections 4 to 6): endpoints, each
  with the account its grants are named by; models, each with its
  endpoint, window, output and prices; accounts. The engine uses the
  models and their prices, which its charters carry, and the endpoints'
  names and accounts, which its charters and grants name. It dials no
  endpoint: their addresses and transports serve the agents.
- **On the worker, smith's agent configuration** (smith's
  `protocol/agent.md`, section 4), named by jig's worker part
  (configuration.md, section 5): the endpoints the agents dial, each with
  its address, transport, dialect, headers and options, and the models
  they serve, with their windows and outputs. The worker's routing
  decodes each charter against these endpoints (jig's
  `protocol/channel.md`, 6.1), and its hello states the agent's bounds,
  which the fleet checks against the engine's charters (jig's
  `domain/engine.md`, section 8).
- **The same names on both,** checked at start: in standalone one
  document names both, so `temper serve` refuses a start whose engine
  names an endpoint, an account or a model the agent's configuration does
  not declare the same way (configuration.md, section 5).
- **Which providers:** those smith's dialects speak over skein's LLM
  client. The process tier gives the agents skein's fake LLM peer as a
  process on loopback; the live tier, a real model through a borrowed
  Codex login (testing.md, 2.6 and 2.7).

## 4. Host tools

- **The vocabulary is the root's `Tool`:** the core's tools (`delegate`,
  `amend`, `message`, `cancel`, `release`, `introduce`, `subscribe`,
  `unsubscribe`, `propose`, `decide`, `decide_escalation`, `withdraw`,
  `note`, `recall`) and the forge's (one read per tool, `read_items` to
  `read_permission`, decoded as `Tool::ReadForge`; `effect_forge`;
  `subscribe_forge`). `temper-engine-smith` declares each with its JSON
  Schema, its description and its effect, a read or a write
  (`tools`), and the charter carries the declarations (section 5).
- **Decoded in the routing** (jig's `protocol/channel.md`, 6.3), in the
  step that routes the call, keeping no state:
  1. the fleet fences the call, and the root hands it out as
     `HostRequest::Decode { to, task, attempt, call }`;
  2. the routing decodes the call's name with smith's channel codec
     (`smith_channel::CallName`), whose activation must be the attempt,
     and its input with `temper_engine_smith::call`;
  3. it hands the typed call back as `WorkerInput::DecodedHostCall { to,
     body }`.
- **A call that does not decode** is answered at once as an error that
  names what is wrong (its `temper_engine_smith::Problem`: malformed,
  too large, a member missing, of the wrong type, out of range, an
  unknown tool), and decides nothing: the routing renders the error as
  smith's text and hands it to the root as `WorkerInput::RenderedHostCall
  { to, answer }`, with the answer marked an error, and the root relays
  it to the run with no commit (`domain/agent.md`, 4.3).
- **Answered in the routing:** once a decided call's commit is durable,
  the root hands its answer out (`HostDelivery::Render`); the routing
  renders it with `temper_engine_smith::answer`, as smith's text with
  whether it is an error, and hands it back as
  `WorkerInput::RenderedHostCall { to, answer }`. An answer that cannot
  be rendered becomes a bounded error, so no decided call is left
  unanswered.
- **Not `jig-protocol-tools`.** temper's root takes its own `Tool`
  vocabulary, not the core's tool events, so `temper-engine-smith` stays
  (section 7).

## 5. The charter and a run's result

- **The charter.** As the routing encodes an assignment,
  `temper_engine_smith::start` writes smith's charter from the root's
  `Assignment` (`domain/agent.md`, section 4: the instructions, the
  brief's sections, the host tools, the result contract, the budget, the
  models and their prices), and smith's codec encodes it
  (`smith_protocol_channel::encode_charter`) against the engine's
  endpoints, in the version its release's smith writes. It crosses as
  smith's charter bytes in jig's assignment, and the worker's routing
  decodes it as the agent starts (jig's `protocol/channel.md`, 6.1). A
  resumed run's charter resumes when its policy allows and the task has
  kept turns.
- **Turns** cross and are kept as smith's bytes; the core reads only what
  jig's records carry beside them (jig's `protocol/channel.md`, 6.1).
- **A run's result.** Once the fleet has fenced an ended answer, the root
  hands its result out as `WorkerRequest::ReadResult { payload, outcome
  }`; the routing decodes smith's result
  (`smith_protocol_channel::decode_declared`), translates it with
  `temper_engine_smith::result` into the root's ending of the task, and
  hands it back as `WorkerInput::ResultRead { payload, end }`. A change's
  resource comes from what the worker and the forge settled, never from
  the LLM's fields. A result that does not decode fails its attempt as
  invalid (jig's `protocol/channel.md`, 6.1).
- **Not `jig-protocol-charter`,** for the same reason as the tools
  (section 4).

## 6. Testing

- **Step tests of `temper-engine-smith`:** each tool's schema against its
  decoder, each `Problem`, each answer's rendering, the charter encoded
  by smith's codec and decoded back, each result's translation.
- **Step tests of the routing:** a call decoded and handed back; a call
  that does not decode answered with no commit; a result read.
- **The one-process world:** smith's agent processes over skein's fake
  LLM on skein's fake machine, calling host tools through bytes
  (testing.md, 2.5).
- **The process tier:** skein's fake LLM peer as a process on loopback;
  **the live tier:** a real model, at most 25 completions and 15 minutes
  in all, the run stopped past either (testing.md, 2.6 and 2.7).
- **No token across a domain boundary** (credentials.md, section 7).

## 7. Open questions

- **jig's host tools' crates** (`jig-protocol-charter`,
  `jig-protocol-tools`). temper's root takes its own `Tool` vocabulary.
  Default: keep `temper-engine-smith` until the root's tools are next
  reconciled with jig's.
- **A run's live text** on a conversation's page: smith's channel carries
  facts without content, so agent processes give no text until it does
  (jig's `protocol/channel.md`, 6.4). Default: a run is shown by its
  activity, and its words as each turn commits.
- **Agents in the worker's process,** `smith-inline-agent`, for one
  person's machine, with no containment and no process per run (jig's
  `protocol/service.md`, section 6). Default: agent processes.
