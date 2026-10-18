# The forge's protocol layer

Provisional, 2026-10-03, revised 2026-10-18. How the engine reaches the
forge: Forgejo v17.0's API over HTTP and JSON, the keys' markers the
engine writes at the head of what it creates, the webhooks Forgejo sends
back to a listener on loopback, and the fake forge's protocol layer that
serves the same API in the worlds. The overview is README.md (sections
5 and 6); the vocabulary carried is the forge client's
(`temper_engine_domain_forge_client::api`, `../domain/forge.md`,
sections 5 and 6); the machines are skein's (skein's `http.md`,
`json.md`, `tls.md`). What others owe this layer is in section 12, what
temper relies on of Forgejo in section 13, and what is open, GitHub
among it, in section 14.

## 1. In one page

- **Translation only.** Each call of the forge client (a `Read` or a
  `Write`) is answered by exactly one `Answer` or `Error`, made of one or
  a few HTTP exchanges on one connection. The forge component chooses the
  requests; the domain decides what to ask, when, and whether to ask
  again.
- **No retries, and every request counted.** A call goes out once. Its
  answer says how many requests it cost, so the client's budget charges
  what the forge actually served, the requests that filled a cache
  included, and nothing for a call that wrote no request.
- **Repositories by the forge's own id.** A call names its repository by
  Forgejo's id, which survives a rename or a transfer; the component
  resolves the id to a path and keeps it. Adoption first locates a
  repository by its owner and name (3.1).
- **Paged, bounded, and decoded as it streams.** Every listing names its
  page and its size; the size keeps a full page within the client's
  answer bound. Answers are decoded from JSON tokens as they arrive:
  what the domain does not carry is skipped as it passes, never held.
- **The engine's marks are keys' markers only.** A creation carries its
  key, which holds the deployment's id, in an HTML comment at the head of
  its body. Nothing else of the engine's is written into the forge.
- **The API token is the component's alone.** It is read at start, never
  granted and never stored; the first exchange with each forge checks
  that the token belongs to the configured user, and a wrong one stops
  the start (3.5; credentials.md, section 5).
- **Webhooks for latency, polling for liveness.** Forgejo's deliveries
  reach a listener of the component's own, on loopback (section 5):
  verified as they stream, answered first, each hint naming what caused
  it. Polling stands as it is and finds what a delivery lost, refused or
  never sent told of; a forge on another host polls alone, since
  deliveries over TLS need skein's TLS server side (setup.md, section
  15).
- **The fake forge speaks the same API,** from the server's side, with
  Forgejo's quirks where they are known (its default order, its page
  cap, its unpaged threads), and counts the requests it serves, so the
  engine's worlds can hold the forge's load to a ceiling.
- **What temper relies on of Forgejo is recorded** against a real
  Forgejo v17.0, by a conformance run against a throwaway instance and
  forgejo-runner v13.2, outside the suites: each behaviour a named fact
  (section 13), each exchange a transcript the decoders are tested
  against. temper requires that release: one patch release of it is
  pinned, and no other version is served.

## 2. In the engine

```
temper-engine-domain-forge-client   Read, Write, Answer, Error, Hint (api.rs)
  ▲  the root: Request::Forge; Event::Forge (the client's Answered, the forge's Hint)
the routing (temper-service)        io ↔ components ↔ root
  ▲
temper-engine-protocol-forge        calls, connections, repositories' paths, users, each forge's start; the webhooks' listener: verified deliveries as hints
  ▲  documents
temper-forge-forgejo                Forgejo's requests and answers, both sides; deliveries' payloads; the org hooks' and the setup's documents
  ▲  tokens
skein-json
  ▲  body stream
skein-http (client; a server for webhooks, plaintext on loopback)
  ▲  plaintext
skein-tls (client, for a forge whose transport is TLS; later, a server for webhooks from another host)
  ▲  ciphertext
skein-io socket
```

- **`temper-forge-forgejo`** depends on skein-lib and skein-json. It
  holds Forgejo's documents and nothing of temper's domains: each
  request's method, path, query and JSON body as a sized encoder; each
  answer's decoder, a small state machine over JSON tokens; error
  bodies; the deliveries' headers and each payload's decoder, of the
  fields a hint reads (section 5); and the setup's documents, which
  reconcile sends (setup.md, section 11). Both sides live there, the
  client's for the engine and reconcile and the server's for the fake
  forge (section 9); for deliveries, the fake forge's encoder and the
  engine's decoder. So the two are also fed transcripts of the real
  Forgejo (section 10).
- **The forge component,** `temper-engine-protocol-forge`, is a
  child of the engine's protocol layer, beside jig's components, with its
  own limits, worst case, deadlines and most outputs per entry point; the
  routing carries what passes between it, io and the root, and decides
  nothing (README.md, sections 5 and 6; jig's `protocol/service.md`,
  section 2). One component serves every configured forge. It holds the
  calls in flight, the forges' connections, each forge's start
  (section 3.5), the repositories' paths and the users, and the
  translation between the forge client's vocabulary and the documents;
  and the webhooks' listener, its connections and each forge's webhook
  secret, verifying each delivery and making it a hint (section 5).
- **What crosses to the root** is the root's boundary: a
  `Request::Forge { call, repository, op }`, which the routing hands to
  the component, ended by exactly one `client::Event::Answered { call,
  cost, result }`, which the routing hands the root as `Event::Forge`
  (the forge's `Event::Client`); and the forge's `Event::Hint { hint }`,
  which no request causes: a verified delivery's (section 5;
  `../domain/forge.md`, 5.2).
- **Room, and the drain,** as jig's components keep them (jig's
  `protocol/service.md`, section 2; jig's `domain/engine.md`, 6.1): an
  answer or a hint the root cannot take yet is held and offered again,
  at most one per call and one per webhook connection, and nothing more
  is read off its connection meanwhile. One the root returns during its
  drain (its `Request::Busy`) is never offered again: it goes when the
  component closes, and the next process reads afresh and settles its
  outbox (`../domain/forge.md`, section 16).
- **What it is told at start:** each entry of `application.forges`
  (configuration.md, section 3): the forge's number, its kind (Forgejo),
  its API's address and transport, its API token's file, the user the
  token belongs to (a number, or a file holding it: setup.md, section 8)
  and its `webhook_secret`'s file, when it has one; and
  `application.webhooks`, the listener's address, when configured. It
  reads no other member: a forge's `git` base is the routing's, for
  runs' remotes (channel.md, section 4), and its `identity` names a git
  identity, which the component never holds.

## 3. Forgejo's API as temper uses it

### 3.1 Conventions

- **Base, transport and authentication.** Every request goes to its
  forge's `api` address, under `/api/v1`, over HTTP/1.1: over TLS with
  the configured server name and trust, or in plaintext to a loopback
  address. Each carries `Authorization: token <t>`, the forge's API
  token, which the component holds from start and never logs.
  `Host` is the TLS server name, or the configured address's host; either
  with `:port` when the port is not the scheme's default. `User-Agent`
  names temper and its version.
- **JSON in, JSON out.** Bodies are written with skein-json's sized writer
  (`Content-Type: application/json`), and answers decoded from its
  tokens. Raw files and job logs are read as bytes. Strings are UTF-8;
  text the domain carries is bytes, and a write whose text is not UTF-8
  is refused before it is sent (3.4).
- **Repositories by id.** A call names `Repository { forge,
  repository }`, `repository` being Forgejo's repository id. Forgejo's
  routes name a repository by path, so each request is made at
  `/api/v1/repos/{owner}/{name}`, from the path the component keeps for
  the id (3.5). A call whose repository has no path kept first resolves
  it with `GET /api/v1/repositories/{id}`, and that request counts in its
  cost. A locate names an owner and a name itself, and is addressed with
  the forge's number and repository 0 (`../domain/forge.md`, section 4).
- **Names in paths** (an owner, a repository's name, a branch, a login,
  a file's path) are percent-encoded, each as one path segment; a slash
  inside a branch's name is `%2F`. A file's path keeps its slashes
  between segments.
- **Times.** Forgejo writes RFC 3339 times with an offset; they are read
  as the forge's clock, in nanoseconds since the epoch, at its resolution
  of a second. `since` is written in RFC 3339, in UTC, for any time the
  domain names, up to the largest (`u64::MAX` nanoseconds, in 2554),
  with which the client reads the forge's clock alone. The time an answer
  was made is its `Date` header, which a listing's answer carries as
  `now`.
- **Commits.** A commit is named by 32 bytes in the domain (a SHA-1 id
  zero-padded). A repository's object format (`object_format_name`, kept
  with its path, 3.5) says whether its ids are 40 or 64 hexadecimal
  digits; anything else is a malformed answer. Ids are written in
  lowercase hexadecimal.
- **Paging is always explicit.** Every listing Forgejo pages is sent
  with `page` and `limit`; a listing without both is never sent: Forgejo
  answers with everything where it ignores the size. The size is fixed
  per forge and per listing: the smallest of the client's `rows`, the
  forge's `max_response_items` (read at the forge's start, 3.5), and the
  number of that listing's largest rows the client's `answer_bytes`
  holds (3.3). A listing has more after it when it came full, or when
  Forgejo says so (`X-HasMore` on a pull request's files, `total_count`
  on a combined status). Listings Forgejo does not page are read whole
  and cut as they stream.
