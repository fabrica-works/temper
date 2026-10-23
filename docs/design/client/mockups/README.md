# Mockups of temper's client

These mockups are reference material for temper's client (`../README.md`): its look and the order of its pages' parts, not its contract. Where they and the client's documents differ, the documents hold.

Serve this directory and open the contents page:

```sh
python3 -m http.server 8765 --directory docs/design/client/mockups
# http://localhost:8765/index.html
```

Each file also opens directly in a browser, with no build step; serving it keeps the chosen world, step and switches from page to page. The only external asset is the optional Google Fonts stylesheet; system fonts are used if it cannot load.

## The pages

| Page | File | Client README, 3.5 |
|---|---|---|
| The contents: the worlds, the step, where to look | `index.html` | not a page of the client |
| What needs the person, then updates | `inbox.html` | `Inbox` |
| The person's chats | `chats.html` | `Chats` |
| A chat: its chatbar, conversation, delegates' cards, proposals, composer (`?task=T1`) | `chat.html` | `Chat { task }` |
| The project's goals | `board.html` | `Board { project }` |
| Every task, as one tree | `tasks.html` | none: jig's `client/domain/pages.md`, 6.3 |
| A task: a goal's plan, an agent's transcript, a change's stages (`?task=T12`) | `task.html` | `Task { task }` |
| The changes of each repository, with their queues and the open findings | `changes.html` | `Changes { project }` |
| The person’s forges: each one’s connection, access and load, and its acts, each folded; the playground’s | `forges.html` | `Forges` |
| Connecting a forge: choose, the details, setting up, ready (?forge=N) | `connect-forge.html` | `ConnectForge` |
| The workers, their slots and health | `workers.html` | none: jig's `client/domain/pages.md`, 7 |
| The person’s LLM accounts: each one’s sources in order, what it pays for, who uses it or waits for it, and its acts, each folded; a sign-in under way; the playground’s | `accounts.html` | `Accounts` |
| Connecting a model: choose, the details, signing in, ready (`?way=chatgpt\|key`) | `connect-model.html` | none yet: drawn ahead, as `ConnectForge` was |
| The welcome: try the playground, or set up temper | `welcome.html` | `Welcome`, drawn ahead (below) |
| The first run, at its first step: the link `temper box` prints | `first-run.html` | opens `welcome.html` at step 0 in the first run |

"Adopt a repository" is in the navigation, in the order of the client README's 5.3, but has no mockup (4.4).

## A world, at a step

