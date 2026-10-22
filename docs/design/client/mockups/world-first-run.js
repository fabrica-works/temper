// A third world: a person's first run, after `temper box` (setup.md, 8.2
// to 8.4). The default document's deployment: one project, the chat and
// coding charters, one worker in temper's process, three forges (the
// demo's fake, GitHub, an owned Forgejo not set up), and the demo's model
// as its one account. The demo model plays its scripts, so every word an
// agent writes here is setup.md 8.4's. Times are minutes from Thursday
// 22 October, 21:00 UTC.
(function () {
  const MOCKUPS = window.MOCKUPS = window.MOCKUPS || { worlds: {} };

  const CSV = 'Add CSV export to widget';
  const DARK = 'Add a dark theme to widget';

  function initial() {
    return {
      now: 0,
      person: 'Sam',
      project: 'Default',

      config: {
        templates: { chat: 'chat', goal: 'goal' },
        charters: ['chat', 'coding'],
        period: 'month',
        allotment: 100,
        ops: false,
        // The owned example's providers: OpenAI, with keys only (setup.md,
        // 8.1), and the demo's.
        ways: ['Use an API key'],
        restart: 'temper box',
      },

      // The demo, as the forge's side reads it (client README, 4.8), and
      // its cues, copied in (setup.md, 8.4). Each cue the mockup plays is
      // drawn at a step.
      demo: {
        provider: 'Demo', forge: 1, repo: 'widget',
        cues: [CSV, DARK, 'Fix widget’s empty search state'],
        plays: { [CSV]: ['2', 'T1'], [DARK]: ['4', 'T4'] },
      },

      repos: {
        widget: {
          name: 'demo/widget', forge: 1, branch: 'main',
          tip: '9f2c0a7',
          ci: { state: 'passing', at: 0 },
          after: null,
          queue: [],
        },
      },

      // The default document's forges, by number (setup.md, 8.3).
      forges: [
        { n: 1, kind: 'Forgejo', name: 'Demo forge', web: 'http://127.0.0.1:8369', mode: 'fake', state: 'connected', login: 'temper', id: 2,
          load: { used: 9, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 5], ['writes', 0], ['keeping up', 4], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['demo/widget', 0]] } },
        { n: 2, kind: 'GitHub', web: 'https://github.com', mode: 'connected', state: 'not connected' },
        { n: 3, kind: 'Forgejo', web: 'http://127.0.0.1:8368', mode: 'owned', state: 'not set up', box: true, versions: { forgejo: '17.0.0', runner: '13.2.0' }, label: 'temper-host', person: 'sam', setup: { org: 'work', repo: 'hello', project: 'hello', newProject: true } },
      ],

      landings: [],
      findings: [],

      workers: [
        { name: 'worker', kind: 'workers', what: 'In temper’s process', slots: 2, logins: [] },
      ],

      // The demo model's key, which the seed adds (setup.md, 8.4); the
      // phrasebook names it Demo model (client README, 5.6).
      accounts: [
        { name: 'Demo', display: 'Demo model', provider: 'Demo', holder: 'Sam',
          sources: [{ name: 'demo', kind: 'key', use: 'Idle', usage: null }] },
      ],

      tasks: {},
      children: {},
      ended: [],
      inbox: [],
      transcripts: {},
      notes: [],
      activity: {},
      history: {},
    };
  }

  const DAY = { day: 'Today · Thursday 22 October' };

  const steps = [
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'The link opened',
      summary: 'Sam opens the link temper box printed. The chats page is the demo’s start: its banner, the context line, three suggested requests. Nothing has run yet, so there is no task and no number.',
      look: [
        ['chats.html', 'Chats: the demo’s start'],
        ['accounts.html', 'Accounts: the demo model’s key, the one account'],
        ['forges.html', 'Forges: the demo’s, connected; GitHub, not connected; a Forgejo here, not set up'],
        ['workers.html', 'Workers: one, in temper’s process'],
        ['changes.html', 'Changes: demo/widget on Demo forge, nothing in flight'],
      ],
      apply() {},
    },
    {
      id: '2', label: '2', hour: 2 / 60, act: 1, title: 'A request sent',
      summary: 'Sam picks “Add CSV export to widget”. The chat starts with those words, T1. Its agent delegates the change T2, whose agent, T3, edits src/main.rs. The card says Running, with no step ticked: the steps are a stand-in until the change’s live stages.',
      look: [
        ['chat.html?task=T1', 'T1: the announcement, and T2’s card, Running'],
        ['task.html?task=T2', 'T2: the change, producing its branch'],
        ['task.html?task=T3', 'T3: the coding agent, calling its tools'],
        ['tasks.html', 'Tasks: the chat, its change and the change’s agent'],
        ['chats.html', 'Chats: the demo’s start, and the chat under it'],
      ],
      apply(w, id) {
        w.tasks.T1 = {
          title: CSV, by: 'Sam', template: 'chat', charter: 'chat', requester: 'Sam',
          model: 'Demo model', account: 'Demo', hostKind: 'workers',
          phase: 'idle', waitsOn: 'delegates', spend: 0, budget: null, made: 1, at: id,
        };
        w.tasks.T2 = {
          title: "Export widget's table as CSV", by: 'T1', kind: 'change', parent: 'T1', repo: 'widget',
          stage: 'producing', made: 1, at: id,
        };
        // Its brief is the change's title, the delegate's words (setup.md,
        // 8.4), so its chip is that title too.
        w.tasks.T3 = {
          title: "Export widget's table as CSV", by: 'T2', briefBy: 'T1', charter: 'coding', parent: 'T2',
          model: 'Demo model', account: 'Demo', hostKind: 'workers', worker: 'worker',
          phase: 'running', run: { n: 1, since: 1, doing: 'calling edit' },
          spend: 0, made: 1, at: id,
          brief: "Export widget's table as CSV",
        };
        w.children = { T1: ['T2'], T2: ['T3'] };
        w.accounts[0].sources[0].use = 'In use';
        w.transcripts.T1 = [
          DAY,
          { who: 'you', at: 1, text: CSV },
          { who: 'agent', at: 1, text: 'I’ll delegate this change in demo/widget. It will push a branch, open a pull request, check CI and land into main on Demo forge.' },
          { who: 'tools', at: 1, calls: [['delegate', 1]], made: ['T2'] },
          { who: 'tools', at: 1, calls: [['wait', 1]] },
        ];
        w.transcripts.T3 = [
          { who: 'tools', at: 1, calls: [['read', 1]] },
        ];
      },
    },
    {
      id: '3', label: '3', hour: 4 / 60, act: 1, title: 'The change landed',
      summary: 'T3 finished with the change, the worker pushed its branch, and T2 opened its pull request, passed the fake’s CI and landed into main. Its result woke T1, which says so in its script’s words. The card says Landed, every step ticked, and the merge commit.',
      look: [
        ['chat.html?task=T1', 'T1: T2’s card, Landed, and the chat’s last words'],
        ['task.html?task=T2', 'T2: landed'],
        ['task.html?task=T3', 'T3: its result'],
        ['changes.html', 'Changes: the landing on demo/widget'],
        ['inbox.html', 'Inbox: nothing needs Sam; T1 replied'],
      ],
      apply(w, id) {
        Object.assign(w.tasks.T3, { phase: 'done', run: null, worker: null, ended: 3, at: id,
          result: '**Add CSV export to widget**\n\n`widget --csv` prints the table as CSV: a header line, `name,count`, then a line for each row.' });
        Object.assign(w.tasks.T2, { stage: 'landed', pr: 1, merge: 'e3b8d4192c0f5a7e', ended: 4, at: id });
        Object.assign(w.tasks.T1, { waitsOn: 'news', wake: { words: true }, at: id });
        w.accounts[0].sources[0].use = 'Idle';
        Object.assign(w.repos.widget, { tip: 'e3b8d41', ci: { state: 'passing', at: 4 } });
        w.landings.unshift({ task: 'T2', repo: 'widget', at: 4, title: "Export widget's table as CSV" });
        w.transcripts.T1[2].read = true;
        w.transcripts.T1.push(
          { who: 'agent', at: 4, text: 'CSV export is available from widget’s table. The change landed on Demo forge. This was a scripted demo change.' },
          { who: 'tools', at: 4, calls: [['wait', 1]] },
        );
        w.transcripts.T3.push(
          { who: 'tools', at: 2, calls: [['edit', 1]] },
          { who: 'tools', at: 3, calls: [['finish', 1]] },
        );
      },
    },
    {
      id: '4', label: '4', hour: 6 / 60, act: 1, title: 'A text-only request',
      summary: 'Back on the demo’s start, Sam picks “Add a dark theme to widget”. Its script is one turn: an answer in words, then finish with a report, as setup.md 8.4 has it. That ends the chat, so its page has no composer: the README flags this as a likely flaw.',
      look: [
        ['chat.html?task=T4', 'T4: answered, then ended, with no composer'],
        ['chats.html', 'Chats: T1 at rest, T4 ended'],
      ],
      apply(w, id) {
        w.tasks.T4 = {
          title: DARK, by: 'Sam', template: 'chat', charter: 'chat', requester: 'Sam',
          model: 'Demo model', account: 'Demo', hostKind: 'workers',
          phase: 'done', contract: 'report', result: 'The demo changes nothing.',
          spend: 0, budget: null, made: 5, ended: 5, at: id,
        };
        w.transcripts.T1[w.transcripts.T1.length - 2].read = true;
        w.transcripts.T4 = [
          DAY,
          { who: 'you', at: 5, text: DARK },
          { who: 'agent', at: 5, text: 'I’d give widget’s table a second set of colours, chosen when your system prefers a dark theme, so that widget follows your system. The demo plays only the CSV export as a change: connect a model to make this one.' },
          { who: 'tools', at: 5, calls: [['finish', 1]] },
        ];
      },
    },
    {
      id: '5', label: '5', hour: 15 / 60, act: 2, title: 'A model connected',
      summary: 'Sam follows “Connect a model” and adds an OpenAI API key. Another provider’s account is usable, so the model’s half of the demo ends: the banner says a demo forge alone, the chats page is a chats page, and T1’s next run takes the first of its charter’s models with an account, gpt-6-luna (setup.md, 8.3).',
      look: [
        ['accounts.html', 'Accounts: the new key, and the demo model’s'],
        ['chats.html', 'Chats: no demo’s start; the banner, a demo forge alone'],
        ['chat.html?task=T1', 'T1: its composer’s line names the new key'],
        ['forges.html', 'Forges: still the demo’s alone, connected'],
      ],
      apply(w, id) {
        w.accounts.unshift({ name: 'OpenAI', provider: 'OpenAI', holder: 'Sam', at: id,
          sources: [{ name: 'sam-key', kind: 'key', use: 'Idle', usage: null }] });
        Object.assign(w.tasks.T1, { model: 'gpt-6-luna', account: 'OpenAI' });
        w.config.next = { account: 'OpenAI', model: 'gpt-6-luna' };
      },
    },
    {
      id: '6', label: '6', hour: 16 / 60, act: 2, title: 'Setting up a Forgejo',
      summary: 'Sam follows “Connect a forge” and keeps the set-up’s defaults: the organization work, the repository hello, and a new project named after it. temper box has fetched Forgejo and its runner and is preparing them; the row shows each step as the box reports it.',
      look: [
        ['forges.html', 'Forges: forge 3 setting up, at its preparation'],
        ['chats.html', 'Chats: the banner still says a demo forge'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        f.state = 'setting up';
        f.setup.step = 'preparing';
      },
    },
    {
      id: '7', label: '7', hour: 19 / 60, act: 2, title: 'A Forgejo connected',
      summary: 'Forgejo and its runner started. temper connected as its bot, made work/hello with a README and a CI workflow, and adopted it into the new project hello; the workflow’s first run passed on this machine. Sam never signed in to Forgejo. A real model and a real forge are connected, so the demo’s banner is gone.',
      look: [
        ['forges.html', 'Forges: forge 3 connected, its repository and its acts'],
        ['changes.html?project=hello', 'Changes in hello: work/hello, CI passing'],
        ['chats.html', 'Chats: no banner'],
        ['forges.html#other-states', 'Forges, below: forge 3’s other states, and a managed and a connected Forgejo'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        Object.assign(f, { state: 'connected', login: 'temper', id: 2,
          load: { used: 14, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 8], ['writes', 3], ['keeping up', 3], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['work/hello', 18]] } });
        delete f.setup.step;
        w.repos.hello = { name: 'work/hello', forge: 3, branch: 'main', tip: 'a41c9e2', ci: { state: 'passing', at: 18 }, after: null, queue: [], project: 'hello' };
      },
    },
  ];

  MOCKUPS.worlds['first-run'] = {
    name: 'First run',
    about: 'the owner’s first run after temper box: the demo’s start, a suggested request landed on the demo forge by the demo model’s script, a text-only one, then a real model connected, and a Forgejo set up here',
    HOUR0: Date.UTC(2026, 9, 22, 21, 0),
    initial, steps, upcoming: [], acts: { 1: 'The demo', 2: 'Going real' },
  };
})();