- **Users by id.** The domain names people by their forge user id; the
  API names them by login in paths and bodies. The component keeps a
  table of the two (3.5).

### 3.2 Each operation

What each call sends, in order; R is `/api/v1/repos/{owner}/{name}`, from
the repository's path, and `<size>` the page size of 3.1. Calls of more
than one request make them in turn on one connection, and end on the
first failure, but where a row says otherwise.

| Call | Requests | Answer from |
|---|---|---|
| `Read::Items { since, page, kind }` | `GET R/issues?state=all&type=issues\|pulls&since=<since>&sort=leastupdate&page=<page>&limit=<size>`, `type` only with a kind | each row a `Summary` (3.3): `number`; a `pull_request` object makes it a pull; `state`; `user.id`; `labels[].name`, for display; `updated_at`; `title`; `body` and its key; `more` when the page came full; `now` from `Date` |
| `Read::Item { number, after }` | `GET R/issues/<number>`; `GET R/issues/<number>/comments` (unpaged) | the issue as a `Summary`; its comments with ids above `after`, oldest first, as many as the answer holds (3.3): `id`, `user.id`, `created_at`, `body` and its key, its provenance and revision; `more` when one is left |
| `Read::Pull { number }` | `GET R/pulls/<number>`; `GET R/commits/<head.sha>/status?page=1&limit=1` | `number`; `state`; `head.ref`, `base.ref`; `head.sha` as `commit`; `base.sha` as `base_commit`, none when empty; `merge_commit_sha` when `merged`; `mergeable`; `requested_reviewers[].id`; CI from the combined `state` and `total_count` |
| `Read::PullFor { head, base }` | `GET R/pulls/<base>/<head>`; when it names a pull request that is not open, `GET R/pulls?state=open&sort=oldest&page=<n>&limit=<size>` from page 1, until a row has the head and base or a page comes short; then the status read, as `Pull`, of the pull request answered | the open pull request for the head and base, else the one the lookup named; as `Pull` |
| `Read::Reviews { number, page }` | `GET R/pulls/<number>/reviews?page=<page>&limit=<size>` | submitted reviews, oldest first: `id`, `user.id`, `state` as the verdict, `commit_id`, `body` and its key, `submitted_at` as `at`, `official`, provenance and revision; `more` when the page came full |
| `Read::Statuses { commit, page }` | `GET R/commits/<commit>/status?page=<page>&limit=<size>` | CI from `state` and `total_count`; each context's latest status: `updated_at`, `context`, `status`, `description`, `target_url` as its link ("Links are absolute"), no job; `more` while the pages read fall short of `total_count` |
| `Read::Remarks { number, review, page }` | `GET R/pulls/<number>/reviews/<review>/comments` (unpaged) | the `page`th slice of `<size>`, cut as it streams: `id`, `user.id`, `path`, `position` as `line`, `body`; `more` when a comment follows the slice |
| `Read::Branch { branch }` | `GET R/branches/<branch>` | `commit.id` |
| `Read::Branches` | `GET R/branches?page=<n>&limit=<size>`, from page 1 until a page comes short | every `name`; past the answer's bound, `TooLarge` |
| `Read::PullFiles { number, head, page }` | `GET R/pulls/<number>`; `GET R/pulls/<number>/files?page=<page>&limit=<size>`; for each file, `GET R/raw/<path>?ref=<merge_base>` (but for `added`; at `previous_filename` for `renamed`) and `GET R/raw/<filename>?ref=<head>` (but for `deleted`) | `head`; each file's `filename` with its contents at the merge base and at the head, a side absent when the file's `status` says so or its read answers `404`; `more` from `X-HasMore`; a `head.sha` other than `head` fails the call `Stale` before any file is read |
| `Read::Compare { before, after }` | `GET R/compare/<before>...<after>?verification=false`; `GET R/compare/<after>...<before>?files=false&verification=false`; for each file, `GET R/raw/<filename>?ref=<before>` and `?ref=<after>` | `files[].filename` with its contents at `before` and at `after`, a side absent when its read answers `404`; `commits[].sha`; `contains_before` when the second comparison's `total_commits` is 0; the whole comparison within the answer's bound, or `TooLarge` |
| `Read::Checks { commit }` | `GET R/commits/<commit>/status?page=<n>&limit=<size>`, every page; for each failing status whose link names a run of this repository's Actions: `GET R/actions/runs?head_sha=<commit>&run_number=<n>&page=1&limit=1`, then `GET R/actions/runs/<id>/jobs` | each context's latest status as for `Statuses`, its `job` the commit, the run's `id`, the job's `id` and its `attempt` when found, none otherwise |
| `Read::Job { attempt, max_bytes }` | `GET R/actions/runs/<run>`; `GET R/actions/jobs/<job>`; `GET R/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>` with `Range: bytes=-<max_bytes>`, `step` the failed step's, absent when no step failed | the failed step, its `number` and `name` (3.3), none when no step failed; the last `max_bytes` bytes of that step's log, or of the job's whole log when no step failed; `truncated` when `Content-Range` names bytes before them; `MissingJob` when a read answers `404`, the run's `commit_sha` is not the attempt's head, or the job's `run_id` or `attempt` is not the attempt's |
| `Read::File { head, path, max_bytes }` | `GET R/raw/<path>?ref=<head>` with `Range: bytes=0-<max_bytes>` | the first `max_bytes` bytes; `truncated` when more came |
| `Read::Protection { branch }` | `GET R/branches/<branch>`; then `GET R/branch_protections/<effective_branch_protection_name>` when it names a rule, or `GET R/branch_protections` when it names none | `rule_name`, `status_check_contexts` when `enable_status_check`, `required_approvals`, `dismiss_stale_approvals`; none when no rule applies; a `403` (a writer who is not an admin) `Forbidden` |
| `Read::Settings` | `GET /api/v1/repositories/<id>` | `default_branch`; `allow_merge_commits`, `allow_squash_merge`, `allow_rebase`; the repository's row kept afresh (3.5) |
| `Read::Collaborators { page }` | `GET R/collaborators?page=<page>&limit=<size>`; for each user, `GET R/collaborators/<login>/permission` | each user's `id` and permission; `more` when the page came full |
| `Read::Permission { user }` | `GET R/collaborators/<login>/permission` | `permission`: `none`, `read`, `write`, `admin`, and `owner` as `Admin` |
| `Read::Locate { owner, name }` | `GET /api/v1/repos/<owner>/<name>` | `Located { repository }` from `id`; the repository's row kept (3.5) |
| `Write::CreateIssue { key, title, body }` | `POST R/issues` `{title, body}`, the body behind the key's marker | `Created` from `number` |
| `Write::OpenPull { title, body, head, base }` | `GET R/branches/<head>`; `GET R/branches/<base>`; `GET R/compare/<base's commit.id>...<head's commit.id>?files=false&verification=false`, of which only `total_commits` is kept; when it is 0, nothing more; else `POST R/pulls` `{head, base, title, body}` | `Created` from `number`; `NothingToMerge` when `total_commits` is 0, the pull request not written |
| `Write::Post { number, key, body }` | `POST R/issues/<number>/comments` `{body}`, behind the key's marker | `Commented` from `id` |
| `Write::Review { number, key, verdict, body }` | `POST R/pulls/<number>/reviews` `{event, body}`, `event` `APPROVED`, `REQUEST_CHANGES` or `COMMENT`, the body behind the key's marker | `Reviewed` from `id` |
| `Write::Edit { number, title, body }` | `PATCH R/issues/<number>` `{title, body}`, each only when given | `Done` |
| `Write::SetReviewers { number, reviewers }` | `GET R/pulls/<number>`; `POST R/pulls/<number>/requested_reviewers` `{reviewers: [logins]}` for those missing; `DELETE R/pulls/<number>/requested_reviewers` with the same body for those extra | `Done` |
| `Write::Close { number }` | `PATCH R/issues/<number>` `{state: closed}` | `Done` |
| `Write::Reopen { number }` | `PATCH R/issues/<number>` `{state: open}` | `Done` |
| `Write::Merge { number, head }` | `POST R/pulls/<number>/merge` `{Do, head_commit_id: <head>}`; `GET R/pulls/<number>` | `Merged` from `merge_commit_sha` (the merge answers with no body) |
| `Write::Update { number }` | `POST R/pulls/<number>/update?style=merge` | `Done` |
| `Write::Status { commit, context, check }` | `POST R/statuses/<commit>` `{state: pending\|success\|failure, context}` | `Done` |
| `Write::CreateBranch { branch, commit }` | `POST R/branches` `{new_branch_name: <branch>, old_ref_name: <commit>}` | `Branch(Created)`; a `409` (the branch exists) `Branch(Exists)` |
| `Write::DeleteBranch { branch }` | `DELETE R/branches/<branch>` | `Done` |

Notes on the table:

- **The keep-up listing** is the only listing of items: always
  `state=all`, always `since`, so its rows are what changed since the
  last pass and its cost follows the rate of change, not the number of
  pull requests held (`../domain/forge.md`, 5.2). A find for an issue
  lists from the entry's start, `type=issues`. Nothing is listed by
  label.
