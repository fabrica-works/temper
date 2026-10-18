# LLMs: setting them up and using them

Draft, 2026-10-18. How a person makes temper's agents run on their LLM
subscriptions and keys, and what they see while they do: the
configuration, signing in from the client, Claude through Claude Code,
several subscriptions, choosing models, the commands, and live tests on
a subscription. jig's `draft/llm-accounts.md` is the model it follows,
and jig's `draft/providers.md` the providers' rules. What changes in
temper's design is section 10; what is open, section 11.

## 1. In one page

- **One person, their subscription, nothing else to set.** A person
  alone on their machine signs in once, and every run of theirs, chats,
  goals and the forge's automatic work, uses that subscription (jig's
  `draft/llm-accounts.md`, 4.2).
- **The configuration names providers, never accounts.** Endpoints,
  models, each provider's sign-in under temper's name, and the programs
  the agents may run: no account, no key, no token (section 2).
- **ChatGPT from the client:** "Continue with ChatGPT", the browser,
  and a one-time notice; "Using ChatGPT plan" wherever a run on it is
  shown (section 3).
- **Claude through Claude Code:** the person declares their Claude
  subscription in the client, under a name of theirs, and signs Claude
  Code in with `temper program-login`, which runs Claude Code's own
  flow in a directory of temper's; temper never holds that login
  (section 5).
- **Keys from the client,** for OpenAI, Anthropic or any provider the
  configuration names, held by the person or, by an owner, by the
  deployment (section 4).
- **Several subscriptions with one provider,** each an account of its
  own, in the person's order; a task keeps its account, and waits for
  it at a limit (section 6).
- **Models are chosen per task:** a chat's model in its composer, a
  goal's planner's, and each task the planner delegates, within what
  the person's role allows (section 7).
- **Live tests on a subscription:** Claude Code signed in once in the
  live tier's own directory, unattended; ChatGPT signed in as each live
  run starts; unattended ChatGPT runs wait for jig's move of an account,
  later (section 9).
- **What goes:** `temper sign-in`, handed-in accounts and the borrowed
  Codex login (section 10).

## 2. The configuration

jig's `engine.llm` becomes its providers (jig's `draft/llm-accounts.md`,
3.1), written out whole in temper's document; the shape below is jig's
to settle, and shows what a person writes:

```
"llm": {
  "providers": [
    {
      "number": 1,
      "name": "OpenAI",
      "endpoints": [{"name": "openai", "dialect": "openai_responses",
                     "address": {"host": "api.openai.com", "port": 443},
                     "transport": "tls", "server_name": "api.openai.com",
                     "path": "/v1/responses"}],
      "models": [{"name": "gpt-6.1-sol", "endpoint": "openai", "window": 272000, "output": 32000,
                  "efforts": null, "prices": {...},
                  "about": "general model for coding and planning"}],
      "sign_in": {"display": "ChatGPT", "plan": true,
                  "manage_usage": "https://chatgpt.com/settings/usage",
                  "registration": {"entry_client_id": "dynamic_agent_client", "agent_name": "temper",
                                   "authorization_url": "https://auth.openai.com/api/accounts/authorize",
                                   "token_url": "https://auth.openai.com/api/accounts/oauth/token",
                                   "revocation_url": ...,
                                   "scopes": ["openid", "profile", "email", "offline_access",
                                              "resource.invoke", "chatgpt.tokens.use.direct"],
                                   "required_scopes": ["chatgpt.tokens.use.direct"],
                                   "resource": "https://api.openai.com/v1",
                                   "redirect": {"port": 1455, "path": "/auth/callback"}}},
      "keys": true,
      "programs": []
    },
    {
      "number": 2,
      "name": "Anthropic",
      "endpoints": [{"name": "anthropic", "dialect": "anthropic_messages", ...}],
      "models": [{"name": "claude-opus-5-5", "endpoint": "anthropic", ...}],
      "sign_in": null,
      "keys": true,
      "programs": [{"adapter": "claude_code", "display": "your Claude subscription, through Claude Code"}]
    }
  ]
}
```

- **Written out, never implied.** `agent_name` is temper's name, the
  same in every deployment; the rest is the provider's, copied from
  `docs/examples/owned.json`, which ships both providers.
