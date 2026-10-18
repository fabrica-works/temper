# The forge kit: temper's side

Draft, 2026-10-18. This is temper's side of skein's `draft/forge.md`. It
says:

- what temper hands to skein;
- what temper keeps;
- what its documents become;
- where the connector's top would be cut later;
- the order of the work.

skein's draft gives the crates themselves, the setup's GitHub plan and
the open questions they raise.

## 1. In one page

- **temper keeps the connector, and gives skein what the connector
  stands on.**
  - **To skein:** Forgejo's documents, the call plans, the forge
    component, the forge client, reconcile's machine, the fake forge and
    the conformance (skein's `draft/forge.md`, section 2).
  - **temper keeps:**
    - the connector's top, `change` and `issues`;
    - the root's translation and the store's records;
    - its configuration and the setup's commands;
    - owned mode and the box;
    - its git component and the fake git program.
- **Nothing changes on a forge.** temper gives skein what it uses today:
  - the word `temper`;
  - its user agent;
  - its tokens' names;
  - its keys' namespace.

  So its markers, its bot's mark, its tokens and its keys read as
  before. The client's rows in temper's store keep their shapes.
- **temper's crates depend on skein's** in place of their own.
  Unchanged:
  - the engine's routing;
  - the forge component's place, as a child of the engine's protocol
    layer;
  - the rules for room and the drain, which become the component's own
    and still match jig's components'.
- **The top's seam is recorded, not cut** (section 4). The top moves when
  a second application that works on a forge is designed.
- **The extraction waits for the client-api run.** It is a plan of its
  own, started once that run has ended (section 5). The real-git bridge
  is designed against the fake forge in its new home in skein.

## 2. What moves, and what stays

| temper's | becomes |
|---|---|
| `temper-forge-forgejo` | `skein-forge`: Forgejo's dialect, its pin among it |
| `temper-engine-domain-forge-client` | `skein-forge-client`. Its `api.rs` becomes `skein-forge`'s vocabulary. `EffectPurpose` and its encoding move into the top, which hands the client the bytes |
| `temper-engine-protocol-forge` | `skein-forge-connection`. Its call plans and its marker go to `skein-forge`, and `conventions.rs`'s user agent becomes the routing's value |
| `temper-forge-setup` | Reconcile, its types and its checks become `skein-forge-setup`. The rest stays: the configuration's reading (`jig-protocol-config`), the preparation, `app.ini`, the runner's files, the processes' specs, the configuration's steps and upgrades |
| `testing/temper-fake-forge-domain`, `testing/temper-fake-forge-protocol`, `tests/fake-forge` | skein's fake forge |
| `testing/temper-throwaway-forgejo`, `tools/forgejo-conformance` | skein's conformance. `tools/git-transcripts` stays and uses skein's throwaway |
| `temper-engine-domain-forge`, `-change`, `-issues` | stay |
| `temper-forge-setup-shell`, `temper-box` | stay. The shell hosts skein's reconcile and maps temper's configuration onto its values |
| `testing/temper-fake-git` | stays: it fakes temper's git component's program (`testing.md`, 4.3) |
| `temper-protocol-store`'s forge records | stay |

- **Tests go with what they test.** The component's machine world and
  reconcile's world move. temper's forge connector world, its protocol
  world (now over skein's crates) and its cost checks stay, because they
  run temper's stories.
- **Owned mode reads skein's pin,** as do the throwaway and the
  conformance (`protocol/setup.md`, 6.1).

## 3. What temper's documents become

When skein's draft is promoted (its section 11), temper's documents are
amended in the same pass. A section that moves keeps its number and
heading, with one line naming the skein document and section that now
hold it.

- **`protocol/forge.md`:**
  - **Keeps:** section 2 (the engine's stack, now naming skein's crates,
    and what crosses to the root); section 4's word and namespace;
    section 10's temper worlds and cost checks; section 12; and section
    14's temper items.
  - **Points to skein for:** sections 3, 5, 7 to 9, and 13.
  - **Section 6 is split** by where each rule is kept: a rule kept in
    skein's crates points there, and temper's values stay.
- **`protocol/setup.md`:**
  - **Section 5:** cites skein's reconcile, and keeps temper's inputs
    (which members become which values) and its files.
  - **Section 11:** its crate tree names `skein-forge-setup`.
  - **Section 13:** 13.1 to 13.3 keep what tests temper's host; the rest
    cites skein.
  - **Section 15:** its GitHub item cites skein's `forge-setup.md`.
- **`domain/forge.md`:**
  - **Section 3:** its structure names `skein-forge-client`.
  - **Section 5:** cites skein for the working set, keeping up, fresh
    reads and the budget, and keeps temper's values.
  - **Section 6:** keeps temper's effects, their kinds and their
    recovery classes, and cites skein for the write lanes and keys.
- **`testing.md`:**
  - **Section 4.2:** cites skein's `fake-forge.md`, and keeps what
    temper's worlds add.
  - **Section 2.6:** the throwaway becomes skein's.
  - **Section 7:** its layout drops the moved crates.
- **`README.md`,** section 5: its crates name skein's.
- **`draft/factory.md` and `draft/qa.md`:** these drafts are the user's
  to amend in their next pass. They ask for connector crates shared by
  applications: factory.md, sections 11 and 12, line "connectors: crates
  on jig"; qa.md, section 9. Part of that is answered: what a forge
  connector stands on is skein's, and the top waits (section 4).

## 4. The top's seam, for a later move

Not cut now. A later extraction separates the following in
`temper-engine-domain-forge`.

- **Neutral:**
  - resources and holds (`held.rs`, `resources.rs`);
  - capabilities (`capabilities.rs`);
  - routing to the client, and keeping its rows;
  - the topics' mechanism;
  - most of `domain.rs`.
- **temper's:**
  - `change` and `issues`;
  - the top's `gates.rs`, `settle.rs` and `judge.rs`;
  - `domain.rs`'s change and issue parts;
  - adoption's checks, in `adoption.rs`;
  - `Repository`'s project, home, prefix, CI and checks fields, in
    `boundary.rs`;
  - the effects offered to agents (`describe_agent_effect`);
  - the `Stored`, `Key`, `Event` and `Request` enums, which carry both
    sides and are split along the line above.
- **Its one use of jig:** `jig_domain_people`'s `Outcome`, `Refusal` and
  `Again`, in adoption (`boundary.rs`, `adoption.rs`). A move replaces
  them with the top's own three-way outcome and an opaque refusal, which
  the root maps, as it maps the top's other words: the forge's crates
  name no type of jig's core (`domain/root.md`, section 1).
- **The root's translation** (`temper-engine-domain`'s `route/`) maps the
  top's words onto jig's connector words. Every application writes its
  own.
- **When:** once a second application that works on a forge is
  designed, such as factory.md's lab. Where the top goes is decided
  then (skein's `draft/forge.md`, section 12).

## 5. Order

1. **This design lands,** with skein's draft. Once reviewed, it is
   promoted: into skein's five documents, and into the amendments of
   section 3 here.
2. **The extraction plan** goes in temper's `docs/development/plans/`,
   since temper is the most downstream repository it changes. It starts
   once client-api's run has ended, because that run changes `api.rs`
   and the component. Its increments move one crate at a time, upstream
   first:
   1. `skein-forge`;
   2. the client;
   3. the component;
   4. reconcile;
   5. the fakes and the conformance.

   Each is a skein landing, then a temper repin that deletes temper's
   crate (skein's `development.md`, section 5.7).
3. **The real-git bridge** (`testing.md`, 2.4 and 10) is designed
   against the fake forge in skein, in a design session of its own.
4. **GitHub's dialect and setup** come in a later plan (skein's
   `draft/forge.md`, section 8; `protocol/setup.md`, section 15).

## 6. Open questions

- **The routing's start values.** The component reads no file in
  skein. Default: temper's routing reads the token's and the secret's
  files, as `protocol/configuration.md`, section 6 reads a secret's
  file, and hands over their values.
- **A temper word other than `temper`.** Default: none. A deployment
  that shares a forge with another temper deployment is told apart by
  its keys' namespace, as now.