- **`sort=leastupdate`** is Forgejo's least-recently-updated order. Its
  default is `latest`, newest *created* first, and a GitHub-style
  `sort=updated&direction=asc` is not in its vocabulary: a client that
  sends the wrong words gets the wrong order with no error. The fake
  forge answers the same way (section 9), and section 13 names the
  real one's facts.
- **An item's comments are not paged by Forgejo.** The call reads the
  whole thread, keeps the comments with ids above `after` as the answer
  holds them, and skips the rest as they stream, bodies unread; the next
  call names the last comment kept as its `after`. Section 14 weighs
  long threads.
- **Issues and pull requests share one numbering,** so an `Item` read of
  a pull request is its conversation, and `Edit`, `Close` and `Reopen`
  are the issue routes for both.
- **Reviews** are mapped by their state: `APPROVED` to `Approve`,
  `REQUEST_CHANGES` to `RequestChanges`, `COMMENT` to `Comment`. Pending
  reviews and review requests (`PENDING`, `REQUEST_REVIEW`) are not
  reviews, and are skipped, though they count towards a page coming
  full. A dismissed review is carried as `Comment`: its words and its key
  stay, its verdict does not (section 14).
- **CI.** A combined state of `failure` or `error` is `Failed`, `pending`
  is `Pending`, `success`, `warning` and `skipped` are `Passed`, and a
  `total_count` of 0 is `None`, whatever the state says. A status's own
  state maps the same way to a `Check`. Forgejo's Actions report each job
  as a status on the commit it ran on, so CI is read from statuses
  alone, never from a history of runs.
- **Job attempts.** An Actions status links to its job's page,
  `<the path of the repository's html_url>/actions/runs/<run
  number>/jobs/<job index>`, relative to the forge's origin, as Forgejo
  writes it (13); the same link with the whole `html_url` before it is
  read the same way. The run's number is its `index_in_repo`, which the
  runs' listing filters by (`run_number`); the job is the one at that
  index of the run's jobs; its attempt is the job's `attempt`. A link of
  another shape, one that starts `//` or names another scheme or host,
  or a run or a job not found, leaves the status's `job` none, and the
  call answers all the same.
- **A failed job's log is its failed step's end.** Forgejo answers a job
  (`actions/jobs/<job>`) with its steps, those of its latest attempt:
  number 0 is the runner's set-up (`Set up job`), the last its completion
  (`Complete job`), and the workflow's own steps are between. The failed
  step is the first whose `status` is `failure`: the completion step
  reports `failure` whenever the job fails, so it is the failed step only
  when none before it failed. The steps are read as they stream, and only
  the failed one is kept, the others skipped as they pass: a job of any
  number of steps is read, its steps costing time, not memory. Its log
  alone is read (`step`), and of it the end, by a suffix range, since a
  step tells its failure last. A job with no step failed (one cancelled,
  or one whose runner was lost) is read whole, its end the same way; one
  never given to a runner has no log (`404`, `MissingJob`). The job's
  `attempt` is its latest: one other than the call's (the job ran again
  since) is `MissingJob`, as a log Forgejo no longer keeps is (`404`). A
  `200` is read through, its last `max_bytes` bytes kept as it streams,
  within `body_bytes_in` like any body (section 7); a range over an empty
  log is an empty log (13). The log's search (`q`) and its NDJSON form
  are not read: neither answers a range.
- **Links are absolute.** A status's link goes to the domain whole: one
  relative to the forge's origin (a `target_url` that starts with one
  `/`) gets the origin (scheme, host and port) of the repository's
  `html_url` before it, so a brief or an agent reading it has a link
  that opens; any other is carried as Forgejo wrote it.
- **Who posted a status is not carried.** A status written by Actions
  has no `creator`, and nothing in the domain reads a status's
  poster: CI is read by context and state, and a gate's verdict temper
  posts is for protection, which reads it by context
  (`../domain/forge.md`, 8.3). `api::Status` has no author.
