# The wire and the client at the base (steps 3 and 4)

The code audit of jig at `a3b53546`, 2026-10-15, by a read-only
sub-agent, from source and grep (the graph was stale for 5 of 8 files).

## 1. The wire

- 801 types: `crates/jig-protocol-client-wire/schema/client-v7.schema`
  (615 records, 125 enums; `family jig-client-wire 7` at line 1) and
  `client-v7-part1.schema` (44 + 17). `VERSION: u32 = 7` at
  `src/lib.rs:12`, copied by hand at
  `crates/jig-domain-client/src/contract/versions.rs:11`
  (`CLIENT_VERSION = 7`).
- The 51 that go, by name (STATUS's 41 + 10):
  - asks (15): StartChat, Write, Close, Disposal, DisposalCancel,
    DisposalTakeOver, TakeOver, SetGoal, GoalExecutor{,Charter,Procedure},
    Prioritise, Priority, SeenReplies, NoteScopeGoal (schema 230-274);
  - outcomes and refusals (10): OwnerOutcome{Closing,GoalStarted,
    GoalProposed,Prioritised}, Made{Chat,Words,Goal}, DisposalIssue,
    RefusalDisposal, LimitConversationWords;
  - lists (9): Read{Conversations,Goals}, SubjectGoals,
    ConversationLine, ConversationPage, Conversations, GoalLine,
    GoalPage, Goals;
  - `me` and inbox (7): Charter, OfferedCharters, BoundsGoal,
    GoalProcedure, ReplyPosition, OwnerInboxObjectReply,
    OwnerHistoryChangePrioritised;
  - the small store (10, part1 49-57 and 70): Place, Draft, JigRecord,
    Address, Address{Inbox,Conversations,Task,Project}, ProjectSection,
    TaskSection.
  - Write, MadeWords and SeenReplies become Message, its outcome and
    Seen.
- What replaces them: Create, Propose, Withdraw, Message, Move, Decide,
  Answer, Seen and `Compound`; one task query whose rows are the task
  line (today's `TaskLine`, schema:609, 5 fields); `me` with merged
  templates and creating bounds; a typed version-not-served answer
  (`Answer`, schema:7, has none); places and drafts in the reference
  UI's own codec.
- Production files naming the shape records: jig-domain-client 33,
  jig-view 15, the link 9, engine-clients 12, jig-scripted-engine 7,
  tests/client 24. Step 3 cannot land without editing the client side.

## 2. `jig-domain-client`

18,803 production lines, 13,242 test lines.

- **Client half (about 8,800):** contract.rs and contract/ 5,198 (the
  shape asks at contract/asks.rs:121-153 go); frame 117 (becomes `me`);
  signin 90; reads 108; watches 52; versions 47; link 11; outcome 50;
  facts 21 (operator facts, not the design's `facts` helper); boundary
  107 (rewritten); limits 138 (split); conversation 1,566 (becomes the
  `thread` helper); the watch spine, domain.rs:830-1167.
- **Page half (about 9,900):** pages 5,042; objects 1,506; confirm 676;
  application 532; windows 459; acts 443; intents 330 (view.md, 18.4);
  drafts 294; notices 148; store 139; fields 120; action 90; address
  77; clock 67; the rest of domain.rs.
- **Goes:** pages/conversations.rs 515, pages/project/goals.rs 335 and
  owner_goals.rs 77 (rebuilt on task queries and `shapes`); the
  by-value hand-offs (boundary.rs:38, 61, 100-104; domain.rs:813);
  `Navigation.home` (pages/navigation.rs:33); `Confirmed` and
  `Retention` (boundary.rs:96, 98) become internal to the client.
- **Where the page half reads the client half:** contract types in 32
  page files; the engine-watch slab (application.rs:73,
  pages/shown.rs:148-156 and 226, task.rs:414, project.rs:243,
  replies.rs:15, routing.rs:58-64); the reads slab tagged by page
  generation (routing.rs:78-107, objects.rs:655-741,
  objects/answer.rs:58-105, task.rs:452-576, project.rs:142); the frame
  document (routing.rs:135-179); the link, confirmed party and
  `blocked` (application.rs:16, shown.rs:166, confirm.rs:141 and 660,
  routing.rs:95 and 159); domain functions (close_watch, signed_out,
  foreign_kept_requests: application.rs:76 and 149, objects.rs:782,
  shown.rs:230, routing.rs:160 and 524); the conversation
  (intents.rs:182-281, routing.rs:126-642).

## 3. `jig-protocol-client-link` (11,464 production lines)

- **HTTP half (about 250):** stream.rs 131 (skein-http's event reader);
  in component.rs the status checks (84-88, 193, 244, 464), paths (144,
  178, 386, 453, 641), stream_bytes (508-547), line_piece (648-657);
  store.rs:168-170; the status, path and stream fields in boundary.rs;
  the skein-http dependency. The native shell builds the requests
  (method, Host, Content-Type, Bearer, three paths) at
  `crates/jig-shell-native/src/connections.rs:87, 92, 117, 150-200`.
- **Stays:** translate/ about 9,840 (shrinks in step 3); component about
  550; store about 320; boundary about 170; limits about 190; watches 87.

## 4. Users

- `jig_domain_client` in jig, files: jig-view 123, tests/client 71,
  jig-scripted-engine 56, the link 21, jig-test-domain-client 14, ops's
  domain-client 12, tests/ops 11, jig-test-view 10, ops's view 6,
  tests/client-protocol 6, engine-clients 2 (dev-dependency), the
  transcript 1, tests/one-process 1.
- `jig_view` in jig: jig-test-view 12, tests/client 11, ops's view 8,
  tests/ops 3, tests/client-protocol 1.
- In the repository downstream of jig (files/references):
  `jig_domain_client` in its client domain 12/94, its terminal 8/36, its
  client view 6/26, its client world 6/24, its one-process world 4/9,
  its binary 1/5; `jig_view` in its terminal 10/74, its client view
  10/51, its client world 4/18, its binary 2/6, its one-process world
  2/6.

## 5. Where the code differs from the design (beyond STATUS's paragraph)

a. The request table is the root's
   (`testing/jig-test-domain-client/src/domain.rs:15`); root.md, section
   4 puts it in the client.
b. No handles, askers or latest documents: watches belong to a page
   generation (watches.rs:18-28), deliveries are pushed into page state
   (domain.rs:830-925); nothing shares one engine watch (client.md, 4
   and 8.1; view.md, 18.2).
c. The link has 3 states (link.rs:4-11) and a `blocked` flag; the design
   has 5 (client.md, 8.2).
d. Siblings hand off by value (Frame/FrameIs, Ask{ByValue}); the design
   has them watch `me` and route by asker (root.md, section 5).
e. Sign-in goes by provider and lives in the shell (boundary.rs:88,
   jig-shell-native sign_in.rs:22-35, the engine's routes.rs:107); the
   design has `SignIn { with }` (client.md, section 12).
f. `me` (Frame, schema:577) is capped at `projects: list 7`, carries chat
   charters and `chat_budget`; creating bounds sit on an offer
   (BoundsGoal, schema:448), not in `me` (contract.md, 5.1 and 5.3).
g. Refusals lack "no one to escalate to", "not well formed", a compound
   request's failing position, and "proposable by whom" (schema:354 has
   `proposable: option bool`) (contract.md, section 4).
h. Eight subjects have two document vocabularies, legacy and `Owner*`,
   in ReadValue and DeliveryValue (part1:61, 68) (protocol/client.md,
   section 3).
i. Releases are folded into asks (`Write.release`, schema:260;
   `Amend.release`, schema:268) instead of a compound request (view.md,
   18.4).
j. The link holds reads and watches behind the step's store writes, and
   a failed write fails them all (store.rs:168-188); the design holds
   only keyed requests (client.md, section 11).
k. Kept work is counted per party, `Kept{scope,requests}`
   (boundary.rs:30), and TableEntry keeps no asker or application key
   (part1:58); the design has `Kept{key,request}`.
l. None of the helpers (`shapes`, `facts`, `thread`, `words`) exist.

## 6. Order

- Needs steps 2 and 3 first: the operations, `Compound`, `me`, the task
  line and task queries; the version refusal per owner; the
  application's scopes and windows; `shapes` (templates in `me`, Create,
  Compound, Move, cancel with a grace); the reference UI's chat and goal
  lists as task queries; `thread`'s read state.
- Independent of steps 2 and 3: the HTTP split; handles and askers; the
  request table moved into the client; the link's states; `SignIn {
  with }`; kept work by key; the reference UI's records leaving the
  client's store (the 10 small-store records).
- The thin end-to-end story fits `tests/client-protocol`: its
  `engine.rs:1-50` already runs the real core under
  `jig_protocol_engine_clients::Component` over HTTP bytes. A story of
  the new client alone (printed link, `me` live, Stop answered once
  durable, kept across a reload) can run before steps 3 and 5; it uses
  Stop, not StartChat (tests/protocol.rs:4), so step 3 does not rewrite
  it.
