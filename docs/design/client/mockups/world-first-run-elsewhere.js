// The configured string-managed Forgejo, after the configuration's restart.
// Every stop returns to the same Details, with only the token emptied.
(function () {
  const MOCKUPS = window.MOCKUPS;
  const first = MOCKUPS.worlds['first-run'];
  function initial() {
    const w = first.initial();
    w.forges = w.forges.filter(f => f.kind === 'GitHub');
    w.forges.push({ n: 3, kind: 'Forgejo', mode: 'managed', admin: true,
      web: 'git.example.org', state: 'not connected', login: 'temper', id: 7,
      setup: { org: 'work', repo: 'hello', project: 'hello', label: 'docker', newProject: true } });
    return w;
  }
  const states = [
    ['Choose', 'not connected', null], ['Details', 'not connected', null],
    ['Token required', 'not connected', 'required'], ['Token over 4 KiB', 'not connected', 'too-long'], ['Invalid runner label', 'not connected', 'label'], ['Organization required', 'not connected', 'org-required'], ['Repository required', 'not connected', 'repository-required'], ['Connecting', 'connecting', null], ['NotAdmin', 'not connected', 'not-admin'],
    ['Refused 401', 'not connected', '401'], ['Refused 403, bot', 'not connected', '403-bot'],
    ['Refused 403, organization', 'not connected', '403-org'], ['Unmarked', 'not connected', 'unmarked'],
    ['Other', 'not connected', 'other'], ['Create, admin', 'not connected', 'first-admin'],
    ['Create, organization', 'not connected', 'first-org'], ['Create, repository', 'not connected', 'first-repository'],
    ['Create, starter workflow', 'not connected', 'first-file'], ['Ready', 'connected', null],
  ];
  const steps = states.map(([title, state, stop], i) => ({
    id: String(i + 1), label: String(i + 1), hour: i / 60, act: 1, title,
    summary: i === 0 ? 'The configuration was restarted: its Forgejo has its own card.' : `${title}; the four repository fields are kept, and no token is drawn.`,
    look: [[`welcome.html?at=forge${i === 0 ? '' : '&forge=3'}`, title], [`connect-forge.html${i === 0 ? '' : '?forge=3'}`, title]],
    apply(w) {
      const f = w.forges.find(f => f.n === 3);
      Object.assign(f, { state, stop });
      if (state === 'connected') {
        w.project = 'hello';
        w.repos['work/hello'] = { name: 'work/hello', forge: 3, branch: 'main', tip: '5d0e9b1', project: 'hello', ci: { state: 'waiting' }, queue: [] };
      }
    },
  }));
  MOCKUPS.worlds['first-run-elsewhere'] = {
    name: 'First run, on a Forgejo elsewhere',
    about: 'the configured Forgejo after restart: its admin token, first repository, every stop and Ready',
    HOUR0: first.HOUR0, initial, steps, upcoming: [], acts: { 1: 'Connecting a Forgejo elsewhere' },
  };
})();