- **Contents of a pull request's files and of a comparison** are read
  file by file, two reads each, since the domain carries each changed
  file whole on both sides (`api::File`); the reads count in the call's
  cost. A side that does not fit what is left of the answer's bound
  fails the call as `TooLarge`, which the domain reads as an overlap
  unknown (`../domain/forge.md`, 7.3). A comparison's files are Forgejo's
  (from the two commits' merge base); their sides are read at `before`
  and at `after`.
- **Merges.** `Do` is the repository's `default_merge_style`, kept with
  its path (3.5): the merge style is the repository's own
  (`../domain/forge.md`, section 4). `head_commit_id` makes the merge
  conditional on the head the domain names, which Forgejo refuses when it
  moved (`Stale`). The merge answers with no body; the read after it
  gives the merge's commit. If that read fails, the call answers
  `Timeout`: the merge's outcome is unknown, and the client finds it by
  reading the pull request (`../domain/forge.md`, section 6).
- **Updates** merge the base into the branch (`style=merge`), never a
  rebase (`../domain/forge.md`, section 10).
- **A pull request for a head and base.** Forgejo keeps at most one pull
  request open for a head and a base. Its lookup, `pulls/{base}/{head}`,
  names one of several, the oldest (13), and its listing has
  no head or base filter. So `PullFor` answers the open one: the
  lookup's, or, when the lookup names one closed or merged, the row of
  the open pull requests' listing whose `head.ref`, `head.repo_id` and
  `base.ref` are the call's head, its repository and its base, every
  other row skipped as it streams. With none open, it answers the one
  the lookup named. A change opens a pull request only when none exists
  for its branches, and reopens it when it was closed
  (`../domain/forge.md`, sections 6 and 8.5), so the oldest is the one it
  follows, and the listing is read only when that one is closed or
  merged, or another hand opened one for its branches. Each page counts
  in the call's cost; the call's deadline bounds them.
- **Comparisons ask for what they keep.** Every comparison sends
  `verification=false`, since no commit's signature is read; one read
  only for its `total_commits` (`OpenPull`'s, and `Compare`'s second)
  also sends `files=false`, so Forgejo lists no commit's files.
  `files=false` leaves the comparison's own `files` empty too (13), so
  `Compare`'s first comparison, which keeps them, never sends it. The
  answer is decoded as before: what the call does not keep is skipped as
  it streams.
- **Opening compares first.** Forgejo opens a pull request whose head
  adds nothing to its base (13), which the forge client's vocabulary
  refuses as `NothingToMerge` (`../domain/forge.md`, section 6). So
  `OpenPull` reads both branches and compares their commits before it
  writes: a head with no commit its base lacks (`total_commits` 0, a
  head equal to its base included) fails the call `NothingToMerge`, and
  nothing is written. The three reads count in the call's cost, and fail
  as 3.4 says for them. A base that moves to contain the head between
  the comparison and the `POST` still gets a pull request with nothing
  to merge; that window is the only way temper opens one.
- **Keys.** `CreateIssue`, `Post` and `Review` write the key's marker,
  then the text (section 4). A pull request's key is its branches.
  `Edit` writes the title and body it is given.
- **Several requests, one call.** `Item`, `Pull`, `PullFor`, `PullFiles`,
  `Compare`, `Checks`, `Job`, `Protection`, `Collaborators`, `OpenPull`,
  `SetReviewers` and `Merge` make more than one; a path to resolve or a
  login to look up adds one (3.5). A `404` on a file's side, or on a
  failing status's run or job, ends nothing: it is an answer. The
  domain asks for what it needs, and the answer's cost tells its budget
  what that took (section 12).

### 3.3 Decoding answers

- **Streams, not documents.** Each answer's decoder is a state machine
  over skein-json's tokens with a bounded stack, pulled as the body
  arrives (skein's `json.md`, 3.1). It keeps only the fields its answer
  needs, as plain values; every other field, nested object and array is
  skipped as it passes.
- **A string is kept whole, or not at all.** skein-json keeps a string
  only when it fits the cap its reader names, and otherwise reads it
  through and says its length (`Text` and `Long`, skein's `json.md`,
  3.1). So:
  - a name, a title, a context, a description, a link or a path is kept
    within `string_bytes`; a longer one makes the answer malformed;
  - but for a job step's name, which Forgejo makes of the step's whole
    script when the workflow names none: a longer one is carried empty,
    and the step is named by its number alone, so that a long script
    never makes a job unreadable;
  - a body (an issue's, a comment's, a review's, a remark's) is kept
    whole within `body_bytes`. A longer one is carried empty, its key
    none, its provenance `Unknown`, and its revision made from its
    length (below); the component counts it among its diagnostics.
    `body_bytes` is at least the largest body temper writes, its marker
    included (the client's `op_bytes`), checked at start, so temper's
    own creations are always read whole and their keys found;
  - every string is read as UTF-8 by skein-json; a body that is not
    valid UTF-8 is a malformed answer.
- **The answer's bound.** The page size (3.1) holds a full page of a
  listing's largest rows within the client's `answer_bytes`, so a paged
  answer never passes it. An item's comments are kept until the next
  would pass `answer_bytes` or `rows`, and `more` is then set. Raw
  contents are read within what is left of `answer_bytes`: a file's
  after its path, a job's log after its step's name, so the answer never
  passes it.
- **Revisions.** A comment's or a review's revision is the first eight
  bytes, big-endian, of the SHA-256 of its whole body as the forge
  returns it, computed as the string streams past; for a body not kept,
  of its length and its `updated_at`. So it changes whenever the body
  does, at any resolution of the forge's clock, and names what the forge
  stored, not what was sent.
- **Provenance.** `Original` when `updated_at` equals `created_at` (a
  review's `submitted_at`), `Revised` when it is later, `Unknown` when
  either is missing or the body was not kept. Forgejo's clock has a
  second's resolution: an edit within the second of the creation reads
  as `Original` (section 14).
- **Markers** at the head of a body, a comment or a review are decoded
  into its `key` (section 4).
- **What may follow** is told by the page coming full, `X-HasMore`, or
  `total_count`. For a call made of several requests, `more` is its
  listing's.
- **A malformed answer** (a document that does not decode, a field
  missing, a number out of range, a commit id of the wrong length, a
  string past its cap, a body past `body_bytes_in`, a log read through
  included) fails a write as `Timeout`: the request reached the forge,
  so what it asked may have been done. It fails a read as
  `InvalidAnswer`. Either way the connection is closed, since what
  followed on it can no longer be trusted. A repository id past the
  domain's 32 bits fails the call as `TooLarge`.

### 3.4 Failures

A call's `Error` is the first failure of its requests. "Nothing written"
means the call failed before any byte of its first request was written;
such a request costs nothing.

| What happened | `Error` |
|---|---|
| nothing written: no connection, a refused connect, a TLS handshake that failed; or the server never saw the request (skein-http's `Closed`); or the call's deadline while it waited | `Unavailable` |
| a `3xx`, as Forgejo answers a moved repository's old path before it acts: the path dropped (3.5) | `Unavailable` |
| `429`, with `Retry-After` (seconds or a date) | `RateLimited { after }`: the wait, a date taken from the answer's `Date` |
| anything after a byte was written: a reset, the end of the stream, the call's deadline, `5xx` | `Timeout` |
| a malformed answer (3.3) | `Timeout` on a write; `InvalidAnswer` on a read |
| `401`, `403` | `Forbidden` |
| `404` | `Missing`, the repository's path dropped (3.5); `MissingJob` on `Job`'s reads |
| `413` (Forgejo's quota) | `Full` |
| `423` (the repository is archived) | `Forbidden` |
| `409` on `OpenPull` (a pull request for the head and base is open) | `Exists` |
| `OpenPull`'s comparison counting no commit (`total_commits` 0): the pull request not written | `NothingToMerge` |
| on `OpenPull`'s reads, before its `POST`: what this table makes a `Timeout` or an `InvalidAnswer` (the pull request was not written) | `Unavailable` |
| `422` on a write whose title or body is empty | `Empty` |
| `405` on `Merge` for a pull request merged or closed | `Closed` |
| `405` on `Merge` for a conflict | `Conflict` |
| `405` on `Merge`, `403` on `DeleteBranch`, for a branch's protection | `Protected` |
| `405` on `Merge` for a style the repository does not allow | `Refused`, the repository's row dropped (3.5) |
| `409` on `Merge`, `head out of date` | `Stale` |
| `409` on `Update`, `merge failed because of conflict` | `Conflict` |
| a pull request's head other than `PullFiles` names | `Stale` |
| an answer past the component's bounds | `TooLarge` |
| a write whose text is not UTF-8, which JSON cannot carry; nothing written | `Refused` |
| any other `4xx` | `Refused` on a write; `InvalidAnswer` on a read |

- **Unavailable is provable only before the first byte.** skein-http's
  client tells a request the server never saw (`Closed`) from one it may
  have read (`Truncated`, `Stream`; skein's `http.md`, 3.4). A
  connection kept alive that the forge closed just as a request went out
  may have been read: it is a `Timeout`. To make that rare, an idle
  connection is closed before the forge's keep-alive timeout would close
  it (`idle`, section 7).
- **One status, several meanings.** Where Forgejo answers two failures
  with one status (`405` on a merge), the decoder reads its `message`
  into the failure; the messages are those the pinned release answers,
  in the conformance's transcripts (section 13). A `4xx` means Forgejo
  did not act, so a message the decoder does not know is `Refused` on a
  write, never a guess at which failure it was; on a read,
  `InvalidAnswer`.
- **Not errors:** a `409` on `CreateBranch` is `Branch(Exists)`, which
  the client follows with a read of the branch; a `404` on
  `DeleteBranch` is `Missing`, which the client counts as the branch
  gone (`../domain/forge.md`, section 6). Forgejo answers the
  deletion of a branch that is not there `500` with an empty body (13):
  a `Timeout`, after which the client's find reads the branch, and that
  read's `404` is `Missing`, the branch gone.
- **Reads before a write.** `OpenPull`'s reads come before its `POST`,
  so a failure among them leaves the pull request provably not written:
  what would be a `Timeout` or an `InvalidAnswer` is `Unavailable`, and
  the client may send the call again. Their other failures are as the
  table says (a `404`, a branch not there, `Missing`; a `429`
  `RateLimited`; a `401` or a `403` `Forbidden`).
- **The deadline.** Each call has the component's `deadline`, which
  configuration checks is at most the client's `lifetime`, so no byte of
  a write is sent past the time the client counts on. A call still
  waiting at its deadline is `Unavailable`; one in flight is a `Timeout`,
  and its connection is aborted.
- **Never `Busy`.** The component holds as many calls as the client may
  have in flight (its `calls`), so it never refuses one: `Busy` is the
  client's own admission.

### 3.5 What the protocol layer keeps

Mechanism only, bounded, and rebuilt by asking again:

- **Each forge's start** (credentials.md, section 5). The first
  exchange with a forge is `GET /api/v1/user`. An `id` other than the
  configured `user`, or a `401` or `403`, stops the engine, naming the
  forge, at start or when the forge first answers: a wrong token would
  make temper act as someone else. Then `GET /api/v1/settings/api`, for
  `max_response_items`, the forge's page cap. A forge that cannot be
  reached, or answers `429`, `5xx` or a malformed answer, is asked again
  after `start_backoff`, doubling to `start_backoff_max` (after the
  `Retry-After` of a `429`, when that is longer). Any other `4xx` to
  either read (the `api` address names no Forgejo API, say) stops the
  engine too, naming the forge and the status: asking again would not
  mend it. Calls to a forge wait until its start is done, each until its
  deadline. These two reads are the component's own, counted in no
  call's cost.
- **Repositories,** by forge and id: the path (owner and name), the
  object format, the `default_merge_style` and the `html_url` (for jobs'
  links). A row is filled by `GET /api/v1/repositories/{id}`, counted in
  the call that needed it, by a locate, and by a `Settings` read, which
  refreshes it. It is dropped when a call made by its path is answered
  `404` or a redirect, or a merge is refused its style, so the next call
  resolves the id again. At most `repositories` rows in all; when the
  table is full, the row used longest ago goes.
- **Users:** a table of logins by user id, filled from every user object
  an answer carries (`user`, a status's `creator` when it is not `null`,
  `requested_reviewers`, a collaborators' listing); an id the table does
  not hold is looked up with `GET /api/v1/users/search?uid=<id>&limit=1`,
  counted in the call, and a user not found fails the call as `Missing`.
  At most `users` rows; when full, the entry used longest ago goes.
- **Each forge's page cap,** from its start.
- **The API tokens,** one per forge, read at start from the files
  configuration names, held in the component's own table, never logged,
  granted or stored (credentials.md, section 5).

Nothing the domain relies on is kept here: a lost row costs one
request, and a restarted engine starts with none.

## 4. The engine's marks

What the engine writes inside what it creates: keys' markers, and
nothing else. No record, outcome or page of the engine's is written into
the forge, and no call of 3.2 writes a label: labels are for display
only, and no decision reads one (`../domain/forge.md`, section 4).

- **A creation's key,** at the head of an issue's body, a comment or a
  review: `<!-- temper:key <the key in lowercase hexadecimal> -->` and a
  line feed, then the text. An HTML comment, which Forgejo's rendering
  hides.
- **The key is the client's** (`identity.rs`, `effect_key`): version 1,
  the namespace's length as two bytes big-endian, the namespace, which
  the root derives from the deployment's id, then the effect's identity
  (its purpose and its numbers). So the marker carries the deployment's
  id, and deployments that share a repository tell their creations
  apart (jig's `domain/connectors.md`, 4.4). The key's first byte is its
  version; the marker has none of its own.
- **Decoded** from a body whose first bytes are `<!-- temper:key `, then
  an even count of lowercase hexadecimal digits, then ` -->` and a line
  feed; any other body has no key. Whether to believe a key is the
  domain's: its version, its namespace, and whether its author is the
  forge's writer (`identity.rs`). A person who copies a marker into
  their own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`, the marker
  twice its length and 21 bytes more, and the body sent the marker and
  the text, which `body_bytes` holds (3.3).
- **The worlds** write the same marker (the forge connector world's
  translation, `tests/engine/forge/client/src/translate.rs`); the
  protocol world (section 10) runs the component's own encoder and
  decoder.

## 5. Webhooks

Forgejo's deliveries, made hints: they cut polling's latency, and decide
nothing. Polling stands as it is with no webhook (section 6;
`../domain/forge.md`, 5.2): its `poll`, `poll_max`, `hinted` and `slow`
are unchanged, and it finds whatever a delivery lost, refused or never
sent told of. A delivery reaches the engine on loopback; a forge on
another host polls alone (setup.md, section 15).

- **The webhooks' listener,** `application.webhooks`, `{"listener":
  address}` (configuration.md, section 3): a socket address on loopback
  (in `127.0.0.0/8`, or `::1`), plaintext HTTP on skein's HTTP server, a
  listener of the forge component's own, apart from the clients'
  listener (`../client/README.md`, section 8). It takes `POST` on one path per
  forge, `/hooks/<number>`, the forge's `number`, one request at a time
  per connection, kept alive.
- **Each forge's secret,** its `webhook_secret`: a member of its
  `forges` entry naming a private file (configuration.md, section 6),
  read at start and held in the component's own table, never logged.
  It is optional, and present only with `application.webhooks`: a
  `webhook_secret` with no `webhooks` refuses the start, naming its
  path. A forge with no secret has no path: a request to it is answered
  `404`, as one to a path that names no forge, and hints nothing.
- **Signed, and verified as it streams.** Forgejo signs each delivery's
  body with its hook's secret: HMAC-SHA256, in hexadecimal, in
  `X-Forgejo-Signature`. The component computes it as the body arrives,
  with skein lib's `sha256::Hmac` keyed by the forge's secret, and
  decodes the body at the same time into the few fields its hint reads;
  nothing of it is believed until the whole body is read and the
  signature holds, compared with `sha256::equal`, in constant time
  (skein's `lib.md`, 6.2). A delivery with no signature is answered
  `403` from its head, its body unread; one with a wrong signature,
  `403` once its body is read; either is dropped.
- **Answered first.** Once the body is read and verified, the response
  (`204`) is queued, and only then does the hint go up. The engine never
  makes Forgejo wait on its work. skein's HTTP server lets the component
  answer when it chooses (skein's `http.md`, 5.4). A verified delivery
  that hints nothing (an event of no row below, a tag's) is answered
  `204` all the same.
- **The events a hook subscribes to** are every event whose delivery the
  table maps to a hint, and no other. The component reads each delivery
  by `X-Forgejo-Event`, the header it arrives under, which several
  events share; each hint's repository is the forge's number and the
  payload's `repository.id` (`run.repository.id` for a job's), so no
  path is needed (3.1):

  | Events subscribed | `X-Forgejo-Event` | `Hint` |
  |---|---|---|
  | `issues`, `issue_assign`, `issue_label`, `issue_milestone` | `issues` | `Change::Item(number)`, with the key of the issue's body |
  | `pull_request`, `pull_request_assign`, `pull_request_label`, `pull_request_milestone`, `pull_request_sync`, `pull_request_review_request` | `pull_request` | `Change::Item(number)`, with no key: a pull request's key is its branches (3.2) |
  | `issue_comment`, `pull_request_comment` | `issue_comment` | `Change::Item(number)`, with the key of the comment's body |
  | `pull_request_review_approved`, `pull_request_review_rejected`, `pull_request_review_comment` | `pull_request_approved`, `pull_request_rejected`, `pull_request_comment` | `Change::Item(number)`, with the key of the review's body |
  | `push` | `push` | of a branch, `Change::BranchMoved { branch, head, actor }`, the commit it moved to and the pusher's id; of a branch deleted (its `after` all zeros), `Change::Branch(branch)` |
  | `create`, `delete` | `create`, `delete` | of a branch, `Change::Branch(branch)` |
  | a job's ends: `workflow_job_success`, `workflow_job_failure`, `workflow_job_cancelled`, `workflow_job_skipped` | the same | `Change::Commit(commit)`, the run's `commit_sha` |
  | none | anything else; a tag's `push`, `create` or `delete` | nothing |

  Each row's events, its header and the fields its hint reads are
  recorded on the pinned release (section 13). A row that went wrong
  would cost a poll's wait, never a decision: a hint only brings a read
  sooner (`../domain/forge.md`, 5.2).

  **Who caused it** is what lets the domain drop its own writes' echoes:
  a creation's key for an item, the pusher for a push
  (`../domain/forge.md`, 5.2). A job's delivery has no top-level
  repository or sender: its `run.repository.id` and `run.commit_sha`
  stand for them, and its hint names the commit and no actor. A hint
  for a repository the deployment has not adopted goes up all the same;
  the domain drops it.
- **CI is hinted as each job ends.** A job's delivery is sent for each
  of its attempts, so a job run again is hinted again when it ends. The
  run's deliveries (`workflow_run_*`), sent once its last job ends, and
  the deprecated `action_run_*` are not subscribed.
- **What has no delivery** is found by polling alone
  (`../domain/forge.md`, 5.2): a commit status, which Forgejo never
  delivers, so CI that a system other than Actions reports and the
  verdicts temper posts.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: section 13) is the domain's to
  read again, within `settle` of the move (`../domain/forge.md`, 5.2);
  the component hints at once.
- **Bounded.** Connections (`hooks`), the head (`hook_head_bytes`) and
  the body (`hook_body_bytes`) are capped, and each request is read
  within `hook_deadline` (section 7). A body whose length passes its cap
  is answered `413` and hints nothing; a method other than `POST` on a
  forge's path, `405`; a verified body the decoder finds malformed,
  `400`, with no hint; a request not read within `hook_deadline` is
  closed with no answer. Push payloads list commits, which the decoder
  skips.
- **A listener that fails** while the engine runs stops the engine, as
  the clients' listener's does: the service drains as at a shutdown, and
  whatever starts `temper serve` again (the box, a service manager)
  starts it with a listener again.
- **Not sent again.** Forgejo does not resend a failed delivery: one
  lost while `temper serve` is down, refused for its size or its
  signature, or never sent is found by polling. A hint that arrives
  before the engine's domain is ready is dropped, and the keep's first
  listing finds what it told of.
- **Hooks are set up by reconcile, or by the person,** at
  `http://<listener>/hooks/<number>`, with the forge's secret, the
  events above, JSON and active (setup.md, sections 4 and 5, step 6, and
  section 10). A repository with no hook polls (`../domain/forge.md`,
  section 4).

## 6. The load rules, and where each is kept

| Rule | Kept by |
|---|---|
| No listing without a filter on a periodic path; no history | the domain's one keep-up listing per repository, always `since` (3.2); the slow pass reads live resources one at a time |
| Calls per pass do not grow with history | one listing per pass, nothing listed by label; checked by the cost checks (10) |
| Cache by change token | comments and reviews by revision (3.3); repositories' paths and users cached (3.5); a pull request read on a backoff (`../domain/forge.md`, 5.2) |
| One path to CI; never a closed pull request's | statuses of the head only (3.2); a closed item leaves the working set |
| API only; `page` and `limit` always; the forge's cap | 3.1 |
| Contents only when a decision reads them | a pull request's files and a comparison, read for a brief or an overlap, each file's reads charged (3.2) |
| Webhooks: hints, answered first, merged, targeted, echoes dropped | section 5; the domain merges hints into one earlier pass, and drops its own echoes by key and by pusher |
| Closed items by deltas, not sweeps | the keep-up listing's `since`; closed items are never listed again |
| Backoff and jitter; honour the rate; no blind retries of writes | the domain's backoff and budget; `RateLimited { after }`; keyed creations found again after a `Timeout`; no retry here (3.4) |
| Slow cadences; webhooks for latency, polling for liveness | the domain's `poll`, `hinted` and `slow`, unchanged by webhooks; a forge with no hook, or on another host, polls alone |

Nothing the engine writes changes often: it writes the forge only to
make an effect (`../domain/forge.md`, section 6), never as a heartbeat.

## 7. Entities, limits and the worst case

**Entities,** each in a slab of the component:

- **a call,** bound to the client's `call` token, holding its operation,
  its repository, the step it is at, what it has decoded so far, its
  cost and its deadline;
- **a forge's start,** per configured forge: where it is (3.5), and its
  backoff;
- **a forge connection,** bound to io's socket and to one forge, holding
  its stack (TLS, HTTP client, JSON tokenizer, the answer's decoder) and
  its deadlines;
- **the tables** of section 3.5: repositories, users, page caps, API
  tokens; and each forge's webhook secret (section 5);
- **the webhooks' listener,** when `webhooks` is configured, bound to
  io's listening socket;
- **a webhook connection,** bound to a socket the listener accepted,
  holding its stack (HTTP server, JSON tokenizer, the delivery's
  decoder), the HMAC state, the fields decoded, and the hint it holds
  until the root has room (section 2).

**Limits** (the forge component's, in temper's `limits` member,
configuration.md, section 7; their values in `temper-service`'s profile
`small`):

- `forges`: the forges served, each with its start, its page cap and
  its API token;
- `token_bytes`: the largest API token, so that every request's head,
  its `Authorization` field among them, fits skein-http's `request`;
- `connections`: the forge connections in all, equal to the client's
  `calls`, so a call never waits for one; an idle connection of one
  forge is closed for a call to another when none is free;
- `head_bytes`, `body_bytes_in`: the largest response head and body read
  (the body is streamed, so this caps time and bytes, not memory);
- `depth`: the JSON nesting followed (Forgejo's documents need about 8);
- `string_bytes`: the largest name, title, context, description, link or
  path kept; at least the forge's `name_bytes`, so every name the
  domain keeps is read;
- `body_bytes`: the largest body kept whole; at least the client's
  `op_bytes` with a marker (section 4);
- `inner`: the most elements kept of a list inside one document (a pull
  request's reviewers, a rule's contexts); `target_bytes`: the longest
  request target, its path and query;
- `repositories`: the paths' table, at most the forge's `repositories`;
  `users`: the logins' table;
- `deadline`: a call's, at most the client's `lifetime`, and each of a
  forge's start reads', from when it is asked;
- `idle`: how long an idle connection is kept, below the keep-alive
  timeout of whatever serves the forge's API (a reverse proxy before
  Forgejo, when there is one) by more than a request's round trip;
- `start_backoff`, `start_backoff_max`: a forge's start asked again;
- what TLS, HTTP and JSON take as their own (skein's `tls.md`, `http.md`,
  `json.md`);
- the webhooks' listener's: `hooks`, the webhook connections, beyond
  which the listener accepts no more until one closes;
  `hook_head_bytes`, `hook_body_bytes`, the largest delivery's head and
  body read; `hook_deadline`, a request's read and answer, from its
  connection's accept or its last answer; `hook_secret_bytes`, the
  largest webhook secret.

**Checked at start** (configuration.md, section 7), with every limit of
temper's and jig's: `connections` equals the client's `calls`;
`deadline` is at most the client's `lifetime`; `body_bytes` holds the
largest body temper writes; and, for each listing, at least one of its
largest rows fits the client's `answer_bytes`, so no page size is 0.

**The worst case** is the sum of: each connection's stack and intake and
output caps; each call's answer as it is built, at most the client's
`answer_bytes`; the tables; each webhook connection's stack, its
decoded fields and its held hint. None of it grows with the forge's
history: a long thread or a large payload costs time, not memory.

## 8. State machines

**A forge connection:**

| State | Holds | Demand | Deadline | On |
|---|---|---|---|---|
| Closed | nothing | none | none | a call: Connecting |
| Connecting | the call | none | the call's | connected: Handshaking; failed: the call `Unavailable`, Closed |
| Handshaking | the call, TLS | TLS's | the call's | done: Sending; failed: `Unavailable`, Closed |
| Sending | the call, its request | room | the call's | sent: Reading; a fault: `Timeout`, or `Unavailable` if nothing was written, Closed |
| Reading | the call, the decoder | the head, then the body as the decoder demands | the call's | answered: the next step's Sending, or Idle with the answer up; a fault, a malformed answer or the deadline: as 3.4, Closed |
| Idle | nothing | none | `idle` | a call to its forge: Sending; the deadline, the forge closing, or a call to another forge with no connection free: Closed |

Without TLS, Connecting goes to Sending. A connection the forge marked
`Connection: close`, or one that read a raw body past `max_bytes`, goes
to Closed after its answer, not to Idle.

- **What holds a connection:** a call, or one of a forge's start reads
  (3.5), which a connection of the forge's own holds where the table
  says the call; a start read's failures go to its start, not to a
  call.
- **Deadlines are the holder's.** Connect and handshake have no limit
  of their own: Connecting, Handshaking, Sending and Reading all wait
  until the holder's deadline, the call's or the start read's, so no
  exchange outlives what it serves. A start read's deadline is the
  component's `deadline` from when the read is asked; one that passes
  is a forge that cannot be reached, asked again after the start's
  backoff (3.5).

**A call:** Waiting for its forge's start (bounded by the client's
`calls`); Resolving, when its repository has no path kept or a login is
missing; then running its steps on its connection; then answered, once.
Its result is the first failure of its steps, or its answer, with its
cost.

**A forge's start:**

| State | On |
|---|---|
| Checking | `GET /user`: the configured user: Settings; another user, `401`, `403`, any other `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed: Waiting |
| Settings | `GET /settings/api`: Ready; a `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed: Waiting |
| Waiting | the backoff: Checking |
| Ready | serves calls |
| Stopped | the engine ends, exit status 1, naming the forge (README.md, section 6) |

**Ending:** the routing closes the component after the root's drain
(jig's `protocol/service.md`, section 5); calls in flight end with their
connections, and no answer goes up once the root has stopped.

**A webhook connection,** from the listener's accept:

| State | Holds | Demand | Deadline | On |
|---|---|---|---|---|
| Head | the head so far | a scan for the head's end | `hook_deadline` | a signed `POST` to the path of a forge with a secret, its length within `hook_body_bytes`: Body; its length past it: `413`, Closing; a path of no such forge: `404`, Closing; another method on a forge's path: `405`, Closing; no signature: `403`, Closing; any other head: answered `4xx`, Closing; past the deadline: Closing, with no answer |
| Body | the HMAC state, the fields decoded | the body, as the decoder demands | `hook_deadline` | read and verified: Answering; the signature wrong: `403`, Closing; a verified body that does not decode: `400`, Closing |
| Answering | the hint, or none | room | `hook_deadline` | `204` queued: the hint offered up, and held while the root has no room (section 2); then Head (kept alive) or Closing |
| Closing | nothing | none | io's close deadline | closed |

A fault or the deadline in any state closes the connection with no
answer and no hint: Forgejo counts the delivery failed, and polling
finds what it told of (section 5).

## 9. The fake forge's protocol layer

The fake forge is a service (`../testing.md`, 4.2): its domain,
`testing/temper-fake-forge-domain`, is the one the forge connector's
world drives, and its protocol layer, `testing/temper-fake-forge-protocol`,
serves Forgejo's API over it, on skein's HTTP server, from the server
side of `temper-forge-forgejo`.

- **The routes of section 3.2,** and those of a forge's start and of
  resolution: `/api/v1/user`, `/api/v1/settings/api`,
  `/api/v1/repositories/{id}`, `/api/v1/repos/{owner}/{name}`,
  `/api/v1/users/search`. Each request is decoded, translated to the
  fake's `Read` or `Write` (as the forge connector world's translation
  does, `tests/engine/forge/client/src/translate.rs`), and its answer
  encoded as Forgejo encodes it. The fake's domain has the reads these
  routes need beyond the forge client's vocabulary: Actions' runs, for
  `Read::Checks` and `Read::Job` (3.2), as a commit's run by its number
  (`actions/runs?head_sha&run_number`), a run by its id
  (`actions/runs/{id}`) and a run's jobs (`actions/runs/{id}/jobs`), each
  job with its id and attempt, and a job by its id (`actions/jobs/{id}`)
  with its steps, its log read by attempt and step
  (`actions/jobs/{id}/logs?attempt&step`) with byte ranges, a suffix
  range among them; the oldest pull request for a head and base, open or
  not, and the open pull requests, oldest first, for
  `Read::PullFor`'s requests (3.2); and the fake's CI statuses link their
  jobs in Forgejo's shape, relative to the origin, `<the path of
  html_url>/actions/runs/<run number>/jobs/<job index>`, with no
  `creator`, as Forgejo writes a status of its Actions.
- **Its names, Forgejo's shape.** The fake names repositories by their
  full name; its protocol layer gives each the id the scenario names,
  and answers `/repositories/{id}` with its path, `sha1` as its object
  format and `squash`, how the fake merges, as its merge style. A commit
  the fake counts is a 40-digit id, its count big-endian in its first
  eight bytes. The user a token stands for is the caller, from a table
  the scenario gives; an unknown token is answered `401`.
- **Forgejo's quirks, kept:** the default order is newest created first;
  `sort=leastupdate` gives least recently updated first; a page is
  capped at the configured `max_response_items`; a size without a page
  is ignored where Forgejo ignores it; an issue's comments, a review's
  comments and a comparison come unpaged; collaborators come without
  their permissions; a merge answers with no body; the lookup
  `pulls/{base}/{head}` names the oldest of several pull requests, open
  or not; a pull request whose head adds nothing to its base is opened
  (`201`); deleting a branch that is not there answers `500` with an
  empty body; a failed job's completion step reports `failure` too; a
  push that moves a pull request's head or base starts its check, which
  reads it not mergeable until it ends, and is delivered while it runs,
  the order Forgejo sometimes takes (section 13). A client that leans on
  Forgejo being better than it is fails in the fake first.
- **Faults move down** (`../testing.md`, 4.2): a call refused for the
  rate is a `429` with `Retry-After`; one failed before it is made is a
  refused connect or a reset before the request is read; one made and
  then timed out is a `5xx` or a dropped connection after the request
  was read; a late answer is a slow one.
- **Requests are counted,** by route, by user and in total, in the
  fake's tally, so a world can hold the engine's load to its ceilings
  (section 10).
- **Deliveries are sent,** for the fake domain's `Request::Hook`, as
  Forgejo sends them: one `POST` per change to the URL of the
  repository's hook (the scenario's, or the org hook reconcile made),
  with section 5's event, headers and fields, its body signed with the
  hook's secret, after the latency the fake draws, late, out of order
  or never: the fake's protocol layer renders the signed request, and
  the world writes it on a connection to the listener. A status of the
  fake's CI is delivered as its job's end; a status anyone else
  reports sends nothing, as on Forgejo.
- **The setup's routes,** for reconcile's world, from the server side of
  `temper-forge-forgejo`'s setup documents, kept in a store of the
  protocol layer's beside its routes, not in its domain's (setup.md,
  sections 11 and 13.2).
- **No git over HTTP:** the fake forge's git is reached through its
  typed transport alone (`../testing.md`, 4.2 and 4.3).

## 10. Testing

The tiers, the fakes and the layout are `../testing.md`'s (sections 2,
4.2 and 7). The forge's own:

- **Step tests** for `temper-forge-forgejo`: each decoder against the
  conformance's transcripts (each answer section 3.2 reads, each error
  section 3.4 names, and each delivery section 5 hints), against answers
  generated from the seed, and against mutations of both; golden bytes
  for each encoder; bounds at their limit and one past; a fuzz target
  for each decoder. Server encoders meet the client decoders, and the
  client encoders meet the server decoders.
- **Transcripts are format-faithful, content-irrelevant.** Ids, times,
  counts and text vary; keys, nesting, types and status lines must
  match. Volatile fields are masked before comparing; tokens and secrets
  never reach a file.
- **Step tests** in the forge component: each call's plan of section 3.2
  against canned answers; the marker's encoder and decoder; the page
  size; names percent-encoded; times both ways; the failures of 3.4.
- **The machine world,** in `tests/engine/protocol-forge`'s crate beside
  the protocol world: the forge component against a scripted HTTP peer,
  over in-memory streams: every state of section 8's machines, faults
  at every byte, deadlines, a wrong user or another `4xx` stopping the
  start, a path dropped on `404` or a redirect and resolved again, a
  body past `body_bytes`. And the webhooks' listener against scripted
  deliveries: a good delivery hinted, its `204` queued before its hint
  goes up; a missing or a bad signature `403`; an oversize body `413`;
  an unknown event `204` and nothing hinted; a forge with no secret
  `404`; a hint held while the root has no room; faults at every byte.
- **The protocol world,** `tests/engine/protocol-forge`: the engine's
  forge component and the fake forge's protocol layer, joined by
  in-memory streams cut at random, running the forge connector world's
  stories (`tests/engine/forge`, `../domain/forge.md`, section 18)
  through bytes: a change produced, opened, checked, queued and landed;
  a creation and a merge whose answers were lost, found after a restart
  and never made twice; a failed head repaired and checked again; a held
  branch moved by another hand, reported as drift. Its invariants: what
  the engine's domain asked is what the fake's domain did; each call is
  answered once, however its streams are cut; the API token reaches no
  record, request or fact. The fake's deliveries reach the engine's
  listener signed, late, out of order or never, so its stories run with
  hints and without them.
- **The cost checks,** in the forge connector's world counting
  `Request::Forge`, and in the protocol world counting HTTP requests:
  - **idle:** over a span with no change on the forge, the calls made are
    at most the keep-up listings, one per repository every `poll`, the
    held pull requests due on their backoff, and the slow pass's reads
    every `slow`; the bound is computed from the limits;
  - **history-free:** the same scenario on a forge preloaded with ten
    times the closed items and comments makes the same calls while idle;
  - **busy:** listing pages per pass are at most the changed items over
    the page size plus one, and item reads at most the changed items
    held times one plus the pages of their new comments;
  - **honest costs:** in the protocol world, the requests the fake served
    equal the sum of the costs the answers reported, the forges' starts'
    reads aside.
- **The real Forgejo check,** outside the two suites:
  `tools/forgejo-conformance`, against the throwaway Forgejo
  (`../testing.md`, 2.6) with a small `MAX_RESPONSE_ITEMS`, so paging
  shows, and webhooks allowed to loopback; the tool makes its admin user
  and tokens. It drives section 13's facts through plain requests, a
  hook made as reconcile makes one, and the forge component itself, each
  call of 3.2 a fact; it records each fact as `holds`, `fails` or
  `unchecked` in its `facts.txt`, and what it sent and received as
  transcripts, masked, with the binary's digest. Actions facts need a
  runner, the forgejo-runner `TEMPER_FORGEJO_RUNNER` names; without it
  the tool records them unchecked.
- **The pin** is one patch release of Forgejo v17.0 and one of
  forgejo-runner v13.2, constants the throwaway Forgejo, the tool and
  owned mode share (`VERSION` and `RUNNER_VERSION`,
  `crates/temper-forge-forgejo/src/pin.rs`; setup.md, 6.1); a binary of
  another release is refused, naming it. Moving to a later patch changes
  those constants and the transcripts and facts the tool records, and
  not this document.
- **The process tier** runs the forge component's start, polling and
  webhooks against an owned Forgejo (`../testing.md`, 2.6; setup.md,
  13.4).

## 12. What others owe

The boundary, the configuration and the setup's hooks are sections 2
and 5's. The forge client (`temper_engine_domain_forge_client`,
`../domain/forge.md`, section 4) owes the component:

- **`Read::Locate { owner, name }`,** answered `Answer::Located {
  repository }`, addressed with the forge's number and repository 0
  (3.2); its answer's shape checked as every other's, a located id never
  0, and its owner and name counted in the call's bytes.
- **Costs,** as the boundary carries them: `Event::Answered { call,
  cost, result }`, `cost` the requests the call wrote, cache fills
  included, 0 for none; the budget charges it (`../domain/forge.md`,
  5.4).
- **Hints,** as the vocabulary has them: `Hint { repository, change, key
  }`, with the pusher on `Change::BranchMoved`, so the domain drops its
  own echoes (section 5).
- **Shared limits:** the client's `calls`, `rows`, `answer_bytes`,
  `op_bytes` and `lifetime`, and the forge's `repositories` and
  `name_bytes`, are given to the component too, which sets and checks
  its own by them (section 7).

## 13. What temper relies on

What the component and the forge connector rely on of Forgejo, on the
pinned release (section 10). Each is named by its fact in
`tools/forgejo-conformance/facts.txt`, where it `holds`; a fact marked
*fails* is phrased the other way there, and its failing is what temper
relies on. What no fact records yet is open (section 14).

- **Every call of 3.2, made by the component.** Each read and write
  answers as its row says, its cost the requests it sent, and the keys
  temper writes are found on reading back; a forge's start is ready, and
  another user or a wrong token stops it (every `component-` fact: one
  per call, from `component-read-items` to
  `component-write-delete-branch`, with
  `component-read-pull-for-open-after-closed` and
  `component-write-open-pull-no-changes`; one per start's outcome, from
  `component-start-ready`; `component-costs-equal-requests-sent`;
  `component-keys-found-on-read-back`).
- **Paging.** With Forgejo's default settings, `max_response_items` is
  50 and `default_paging_num` 30 (`api-default-page-cap-50-and-30`). A
  pull request's files are paged, with `X-HasMore`
  (`pull-files-page-with-x-hasmore`); a comparison ignores `page` and
  `limit` and answers everything (`compare-ignores-page-and-limit`).
- **The keep-up listing.** `sort=leastupdate` lists the least recently
  updated first, ties by number
  (`leastupdate-orders-least-recently-updated-first`,
  `leastupdate-ties-ordered-by-number`); `since` is inclusive at the
  second (`since-is-inclusive-at-the-second`), and in 2554 answers an
  empty page with its `Date`
  (`since-in-2554-answers-an-empty-page-with-date`). An item's updated
  time moves on a comment, a review, a push to its head, closing and
  reopening (`updated-moves-on-comment`, `updated-moves-on-review`,
  `updated-moves-on-head-push`, `updated-moves-on-close`,
  `updated-moves-on-reopen`), and stays on a comment's edit or deletion,
  a status and a push to its base (`updated-stays-on-comment-edit`,
  `updated-stays-on-comment-delete`, `updated-stays-on-head-status`,
  `updated-stays-on-base-push`): so a pull request's CI and base are
  read on a backoff (`../domain/forge.md`, 5.2).
- **Bodies and their versions.** A marker's line feed is kept
  (`marker-line-feed-kept`); bodies of 64 KiB and 1 MiB are kept whole
  (`body-of-65536-bytes-kept-whole`, `body-of-1048576-bytes-kept-whole`).
  An unedited comment's or review's updated time is its creation's
  (`unedited-comment-updated-equals-created`,
  `unedited-review-updated-equals-submitted`), and an edit moves it
  (`edit-moves-comment-updated`). An issue's comments and a pull
  request's reviews come oldest first (`comments-come-oldest-first`,
  `reviews-come-oldest-first`); a dismissed review keeps its body
  (`dismissed-review-keeps-its-body`); a remark's `position` is its line
  (`remark-position-is-its-line`).
- **Pull requests.** `PATCH issues/{index}` edits a pull request's title
  and body (`patch-issue-edits-a-pull-request`); its `base.sha` is its
  base's tip (`base-sha-is-the-base-tip`). The lookup
  `pulls/{base}/{head}` takes a percent-encoded slash
  (`pull-for-accepts-encoded-slash`) and names the oldest of several
  pull requests, a closed one before a newer open one
  (`pull-for-names-the-newest-of-several`, *fails*). Opening one whose
  head adds nothing to its base answers `201` and opens it
  (`open-pull-no-changes-is-422`, *fails*); opening one for a head and
  base that have one open answers `409` (`open-pull-exists-is-409`).
- **Merges and updates.** A merge answers `200` with no body
  (`merge-answers-200-with-no-body`); one naming an old head `409`,
  `head out of date` (`merge-old-head-is-409-head-out-of-date`); one
  refused for a closed pull request, a conflict, a protection or a style
  `405` (`merge-closed-is-405`, `merge-conflict-is-405`,
  `merge-protected-is-405`, `merge-style-is-405`). An update refused for
  a conflict answers `409`, `merge failed because of conflict`
  (`update-conflict-is-409-merge-failed`).
- **Branches.** `branches/{branch}` takes a percent-encoded slash
  (`branch-path-takes-encoded-slash`). A branch is created at a full
  commit id (`create-branch-at-full-commit-id`), and creating one that
  exists answers `409` (`create-branch-exists-is-409`). Deleting a
  protected branch answers `403` (`delete-protected-branch-is-403`), and
  deleting one that is not there `500`, with an empty body
  (`delete-missing-branch-is-404`, *fails*).
- **Protection.** A write collaborator who is not an admin is refused
  protection reads, `403` (`writer-protection-reads-are-403`), and reads
  a branch's `effective_branch_protection_name` empty
  (`effective-protection-name-for-a-writer`, *fails*), which names the
  rule for an admin (`effective-protection-name-for-an-admin`): so
  adoption records protection as unknown, never absent
  (`../domain/forge.md`, section 4).
- **CI.** A combined status with no status counts 0
  (`combined-status-without-status-counts-0`), and `skipped` with
  `success` combines as passed (`skipped-and-success-combine-as-passed`).
  Actions report each job as a status on its head
  (`actions-report-each-job-as-a-status-on-its-head`), linking
  `<…>/actions/runs/<run number>/jobs/<job index>` and naming no attempt
  (`actions-status-link-is-runs-n-jobs-i`,
  `actions-status-link-names-no-attempt`); a run's number is its
  `index_in_repo` (`run-number-is-index-in-repo`), and its jobs come in
  index order (`run-jobs-come-in-index-order`).
- **Jobs and their logs.** A job's steps are numbered from 0, `Set up
  job` first and `Complete job` last
  (`job-steps-number-from-0-set-up-first-complete-last`); a failed
  workflow step is `failure`, and the completion step with it
  (`failed-workflow-step-and-completion-step-are-failure`). A step's log
  holds that step's lines alone (`step-log-holds-that-steps-lines-alone`),
  and a step past the last answers `404` (`step-past-the-last-is-404`).
  A range answers `206`, over a job's log and over a raw file
  (`job-log-range-answers-206`, `raw-range-answers-206`); a suffix range
  over a step's log, its last bytes, with a `Content-Range` naming where
  they start (`step-log-suffix-range-answers-206-with-its-last-bytes`,
  `step-log-suffix-content-range-names-its-first-byte`); over an empty
  log, nothing (`empty-log-suffix-range-answers-206-empty`).
- **Comparisons.** `files=false` lists no commit's files and leaves the
  comparison's own `files` empty, and keeps `total_commits` and the
  commits' ids (`compare-files-false-lists-no-commits-files`,
  `compare-files-false-keeps-its-own-files-empty`,
  `compare-files-false-keeps-total-commits-and-shas`);
  `verification=false` changes nothing the calls keep
  (`compare-verification-false-changes-nothing-kept`).
- **Access and errors.** A repository the token cannot read answers
  `404`, by path and by id (`unreadable-repository-by-path-is-404`,
  `unreadable-repository-by-id-is-404`); a moved repository's old path
  answers a redirect (`moved-repository-is-a-redirect`); a write to an
  archived repository `423` (`archived-write-is-423`); an empty title
  `422` (`empty-title-is-422`). No rate-limit headers are sent
  (`no-rate-limit-headers`).
- **Deliveries.** Each row of section 5's table arrives under its
  `X-Forgejo-Event` with the fields its hint reads
  (`delivery-issues-row`, `delivery-pull-requests-row`,
  `delivery-comments-row`, `delivery-reviews-row`, `delivery-push-row`,
  `delivery-create-delete-row`, `delivery-job-ends-row`,
  `delivery-cancelled-job-end-is-delivered`), with its repository's id
  (`delivery-carries-the-repository-id`); a hook's events read back as
  made (`delivery-subscribed-events-read-back`); the signature is
  HMAC-SHA256 in hexadecimal (`delivery-signature-is-hmac-sha256-hex`);
  refs are full, and a tag's told from a branch's
  (`delivery-refs-are-full-and-tags-are-told-apart`); a branch's
  deletion is delivered as a push too
  (`delivery-branch-deletion-sends-no-push`, *fails*). Forgejo delivers
  the actor's own writes (`delivery-echoes-the-actors-own-writes`), so
  the domain drops its echoes; it refuses to deliver to loopback by
  default, and delivers there when its settings name the address
  (`delivery-loopback-is-refused-by-default`,
  `delivery-loopback-is-allowed-when-named`); a failed delivery is not
  sent again (`delivery-failed-delivery-is-not-sent-again`).
- **What it settles after it delivers.** A pull request reads `mergeable`
  false while Forgejo checks it
  (`delivery-pull-reads-not-mergeable-while-checked`). In the recorded
  run, a pull request was still being checked when the push that moved
  its head or base was delivered, and settled within 30 s
  (`delivery-pull-is-checking-when-its-push-is-delivered`;
  `delivery-pull-is-settled-when-its-push-is-delivered`, *fails*); a
  job's commit status was written before its end was delivered
  (`delivery-job-end-follows-its-commit-status`;
  `delivery-job-end-precedes-its-commit-status`, *fails*). temper relies
  on neither order, only on each settling within `settle` of the move,
  30 s in the profile `small` (`../domain/forge.md`, 5.2).
- **The setup's, and git's.** What the setup relies on (Forgejo's
  command line, its admin routes, orgs, teams and org hooks, the runner)
  is setup.md, 13.3's; what git relies on of Forgejo (a renamed
  repository's redirect, a protected branch's refusal of a push) is
  git.md's (section 3 and 4.8), named in
  `tools/git-transcripts/facts.txt`.

## 14. Open questions

- **Long threads, read whole.** Each `Item` read of a participating
  object streams its whole thread, since Forgejo does not page comments.
  Its timeline is paged (`issues/{n}/timeline?since&page&limit`) but
  mixes every event with comments and pages by offset, which a deletion
  shifts; or the component could send `since`, the creation time of
  comment `after`, if the client named it. Default: the whole thread,
  its cost in time, not memory.
- **Dismissed reviews:** whether one counts as no verdict, or as the
  comment it was. Default: the comment it was.
- **Contents on demand.** A pull request's files and a comparison cost
  two reads per file, since `api::File` carries contents; an overlap
  needs paths only. Default: contents always read, as the domain asks;
  a read of paths alone would be the client's to name.
- **A file too large for the answer** fails its call as `TooLarge`, which
  makes an overlap unknown and a brief's section fail. Default: so; a
  side cut with a mark would need `api::File` to carry one.
- **Bodies past `body_bytes`** are carried empty. Default: so, the
  component counting them; a skein-json read that keeps a string's first
  bytes and digests the rest would let it keep their heads and keys.
- **An edit within a creation's second** reads as `Original`, so a person
  who edits temper's keyed comment within that second is not heard.
  Default: `Original` when the times are equal.
- **The merge style:** the repository's `default_merge_style`. A style a
  project names per repository (`../domain/forge.md`, section 4) would
  need `Write::Merge` to carry it. Default: the repository's.
- **An expired or revoked API token** reads as `Forbidden` on every call
  until the next start, which it stops (3.5). Default: so; the domain
  could tell it apart, so a person is told once.
- **What no fact records yet,** read from Forgejo's API description and
  sources alone (section 13):
  - that an issue's comments and a review's comments take no page and
    come whole however many there are (3.2, `Item` and `Remarks`);
  - that a job no runner took has no steps, and that its log, and a log
    Forgejo no longer keeps, answer `404` (3.2, "A failed job's log");
  - that the push workflow an update starts runs on exactly the head
    the update made, so CI is read again there (`../domain/forge.md`,
    section 10);
  - that a merge after its pull request's base was changed lands in the
    new base: the pull request is read afresh before a merge, and one
    retargeted since the decision is not merged, but nothing makes the
    merge itself conditional on its base (`../domain/forge.md`, section
    6).
- **GitHub,** later: nothing above `temper-forge-forgejo` and the call
  plans of 3.2 assumes Forgejo, so a provider is a crate of documents
  and a set of call plans, each call made of whatever requests it takes,
  its cost reported as Forgejo's is. What its API would add to the
  forge client's vocabulary is open: validators for conditional
  requests, which GitHub does not count against its rate; reads batched
  into one query; the rate's window, reported on every answer.
