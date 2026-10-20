// Independent sample state, reset on reload. No requests, credentials or storage.
(() => {
  'use strict';
  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
  const input = $('textarea');
  const messages = $('.terminal-added-messages');
  const thread = $('.terminal-thread');
  const suggestions = $$('[data-suggest]');
  const scripts = [
    { request: suggestions[0].textContent, branch: 'csv-export', result: 'CSV export is available from widget’s table.' },
    { request: suggestions[1].textContent, branch: 'dark-theme', result: 'Widget now has a dark theme that follows your system preference.' },
    { request: suggestions[2].textContent, branch: 'empty-search', result: 'Widget now explains when a search has no results and offers to clear it.' }
  ];
  let busy = false;
  let nextTask = 1;
  const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
  const reveal = () => { thread.scrollTop = thread.scrollHeight; };

  function autosize() {
    input.style.height = '22px';
    input.style.height = `${Math.min(Math.max(input.scrollHeight, 22), 160)}px`;
    input.style.overflowY = input.scrollHeight > 160 ? 'auto' : 'hidden';
    $('.terminal-send').disabled = busy || !input.value.trim();
  }

  function turn(words, user = false) {
    const node = document.createElement('section');
    node.className = `terminal-turn${user ? ' terminal-turn-user' : ''}`;
    node.innerHTML = `<div class="terminal-role">${user ? 'You' : 'Temper'}<time>Just now</time></div><div class="terminal-turn-content"><span class="terminal-prompt${user ? '' : ' agent'}" aria-hidden="true">${user ? '›' : '✳'}</span><div><p></p></div></div>`;
    $('p', node).textContent = words;
    messages.append(node);
    reveal();
    return $('.terminal-turn-content > div', node);
  }

  async function send(words = input.value.trim()) {
    if (busy || !words) return;
    busy = true;
    input.value = '';
    autosize();
    suggestions.forEach(button => { button.disabled = true; });
    $('.first-run-empty').hidden = true;
    $('.terminal-phase').textContent = 'Writing';
    turn(words, true);
    await pause(500);
    const script = scripts.find(item => item.request.toLowerCase() === words.toLowerCase());
    if (!script) {
      const reply = turn('I’m the demo model. I can only play the suggested requests from my script. Connect a model to ask for other work, or try one of these requests.');
      const actions = document.createElement('div');
      actions.className = 'terminal-proposal-actions';
      const connect = document.createElement('button');
      connect.textContent = 'Connect a model';
      connect.addEventListener('click', () => openPanel('accounts'));
      actions.append(connect);
      scripts.forEach(item => {
        const button = document.createElement('button');
        button.textContent = item.request;
        button.addEventListener('click', () => send(item.request));
        actions.append(button);
      });
      reply.append(actions);
    } else {
      const id = nextTask++;
      turn(`I’ll delegate this change to T${id} in demo/widget. It will push a branch, open a pull request, check CI and land into main on Demo forge.`);
      const task = document.createElement('article');
      task.className = 'terminal-proposal first-run-task';
      task.setAttribute('aria-label', `Task T${id}`);
      task.innerHTML = `<div class="terminal-proposal-heading"><span class="terminal-proposal-type">T${id} · CHANGE</span><span data-phase>Producing</span></div><h2></h2><p data-detail></p><div class="steps"><div class="step">Branch pushed</div><div class="step">Pull request opened</div><div class="step">CI</div><div class="step">Landed</div></div><a class="text-link" hidden target="_blank" rel="noopener" data-pull></a>`;
      $('h2', task).textContent = script.request;
      $('[data-detail]', task).textContent = 'Producing the change in demo/widget.';
      const link = $('[data-pull]', task);
      // Illustrative local forge URL: its pull request page may answer 404.
      link.href = `http://127.0.0.1:3000/demo/widget/pulls/${id}`;
      link.textContent = `demo/widget#${id} on Demo forge ↗`;
      messages.append(task);
      $('.terminal-phase').textContent = 'Waiting for news';
      const stages = [
        ['Branch pushed', `Branch temper/${script.branch}-${id} pushed on Demo forge.`],
        ['Pull request opened', `Pull request #${id} opened into main on Demo forge.`],
        ['Checking CI', 'CI is running on the pull request’s head.'],
        ['Landed', `CI passed. Landed into main · merge commit de00${String(id).padStart(3, '0')}.`]
      ];
      for (const [index, [phase, detail]] of stages.entries()) {
        await pause(900);
        $('[data-phase]', task).textContent = phase;
        $('[data-detail]', task).textContent = detail;
        $$('.step', task).forEach((step, i) => {
          step.classList.toggle('done', i < index || (i === index && index !== 2));
          step.classList.toggle('current', i === index && index === 2);
          step.textContent = i === 2 && index === 3 ? 'CI passed' : ['Branch pushed', 'Pull request opened', 'CI', 'Landed'][i];
        });
        if (index >= 1) link.hidden = false;
        reveal();
      }
      turn(`${script.result} T${id} landed on Demo forge. This was a scripted demo change.`);
    }
    busy = false;
    suggestions.forEach(button => { button.disabled = false; });
    $('.terminal-phase').textContent = 'Waiting for you';
    autosize();
    reveal();
  }

  function openPanel(id) {
    $(`#${id}`).showModal();
  }
  $$('[data-panel]').forEach(button => button.addEventListener('click', () => openPanel(button.dataset.panel)));
  $$('[data-dismiss]').forEach(button => button.addEventListener('click', () => button.closest('dialog').close()));
  $('[data-elsewhere]').addEventListener('click', () => {
    $('#elsewhere').hidden = false;
    $('#elsewhere').scrollIntoView({ block: 'nearest' });
  });

  const connections = {
    chatgpt: ['accounts', 'ChatGPT · owner@example.com', 'Signing in to ChatGPT…', 'Signed in to ChatGPT as owner@example.com.'],
    claude: ['accounts', 'Claude subscription · claude-owner', 'Checking Claude Code’s login on this worker…', 'Claude subscription ready on this worker.'],
    key: ['accounts', 'OpenAI API key', 'Checking the sample key…', 'OpenAI API key ready.'],
    'github-create': ['forges', 'GitHub · Managed', 'Creating the sample App and checking its installation…', 'Connected as temper-sample (42) · selected repositories on demo-owner.'],
    'github-paste': ['forges', 'GitHub · Connected', 'Checking the sample App’s id, private key and installation…', 'Connected as temper-sample (42) · selected repositories on demo-owner.'],
    forgejo: ['forges', 'Forgejo here · Owned', 'Downloading Forgejo and its runner…', 'Connected as temper-bot (2) · admin’s token kept.']
  };
  const added = new Map();
  const pending = new Set();
  $$('[data-connect]').forEach(button => button.addEventListener('click', async () => {
    const key = button.dataset.connect;
    const [panelId, name, progress, result] = connections[key];
    if (pending.has(panelId)) return;
    pending.add(panelId);
    const panel = $(`#${panelId}`);
    const buttons = $$('[data-connect]', panel);
    buttons.forEach(item => { item.disabled = true; });
    const status = $('[data-result]', panel);
    status.textContent = progress;
    await pause(650);
    if (key === 'forgejo') {
      status.textContent = 'Starting Forgejo and its runner…';
      await pause(650);
    }
    // An App's two connect methods lead to the same configured GitHub forge.
    const record = key.startsWith('github') ? 'github' : key;
    let row = added.get(record);
    if (!row) {
      row = document.createElement('div');
      row.className = 'kv';
      row.innerHTML = '<span></span><strong></strong>';
      $(panelId === 'accounts' ? '#account-list' : '#forge-list').append(row);
      added.set(record, row);
    }
    $('span', row).textContent = name;
    if (panelId === 'accounts') {
      $$('#account-list strong').forEach(item => { item.textContent = 'Ready'; });
      $('strong', row).textContent = 'Ready · Default';
    } else {
      $('strong', row).textContent = result;
    }
    status.textContent = `${result} ${panelId === 'accounts' ? 'Deployment default updated. Default’s chat still uses Demo model.' : 'demo/widget stays on Demo forge.'} (Sample result.)`;
    pending.delete(panelId);
    buttons.forEach(item => {
      const record = item.dataset.connect.startsWith('github') ? 'github' : item.dataset.connect;
      item.disabled = added.has(record);
    });
  }));

  suggestions.forEach(button => button.addEventListener('click', () => send(button.textContent)));
  input.addEventListener('input', autosize);
  input.addEventListener('keydown', event => {
    if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      send();
    }
  });
  $('.terminal-send').addEventListener('click', () => send());
  $('[data-reset]').addEventListener('click', () => location.reload());
  autosize();
})();