- **The worlds.** `world-reference.js` holds the reference case (`../../draft/reference-case.md`): a director, owners on two machines, pools, findings, an ops application, at hour 0, with the steps that move it on. `world-solo.js` holds a workflow unlike it: one developer, one repository, one laptop, chats and a goal set by hand, no coordinator, timers, pools or ops application. `world-first-run.js` holds a person's first run after `temper box`, from the welcome through the demo and its end (below). `world-first-run-real.js` follows the welcome’s real setup, without starting the demo. A page that reads wrong in one world assumed another's workflow.
- **The step.** `pages.js` draws every page from the world at the step in the address (`?world=`, `?step=`), so the pages agree at each step. The dark bar above each page is the mockups' own: ‹ and › move the step, **Steps** lists the whole case, the step's title opens what it shows and where to look, and **View** switches the world and what is marked.
- **What is drawn:** the reference case's hour 0, the solo world's one step, and the first run’s eight and real setup’s five. The case's later steps are listed, and are added one act at a time.
- **Acts:** a button or link for an act shows a note instead of acting, unless a later step draws what it does: then it leads to that step (the first run's suggested requests). Nothing is sent anywhere.
- **For a capture:** `?bare` draws the page alone, without the bar and the marks, and every link keeps it (`chat.html?world=solo&task=T10&bare`). A page's state is its address: the world, the step and the task. `?project=` shows another project; the chats, tasks, board and changes pages follow it.

## Where each value comes from

temper is a generic engine, so its pages may not assume a workflow. Every value a page shows comes from a named source, and `pages.js` draws each through one helper per source:

| Source | What it is | Examples |
|---|---|---|
| Engine fact | what jig or temper keeps or computes | a phase; what a task waits on; a wake policy's next time; a change's stage; slots in use |
| Configuration | a name the deployment's configuration gives | a template (`chat`, `goal`); a charter (`Coordinator`); a host kind; the period and allotment |
| As written | words a person or an agent wrote, shown as written, with the writer | a title; a brief; a reply, rendered from Markdown; a note; a reason in a task's history |
| Ops application | what a later application reports, beside temper's facts and never as them | a worker's load and disk |

What a task waits on is jig's, computed by the engine: a party, its delegates, news. Where it is news, the page narrows it from the task's wake policy, mechanically: *your words* when a party's words wake a task that party requested (a chat at rest), *T12's end* for a subscription to another task's state, *until 09:05* for a timer alone. *Waiting on your words* is not *needs you*: nothing is held up, and only an entry (a question, a proposal, an escalation, a person task) is in the inbox, where its task waits on *an answer from you*. temper's client words these statuses its own way ("Waiting for you", "Parked"; the client README, 3.5).

Anything else on a page is the UI's own words, and those name no workflow: the director, an owner or a check appear only in what was written or configured. The inbox, for example, says "Nothing needs you." and stops; "Next check at 09:10" appears only where the director wrote it, in its reply. Its updates are derived, as jig's inbox derives them: replies in the person's chats, and results of the tasks the person requested.

**View** has two switches. **Annotations** (on by default) shows the case's names beside temper's numbers (`B`, `f1`) and the gaps of temper-on-temper.md's section 6 that a page draws ahead of temper (`gap 6.4`), all in slate, never the product's palette. **Sources** (off by default) underlines each value in its source's colour, and hovering one names where it comes from.

## Drawn ahead of temper

What temper does not have yet, drawn as temper-on-temper.md's section 6 wants it and marked with its gap while **Annotations** is on:

- **6.3, plan usage:** a program's 5-hour and weekly windows on the accounts page, the week with its pace; the director's check reads them. A ChatGPT plan shows none: OpenAI reports no windows to temper, so its row shows its limit's state and "Manage usage ↗", which opens ChatGPT's settings.
- **6.4, worker health:** the ops strip on the workers page, an ops application's report of load and disk.
- **6.5, open findings:** a finding on a goal's page, the board and the changes page, where a failure that matches an open finding attaches to it and leaves the queue running.
- **6.7, fleet occupancy:** a worker's free slots, and the director's read of them.

Marked "ahead" (slate, with the annotations): what the owned Forgejo’s rows and the first run draw before temper has a mission for it. **B4 M4:** the set-up’s first repository, and "New repository", made by temper with the admin’s token once temper-15 moves that token into the engine. **jig:** a project made after the first start; jig makes projects only at a deployment’s first start. **welcome:** the first run’s welcome and an explicitly chosen demo, with no mission yet. **demo’s end:** retirement after the first personal forge connects, with no mission yet. **sign-in:** "Open Forgejo" signing the person in through temper, as Forgejo’s OpenID provider; until then, the person signs in with the password the preparation wrote. **display:** an account's email, "ChatGPT · sam@example.com"; jig carries no display yet, and the client README's 5.5 drops it from the heading until it does. **callback page:** the tab ChatGPT sends the browser back to, which the engine's loopback listener answers; skein's `oauth.md` draws none. **words:** a sign-in's endings that 5.5 has no words for: a subject another person holds, cancelled or expired, and the port taken. **sketch:** Claude's subscription in Connect a model, in outline. **limited chat:** where a chat shows its account's limit.

Two pages follow jig's pages, which temper's client leaves out (the client README, 5.3 and 11): **Tasks**, every task of the project as one tree (jig's `client/domain/pages.md`, 6.3), and **Workers**, the deployment's hosts (7).

Not drawn: two ideas that would change temper's engine, not only its pages. A charter or template would name the pages where its tasks lead, so the board could open with the director's line; an agent would set a short status line on its task with a tool, shown as written beside its phase. They wait for a design of their own.

## The first run

`world-first-run.js` is the deployment `temper init` writes (`../../protocol/setup.md`, 8.2 and 8.3): the project Default, the charters `chat` and `coding`, one worker in temper's process, three forges (the demo's fake, GitHub not connected, a Forgejo here not set up), and no account, repository or forge started yet. Every word an agent says is the demo's script (8.4). [first-run.html](first-run.html), the address the documents cite, opens it at its first step. Its steps:

0. **The link opened:** the welcome offers Try the playground or Set up temper. Chats behind it has the setup strip, no model and a disabled send button.
1. **The demo chosen:** the engine asks temper box for the demo’s fakes, adds the Scripted model’s account and adopts playground/widget into Default. The chats page is the demo's start (client README, 5.6): the banner under the navigation, as on every page; a chatbar with no task and no number; the context line; the three suggested requests.
2. **A request sent:** "Add CSV export to widget" starts the chat T1; its agent delegates the change T2, whose agent T3 edits `src/main.rs`. T2's card says Running with no step ticked (5.7).
3. **The change landed:** T2's card says Landed, every step ticked, "CI passed. Landed · merge commit e3b8d41"; T1 says so in its script's third turn.
4. **A text-only request:** "Add a dark theme to widget", the chat T4, answered in words; its script's `finish` ends only the turn (setup.md 8.4), so T4 waits at rest for Sam's next words, its composer open, as T1 does.
5. **A model connected:** Sam follows the banner's Connect a model, chooses ChatGPT and signs in as sam@example.com; the flow ends on Ready, with Start a chat and Manage usage. The Scripted model leaves Accounts, where ChatGPT · sam@example.com is the one row. The model's half of the demo ends (4.8): the banner says a playground forge alone, the chats page has no demo's start, and the composers' line reads "Using ChatGPT plan · gpt-6-luna · Manage usage". The galleries below Connect a model, Accounts and Chats draw the rest of the ChatGPT experience (below).
6. **Setting up a Forgejo:** chosen in the flow, its defaults kept (work/hello, a new project hello); the box has fetched and is preparing. The flow shows each step, and the banner says where it is.
7. **A Forgejo connected:** temper connected as its bot, made work/hello and adopted it into the new project hello; its first CI run passed. The flow ends on the new project, opened with no chat yet. The demo is retired: forge 1 stops and disappears, T1 and T4 end at 21:19 with the reason, and playground/widget leaves Default’s repository lists. Its landing and transcripts stay readable; Recent landings names it “playground/widget · the playground’s, ended”. The banner is gone.

## The first run, set up

`welcome.html` has a bare topbar (temper, Sam, Sign out), and Welcome → Forge → Model → Ready. Its default view follows the deployment: the welcome before any forge is acted on, Model while a forge sets up without a model, then Ready. Once the demo runs it shows a mock-note linking to Chats. `?at=welcome|forge|model|ready` selects a view; `?forge=2|3|elsewhere` keeps the forge’s details inside the wizard. The forge cards and setup pieces are shared with `connect-forge.html`. Later opens Chats; trying the demo explicitly switches to first-run step 1.

`world-first-run-real.js` reuses the first-run world’s fresh `initial`:

1. **The link opened:** Sam chooses Set up temper, then Forgejo on this machine, with its defaults.
2. **Setting up a Forgejo:** the box fetches Forgejo and its runner; Model opens while that proceeds, with an Each step disclosure: ChatGPT or an OpenAI API key, as the default document offers them. Sam chooses ChatGPT, whose details say what plan usage means and where to sign in. The gallery sketches Claude's subscription, where a configuration offers it.
3. **Signing in to ChatGPT:** the sign-in's address, to open in a browser on the engine's machine, and its deadline; Cancel. Accounts shows the sign-in under way, with the nav's amber mark; the setup strip shows its status beside the forge's.
4. **Signed in:** ChatGPT · sam@example.com is the only account, and Almost ready waits while the forge starts. Open hello is disabled.
5. **Ready:** the forge is connected as temper; work/hello is adopted into hello with its first CI passed. Open hello opens that project with no chat yet. The demo, never started, is retired and nothing of it shows.

The person meets the demo as **the playground** (with the user, 2026-10-23): "Try the playground", the banner "This is the playground: a scripted model and a playground forge.", Scripted model, Playground forge and playground/widget; the scripts say "Playground forge" and "a scripted change". The design keeps the demo as its concept's name (`application.demo`, the demo's halves); setup.md 8.4 holds the scripts' words, and the client README 5.6 the page's (`../../draft/first-run.md`, 2.3).