- **A provider temper does not ship** is another entry: a key and a
  dialect skein speaks, or a local model with an endpoint on loopback
  and no source to hold, whose one account is the deployment's (jig's
  `draft/llm-accounts.md`, 3.1).
- **The agent's configuration** (`../protocol/configuration.md`, section 5) gains
  each program's path and the directory that holds its configuration
  directories: `"programs": {"claude_code": {"path":
  "/usr/local/bin/claude", "directory": "/home/me/temper/programs"}}`.
  A program source's directory is `<directory>/<its name>` (section 5),
  private, made by `temper program-login`.
- **Charters** allow models rather than fix one (jig's
  `draft/llm-accounts.md`, 5.1): the chat's charter allows every
  configured model, and its default is the first model, in the
  charter's order, for which the person has a usable account.

## 3. A first run

1. **Start and open,** as `temper box CONFIG.json` and `temper open
   DIRECTORY LINK` do (`../protocol/setup.md`, section 7;
   `../client/README.md`, section 7).
2. **The banner,** on every page while no account is usable:

   > Agents need an LLM account to run.
   > [Continue with ChatGPT] [Use your Claude subscription] [Use an API key]

   A chat may be started meanwhile; its page says "starting: waiting
   for an account".
3. **Continue with ChatGPT.** The client shows the sign-in's address,
   as a link the terminal prints and the browser opens. The person
   approves in ChatGPT's page; ChatGPT redirects to the engine on
   `127.0.0.1`, on the person's machine; the client says "Signed in to
   ChatGPT as alice@example.com".
4. **The first-use notice,** once, dismissable:

   > **You're using your ChatGPT plan.** Eligible requests from temper
   > use your plan's included usage. You can see it, and set a weekly
   > cap for temper, in ChatGPT's settings.
   > [Manage usage] [Got it]

5. **Working.** The chat's composer line reads "Using ChatGPT plan ·
   gpt-6.1-sol · Manage usage", and the chat's run starts.

Nothing else is asked: a plan pays for its holder's own work and for
the automatic work of the projects in which they are the only party,
which on one person's machine is all of it.

## 4. Screens

### 4.1 Accounts

The system page's "LLM accounts" (jig's `client/domain/pages.md`,
section 7) becomes the person's accounts, and, for an owner, the
deployment's:

- **Each account:** its provider and display name ("ChatGPT ·
  alice@example.com"), each source and its state, the runs using it and
  the tasks waiting for it.
- **Its actions:** sign out; remove; up and down in the person's order,
  and its sources' order; resume, when limited; check again, for a
  program's login; add a key to it.
- **Adding:** [Continue with ChatGPT]; [Use your Claude subscription];
  [Use an API key]: a provider, then the key, pasted once and never
  shown again, as an account of its own unless the person adds it to
  one of theirs; "billed to your API organization".

### 4.2 What each state says

| State | Words | Actions |
|---|---|---|
| limited, a plan | "temper reached its usage limit on your ChatGPT plan. Tasks on it wait." | [Manage usage] [Resume] |
| limited, with a time | "Limited by OpenAI until 14:00. Tasks on it wait." | — |
| plan usage declined | "You signed in without plan usage." | [Enable plan usage] [Use an API key] |
| not eligible | "This ChatGPT account can't be used for plan usage in temper. Plan usage needs an eligible ChatGPT plan." | [Use an API key] [Sign in to another account] |
| needs sign-in | "Signed out of ChatGPT." | [Continue with ChatGPT] |
| a program's login, on a host | "Claude Code isn't signed in for 'claude-alice' on this machine." and the command (section 5) | [Check again] |
| misconfigured | "OpenAI refused temper's sign-in. Check the configuration's registration." | — |

- **Never a reset time a provider did not give,** and never "your plan
  is empty" (jig's `draft/providers.md`, 2.5).
- **A limit may be temper's alone:** the person's weekly cap for temper
  in ChatGPT's settings, which [Manage usage] reaches.
- **A limited plan holds its tasks,** even where the account also has a
  key: the person moves them to the key by putting it first, never
  temper (jig's `draft/llm-accounts.md`, 3.4).

### 4.3 Where the account shows

- **The composer,** in a chat (jig's `client/domain/pages.md`, section 4): "Using
  ChatGPT plan · Manage usage"; "Using your Claude subscription,
  through Claude Code"; "Using your OpenAI API key".
- **The task page** (jig's `client/domain/pages.md`, section 5): "runs on", the model,
  the effort and the account, each changeable by an amendment
  (section 7).
- **A held task** whose account is unusable says so, with the account's
  actions (jig's `client/domain/objects.md`, section 5).

### 4.4 Signing out

- "Signed out of ChatGPT." when the revocation is confirmed;
- "temper removed its tokens, but ChatGPT didn't confirm. Disconnect
  temper in ChatGPT's settings, under Apps." otherwise
  (jig's `draft/providers.md`, 2.3);
- for Claude Code, "Signed out on this machine." once each host has run
  Claude Code's own sign-out, naming any that did not.

## 5. Claude through Claude Code

- **[Use your Claude subscription]** asks a short name for it,
  `claude-alice` by default (1 to 32 of `a-z`, `0-9` and `-`, never
  reused), and makes the account: the person its holder, Claude Code its
  source (jig's `draft/llm-accounts.md`, 3.4). The page then shows the
  command to run on this machine:

  ```
  temper program-login CONFIG.json claude-alice
  ```

- **`temper program-login CONFIG.json NAME`** reads the agent's
  configuration that the document names, makes `<directory>/NAME`
  private if it is missing, and replaces itself with the program's own
  sign-in, here Claude Code's `claude auth login` with
  `CLAUDE_CONFIG_DIR` set to it, handing it the terminal: a browser, or
  a code to paste over SSH (jig's `draft/providers.md`, 7.3). temper
  reads nothing of it, and checks it only with `claude auth status`.
- **The worker checks** the login when told of the source, at its
  hello, at an interval, and on [Check again], and the account becomes
  usable on this machine (jig's `draft/llm-accounts.md`, 3.4). A worker
  on another machine is signed in there, with its own document.
- **Then** Anthropic's models run on Claude Code, and the composer says
  so. A person who also adds an Anthropic key to the account has Claude
  Code first and the key second, by default; the key is used only on a
  machine where Claude Code is not signed in.
- **Several logins on one machine,** one per name, each its own
  subscription. Every run on the machine runs as its user and could read
  them, which one person's machine accepts (jig's
  `draft/llm-accounts.md`, 3.4).
- **Plain words only:** temper says that it runs Claude Code, and names
  no command, page, button or feature after it (jig's
  `draft/providers.md`, 3.2 and 7.5).

## 6. Several subscriptions

- **Another ChatGPT account:** [Continue with ChatGPT] again, signed in
  to ChatGPT, in the browser, as the other account; a new account
  appears beside the first. A sign-in that lands on an account the
  person already has renews it, and says: "That's alice@example.com
  again. To add another account, sign out of ChatGPT in your browser
  first, then Continue with ChatGPT."
- **Another Claude subscription:** [Use your Claude subscription] again,
  with another name, `claude-work`, and `temper program-login
  CONFIG.json claude-work` with the other email.
- **The person's order** decides which a new task takes; a task keeps
  its account, and its page shows which. A person moves a task to
  another account by amending it, but not while its account is limited:
  temper never moves work at a limit, and the task waits for its own
  account.

## 7. Choosing models

- **A chat:** the composer's model picker lists the models the person
  may use, by provider, each with its efforts, and "needs an account"
  beside a provider they have none for. A change applies from the
  chat's next run.
- **A goal:** setting a goal names its planner's model and effort, the
  template's by default, beside the planner it already names
  (jig's `client/domain/pages.md`, section 6). The planner gives each task it delegates
  a model and an effort of its own, from the catalogue its brief shows,
  with each model's `about` and prices (jig's `draft/llm-accounts.md`,
  5.2).
- **Every task's page** shows its model, effort and account, and a
  person changes them by amending the task.
- **A project's policy** may narrow the models its roles allow
  (jig's `client/domain/pages.md`, section 6); one person rarely needs it.

## 8. Commands

- **`temper sign-in` goes:** a sign-in is the client's, and its record
  is kept in the store (jig's `draft/llm-accounts.md`, sections 6 and
  7).
- **`temper program-login CONFIG.json NAME`** is new (section 5).
- **`temper ctl`'s tools** (`../tools.md`, section 5) gain the accounts:
  `llm-accounts`; `llm-sign-in PROVIDER`, which prints the address and
  waits for its end; `llm-sign-out ACCOUNT`; `add-key PROVIDER FILE
  [ACCOUNT]`, a new account when none is named; `replace-key ACCOUNT
  FILE`; `remove ACCOUNT`; `order ACCOUNT POSITION`; `resume ACCOUNT`;
  `declare-program PROVIDER ADAPTER NAME`; `check ACCOUNT`; and later,
  with jig's move, `move-out` and `move-in`. A key's file is read by the
  tool and sent once, never an argument. An agent driving temper for a
  person, with the person's credential or a service's (`../tools.md`,
  section 9), and a script use them as the person's pages do.

## 9. The live tier on a subscription

- **Claude, unattended.** The live configuration's programs directory
  is `~/.local/state/temper/live/programs`. The developer runs `temper
  program-login live.json live` once, on each machine that runs the
  tier. Each run's fresh deployment declares the program source named
  `live` (`temper ctl ... declare-program`), whose directory is then
  found signed in: the name, not the account's number, ties the two
  (section 5; jig's `draft/llm-accounts.md`, 3.4). Claude Code refreshes
  its own login, and the run needs nobody.
- **ChatGPT, signed in as the run starts.** `scripts/proof-live.sh`
  starts the box, runs `temper ctl ... llm-sign-in 1`, prints its
  address and waits, within the sign-in's deadline, for the developer to
  approve it in a browser on the same machine. The account is the run's
  fresh deployment's, and goes with it.
- **Unattended ChatGPT runs, later,** with jig's move of an account
  (jig's `draft/llm-accounts.md`, section 8): one account moved into
  each run's deployment and out again at its end, never copied, since a
  copy that refreshes signs the other out.
- **A key, when neither is at hand:** `add-key` from a file the script
  names.
- **Which one runs** is the script's argument; Claude is its default,
  since it needs nobody.
- **As now:** at most 25 completions and 15 minutes, one run at a time,
  never at the gate (`../testing.md`, 2.7).

## 10. What changes in temper's design

- **`../protocol/credentials.md`,** sections 1, 2, 3, 7 and 8: LLM accounts in jig's
  store, signed in from the client; `temper sign-in` and handed-in
  files removed.
- **`../protocol/configuration.md`,** sections 1, 2, 5, 6, 8 and 9: providers;
  the agent's programs; the agent's configuration no longer checked for
  "the same account" as the engine's endpoints, since an endpoint names
  none; the secrets' table without LLM rows; the commands.
- **`../protocol/llm.md`,** sections 1 to 3 and 6: providers and accounts, the
  agent's two backends.
- **`../protocol/README.md`,** sections 4, 5 and 11: the commands, the
  crates, and their tests.
- **`../protocol/setup.md`,** 8.1: the owned example's LLM provider in place
  of a handed-in account.
- **`../testing.md`,** 2.7 and 4.1: the live tier on a subscription; a fake
  `claude` beside skein's fake LLM peer and fake issuer.
- **`../tools.md`,** sections 5 and 9: the accounts' tools; the agent's
  account no longer handed in by configuration.
- **`../client/README.md`,** on jig's pages (jig's `client/domain/pages.md`,
  sections 4 to 7): the screens of sections 3 to 7, in the browser and
  the terminal.
- **`../README.md`,** section 2: the first milestone's account.

## 11. Open questions

- **A browser on another machine.** The sign-in's redirect reaches the
  engine on `127.0.0.1`, so a person who reaches temper through a
  tunnel signs in on the engine's machine, or, later, moves an account
  in, until a client relays the redirect (jig's `draft/llm-accounts.md`,
  section 12). Default: as said.
- **The first-use notice in the terminal:** a line once, or a card that
  waits to be dismissed. Default: a card.
- **QA's evals** (`qa.md`): on keys, or on QA's own subscriptions.
  Default: keys.
