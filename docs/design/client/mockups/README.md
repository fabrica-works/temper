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
| The workers, their slots and health | `workers.html` | none: jig's `client/domain/pages.md`, 7 |
| The LLM accounts, their sources and plans' windows | `accounts.html` | `Accounts` |
| The owner's first chat after `temper box` | `first-run.html` | `Chats`, `Chat { task }` |

"Adopt a repository" and "Forges" are in the navigation, in the order of the client README's 5.3, but have no mockup (4.4; `forge.md`, 5.4).

## A world, at a step

- **The worlds.** `world-reference.js` holds the reference case (`../../draft/reference-case.md`): a director, owners on two machines, pools, findings, an ops application, at hour 0, with the steps that move it on. `world-solo.js` holds a workflow unlike it: one developer, one repository, one laptop, chats and a goal set by hand, no coordinator, timers, pools or ops application. A page that reads wrong in one world assumed the other's workflow.
- **The step.** `pages.js` draws every page from the world at the step in the address (`?world=`, `?step=`), so the pages agree at each step. The dark bar above each page is the mockups' own: ‹ and › move the step, **Steps** lists the whole case, the step's title opens what it shows and where to look, and **View** switches the world and what is marked.
- **What is drawn:** the reference case's hour 0, and the solo world's one step. The case's later steps are listed, and are added one act at a time.
- **Acts:** a button or link for an act shows a note instead of acting. Nothing is sent anywhere.
- **For a capture:** `?bare` draws the page alone, without the bar and the marks, and every link keeps it (`chat.html?world=solo&task=T10&bare`). A page's state is its address: the world, the step and the task.

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

Two pages follow jig's pages, which temper's client leaves out (the client README, 5.3 and 11): **Tasks**, every task of the project as one tree (jig's `client/domain/pages.md`, 6.3), and **Workers**, the deployment's hosts (7).

Not drawn: two ideas that would change temper's engine, not only its pages. A charter or template would name the pages where its tasks lead, so the board could open with the director's line; an agent would set a short status line on its task with a tool, shown as written beside its phase. They wait for a design of their own.

## first-run.html

[first-run.html](first-run.html) and `first-run.js` are the owner's first chat after opening the link from `temper box`, a story of their own rather than a world: the demo's start with its suggested requests, the demo's banner, a delegation's card with its steps, on sample model and forge connections. The client README cites it for the demo (4.8, 5.6) and for what the view leaves out (section 11). It loads no external assets, and **Reset demo** or reloading clears its local state.

## The files

- `style.css`, `terminal-chat.css` and `terminal-pages.css`: the product's look, from which `temper.css` takes its rules (`../browser.md`, 2.5).
- `mockup.css`: the mockups' own marks and bar, never the product.
- `pages.js`, `world-reference.js` and `world-solo.js`: every page but first-run, and its worlds; each HTML file is a shell that loads them.