The demo’s explicit states are `not started`, `running`, and `retired`; an absent state retains the older worlds’ running behavior. Its model and forge halves and context draw only while running. The first usable account from another provider ends the model half and removes the Scripted model’s account; the first connected forge of the person’s retires the demo’s forge and repository and ends its open chats, keeping history.

While the demo is not started and a forge or model is missing, the setup strip takes the demo strip’s place. It states what is missing, shows any forge setup status with View, and links to Continue setting up and (until a personal forge connects) Try the playground. Welcome, Accounts, Forges, Connect a forge and Connect a model omit it. With no account, a composer says “No model yet · Connect a model” and cannot send.

Where the documents disagree, or leave a word open, the mockup takes one side and says so here:

- **T1's next run takes the new account** (step 5), where temper's stays on the demo model. The draft says the demo model "stays the demo project's" (`../../draft/first-run.md`, 2.2). setup.md 8.3 says a task with no model chosen takes the first of its charter's models that has a usable account, and the demo model is last in both charters. jig resolves that default once, at the task's Create, and records it, so a new chat's first run takes gpt-6-luna on the ChatGPT plan while T1, created before it, keeps the demo model. The mockup draws T1 on gpt-6-luna too: re-resolving a task's default for each run is jig's to decide.
- **The coding agent's title is the change's** (T3, "Export widget's table as CSV"): setup.md 8.4 makes the change's title its agent's brief, and a chip is the spec's first line. The other worlds title that agent "Produce TN's branch", a stand-in for a title temper's procedure does not word yet.
- **The dark theme's words** are a stand-in. Their content is the demo's follow-up (`../../draft/first-run.md`, section 8). The empty search and the fallback are not drawn: they take step 4's shape.
- **"Connect a forge" is a link,** to `connect-forge.html`. The client has no `ConnectForge` yet, so until it does the button is disabled (client README, 5.6).
- **The forges page** shows only the person’s forges (connecting, setting up, set up failed, connected, stopped), each folded until opened, and the playground’s. A forge not acted on is a choice in the “Connect a forge” flow: Choose → Details → Setting up (Connecting for GitHub) → Ready, ending on the new project. The banner shows the set-up’s status and a link to it; the Forges nav item’s dot is amber while setting up or connecting, red when a forge needs the person. The page and the flow are forge.md 5.4's. Mockup-only galleries on both pages compare the flow’s other paths and the other forge rows, including managed and connected Forgejo. The flow’s “A Forgejo elsewhere” includes the setup entry that connecting as its admin needs.
- **Accounts and Connect a model** follow the forges' shape (with the user, 2026-10-23). The page stays Accounts; the flow and every entry point say "Connect a model": the banner, the composer's "No model yet", the page's "+ Connect a model". The flow is Choose → Details → Signing in → Ready, and the welcome's Model reuses its Choose, Details and Signing in. ChatGPT is drawn whole; an API key's form and Claude's subscription are there for the context, not designed. Each account is a folded row: its name and holder, its deciding source's state, and a line of its models, what it pays for and its tasks; opened, the state's words and acts from 5.5's table, its sources in order, Pays for, Used by, Waiting for it, and its acts. The Accounts nav item's dot is amber while a sign-in is under way or an account is limited, red while one needs the person. Mockup-only galleries draw, below Connect a model, the sign-in waiting, the browser's tab, and each other ending; below Accounts, the ChatGPT row in each state of 5.5's table, and a key's and a Claude subscription's rows; below Chats, a plan's first-use card and a chat whose plan is limited.
- **A plan pays for its holder's own work, and for projects where the holder is the only member;** a key or a program's login, with no uses given, for its holder's own work. A limited plan's tasks wait for it; where the account also holds a key, "Use the API key first" reorders its sources, and never moves a task to another account.
- **The sign-in is from a browser on the engine's machine.** ChatGPT's registration fixes the redirect at `127.0.0.1:1455`, so a browser elsewhere needs `ssh -L 1455:127.0.0.1:1455 HOST`, folded under the details and the address; Codex's own sign-in takes the same port.
- **A key reads Ready until a run is refused:** nothing checks it when it is added, so a wrong key shows only at its first request. The key's form says so, as a note.

The earlier first-run mockup, a story of its own, also drew "Reset demo", a turn's "Just now", the composer's placeholder, "Landed into main" and the pull request's link on the card. The client leaves each out (client README, section 11), and so does this world.

## The files

- `style.css`, `terminal-chat.css` and `terminal-pages.css`: the product's look, from which `temper.css` takes its rules (`../browser.md`, 2.5).
- `mockup.css`: the mockups' own marks and bar, never the product.
- `pages.js`, `world-reference.js`, `world-solo.js`, `world-first-run.js` and `world-first-run-real.js`: every page and its worlds; each HTML file is a shell that loads them, and `first-run.html` opens the first run.
