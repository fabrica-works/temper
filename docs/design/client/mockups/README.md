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
| The person’s forges: each one’s connection, access and load, and its acts; the demo’s, folded | `forges.html` | `Forges` |
| Connecting a forge: choose, the details, setting up, ready (?forge=N) | `connect-forge.html` | none: drawn ahead (below) |
| The workers, their slots and health | `workers.html` | none: jig's `client/domain/pages.md`, 7 |
| The LLM accounts, their sources and plans' windows | `accounts.html` | `Accounts` |
| The first run, at its first step: the link `temper box` prints | `first-run.html` | opens `chats.html` in the first run |

"Adopt a repository" is in the navigation, in the order of the client README's 5.3, but has no mockup (4.4).

## A world, at a step

- **The worlds.** `world-reference.js` holds the reference case (`../../draft/reference-case.md`): a director, owners on two machines, pools, findings, an ops application, at hour 0, with the steps that move it on. `world-solo.js` holds a workflow unlike it: one developer, one repository, one laptop, chats and a goal set by hand, no coordinator, timers, pools or ops application. `world-first-run.js` holds a person's first run after `temper box`, on the demo's model and forge (below). A page that reads wrong in one world assumed another's workflow.
- **The step.** `pages.js` draws every page from the world at the step in the address (`?world=`, `?step=`), so the pages agree at each step. The dark bar above each page is the mockups' own: ‹ and › move the step, **Steps** lists the whole case, the step's title opens what it shows and where to look, and **View** switches the world and what is marked.
- **What is drawn:** the reference case's hour 0, the solo world's one step, and the first run's seven. The case's later steps are listed, and are added one act at a time.
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

- **6.3, plan usage:** each source's 5-hour and weekly windows on the accounts page, the week with its pace; the director's check reads them.
- **6.4, worker health:** the ops strip on the workers page, an ops application's report of load and disk.
- **6.5, open findings:** a finding on a goal's page, the board and the changes page, where a failure that matches an open finding attaches to it and leaves the queue running.
- **6.7, fleet occupancy:** a worker's free slots, and the director's read of them.

Marked "ahead" (slate, with the annotations): what the owned Forgejo’s rows draw before temper designs it. **client:** the flow as a page of its own. **B4 M4:** the set-up’s first repository, and "New repository", made by temper with the admin’s token once temper-15 moves that token into the engine. **jig:** a project made after the first start; jig makes projects only at a deployment’s first start. **sign-in:** "Open Forgejo" signing the person in through temper, as Forgejo’s OpenID provider; until then, the person signs in with the password the preparation wrote.

Two pages follow jig's pages, which temper's client leaves out (the client README, 5.3 and 11): **Tasks**, every task of the project as one tree (jig's `client/domain/pages.md`, 6.3), and **Workers**, the deployment's hosts (7).

Not drawn: two ideas that would change temper's engine, not only its pages. A charter or template would name the pages where its tasks lead, so the board could open with the director's line; an agent would set a short status line on its task with a tool, shown as written beside its phase. They wait for a design of their own.

## The first run

`world-first-run.js` is the deployment `temper init` writes (`../../protocol/setup.md`, 8.2 and 8.3): the project Default, the charters `chat` and `coding`, one worker in temper's process, three forges (the demo's fake, GitHub not connected, a Forgejo here not set up), and the demo model's key as the one account. Every word an agent says is the demo's script (8.4). [first-run.html](first-run.html), the address the documents cite, opens it at its first step. Its steps:

1. **The link opened:** the chats page is the demo's start (client README, 5.6): a chatbar with no task and no number, the banner, the context line, the three suggested requests.
2. **A request sent:** "Add CSV export to widget" starts the chat T1; its agent delegates the change T2, whose agent T3 edits `src/main.rs`. T2's card says Running with no step ticked (5.7).
3. **The change landed:** T2's card says Landed, every step ticked, "CI passed. Landed · merge commit e3b8d41"; T1 says so in its script's third turn.
4. **A text-only request:** "Add a dark theme to widget", the chat T4, answered in words; then its script's `finish` ends it, so its page has no composer.
5. **A model connected:** an OpenAI API key added. The model's half of the demo ends (4.8): the banner says a demo forge alone, the chats page has no demo's start, and the composers' line names the new key.
6. **Setting up a Forgejo:** chosen in the flow, its defaults kept (work/hello, a new project hello); the box has fetched and is preparing. The flow shows each step, and the banner says where it is.
7. **A Forgejo connected:** temper connected as its bot, made work/hello and adopted it into the new project hello; its first CI run passed. The flow ends on the new project, opened with no chat yet. The banner is gone.

Where the documents disagree, or leave a word open, the mockup takes one side and says so here:

- **A text-only answer ends its chat** (step 4). setup.md 8.4 scripts the dark theme, the empty search and the fallback as one turn: words, then `finish` with the report "The demo changes nothing.", since a text-only stop is nudged and then fails the run. Finishing ends the chat, so the person cannot reply, and the page they are on loses its composer just after they asked. This is likely a flaw: a chat at rest should wait for the person's words, as the CSV's does with `wait`. The mockup draws the script as written.
- **T1's next run takes the new account** (step 5), where temper's stays on the demo model. The draft says the demo model "stays the demo project's" (`../../draft/first-run.md`, 2.2). setup.md 8.3 says a task with no model chosen takes the first of its charter's models that has a usable account, and the demo model is last in both charters. jig resolves that default once, at the task's Create, and records it, so a new chat's first run takes gpt-6-luna on the new key while T1, created before it, keeps the demo model. The mockup draws T1 on gpt-6-luna too: re-resolving a task's default for each run is jig's to decide.
- **The coding agent's title is the change's** (T3, "Export widget's table as CSV"): setup.md 8.4 makes the change's title its agent's brief, and a chip is the spec's first line. The other worlds title that agent "Produce TN's branch", a stand-in for a title temper's procedure does not word yet.
- **The dark theme's words** are a stand-in. Their content is the demo's follow-up (`../../draft/first-run.md`, section 8). The empty search and the fallback are not drawn: they take step 4's shape.
- **"Connect a forge" is a link,** to `connect-forge.html`. The client has no forges page yet, so until it does the button is disabled (client README, 5.6).
- **The forges page** shows only the person’s forges (connecting, setting up, set up failed, connected, stopped), and the demo’s, folded. A forge not acted on is a choice in the “Connect a forge” flow: Choose → Details → Setting up (Connecting for GitHub) → Ready, ending on the new project. The banner shows the set-up’s status and a link to it; the Forges nav item’s dot is amber while setting up or connecting, red when a forge needs the person. The rows draw forge.md 5.4 with an Account line, an Access line in place of the mode, and an owned forge’s Runs line. Mockup-only galleries on both pages compare the flow’s other paths and the other forge rows, including managed and connected Forgejo. The flow’s “A Forgejo elsewhere” includes the setup entry that connecting as its admin needs.

The earlier first-run mockup, a story of its own, also drew "Reset demo", a turn's "Just now", the composer's placeholder, "Landed into main" and the pull request's link on the card. The client leaves each out (client README, section 11), and so does this world.

## The files

- `style.css`, `terminal-chat.css` and `terminal-pages.css`: the product's look, from which `temper.css` takes its rules (`../browser.md`, 2.5).
- `mockup.css`: the mockups' own marks and bar, never the product.
- `pages.js`, `world-reference.js`, `world-solo.js` and `world-first-run.js`: every page and its worlds; each HTML file is a shell that loads them, and `first-run.html` opens the first run.
