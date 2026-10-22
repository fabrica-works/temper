// The welcome's real setup: the same fresh deployment as the demo path,
// with no demo started. Times are minutes from Thursday 22 October, 21:00.
(function () {
  const MOCKUPS = window.MOCKUPS;
  const first = MOCKUPS.worlds['first-run'];
  const steps = [
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'The link opened',
      summary: 'temper box started temper serve alone. The link opens the welcome; Sam chooses Set up temper, then Forgejo on this machine, and keeps its defaults.',
      look: [
        ['welcome.html', 'The welcome: the demo, or set up'],
        ['welcome.html?at=forge', 'Connect a forge'],
        ['welcome.html?at=forge&forge=3', 'Forgejo on this machine: its defaults'],
        ['chats.html', 'Chats behind it: the setup strip, no model'],
      ],
      apply() {},
    },
    {
      id: '2', label: '2', hour: 1 / 60, act: 1, title: 'Setting up a Forgejo',
      summary: 'temper box fetches Forgejo and its runner. The wizard moves on to the model while it does.',
      look: [
        ['welcome.html?at=model', 'Connect a model, while Forgejo sets up'],
        ['welcome.html?at=model#other-states', 'Other ways, where the configuration offers them'],
        ['forges.html', 'Forges: forge 3, setting up; no demo'],
        ['chats.html', 'Chats: the setup strip’s status'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        f.state = 'setting up';
        f.setup.step = 'fetching';
      },
    },
    {
      id: '3', label: '3', hour: 2 / 60, act: 1, title: 'A model connected',
      summary: 'Sam adds an OpenAI API key. The forge is still starting, so Ready waits for it.',
      look: [
        ['welcome.html?at=ready', 'Almost ready: waiting for Forgejo'],
        ['accounts.html', 'Accounts: the key alone'],
      ],
      apply(w, id) {
        w.accounts.unshift({ name: 'OpenAI', provider: 'OpenAI', holder: 'Sam', at: id,
          sources: [{ name: 'sam-key', kind: 'key', use: 'Idle', usage: null }] });
        w.config.next = { account: 'OpenAI', model: 'gpt-6-luna' };
        w.forges.find(f => f.n === 3).setup.step = 'starting';
      },
    },
    {
      id: '4', label: '4', hour: 4 / 60, act: 1, title: 'Ready',
      summary: 'temper connected as the forge’s bot, made work/hello and adopted it into the new project hello. A forge of Sam’s is connected, so the demo, never started, is retired: nothing of it shows.',
      look: [
        ['welcome.html?at=ready', 'work/hello is ready'],
        ['chats.html?project=hello', 'Chats in hello: no chat yet, no strip'],
        ['forges.html', 'Forges: forge 3 alone'],
        ['accounts.html', 'Accounts: the key alone'],
        ['chats.html', 'Chats in Default: empty'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        Object.assign(f, { state: 'connected', login: 'temper', id: 2,
          load: { used: 14, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 8], ['writes', 3], ['keeping up', 3], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['work/hello', 18]] } });
        delete f.setup.step;
        w.repos.hello = { name: 'work/hello', forge: 3, branch: 'main', tip: 'a41c9e2', ci: { state: 'passing', at: 18 }, after: null, queue: [], project: 'hello' };
        w.demo.state = 'retired';
        w.forges[0].state = 'retired';
      },
    },
  ];

  MOCKUPS.worlds['first-run-real'] = {
    name: 'First run, set up',
    about: 'the owner’s first run after temper box, choosing to set up temper: a Forgejo on this machine and an OpenAI key, through the welcome’s wizard; the demo never starts',
    HOUR0: first.HOUR0, initial: first.initial,
    steps, upcoming: [], acts: { 1: 'Setting up' },
  };
})();
