// temper's client, drawn from a world at the step in the address
// (?world=, ?step=), with the mockups' bar above each page. Every page
// reads the same world, so the pages agree at each step.
//
// Every value a page shows goes through src(): an engine fact, a name
// from the configuration, words as a person or an agent wrote them, or
// an ops application's report. What the page code writes itself is the
// UI's own words, which name no workflow: a workflow's words come from
// its world's data, with their writer.
(function () {
  const C = window.MOCKUPS;
  const params = new URLSearchParams(location.search);
  const page = document.body.dataset.page;

  function stored(key) { try { return localStorage.getItem(key); } catch { return null; } }
  function store(key, value) { try { localStorage.setItem(key, value); } catch { /* no storage in previews */ } }

  // ---- The world, and the step ----------------------------------------

  let worldKey = params.get('world') || stored('mockups-world') || 'reference';
  if (!C.worlds[worldKey]) worldKey = 'reference';
  store('mockups-world', worldKey);
  const W = C.worlds[worldKey];

  const ids = W.steps.map(s => s.id);
  let stepId = params.get('step') || stored(`mockups-step-${worldKey}`) || ids[0];
  if (!ids.includes(stepId)) stepId = ids[0];
  store(`mockups-step-${worldKey}`, stepId);
  const stepIndex = ids.indexOf(stepId);
  const step = W.steps[stepIndex];

  const world = W.initial();
  for (const s of W.steps.slice(0, stepIndex + 1)) {
    world.now = s.hour * 60;
    s.apply(world, s.id);
  }

  // Annotations show unless hidden; sources show when asked. `?bare`
  // draws the product alone, for a capture: no bar, no marks.
  const FLAGS = { plain: 'mock-plain', sources: 'mock-sources' };
  const bare = params.has('bare');
  if (bare) document.body.classList.add(FLAGS.plain);
  else {
    for (const [flag, cls] of Object.entries(FLAGS)) if (stored(cls) === '1') document.body.classList.add(cls);
    document.body.classList.add('mock');
  }

  // ---- Words and links ------------------------------------------------

  const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  function esc(s) { return String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c])); }
  function date(min) { return new Date(W.HOUR0 + min * 60000); }
  function dayOf(min) { return Math.floor((W.HOUR0 + min * 60000) / 86400000); }
  function hm(min) { const d = date(min); return `${String(d.getUTCHours()).padStart(2, '0')}:${String(d.getUTCMinutes()).padStart(2, '0')}`; }
  function clock(min) { return dayOf(min) === dayOf(world.now) ? hm(min) : `${DAY[date(min).getUTCDay()]} ${hm(min)}`; }
  function span(minutes) {
    const m = Math.round(minutes);
    if (m < 60) return `${m} min`;
    const h = Math.floor(m / 60), r = m % 60;
    return r ? `${h} h ${r} min` : `${h} h`;
  }
  function ago(min) {
    const d = world.now - min;
    if (d < 1) return 'just now';
    if (d < 60 * 12) return `${span(d)} ago`;
    const days = dayOf(world.now) - dayOf(min);
    if (days === 0) return `at ${hm(min)}`;
    if (days === 1) return 'yesterday';
    return DAY[date(min).getUTCDay()];
  }
  function ordinal(n) { return `${n}${['th', 'st', 'nd', 'rd'][(n % 100 - 20) % 10] || ['th', 'st', 'nd', 'rd'][n % 100] || 'th'}`; }
  function plural(n, one, many) { return `${n} ${n === 1 ? one : (many || `${one}s`)}`; }
  const cap = s => s.charAt(0).toUpperCase() + s.slice(1);

  // A link within the mockups keeps the world and the step.
  function u(target) {
    if (/^(https?:|#)/.test(target)) return target;
    const [path, hash] = target.split('#');
    const [file, query] = path.split('?');
    const p = new URLSearchParams(query || '');
    p.set('world', worldKey);
    p.set('step', stepId);
    if (bare) p.set('bare', '');
    return `${file}?${p}${hash ? `#${hash}` : ''}`;
  }

  const T = id => world.tasks[id];
  const kids = id => world.children[id] || [];
  const requesterOf = t => t.requester || t.parent;
  const isChat = t => t.template === world.config.templates.chat;
  function isLive(id) {
    const t = T(id);
    return t.kind === 'change' ? t.stage !== 'landed' : !['done', 'failed', 'cancelled'].includes(t.phase);
  }
  const liveKids = id => kids(id).filter(isLive);
  function descendants(id) { return kids(id).flatMap(k => [k, ...descendants(k)]); }
  function href(id) {
    const t = T(id);
    if (!t) return '#';
    return u(isChat(t) ? `chat.html?task=${id}` : `task.html?task=${id}`);
  }
  function who(by) { const t = T(by); return t ? `${by} ${t.title}` : by; }
  function liveRuns(worker) { return Object.keys(world.tasks).filter(k => T(k).worker === worker && isLive(k) && T(k).kind !== 'change' && (T(k).phase === 'running' || T(k).run)); }
  const fresh = thing => (thing && thing.at === stepId ? ' fresh' : '');

  // A subset of Markdown, as jig's client renders what agents write.
  function inline(s) { return esc(s).replace(/`([^`]+)`/g, '<code>$1</code>').replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>'); }
  // A person's words keep their line breaks; an agent's lines join.
  function md(text, br = ' ') {
    const lines = String(text).split('\n');
    let out = '';
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];
      if (!line.trim()) { i++; continue; }
      if (line.startsWith('|')) {
        const rows = [];
        while (i < lines.length && lines[i].startsWith('|')) rows.push(lines[i++]);
        const cells = r => r.slice(1, -1).split('|').map(c => c.trim());
        const [head, , ...body] = rows;
        const h = cells(head);
        out += `<table class="md-table">${h.some(Boolean) ? `<thead><tr>${h.map(c => `<th>${inline(c)}</th>`).join('')}</tr></thead>` : ''}<tbody>${body.map(r => `<tr>${cells(r).map(c => `<td>${inline(c)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
        continue;
      }
      if (line.startsWith('- ')) {
        const items = [];
        while (i < lines.length && lines[i].startsWith('- ')) items.push(lines[i++].slice(2));
        out += `<ul>${items.map(x => `<li>${inline(x)}</li>`).join('')}</ul>`;
        continue;
      }
      const para = [];
      while (i < lines.length && lines[i].trim() && !lines[i].startsWith('|') && !lines[i].startsWith('- ')) para.push(lines[i++]);
      out += `<p>${para.map(inline).join(br)}</p>`;
    }
    return `<div class="md">${out}</div>`;
  }
  const firstLine = text => String(text).split('\n').find(l => l.trim()).replace(/\*\*|`/g, '');

  // ---- Sources --------------------------------------------------------

  const KINDS = {
    engine: 'Engine fact',
    config: 'Configuration',
    written: 'As written',
    ops: 'Ops application',
  };
  // A value of blocks is marked as a block; anything else, inline.
  function src(kind, where, html) {
    const tag = /^\s*<(div|table|ul|ol|p|dl)\b/.test(html) ? 'div' : 'span';
    return `<${tag} class="src src-${kind}" data-src="${esc(where)}">${html}</${tag}>`;
  }
  const fact = (where, html) => src('engine', where, html);
  const conf = (where, html) => src('config', where, html);
  function said(by, html, at) { return src('written', `Written by ${who(by)}${at != null ? `, ${clock(at)}` : ''}`, html); }

  // Annotations, beside the product's words: the reference case's names,
  // and the gaps temper lacks for what a page draws (the README lists
  // them).
  function ref(name) { return name ? `<i class="case-ref" title="The reference case's name">${esc(name)}</i>` : ''; }
  function gap(id, what) { return `<i class="gap" title="temper-on-temper.md ${esc(id)}: ${esc(what)}. Drawn ahead of temper's design.">gap ${esc(id)}</i>`; }

  // ---- A task's facts, in words ---------------------------------------

  // When jig computes news, what would wake the task, from its wake
  // policy: a party's words, for a task a party requested; another task's
  // state, by a subscription. A timer alone is a time (phaseWords).
  function newsWords(t) {
    const w = t.wake || {};
    const req = requesterOf(t);
    if (w.words && req && !T(req)) return req === world.person ? 'your words' : `${req}’s words`;
    if (w.on) return `${w.on}’s end`;
    return null;
  }
  function waitsWords(id) {
    const t = T(id);
    switch (t.waitsOn) {
      case 'delegates': return plural(liveKids(id).length, 'delegate');
      case 'news': return newsWords(t) || 'news';
      case 'party': return t.waitsFor === world.person ? 'an answer from you' : `an answer from ${t.waitsFor}`;
      default: return 'nothing';
    }
  }
  function wakeWords(t, short) {
    if (!t.wake) return '';
    if (t.wake.every) return short ? `wakes ${hm(t.wake.next)}` : `wakes ${hm(t.wake.next)}, every ${span(t.wake.every)}`;
    return '';
  }
  function changeWords(id, t, short) {
    const pos = world.repos[t.repo].queue.indexOf(id);
    const repo = short ? '' : ` in ${t.repo}`;
    switch (t.stage) {
      case 'checking': return short ? 'checking' : 'checking · CI at its head';
      case 'queued': return `queued · ${ordinal(pos + 1)}${repo}`;
      case 'first': return `first${repo}${t.updating ? (short ? ' · updating' : ' · updating from main') : ''}`;
      case 'landed': return `landed ${clock(t.ended)}`;
      default: return t.stage;
    }
  }
  function phaseWords(id, short) {
    const t = T(id);
    if (t.kind === 'change') return changeWords(id, t, short);
    switch (t.phase) {
      case 'idle': {
        if (t.waitsOn === 'news' && !newsWords(t) && t.wake && t.wake.every) return `waiting until ${hm(t.wake.next)}${short ? '' : `, every ${span(t.wake.every)}`}`;
        const w = wakeWords(t, short);
        return `waiting on ${waitsWords(id)}${w ? ` · ${w}` : ''}`;
      }
      case 'running': return `running · ${t.run.doing}${short ? '' : ` · ${span(world.now - t.run.since)}`}`;
      case 'done': return `done ${clock(t.ended)}`;
      default: return t.phase;
    }
  }
  function phaseWhere(id) {
    const t = T(id);
    if (t.kind === 'change') return `temper: ${id}'s stage, in its forge connector's words`;
    const parts = [`jig: ${id}'s phase`];
    if (t.phase === 'idle') parts.push(`what the engine computes it waits on: ${t.waitsOn === 'party' ? 'a party, for an entry in its subtree' : t.waitsOn}`);
    if (t.waitsOn === 'news' && t.wake && t.wake.words && !T(requesterOf(t))) parts.push(`narrowed by its wake policy: a party's words wake it, and ${requesterOf(t) === world.person ? 'you' : requesterOf(t)} requested it`);
    if (t.waitsOn === 'news' && t.wake && t.wake.on) parts.push(`narrowed by its subscription to ${t.wake.on}'s state: done, held or its result`);
    if (t.wake && t.wake.every) parts.push(`its wake policy's timer, every ${span(t.wake.every)}`);
    if (t.phase === 'running') parts.push("its host's last report");
    return parts.join('; ');
  }
  const phase = (id, short) => fact(phaseWhere(id), esc(phaseWords(id, short)));
  function stateClass(id) {
    const t = T(id);
    if (t.kind === 'change') return t.stage === 'landed' ? 'done' : 'queued';
    return t.phase === 'idle' ? 'waiting' : t.phase;
  }

  function kind(id) {
    const t = T(id);
    if (t.kind === 'change') return fact(`temper: a change, its forge connector's procedure, into ${world.repos[t.repo].name}`, esc(`change · ${t.repo}`));
    return conf(`${t.template ? `template "${t.template}" and ` : ''}charter "${t.charter}", named in the deployment's configuration`, esc(t.template ? `${t.template} · ${t.charter}` : t.charter));
  }
  function chip(id) {
    const t = T(id);
    if (!t) return esc(id);
    return fact(`${id}'s chip: its number, and its title as ${who(t.by)} wrote it`, `<a class="task-chip" href="${href(id)}"><b>${id}</b> ${esc(t.title)}</a>`) + ref(t.ref);
  }
  // A task's card, in temper's words (client README, 5.7): its kind is its
  // result contract's, its state the document's.
  function contract(id) { const t = T(id); return t.kind === 'change' ? 'CHANGE' : (t.contract || 'report').toUpperCase(); }
  function cardState(id) {
    const t = T(id);
    if (t.kind === 'change') return t.stage === 'landed' ? 'Landed' : 'Running';
    if (t.phase === 'held') return 'Held';
    return { done: 'Completed', failed: 'Failed', cancelled: 'Cancelled' }[t.phase] || (isLive(id) ? 'Running' : 'Ended');
  }
  const spentThisPeriod = () => Object.values(world.tasks).reduce((n, t) => n + (t.spend || 0), 0) + world.ended.reduce((n, e) => n + (e.spend || 0), 0);

  // An account and the line naming what a run takes, in temper's words
  // (client README, 5.5).
  const account = name => world.accounts.find(a => a.name === name);
  function accountName(a) {
    const program = a.sources.find(s => s.kind === 'program');
    const signIn = a.sources.find(s => s.kind === 'sign-in');
    if (program) return `${a.plan} · ${program.name}`;
    if (signIn) return `${a.name} · ${signIn.name}`;
    return `${a.provider} API key`;
  }
  function usingLine(id) {
    const t = T(id);
    const a = account(t.account);
    const first = a.sources[0];
    const words = first.kind === 'program' ? `Using your ${a.plan}, through ${first.program} · ${t.model}`
      : first.kind === 'sign-in' ? `Using ${a.plan} · ${t.model} · Manage usage`
      : `Using your ${a.provider} API key · ${t.model}`;
    return fact(`jig: the account and model ${id}'s next run takes; the account's first source, in temper's words`, esc(words));
  }

  const title = id => said(T(id).by, esc(T(id).title));
  const spent = id => fact(`jig: spent by ${id} itself`, esc(T(id).spend ?? ''));
  const model = id => fact(`jig: ${id}'s LLM choice, given by ${who(T(id).by)}, among the configuration's models`, esc(T(id).model || ''));
  const project = () => fact('jig: the project', esc(world.project));

  // ---- The step bar ---------------------------------------------------

  function mockbar() {
    const prev = W.steps[stepIndex - 1];
    const next = W.steps[stepIndex + 1];
    const upcoming = W.upcoming.find(([, id]) => !ids.includes(id));
    const nextTitle = next ? `Next: ${next.id} ${next.title}` : upcoming ? `Next in the case: ${upcoming[1]} ${upcoming[2]} (not drawn yet)` : 'The last step';
    const when = `${step.label} · ${DAY[date(step.hour * 60).getUTCDay()]} ${hm(step.hour * 60)}`;
    const legend = Object.entries(KINDS).map(([k, name]) => `<span class="legend-${k}">${esc(name)}</span>`).join('');
    return `<div class="mockbar" role="region" aria-label="The mockups’ controls">
      <a class="mockbar-mark" href="${u('index.html')}">${esc(W.name.toUpperCase())}</a>
      <div class="mockbar-step">
        <button type="button" data-go="${prev ? prev.id : ''}" ${prev ? '' : 'disabled'} title="${prev ? `Back to ${prev.id} ${esc(prev.title)}` : 'The first step'}" aria-label="Previous step">‹</button>
        <button type="button" data-pop="step" aria-expanded="false">${esc(step.label)} · ${esc(step.title)} ⌄</button>
        <button type="button" data-go="${next ? next.id : ''}" ${next ? '' : 'disabled'} title="${esc(nextTitle)}" aria-label="Next step">›</button>
      </div>
      <div class="mockbar-now"><b>${esc(when)}</b><span>${esc(W.acts[step.act])}</span></div>
      <div class="mockbar-right">
        <div class="mockbar-legend" aria-hidden="true">${legend}</div>
        <button type="button" data-pop="steps" aria-expanded="false">Steps ⌄</button>
        <button type="button" data-pop="view" aria-expanded="false">View ⌄</button>
      </div>
    </div>`;
  }

  function stepPop() {
    return `<div class="mockbar-pop" data-popover="step" hidden style="left:120px">
      <h4>${esc(step.label)} · ${esc(step.title)}</h4>
      <p>${esc(step.summary)}</p>
      <h4>Where to look</h4>
      <ol>${step.look.map(([h, text]) => `<li><a href="${u(h)}">${esc(text)}</a></li>`).join('')}</ol>
    </div>`;
  }

  function stepsPop() {
    const drawn = W.steps.map(s => [s.act, s.id, s.title, s.label]);
    const later = W.upcoming.filter(([, id]) => !ids.includes(id));
    let act = 0;
    const items = [...drawn, ...later].map(([a, id, stepTitle, label]) => {
      const head = a !== act ? `<li class="act">${esc(W.acts[a])}</li>` : '';
      act = a;
      const isDrawn = ids.includes(id);
      const cls = id === stepId ? ' current' : isDrawn ? '' : ' later';
      const body = `<span>${esc(label || id)}</span><span>${esc(stepTitle)}${isDrawn ? '' : ' · not drawn yet'}</span>`;
      if (isDrawn && id !== stepId) {
        const p = new URLSearchParams(location.search);
        p.set('world', worldKey);
        p.set('step', id);
        return `${head}<li><a class="step-item" href="${location.pathname.split('/').pop()}?${p}">${body}</a></li>`;
      }
      return `${head}<li class="step-item${cls}">${body}</li>`;
    }).join('');
    return `<div class="mockbar-pop" data-popover="steps" hidden style="right:12px"><ul>${items}</ul></div>`;
  }

  function viewPop() {
    const file = location.pathname.split('/').pop() || 'index.html';
    const worlds = Object.entries(C.worlds).map(([key, w]) => {
      const p = new URLSearchParams(location.search);
      p.set('world', key);
      p.delete('step');
      if (!['chat', 'task'].includes(page)) p.delete('task');
      const target = ['chat', 'task'].includes(page) && key !== worldKey ? 'index.html' : file;
      return key === worldKey
        ? `<li class="step-item current"><span>●</span><span>${esc(w.name)}</span></li>`
        : `<li><a class="step-item" href="${target}?${p}"><span>○</span><span>${esc(w.name)}</span></a></li>`;
    }).join('');
    const toggle = (flag, label, what) => {
      const on = flag === 'plain' ? !document.body.classList.contains(FLAGS.plain) : document.body.classList.contains(FLAGS[flag]);
      return `<label class="view-toggle"><input type="checkbox" data-flag="${flag}" ${on ? 'checked' : ''}><span><b>${label}</b> ${what}</span></label>`;
    };
    const legend = Object.entries(KINDS).map(([k, name]) => `<li><span class="legend-${k}">${esc(name)}</span></li>`).join('');
    return `<div class="mockbar-pop view-pop" data-popover="view" hidden style="right:12px">
      <h4>World</h4><ul>${worlds}</ul>
      <h4>Show</h4>
      ${toggle('plain', 'Annotations', 'the reference case’s names, and the gaps temper lacks for what a page draws.')}
      ${toggle('sources', 'Sources', 'each value underlined by where it comes from; hover one for the detail. Unmarked text is the UI’s own words.')}
      <h4>Sources</h4><ul class="view-legend">${legend}</ul>
    </div>`;
  }

  function bindCasebar() {
    document.querySelectorAll('[data-go]').forEach(b => b.addEventListener('click', () => {
      if (!b.dataset.go) return;
      const p = new URLSearchParams(location.search);
      p.set('world', worldKey);
      p.set('step', b.dataset.go);
      location.search = p.toString();
    }));
    document.querySelectorAll('[data-pop]').forEach(b => b.addEventListener('click', e => {
      e.stopPropagation();
      const pop = document.querySelector(`[data-popover="${b.dataset.pop}"]`);
      const open = pop.hidden;
      document.querySelectorAll('[data-popover]').forEach(p => { p.hidden = true; });
      document.querySelectorAll('[data-pop]').forEach(x => x.setAttribute('aria-expanded', 'false'));
      pop.hidden = !open;
      b.setAttribute('aria-expanded', String(open));
    }));
    document.addEventListener('click', e => {
      if (e.target.closest('.mockbar-pop')) return;
      document.querySelectorAll('[data-popover]').forEach(p => { p.hidden = true; });
    });
    document.querySelectorAll('[data-flag]').forEach(box => box.addEventListener('change', () => {
      const cls = FLAGS[box.dataset.flag];
      const on = box.dataset.flag === 'plain' ? !box.checked : box.checked;
      document.body.classList.toggle(cls, on);
      store(cls, on ? '1' : '0');
    }));
  }

  // The detail of a source, on hover, while sources show.
  function bindSources() {
    const tip = document.createElement('div');
    tip.className = 'src-tip';
    tip.hidden = true;
    document.body.append(tip);
    document.addEventListener('mouseover', e => {
      const s = document.body.classList.contains(FLAGS.sources) && e.target.closest('.src');
      if (!s) { tip.hidden = true; return; }
      const k = [...s.classList].find(c => c.startsWith('src-')).slice(4);
      tip.innerHTML = `<b class="legend-${k}">${esc(KINDS[k])}</b><span>${esc(s.dataset.src)}</span>`;
      tip.hidden = false;
      const r = s.getBoundingClientRect();
      const x = Math.min(Math.max(8, r.left), window.innerWidth - tip.offsetWidth - 8);
      const below = r.bottom + 6 + tip.offsetHeight < window.innerHeight;
      tip.style.left = `${x}px`;
      tip.style.top = `${below ? r.bottom + 6 : r.top - tip.offsetHeight - 6}px`;
    });
  }

  // ---- The product's chrome -------------------------------------------

  // In client README 5.3's order. Tasks and Workers are jig's pages
  // (pages.md, 6.3 and 7), not temper's yet; Adopt and Forges have no
  // mockup (client README, 4.4; forge.md, 5.4).
  const NAV = [
    ['inbox', 'inbox.html', 'Inbox'],
    ['chats', 'chats.html', 'Chats'],
    ['board', 'board.html', 'Board'],
    ['changes', 'changes.html', 'Changes'],
    ['tasks', 'tasks.html', 'Tasks'],
    ['adopt', '', 'Adopt a repository', 'No mockup: the adoption form is client README 4.4.'],
    ['forges', '', 'Forges', 'No mockup: the forges’ page is forge.md 5.4.'],
    ['workers', 'workers.html', 'Workers'],
    ['accounts', 'accounts.html', 'Accounts'],
  ];
  const navLink = ([key, h, label, later], active, n = '') => h
    ? `<a href="${u(h)}"${key === active ? ' class="active" aria-current="page"' : ''}>${label}${n}</a>`
    : `<a href="#" data-later="${esc(later)}">${label}</a>`;

  function topbar(active) {
    const links = NAV.map(e => navLink(e, active, e[0] === 'inbox' ? ` <span class="terminal-count">${world.inbox.length || ''}</span>` : '')).join('');
    return `<header class="terminal-topbar">
      <a class="terminal-brand" href="${u('chats.html')}" aria-label="temper chats"><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 1.5c1.5 5.6 2.9 7 8.5 8.5-5.6 1.5-7 2.9-8.5 8.5C10.5 12.9 9.1 11.5 3.5 10 9.1 8.5 10.5 7.1 12 1.5Z"/><circle cx="18.5" cy="18.5" r="2" fill="currentColor"/></svg><span>temper</span></a>
      <span class="terminal-divider" aria-hidden="true"></span>
      <span class="terminal-project">${project()} <span aria-hidden="true">⌄</span></span>
      <nav class="terminal-nav" aria-label="Main navigation">${links}</nav>
      <div class="terminal-top-right"><span class="terminal-live"><i class="status-dot"></i> ${fact('jig-client: the link’s state', 'Live')}</span><span class="terminal-user">${fact('jig: the party signed in', esc(world.person))}</span><a class="terminal-user" href="#" data-later="Mockup: signing out is not drawn.">Sign out</a></div>
      <details class="terminal-mobile-menu"><summary aria-label="Open navigation">☰</summary><nav aria-label="Mobile navigation">${NAV.map(e => navLink(e, active)).join('')}</nav></details>
    </header>`;
  }

  function listPage(active, crumbs, body, actions = '') {
    const trail = crumbs.map((c, i) => i === crumbs.length - 1 ? `<strong>${c[0]}</strong>` : `<a href="${u(c[1])}">${c[0]}</a><span class="sep">/</span>`).join('');
    document.body.classList.add('terminal-pages');
    return `${topbar(active)}<div class="app"><main class="main">
      <header class="topbar"><div class="breadcrumbs">${trail}</div><div class="top-actions">${actions}<button class="icon-button" aria-label="More" title="Visual mockup"></button></div></header>
      <div class="page">${body}</div>
    </main></div>`;
  }

  // ---- The tree -------------------------------------------------------

  function treeRows(roots, withRoots) {
    const rows = [];
    function walk(id, depth, lasts, ancestors) {
      const t = T(id);
      const children = kids(id);
      const glyph = depth === 0 ? '' : lasts.slice(0, -1).map(l => (l ? '   ' : '│  ')).join('') + (lasts[lasts.length - 1] ? '└─ ' : '├─ ');
      const running = t.worker && isLive(id);
      const where = t.kind === 'change'
        ? fact('temper: a change has no host of its own', '—')
        : fact(running ? `jig: the worker its run is on` : `jig: its charter's host kind; no run now`, esc(t.worker || t.hostKind));
      rows.push(`<div class="tree-row${isLive(id) ? '' : ' is-done'}${fresh(t)}" role="treeitem" style="--d:${depth}" data-id="${id}" data-anc="${ancestors.join(' ')}" data-live="${isLive(id)}" ${children.length ? 'aria-expanded="true"' : ''}>
        <span class="tree-name"><span class="tree-glyph">${glyph}</span><button type="button" class="tree-toggle" ${children.length ? `data-toggle="${id}" aria-label="Fold ${id}"` : 'disabled tabindex="-1" aria-hidden="true"'}>${children.length ? '▾' : ''}</button><a href="${href(id)}"><b>${id}</b>${title(id)}</a>${ref(t.ref)}</span>
        <span class="muted">${kind(id)}</span>
        <span class="muted">${model(id)}</span>
        <span class="${running ? '' : 'muted'}">${where}</span>
        <span class="tree-state ${stateClass(id)}">${phase(id, true)}</span>
        <span class="tree-spend">${spent(id)}</span>
      </div>`);
      children.forEach((k, i) => walk(k, depth + 1, [...lasts, i === children.length - 1], [...ancestors, id]));
    }
    if (withRoots) roots.forEach(r => walk(r, 0, [], []));
    else roots.forEach((r, i) => walk(r, 1, [i === roots.length - 1], []));
    return rows.join('');
  }

  function tree(roots, withRoots) {
    return `<div class="tree" role="tree">
      <div class="tree-row tree-head" aria-hidden="true"><span>Task</span><span>Kind</span><span>Model</span><span>On</span><span>State</span><span class="tree-spend">Spent</span></div>
      ${treeRows(roots, withRoots)}
    </div>`;
  }

  function bindTrees() {
    const folded = new Set();
    let liveOnly = false;
    function apply() {
      document.querySelectorAll('.tree-row[data-id]').forEach(row => {
        const anc = row.dataset.anc ? row.dataset.anc.split(' ') : [];
        const endedLeaf = row.dataset.live === 'false' && descendants(row.dataset.id).every(k => !isLive(k));
        row.hidden = anc.some(a => folded.has(a)) || (liveOnly && endedLeaf);
      });
    }
    document.querySelectorAll('[data-toggle]').forEach(b => b.addEventListener('click', () => {
      const id = b.dataset.toggle;
      const row = b.closest('.tree-row');
      if (folded.has(id)) { folded.delete(id); b.textContent = '▾'; row.setAttribute('aria-expanded', 'true'); }
      else { folded.add(id); b.textContent = '▸'; row.setAttribute('aria-expanded', 'false'); }
      apply();
    }));
    const set = on => {
      liveOnly = on;
      document.querySelectorAll('[data-live-only]').forEach(x => x.classList.toggle('active', on));
      document.querySelectorAll('[data-all]').forEach(x => x.classList.toggle('active', !on));
      apply();
    };
    document.querySelectorAll('[data-live-only]').forEach(b => b.addEventListener('click', () => set(!liveOnly)));
    document.querySelectorAll('[data-all]').forEach(b => b.addEventListener('click', () => set(false)));
  }

  // ---- Shared parts ---------------------------------------------------

  function slotsGlyph(used, total) { return `<span class="slots" role="img" aria-label="${used} of ${total} slots in use">${'<i class="on"></i>'.repeat(used)}${'<i></i>'.repeat(Math.max(0, total - used))}</span>`; }

  function repoHealth(r) {
    const red = r.ci.state !== 'passing' || (r.after && r.after.state !== 'passing');
    return `<span class="tag ${red ? 'red' : 'green'}">${fact(`temper: CI at ${r.name}'s tip, and what runs after a landing`, red ? `${r.branch} red` : `${r.branch} healthy`)}</span>`;
  }

  function findingRow(f) {
    const fix = T(f.fix);
    return `<div class="finding${fresh(f)}">
      <span class="finding-id">${esc(f.id)}</span>
      <div>
        <h3>${esc(f.title)} <span class="muted" style="font-weight:400">· ${esc(world.repos[f.repo].name)}</span></h3>
        <p><code>${esc(f.signature)}</code></p>
        <p>${esc(f.rate)} · matched at heads ${f.seen.heads} · at the tip ${f.seen.tip} · opened ${ago(f.opened)} from ${chip(f.from)}</p>
        <p>Goal ${chip(f.goal)} · fix ${chip(f.fix)} · ${phase(f.fix)}</p>
      </div>
      <span class="tag amber">${f.state === 'open' ? 'Open' : esc(f.state)}</span>
    </div>`;
  }

  function usageLine(label, pct, reset, pace) {
    const over = pace != null && pct > pace;
    return `<div class="usage-line"><div><span>${esc(label)}</span><span>${pct}%${reset ? ` · resets ${esc(reset)}` : ''}</span></div>
      <div class="usage-bar"><span class="${over ? 'over' : ''}" style="width:${pct}%"></span>${pace != null ? `<i style="left:${pace}%" title="Pace: where the week would be if spent evenly"></i>` : ''}</div></div>`;
  }

  function event(e, body) { return `<div class="event${fresh(e)}"><time>${fact('jig: when it was committed', clock(e.at))}</time><div>${body}</div></div>`; }

  function toast(message) {
    const el = document.createElement('div');
    el.className = 'demo-toast';
    el.textContent = message;
    document.body.append(el);
    setTimeout(() => el.remove(), 2600);
  }
  function bindLaterActs() {
    document.querySelectorAll('[data-later]').forEach(b => b.addEventListener('click', () => toast(b.dataset.later || 'Mockup: this act is not drawn.')));
  }

  // ---- A conversation, its cards and proposals ------------------------

  // A delegation's card, at the call that made it (client README, 5.7).
  function taskCard(k) {
    const t = T(k);
    const result = isLive(k) ? '' : t.kind === 'change' ? (t.merge ? `CI passed. Landed · merge commit ${t.merge.slice(0, 7)}` : '') : (t.result || '');
    const steps = t.kind === 'change'
      ? fact('temper: the steps, a stand-in, all ticked once landed (client README, 5.7)', `<div class="steps" role="list" aria-label="Steps">${['Branch pushed', 'Pull request opened', 'CI passed', 'Landed'].map(w => `<div class="step${t.stage === 'landed' ? ' done' : ''}" role="listitem">${w}</div>`).join('')}</div>`)
      : '';
    return `<article class="terminal-proposal task-card${fresh(t)}" aria-label="Task ${k.slice(1)}">
      <div class="terminal-proposal-heading"><span class="terminal-proposal-type">${fact(`jig: ${k}'s number, and its result contract`, `${k} · ${contract(k)}`)}</span><span>${fact(`jig: ${k}'s document, in temper's words`, cardState(k))}</span></div>
      <h2>${title(k)}</h2>
      ${result ? `<div class="task-card-result">${said(k, md(result), t.ended)}</div>` : ''}${steps}
      <a class="text-link" href="${href(k)}">Task ${k.slice(1)} ↗</a>
    </article>`;
  }

  // A proposal (jig's client objects.md, section 4), in the inbox or at
  // the call that made it.
  const number = n => String(+n.toFixed(1));
  function proposalCard(e, where) {
    const b = e.batch[0];
    const left = world.config.allotment - spentThisPeriod();
    const items = [
      ['Why', said(e.task, `“${esc(e.why)}”`, e.at)],
      ['Goal', `${e.own ? 'yours · ' : ''}${conf('the template and charter it names, in the configuration', esc(`${b.template} · ${b.charter}`))}`],
      ['Landing', fact('jig: the grant it asks for, in words', esc(`${b.landing} · priority ${b.priority}`))],
      ['Budget', fact('jig: what it needs, and who would fund it, from the proposal’s document', esc(`Funds ${b.budget} from your pool · ${number(left)} left, ${number(left - b.budget)} after`))],
    ];
    const waiting = w => fact('jig: since it was proposed', `${w} ${span(world.now - e.at)}`);
    if (where === 'chat') {
      return `<article class="terminal-proposal${fresh(e)}" aria-label="Proposal from ${e.task}">
        <div class="terminal-proposal-heading"><span class="terminal-proposal-type">PROPOSAL</span><span class="terminal-proposal-wait">${waiting('waiting')}</span></div>
        <h2>${said(e.task, esc(e.title), e.at)}</h2><p>${said(e.task, inline(e.text), e.at)}</p>
        <dl>${items.map(([k, v]) => `<div><dt>${k}</dt><dd>${v}</dd></div>`).join('')}</dl>
        <div class="terminal-proposal-actions"><button type="button" class="terminal-primary" data-later>Accept goal</button><button type="button" data-later>Reject with reason</button><a href="${u('inbox.html')}">Open in inbox ↗</a></div>
      </article>`;
    }
    return `<article class="card inbox-card${fresh(e)}"><div class="card-top"><div class="meta-row"><span class="tag accent">Proposal</span><span>${lineage(e.task)}</span></div><span class="muted tiny">${waiting('Waiting')}</span></div>
      <h3>${said(e.task, esc(e.title), e.at)}</h3><p>${said(e.task, inline(e.text), e.at)}</p>
      <div class="proposal-items">${items.map(([k, v]) => `<div class="proposal-item"><span class="label">${k}</span><span>${v}</span></div>`).join('')}</div>
      <div class="button-row"><button class="button primary" data-later>Accept goal</button><button class="button" data-later>Reject with reason</button><a class="button ghost" href="${href(e.task)}">Open ${esc(e.task)} ↗</a></div></article>`;
  }

  // A conversation's lines (jig's client objects.md, section 10), its
  // turns named as client README 5.1 names them: "You", "Temper", or the
  // party's name. A delegation shows as its tasks' cards, and a proposal
  // as its card, at the call that made them.
  function conversation(id, shown) {
    const t = T(id);
    return (world.transcripts[id] || []).map(x => {
      if (x.day) return `<div class="terminal-day">${esc(x.day)}${x.earlier ? ` · <button type="button" data-later="Mockup: the earlier turns are not drawn.">${x.earlier} earlier turns</button>` : ''}</div>`;
      if (x.who === 'you') return `<section class="terminal-turn terminal-turn-user${fresh(x)}"><div class="terminal-role">You <time>${clock(x.at)}</time></div><div class="terminal-turn-content"><span class="terminal-prompt">›</span>${said(world.person, md(x.text, '<br>'), x.at)}</div></section>`;
      if (x.who === 'agent' && x.live) return `<section class="terminal-turn provisional${fresh(x)}"><div class="terminal-role">Temper <span class="provisional-mark">${fact(`jig: ${id}'s provisional text, before its turn commits`, 'writing')}</span></div><div class="terminal-turn-content"><span class="terminal-prompt agent">✳</span>${said(id, md(x.text).replace(/<\/p><\/div>$/, '<span class="stream-caret" aria-hidden="true"></span></p></div>'), x.at)}</div></section>`;
      if (x.who === 'agent') return `<section class="terminal-turn${fresh(x)}"><div class="terminal-role">Temper <time>${clock(x.at)}</time></div><div class="terminal-turn-content"><span class="terminal-prompt agent">✳</span>${said(id, md(x.text), x.at)}</div></section>`;
      if (x.who === 'tools') {
        const made = (x.made || []).filter(k => T(k));
        const calls = x.calls.filter(([name]) => !(name === 'delegate' && made.length) && !(name === 'propose' && x.proposes));
        const n = calls.reduce((sum, [, c]) => sum + c, 0);
        const work = n ? `<details class="terminal-work${fresh(x)}"><summary>${fact(`jig: ${id}'s run, its tool calls folded`, plural(n, 'tool call'))} <span>${calls.map(([name, c]) => `${esc(name)}${c > 1 ? ` ×${c}` : ''}`).join(' · ')}</span></summary><div class="terminal-work-detail">${calls.map(([name, c, g]) => `<div>${fact('jig: a tool call, by its tool’s name', `${esc(name)}${c > 1 ? ` ×${c}` : ''}`)}${g ? ` ${gap(g, GAPS[g])}` : ''}</div>`).join('')}</div></details>` : '';
        made.forEach(k => shown.add(k));
        const proposal = x.proposes ? world.inbox.find(e => e.id === x.proposes) : null;
        return work + made.map(taskCard).join('') + (proposal ? proposalCard(proposal, 'chat') : '');
      }
      return '';
    }).join('');
  }
  const GAPS = { '6.3': 'plan usage', '6.7': 'fleet occupancy' };

  // ---- Pages ----------------------------------------------------------

  const pages = {};

  pages.index = () => {
    const look = step.look.map(([h, text]) => `<li><a href="${u(h)}">${esc(text)}</a></li>`).join('');
    const worlds = Object.entries(C.worlds).map(([key, w]) => key === worldKey
      ? `<li><b>${esc(w.name)}</b> · ${esc(w.about)}</li>`
      : `<li><a href="index.html?world=${key}">${esc(w.name)}</a> · ${esc(w.about)}</li>`).join('');
    const pagesList = [['inbox.html', 'Inbox'], ['chats.html', 'Chats'], ['board.html', 'Board'], ['changes.html', 'Changes'], ['tasks.html', 'Tasks'], ['workers.html', 'Workers'], ['accounts.html', 'Accounts']]
      .map(([h, label]) => `<a href="${u(h)}">${label}</a>`).join(' · ');
    return `<div class="mock-index">
      <h1>temper's client, in mockups</h1>
      <p>Each page is drawn from a world at a step, in the bar above: ‹ and › move the step, <b>Steps</b> lists them, and <b>View</b> switches the world and what is marked. The README says what each page follows, and what it draws ahead of temper's design.</p>
      <h2>Worlds</h2>
      <ul>${worlds}</ul>
      <h2>${esc(W.name)} · ${esc(step.label)} · ${esc(step.title)}</h2>
      <p>${esc(step.summary)}</p>
      <ol>${look}</ol>
      <h2>Pages</h2>
      <p>${pagesList}; a chat, a task and a change open from them. <a href="first-run.html">first-run.html</a> is the owner's first chat after <code>temper box</code>, its own story on the demo's fake forge and scripted model.</p>
      <h2>Marks</h2>
      <p>Slate marks are annotations, not the product: the reference case's names (<i class="case-ref">B</i>, <i class="case-ref">f1</i>) and the gaps temper lacks for what a page draws (<i class="gap">gap 6.5</i>). <b>Sources</b>, in View, underlines each value by where it comes from: <span class="legend-engine">an engine fact</span>, <span class="legend-config">a name from the configuration</span>, <span class="legend-written">words as a person or an agent wrote them</span>, or <span class="legend-ops">an ops application</span>. Unmarked text is the UI's own words, and names no workflow.</p>
    </div>`;
  };

  function lineage(id) {
    const chain = [];
    for (let k = id; k && T(k); k = T(k).parent) chain.unshift(k);
    return `${project()} › ${chain.map(k => fact(`${k}: its number and title`, `${k} ${esc(T(k).title)}`)).join(' › ')}`;
  }

  function entryCard(e) {
    if (e.kind === 'proposal') return proposalCard(e, 'inbox');
    if (e.kind === 'question') {
      return `<article class="card inbox-card${fresh(e)}"><div class="card-top"><div class="meta-row"><span class="tag amber">Question</span><span>${lineage(e.task)}</span></div><span class="muted tiny">${fact('jig: since it was asked', `Waiting ${span(world.now - e.at)}`)}</span></div>
        ${said(e.task, md(e.text), e.at)}
        <div class="button-row">${e.options.map(o => `<button class="button" data-later>${said(e.task, esc(o), e.at)}</button>`).join('')}<button class="button ghost" data-later>Answer in words</button><a class="button ghost" href="${href(e.task)}">Open ${esc(e.task)} ↗</a></div></article>`;
    }
    return '';
  }

  // Replies in the person's chats, and results of what they asked for.
  function updates() {
    const list = [];
    for (const [id, lines] of Object.entries(world.transcripts)) {
      const t = T(id);
      if (!t || requesterOf(t) !== world.person) continue;
      const replies = lines.filter(x => x.who === 'agent');
      if (!replies.length) continue;
      const last = replies[replies.length - 1];
      const unread = replies.filter(x => !x.read).length;
      list.push({ at: last.at, unread, html: `<a class="card update" href="${href(id)}"><span class="update-icon">✳</span><div><h3>${fact(`${id}: its number and title`, `${id} ${esc(t.title)}`)} replied</h3><p>${said(id, esc(firstLine(last.text)), last.at)}</p><p>${fact("jig: your read position in it", `${unread ? `${unread} new · ` : ''}${ago(last.at)}`)}</p></div></a>` });
    }
    for (const [id, t] of Object.entries(world.tasks)) {
      if (requesterOf(t) !== world.person || isChat(t) || isLive(id)) continue;
      list.push({ at: t.ended, unread: 1, html: `<a class="card update" href="${href(id)}"><span class="update-icon">✓</span><div><h3>${fact(`${id}: its number and title`, `${id} ${esc(t.title)}`)} is done</h3><p>${said(id, esc(firstLine(t.result || '')), t.ended)}</p></div></a>` });
    }
    return list.sort((a, b) => b.at - a.at);
  }

  pages.inbox = () => {
    const needs = world.inbox.map(entryCard).join('') || '<div class="empty-state">Nothing needs you.</div>';
    const ups = updates();
    const unread = ups.filter(x => x.unread).length;
    return listPage('inbox', [['Inbox']], `<div class="inbox-split"><div>
      <div class="section-heading" style="margin-top:0"><h2>Needs you <span class="subtle-count">${world.inbox.length}</span></h2><div class="filter-row"><a class="button" href="${u('chats.html')}">+ New chat</a><button class="filter active">All goals</button><button class="filter">Kind ⌄</button></div></div>
      ${needs}
    </div><aside><div class="section-heading" style="margin-top:0"><h2>Updates <span class="subtle-count">${unread ? `${unread} new` : ups.length}</span></h2></div><div class="updates-list">${ups.map(x => x.html).join('') || '<div class="empty-state">No updates.</div>'}</div></aside></div>`);
  };

  pages.chats = () => {
    const live = Object.keys(world.tasks).filter(k => isChat(T(k)) && requesterOf(T(k)) === world.person && isLive(k));
    const rows = live.map(id => {
      const t = T(id);
      const lines = world.transcripts[id] || [];
      const replies = lines.filter(x => x.who === 'agent');
      const last = replies[replies.length - 1];
      const activity = Math.max(...lines.filter(x => x.at != null).map(x => x.at), t.made);
      return `<a class="card chat-preview${fresh(t)}" href="${href(id)}"><div><div class="meta-row"><span class="tag ${stateClass(id) === 'running' ? 'green' : 'amber'}">${phase(id, true)}</span><span>${project()} · ${id}</span>${ref(t.ref)}</div><h3>${title(id)}</h3>${last ? `<p>${said(id, esc(firstLine(last.text)), last.at)}</p>` : ''}</div><div class="meta-row"><span>${spent(id)} spent${kids(id).length ? ` · ${fact('jig: its delegates, live or ended', liveKids(id).length ? plural(liveKids(id).length, 'delegate') : `${plural(kids(id).length, 'delegate')}, ended`)}` : ''}</span><span>${fact('jig: its last activity', ago(activity))}</span></div></a>`;
    });
    const ended = world.ended.filter(e => e.by === world.person).map(e => `<a class="card chat-preview" href="#"><div><div class="meta-row"><span class="tag">Closed</span><span>${project()} · ${e.task}</span></div><h3>${said(e.by, esc(e.title))}</h3><p>${fact('jig: its retention class, kept', 'Closed. Its transcript is kept.')}</p></div><div class="meta-row"><span>${fact('jig: what it spent', esc(e.spend))} spent</span><span>${fact('jig: when it ended', ago(e.ended))}</span></div></a>`);
    return listPage('chats', [['Chats']], `<div class="welcome"><div class="section-heading" style="margin-top:0"><h2>Start a chat</h2></div><div class="terminal-entry-composer"><span class="entry-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Start a new chat" placeholder="Ask temper anything…"></textarea><span class="entry-hint">temper · Enter to start · Shift+Enter for newline</span><button class="send-button" aria-label="Start chat" data-later>↑</button></div></div>
      <div class="section-heading"><h2>Your chats</h2><div class="filter-row"><button class="filter active">All</button><button class="filter">Live</button><button class="filter">Closed</button></div></div><div class="chat-list">${[...rows, ...ended].join('')}</div>`);
  };

  pages.board = () => {
    const goals = Object.keys(world.tasks).filter(k => T(k).tracked && isLive(k)).sort((a, b) => T(a).priority - T(b).priority);
    const changes = Object.keys(world.tasks).filter(k => T(k).kind === 'change' && isLive(k));
    const ready = changes.filter(k => ['queued', 'first'].includes(T(k).stage)).length;
    const total = spentThisPeriod();
    const held = Object.keys(world.tasks).filter(k => T(k).phase === 'held').length;
    const rows = goals.map(g => {
      const t = T(g);
      const sub = descendants(g);
      const queued = sub.filter(k => T(k).kind === 'change' && ['queued', 'first'].includes(T(k).stage)).length;
      const running = sub.filter(k => T(k).kind !== 'change' && T(k).phase === 'running').length;
      const landed = sub.filter(k => T(k).kind === 'change' && T(k).stage === 'landed').length;
      const findings = world.findings.filter(f => f.goal === g && f.state === 'open').map(f => f.id);
      const pct = Math.round((t.spend / t.budget) * 100);
      const req = requesterOf(t);
      return `<article class="card board-row${fresh(t)}"><div class="rank">${fact('jig: its priority among the project’s goals', t.priority)}</div><div><div class="meta-row"><span class="tag ${stateClass(g) === 'running' ? 'green' : 'amber'}">${phase(g, true)}</span><span class="tiny">${g} · ${kind(g)}</span>${ref(t.ref)}</div><h3 style="margin-top:7px"><a href="${href(g)}">${title(g)}</a></h3><p>${fact('jig and temper: the tasks and changes in its tree', `${plural(landed, 'change')} landed · ${queued} queued · ${plural(running, 'delegate')} running`)}${findings.length ? ` · finding ${findings.join(', ')} open` : ''} · requested by ${req === world.person ? 'you' : chip(req)}</p></div><div><div class="progress"><span style="width:${pct}%"></span></div><p>${fact('jig: its budget, and what its tree spent', `${t.spend} of ${t.budget} spent`)} · ${fact('jig: its deadline', t.deadline ? `due ${esc(t.deadline)}` : 'no deadline')} · ${fact(`jig: ${t.worker ? 'the worker its run is on' : 'its host kind'}`, `on ${esc(t.worker || t.hostKind)}`)}</p></div><div class="right"><a class="text-link" href="${href(g)}">View goal ↗</a></div></article>`;
    }).join('') || '<div class="empty-state">No live goal.</div>';
    const repos = Object.values(world.repos).map(r => `<div class="kv"><span>${esc(r.branch)} in ${esc(r.name)}</span><strong>${fact(`temper: CI at ${r.name}'s tip`, r.ci.state === 'passing' ? '<span class="ok">CI passing at the tip</span>' : '<span class="bad">CI failing at the tip</span>')}${r.after ? ` · ${fact('temper: the check that runs after a landing, by the forge’s name', `${esc(r.after.name)} ${esc(r.after.state)}`)}` : ''} · ${fact('temper: its landing queue', plural(r.queue.length, 'change'))} queued</strong></div>`).join('');
    const open = world.findings.filter(f => f.state === 'open');
    const workers = world.workers.map(w => `${w.name} ${liveRuns(w.name).length}/${w.slots}`).join(' · ');
    return listPage('board', [[project(), 'board.html'], ['Board']], `
      <div class="stats-row"><div class="card stat-card"><span class="muted tiny">Live goals</span><strong class="value">${fact('jig: the project’s live goals', goals.length)}</strong><small>${fact('jig: held tasks in their trees', held ? `${held} held` : 'none held')}</small></div><div class="card stat-card"><span class="muted tiny">Changes in flight</span><strong class="value">${fact('temper: live changes', changes.length)}</strong><small>${fact('temper: changes in landing queues', `${ready} ready, in queues`)}</small></div><div class="card stat-card"><span class="muted tiny">Needs you</span><strong class="value">${fact('jig: your inbox’s entries', world.inbox.length)}</strong><small><a class="text-link" href="${u('inbox.html')}">Open inbox ↗</a></small></div><div class="card stat-card"><span class="muted tiny">Spent this ${conf('the configuration’s period', esc(world.config.period))}</span><strong class="value">${fact('jig: spent under your authority this period', total.toFixed(1))}</strong><small>${conf('your allotment per period, in the configuration', `of ${world.config.allotment} allotted`)}</small></div></div>
      <div class="section-heading"><h2>Priority order</h2><div class="filter-row"><button class="button primary" data-later>+ Set a goal</button><button class="filter active">Live</button><button class="filter">Waiting</button><button class="filter">Recently ended</button></div></div>
      <div class="stack">${rows}</div>
      <div class="section-heading"><h2>Project health</h2><a class="text-link" href="${u('changes.html')}">See changes and queues ↗</a></div>
      <div class="card card-pad">${repos}<div class="kv"><span>Open findings</span><strong>${open.map(f => `${esc(f.id)} · ${esc(f.title)}`).join(', ') || 'none'} ${gap('6.5', 'an open finding at the landing tip')}</strong></div><div class="kv"><span>Workers</span><strong><a class="text-link" href="${u('workers.html')}">${fact('jig: workers’ slots and live runs', `${esc(workers)} slots`)} ↗</a></strong></div></div>`);
  };

  pages.tasks = () => {
    const roots = Object.keys(world.tasks).filter(k => requesterOf(T(k)) === world.person && isLive(k)).sort((a, b) => T(a).made - T(b).made);
    const all = roots.flatMap(r => [r, ...descendants(r)]);
    const running = all.filter(k => T(k).phase === 'running').length;
    const waiting = all.filter(k => T(k).phase === 'idle').length;
    const queued = all.filter(k => ['queued', 'first'].includes(T(k).stage)).length;
    return listPage('tasks', [[project(), 'board.html'], ['Tasks']], `
      <div class="section-heading" style="margin-top:0"><div><h2>Every task <span class="subtle-count">${all.length}</span></h2><p class="muted tiny" style="margin:5px 0 0">What you requested, and what each delegated: ${fact('jig: the tasks’ phases', `${running} running · ${waiting} waiting · ${queued} in queues`)}.</p></div><div class="filter-row"><button class="filter active" data-all>All</button><button class="filter" data-live-only>Live only</button></div></div>
      <section class="card card-pad">${tree(roots, true)}</section>`);
  };

  pages.task = () => {
    const id = params.get('task') || Object.keys(world.tasks).find(k => T(k).tracked);
    const t = T(id);
    if (!t) return listPage('tasks', [[project(), 'board.html'], ['Tasks', 'tasks.html'], [esc(id)]], `<div class="empty-state">${esc(id)} does not exist at this step.</div>`);
    if (isChat(t)) { location.replace(href(id)); return ''; }
    if (t.tracked) return goalPage(id, t);
    if (t.kind === 'change') return changePage(id, t);
    return agentPage(id, t);
  };

  function goalPage(id, t) {
    const all = descendants(id);
    const pct = Math.round((t.spend / t.budget) * 100);
    const holders = all.filter(k => T(k).pool && isLive(k));
    const findings = world.findings.filter(f => f.goal === id);
    const notes = world.notes.filter(n => n.scope === 'goal' && n.goal === id);
    const worker = world.workers.find(w => w.name === t.worker);
    const req = requesterOf(t);
    const reqWords = req === world.person ? 'you' : chip(req);
    const activity = (world.activity[id] || []).map(e => event(e, `<strong>${fact(`jig: ${e.task}, ${e.what}`, `${e.task} ${esc(T(e.task) ? T(e.task).title : '')} · ${esc(e.what)}`)}</strong>${e.gap ? ` ${gap(e.gap, 'an open finding')}` : ''}<p>${fact('jig and temper: what the commit records', esc(e.text))}</p>`)).join('');
    const history = (world.history[id] || []).map(e => event(e, `<strong>${fact('jig: the task’s history, by whom', `${esc(who(e.by))} ${esc(e.what)}`)}</strong>${e.reason ? `<p>${said(e.by, `“${esc(e.reason)}”`, e.at)}</p>` : ''}`)).join('');
    const changes = all.filter(k => T(k).kind === 'change' && isLive(k));
    const run = t.run ? fact(`jig: run ${t.run.n} of ${id}`, `Run ${t.run.n}${t.run.waiting ? ' · waits, holding its slot' : ''}`) : fact('jig: no live run', 'No run now');
    return listPage('tasks', [[project(), 'board.html'], ['Board', 'board.html'], [id]], `
      <div class="page-header"><div class="task-heading">
        <div class="meta-row"><span class="tag accent">${fact('jig: tracked, a goal, with its priority', `Goal · priority ${t.priority}`)}</span><span class="task-chip"><b>${id}</b> ${kind(id)}</span>${ref(t.ref)}<span>${project()}</span></div>
        <h1>${title(id)}</h1>
        <div class="meta-row" style="margin-top:12px"><span class="tag ${stateClass(id) === 'running' ? 'green' : 'amber'}"><i class="pill-dot"></i> ${phase(id)}</span><span>Requested by ${reqWords} · ${fact('jig: when it was made', `made ${ago(t.made)}`)}</span></div>
        <div class="facts-line"><span>Model <b>${model(id)}</b></span><span>Account <b>${fact('jig: the account its runs use, kept from run to run', esc(t.account))}</b></span><span>Host kind <b>${conf('its charter’s host kind, in the configuration', esc(t.hostKind))}</b></span><span>${run}</span></div>
      </div><div class="task-toolbar"><button class="button" data-later>Amend</button><button class="button" data-later>Stop</button><button class="button danger" data-later>Cancel</button></div></div>
      <div class="tabbar"><a class="active" href="#overview">Overview</a><a href="#plan">Plan</a><a href="#findings">Findings</a><a href="#notes">Notes</a><a href="#activity">Activity</a><a href="#history">History</a></div>
      <div class="two-col"><div class="stack">
        <section id="overview" class="card card-pad"><div class="card-top"><h2>What was asked</h2><span class="tag outline">${fact('jig: its result contract', 'Result · report')}</span></div>
          <div style="margin:12px 0 0;font-size:12px;line-height:1.7">${said(t.by, md(t.brief), t.made)}</div>
          <div class="rule"></div>
          <div class="kv"><span>Input</span><strong>${fact('jig: an input its batch named', esc(t.input))}</strong></div>
          <div class="kv"><span>Writes</span><strong>${fact('jig: the resources it writes, held alone', esc(t.writes))}</strong></div>
          <div class="kv"><span>Wakes on</span><strong>${fact('jig: its wake policy, by kind of message', esc(t.wakesOn.join(', ')))}</strong></div>
        </section>
        <section id="plan" class="card card-pad"><div class="card-top"><div><h2>Plan <span class="subtle-count">${plural(all.length, 'task')}</span></h2><p class="muted tiny" style="margin:5px 0 0">Its delegates: ${fact('jig: its delegates’ phases', `${all.filter(k => T(k).phase === 'running').length} running, ${all.filter(k => ['queued', 'first'].includes(T(k).stage)).length} in queues, ${all.filter(k => !isLive(k)).length} ended`)}.</p></div><div class="filter-row"><button class="filter active" data-all>All</button><button class="filter" data-live-only>Live only</button></div></div>
          <div style="margin-top:10px">${tree(kids(id), false)}</div>
          ${t.pool ? `<div class="rule"></div><div class="meta-row"><span>Pool <b style="color:var(--term-ink)">${fact(`jig: a pool ${id} holds, its name as ${who(t.by)} gave it`, esc(t.pool.name))}</b></span>${slotsGlyph(holders.length, t.pool.size)}<span>${fact('jig: the pool’s holders', `${holders.length} of ${t.pool.size} held: ${holders.join(', ') || 'none'}`)}</span><span>· a new delegate waits for a slot</span></div>` : ''}
        </section>
        <section id="findings" class="card card-pad"><div class="card-top"><h2>Findings on this goal <span class="subtle-count">${findings.length}</span></h2>${gap('6.5', 'an open finding at the landing tip')}</div>${findings.map(findingRow).join('') || '<p class="muted">None.</p>'}</section>
        <section id="notes" class="card card-pad"><div class="card-top"><div><h2>Notes <span class="subtle-count">${notes.length}</span></h2><p class="muted tiny" style="margin:5px 0 0">In its scope: each brief in its tree carries their index.</p></div></div>${notes.length ? `<ul class="note-list">${notes.map(n => `<li>${said(n.by, `<b>${esc(n.name)}</b> · ${inline(n.text)}`, n.at)}</li>`).join('')}</ul>` : '<p class="muted">None.</p>'}</section>
        <section id="activity" class="card card-pad"><div class="card-top"><h2>Latest activity</h2><span class="tag green">Live</span></div><div class="timeline">${activity || '<p class="muted">None.</p>'}</div></section>
        <section id="history" class="card card-pad"><div class="card-top"><h2>History</h2><button class="filter">Whole subtree ⌄</button></div><div class="timeline">${history}</div></section>
      </div>
      <aside class="stack">
        <section class="card side-card"><h3>Budget & authority</h3><div class="progress"><span style="width:${pct}%"></span></div><p class="mini-stat">${fact('jig: its budget, and what its tree spent', `${t.spend} spent · ${(t.budget - t.spend).toFixed(1)} left of ${t.budget}`)}</p><div class="rule"></div><div class="kv"><span>Funded by</span><strong>${req === world.person ? fact('jig: who funded its budget', esc(world.person)) : chip(req)}</strong></div><div class="kv"><span>Deadline</span><strong>${fact('jig: its deadline', esc(t.deadline || 'none'))}</strong></div></section>
        <section class="card side-card"><h3>Placement</h3><div class="kv"><span>Host kind</span><strong>${conf('its charter’s host kind, in the configuration', esc(t.hostKind))}</strong></div><div class="kv"><span>Its run</span><strong>${worker ? `<a class="text-link" href="${u('workers.html')}">${fact('jig: the worker its run is on', `${esc(worker.name)} · ${liveRuns(worker.name).length} of ${worker.slots} slots`)} ↗</a>` : fact('jig: no live run', 'none now')}</strong></div>${t.pool ? `<div class="kv"><span>Pool</span><strong>${fact('jig: the pool’s holders', `${holders.length} of ${t.pool.size} ${esc(t.pool.name)}`)}</strong></div>` : ''}</section>
        <section class="card side-card"><h3>Its changes</h3><div class="aside-list">${changes.map(k => `<a href="${href(k)}"><span class="task-chip"><b>${k}</b> ${esc(T(k).title)}</span>${ref(T(k).ref)}<small>${phase(k, true)} · ${fact('temper: its pull request', T(k).pr ? `PR #${T(k).pr}` : 'no pull request yet')}</small></a>`).join('') || '<p class="muted tiny" style="margin:0">None live.</p>'}</div></section>
      </aside></div>`);
  }

  function changePage(id, t) {
    const repo = world.repos[t.repo];
    const pos = repo.queue.indexOf(id);
    const order = ['producing', 'opening', 'checking', 'gating', 'queued', 'landed'];
    const at = { first: 4, landing: 5 }[t.stage] ?? order.indexOf(t.stage);
    const names = order.map((s, i) => (i === 4 && pos >= 0 ? `Queued · ${ordinal(pos + 1)}` : cap(s)));
    const steps = names.map((n, i) => `<div class="step ${i < at || t.stage === 'landed' ? 'done' : i === at ? 'current' : ''}">${esc(n)}</div>`).join('');
    const fixes = t.fixes ? world.findings.find(f => f.id === t.fixes) : null;
    const goal = [id, ...[]].reduce(k => { let g = T(k).parent; while (g && !T(g).tracked) g = T(g).parent; return g; }, id);
    const ci = at > 2 ? 'Passed' : at === 2 ? 'Running' : 'Not yet';
    const gates = repo.gates || [];
    return listPage('changes', [[project(), 'board.html'], ['Changes', 'changes.html'], [id]], `
      <div class="page-header"><div class="task-heading"><div class="meta-row"><span class="tag accent">${fact('temper: a change', `Change · ${id}`)}</span>${ref(t.ref)}<span>Goal ${chip(goal)}</span></div><h1>${title(id)}</h1><p class="intro">${fact('temper: its pull request, branch and landing branch', `${t.pr ? `PR #${t.pr}` : 'No pull request yet'} · ${esc(repo.name)} · temper/${id.slice(1)}/${esc(t.title.toLowerCase().replace(/[^a-z0-9]+/g, '-'))} → ${esc(repo.branch)}`)}</p></div>
      <div class="task-toolbar"><a class="button" href="https://github.com/" target="_blank" rel="noreferrer">View on forge ↗</a></div></div>
      <section class="card card-pad">${fact(`temper: ${id}'s stage (forge.md, section 8)`, `<div class="steps">${steps}</div>`)}
        ${fixes ? `<div class="notice"><span>◇</span><div><strong>Fixes finding ${esc(fixes.id)}.</strong> ${esc(fixes.title)}: ${esc(fixes.rate)}. ${gap('6.5', 'an open finding at the landing tip')}</div></div>` : ''}
        <div class="section-heading"><h3>Its tasks</h3><span class="muted tiny">made by the change procedure: producing, repairs, reviews, resolving</span></div>
        ${kids(id).map(k => `<div class="gate"><strong><a href="${href(k)}">${k} ${title(k)}</a> <span class="muted" style="font-weight:400">${kind(k)}</span></strong><span>${phase(k, true)}</span></div>`).join('') || '<p class="muted">None yet.</p>'}
        <div class="section-heading"><h3>Gates at its head</h3></div>
        <div class="gate"><strong>CI at its head</strong><span>${fact('temper: CI on the change’s exact head', ci)}</span></div>
        ${gates.map(g => `<div class="gate"><strong>${conf(`a gate ${repo.name}'s landing rules add`, esc(g))}</strong><span>Not yet</span></div>`).join('') || `<div class="gate"><strong class="muted">No other gate</strong><span>${conf(`${repo.name}'s landing rules, in the project’s configuration`, 'its landing rules add none')}</span></div>`}
      </section>`);
  }

  function agentPage(id, t) {
    document.body.classList.add('terminal-variant', 'agent-task-page');
    const parent = T(t.parent);
    const live = t.phase === 'running';
    const body = !isLive(id)
      ? `<div class="agent-run-heading"><strong>Run 1 · ended</strong></div><div class="agent-result"><strong>RESULT</strong>${said(id, md(t.result), t.ended)}</div>`
      : `<div class="agent-run-heading"><strong>${t.run ? fact(`jig: run ${t.run.n} of ${id}`, `Run ${t.run.n} · ${live ? 'live now' : 'waiting'}`) : 'No run now'}</strong></div>
         <div class="agent-run-meta">${t.run ? fact('jig: when its run started, and on which worker', `Started ${clock(t.run.since)}${t.worker ? ` on ${esc(t.worker)}` : ''}`) : conf('its charter’s host kind', `Host kind ${esc(t.hostKind)}`)} · ${spent(id)} spent so far</div>
         ${world.transcripts[id] ? conversation(id, new Set()) : `<div class="agent-event"><strong>${phase(id)}</strong> <span class="mock-note">Its transcript is not drawn in these mockups.</span></div>`}`;
    return `<div class="terminal-shell">${topbar('tasks')}
      <div class="agent-lineage"><a href="${href(t.parent)}">← ${t.parent} ${esc(parent.title)}</a><span>/</span><span class="agent-lineage-current">${id} ${esc(t.title)}</span></div>
      <div class="terminal-chatbar agent-taskbar"><div class="terminal-chat-identity"><h1 class="chatbar-heading"><span class="terminal-number">${id}</span><span class="visually-hidden"> · </span><strong class="terminal-title">${title(id)}</strong></h1>${ref(t.ref)}<span class="terminal-phase${stateClass(id) === 'waiting' ? ' waiting' : stateClass(id) === 'done' ? ' closed' : ''}"><i class="pill-dot"></i> ${phase(id, true)}</span></div>
        <div class="terminal-chat-meta"><span class="agent-charter">${kind(id)} · ${model(id)}</span><span class="agent-spend">${spent(id)} spent</span>${isLive(id) ? '<button type="button" data-later>Amend</button><button type="button" data-later>Stop</button>' : ''}</div></div>
      <main class="terminal-thread" aria-label="Agent task"><div class="terminal-thread-inner">
        <div class="agent-brief"><span class="agent-kicker">TASK BRIEF</span><div class="agent-brief-text">${said(t.briefBy || t.by, md(t.brief), t.made)}</div><div class="agent-brief-meta">Asked by ${esc(t.parent)}${t.briefBy && t.briefBy !== t.parent ? `, from ${esc(t.briefBy)}’s spec` : ''} · ${fact('jig: the account its runs use', esc(t.account))} · ${model(id)}${t.pool ? ` · ${fact(`jig: a slot of ${t.parent}'s pool`, `holds a slot of ${t.parent}’s pool`)}` : ''}</div></div>
        ${body}
      </div></main>
      ${isLive(id) ? `<footer class="terminal-composer-area agent-steer-area"><div class="agent-steer-note">Steer this task · words reach its live run at the next turn.</div><div class="terminal-composer"><span class="terminal-input-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Write to agent task" placeholder="Write to ${esc(id)}…"></textarea><span class="terminal-input-hint">Enter to send · Shift+Enter for newline</span><button type="button" class="terminal-send" aria-label="Send" data-later>↑</button></div></footer>` : ''}
    </div>`;
  }

  pages.chat = () => {
    const id = params.get('task') || Object.keys(world.tasks).find(k => isChat(T(k)));
    const t = T(id);
    document.body.classList.add('terminal-variant');
    const shown = new Set();
    const lines = conversation(id, shown);
    // A live delegate whose call the window does not hold shows after the
    // conversation (client README, 5.7).
    const after = liveKids(id).filter(k => !shown.has(k)).map(taskCard).join('');
    const notes = world.notes.filter(n => n.by === id);
    const scopes = [...new Set(notes.map(n => n.scope))];
    return `<div class="terminal-shell">${topbar('chats')}
      <div class="terminal-chatbar"><div class="terminal-chat-identity"><h1 class="chatbar-heading"><span class="terminal-number">${id}</span><span class="visually-hidden"> · </span><strong class="terminal-title">${title(id)}</strong></h1>${ref(t.ref)}<span class="terminal-phase waiting"><i class="pill-dot"></i> ${phase(id)}</span></div>
        <div class="terminal-chat-meta"><span>${kind(id)} · ${model(id)} · ${conf('its charter’s host kind', esc(t.hostKind))}</span><span>${fact('jig: what it spent, and its budget', `${t.spend} spent · ${t.budget ? `of ${t.budget}` : 'as allowed'}`)}</span>${notes.length ? '<button type="button" data-notes aria-expanded="false">Notes</button>' : ''}${kids(id).length ? `<a href="${u('tasks.html')}" class="chatbar-link">${fact('jig: its delegates, live or ended', liveKids(id).length ? plural(liveKids(id).length, 'delegate') : `${plural(kids(id).length, 'delegate')}, ended`)}</a>` : ''}<button type="button" data-later>Stop</button><button type="button" data-later>Close</button></div></div>
      <main class="terminal-thread" aria-label="Conversation"><div class="terminal-thread-inner">${lines}${after}</div></main>
      <footer class="terminal-composer-area"><div class="composer-line">${usingLine(id)}</div><div class="terminal-composer"><span class="terminal-input-prompt" aria-hidden="true">›</span><textarea rows="1" aria-label="Message ${id}" placeholder="Message ${esc(id)}…"></textarea><span class="terminal-input-hint">Enter to send · Shift+Enter for newline</span><button type="button" class="terminal-send" aria-label="Send message" data-later>↑</button></div></footer>
    </div>
    ${notes.length ? `<aside class="drawer" id="notes" hidden aria-label="Notes ${id} wrote">
      <div class="drawer-head"><h2>${id} · the notes it wrote</h2><button type="button" data-notes-close>Close ✕</button></div>
      <p class="muted" style="margin:0">Notes are hints: where a note and the facts disagree, the facts are right.</p>
      ${scopes.map(s => `<h3>${fact('jig: the note’s scope', `${esc(s)} scope${s === 'project' ? ` · ${esc(world.project)}` : ''}`)}</h3><ul>${notes.filter(n => n.scope === s).map(n => `<li>${said(n.by, `<b>${esc(n.name)}</b>${md(n.text)}`, n.at)}</li>`).join('')}</ul>`).join('')}
      <p class="drawer-foot">Each an entry: a one-line description, a body, a scope and its writer.</p>
    </aside>` : ''}`;
  };

  pages.changes = () => {
    const repos = Object.entries(world.repos);
    const cards = repos.map(([key, r]) => `<section class="card card-pad repo-card"><div class="card-top"><div><h2>${fact('temper: an adopted repository, and its landing branch', `${esc(r.branch)} · ${esc(r.name)}`)}</h2><p class="muted tiny" style="margin:4px 0 0">${fact('temper: its landing queue', plural(r.queue.length, 'change'))} queued</p></div>${repoHealth(r)}</div>
      <div class="repo-lines"><div class="kv"><span>Tip</span><strong>${fact('temper: the branch’s tip', esc(r.tip))}</strong></div><div class="kv"><span>CI at the tip</span><strong class="${r.ci.state === 'passing' ? 'ok' : 'bad'}">${fact('temper: CI at the tip', `${r.ci.state === 'passing' ? 'Passing' : 'Failing'} · ${clock(r.ci.at)}`)}</strong></div>${r.after ? `<div class="kv"><span>After landing</span><strong class="${r.after.state === 'passing' ? 'ok' : 'bad'}">${fact('temper: the check that runs after a landing, by the forge’s name', `${esc(r.after.name)} ${esc(r.after.state)} at ${esc(r.after.head)} · ${clock(r.after.at)}`)}</strong></div>` : ''}</div></section>`).join('');
    const changeRow = (k, i, tag) => {
      const t = T(k);
      let g = t.parent;
      while (g && !T(g).tracked) g = T(g).parent;
      return `<div class="queue-row${fresh(t)}"><span class="queue-no">${i == null ? '·' : i + 1}</span><div><h3><a href="${href(k)}">${k} ${title(k)}</a>${ref(t.ref)}</h3><p>${g ? `Goal ${fact('jig: its goal, and the goal’s priority', `${g} ${esc(T(g).title)} · priority ${T(g).priority}`)}` : ''}${t.ready != null ? ` · ${fact('temper: since it was ready', `ready ${span(world.now - t.ready)}`)}` : ''}${t.fixes ? ` · fixes ${esc(t.fixes)}` : ''}</p></div><span class="tag ${tag}">${phase(k, true)}</span></div>`;
    };
    const queues = repos.map(([key, r]) => {
      const rows = r.queue.map((k, i) => changeRow(k, i, i === 0 ? 'accent' : '')).join('') || '<div class="empty-state">Nothing queued.</div>';
      return `<section class="card queue-col"><div class="card-pad" style="padding-bottom:8px"><div class="card-top"><div><h2>${esc(key)} · landing queue</h2><p class="muted tiny" style="margin:5px 0 0">By goal priority, then by when ready.</p></div><span class="tag ${r.queue.length ? 'green' : ''}">${fact('temper: its landing queue', `${r.queue.length} ready`)}</span></div></div>${rows}</section>`;
    }).join('');
    const notReady = Object.keys(world.tasks).filter(k => T(k).kind === 'change' && ['producing', 'opening', 'checking', 'gating'].includes(T(k).stage));
    const landings = world.landings.map(l => `<a href="${href(l.task)}">${fact('temper: a landing', `${l.task} ${esc(T(l.task) ? T(l.task).title : l.title)}`)}<small>${fact('temper: where and when it landed', `${esc(world.repos[l.repo].name)} · ${clock(l.at)}${T(l.task) && T(l.task).parent ? ` · goal ${T(l.task).parent}` : ''}`)}</small></a>`).join('');
    const open = world.findings.filter(f => f.state === 'open');
    return listPage('changes', [[project(), 'board.html'], ['Changes']], `
      <div class="compact-context"><span>${fact('temper: the forge and the adopted repositories', `Forge · GitHub · ${repos.map(([, r]) => esc(r.name)).join(', ')}`)}</span></div>
      <div class="repo-grid${repos.length === 1 ? ' one' : ''}">${cards}</div>
      <div class="section-heading"><h2>Landing queues</h2></div>
      <div class="repo-grid${repos.length === 1 ? ' one' : ''}">${queues}</div>
      <div class="section-heading"><h2>Not ready yet</h2><span class="muted tiny">${plural(notReady.length, 'change')}</span></div>
      ${notReady.length ? `<section class="card queue-col">${notReady.map(k => changeRow(k, null, '')).join('')}</section>` : '<div class="empty-state">No change is being produced, opened, checked or gated.</div>'}
      <div class="section-heading"><h2>Open findings <span class="subtle-count">${open.length}</span></h2>${gap('6.5', 'a failure that matches an open finding attaches to it and leaves the queue running')}</div>
      <section class="card card-pad">${world.findings.map(findingRow).join('') || '<p class="muted" style="margin:0">No open finding.</p>'}${world.findings.length ? '<p class="gap-note">Gap 6.5, as the case wants it: a failure at a tip or a head that matches an open finding is attached to it; the queue keeps landing, and the finding’s goal hears.</p>' : ''}</section>
      <div class="section-heading"><h2>Recent landings</h2></div>
      <section class="card side-card"><div class="aside-list">${landings}</div></section>`);
  };

  pages.workers = () => {
    const used = world.workers.reduce((n, w) => n + liveRuns(w.name).length, 0);
    const total = world.workers.reduce((n, w) => n + w.slots, 0);
    const cards = world.workers.map(w => {
      const runs = liveRuns(w.name);
      const h = w.health;
      const runRows = runs.map(k => {
        const t = T(k);
        const what = t.run && t.run.waiting
          ? fact(`jig: run ${t.run.n} of ${k}, waiting`, `run ${t.run.n} waits, holding its slot`)
          : `${kind(k)}${t.pool ? ` · ${fact(`jig: a slot of ${t.parent}'s pool`, `${t.parent}’s pool`)}` : ''}`;
        return `<div class="worker-run"><span>${chip(k)}</span><small>${what} · ${model(k)}</small></div>`;
      }).join('') || '<p class="muted tiny" style="margin:4px 0 0">No run here now.</p>';
      const idle = Object.keys(world.tasks).filter(k => T(k).hostKind === w.kind && !runs.includes(k) && isLive(k) && T(k).kind !== 'change');
      return `<section class="card card-pad worker-card"><div class="card-top"><h2>${conf('the worker’s name, in its configuration', esc(w.name))} <small>${conf('the worker’s description and host kind, in its configuration', `${esc(w.what)} · host kind ${esc(w.kind)}`)}</small></h2><span class="tag green">${fact('jig: the worker’s connection', 'Connected')}</span></div>
        <div class="meta-row" style="margin-top:10px">${slotsGlyph(runs.length, w.slots)}<span>${fact('jig: the slots it declares, and its live runs', `${runs.length} of ${w.slots} slots in use · ${w.slots - runs.length} free`)}</span>${w.note ? `<span>· ${conf('the worker’s description, in its configuration', esc(w.note))}</span>` : ''}</div>
        <div class="worker-runs">${runRows}</div>
        ${idle.length ? `<p class="muted tiny" style="margin:8px 0 0">Of host kind ${esc(w.kind)}, with no run now: ${idle.map(k => `<a class="text-link" href="${href(k)}">${fact(`${k}: its number and title`, `${k} ${esc(T(k).title)}`)}</a>`).join(', ')}.</p>` : ''}
        <div class="rule"></div>
        <div class="worker-logins"><span>Programs’ logins on it</span>${w.logins.map(([n, s]) => `<span>${fact('temper: a login the worker declares, and its last check', `<b>${esc(n)}</b> · <span class="ok">${esc(s)}</span>`)}</span>`).join('')}</div>
        ${world.config.ops && h ? src('ops', 'the ops application: a later application feeding temper, not temper’s facts', `<div class="ops-strip"><div class="ops-strip-head"><span>From the ops application · not temper’s facts</span>${gap('6.4', 'worker health')}</div>
          <dl><div><dt>Load per core</dt><dd>${esc(h.load)}</dd></div><div><dt>Builds queued</dt><dd>${esc(h.builds)}</dd></div><div><dt>Disk free · floor</dt><dd>${esc(h.disk)} · ${esc(h.floor)}</dd></div><div><dt>Memory</dt><dd>${esc(h.memory)}</dd></div></dl></div>`) : ''}
      </section>`;
    }).join('');
    const waiting = Object.values(world.tasks).filter(t => t.phase === 'waiting-slot').length;
    return listPage('workers', [[project(), 'board.html'], ['Workers']], `
      <div class="stats-row"><div class="card stat-card"><span class="muted tiny">Workers</span><strong class="value">${fact('jig: the workers registered', world.workers.length)}</strong><small>${fact('jig: their connections', 'all connected')}</small></div><div class="card stat-card"><span class="muted tiny">Slots in use</span><strong class="value">${fact('jig: live runs over declared slots', `${used} of ${total}`)}</strong><small>${fact('jig: per worker', world.workers.map(w => `${w.name} ${liveRuns(w.name).length}/${w.slots}`).join(' · '))}</small></div><div class="card stat-card"><span class="muted tiny">Free slots</span><strong class="value">${fact('jig: declared slots with no run', total - used)}</strong><small>${fact('jig: per worker', world.workers.map(w => `${w.name} ${w.slots - liveRuns(w.name).length}`).join(' · '))} ${gap('6.7', 'fleet occupancy, read by a coordinator')}</small></div><div class="card stat-card"><span class="muted tiny">Waiting for a slot</span><strong class="value">${fact('jig: runs waiting for a slot', waiting)}</strong><small>runs placed by host kind, model and source</small></div></div>
      <div class="section-heading"><h2>Workers</h2><span class="muted tiny">Each declares its slots. A run goes to a worker of its task’s host kind.</span></div>
      <div class="worker-grid${world.workers.length === 1 ? ' one' : ''}">${cards}</div>`);
  };

  // The person's LLM accounts, in temper's words (client README, 5.5);
  // the plans' windows are gap 6.3's.
  function sourceRow(s, i) {
    const what = s.kind === 'program' ? `${s.program}’s login · on ${s.hosts.map(esc).join(', ')}`
      : s.kind === 'sign-in' ? 'Signed in' : 'A key';
    const usage = s.usage
      ? fact('gap 6.3: the plan’s usage per window, as the provider reports it', `${usageLine('5-hour window', s.usage.window, s.usage.windowReset != null ? hm(s.usage.windowReset) : null, null)}${usageLine('Week', s.usage.week, s.usage.weekReset, s.usage.pace)}`)
      : `<p class="muted tiny" style="margin:0">${fact('jig: a key has no plan', 'No plan windows.')}</p>`;
    return `<div class="source-row"><span class="queue-no">${i + 1}</span><div><h3>${conf('a source of the account, in its order', esc(s.name))} <span class="tag green">${fact('jig: the source’s state, in temper’s words', 'Ready')}</span>${s.use === 'In use' ? ` <span class="tag">${fact('jig: a run uses it now', 'In use')}</span>` : ''}</h3><p>${fact('jig: the source’s kind, and for a program’s, the hosts it is signed in on', esc(what))}</p></div><div class="usage">${usage}</div></div>`;
  }

  pages.accounts = () => {
    const cards = world.accounts.map(a => {
      const users = Object.keys(world.tasks).filter(k => T(k).account === a.name && isLive(k) && T(k).kind !== 'change').sort((x, y) => x.slice(1) - y.slice(1));
      const windows = a.sources.some(s => s.usage);
      return `<section class="card card-pad"><div class="card-top"><div><h2>${fact('jig: the account’s provider and display name, in temper’s words', esc(accountName(a)))} <span class="muted" style="font-weight:400;font-size:11px">· ${fact('jig: its LLM provider, and the party holding it', `${esc(a.provider)} · held by ${esc(a.holder)}`)}</span></h2><p class="muted tiny" style="margin:5px 0 0">Its sources in order: a run takes the first usable one on its worker.</p></div><div class="filter-row">${windows ? gap('6.3', 'plan usage') : ''}<span class="tag green">${fact('jig: whether it is usable now', 'Ready')}</span></div></div>
        <div style="margin-top:8px">${a.sources.map(sourceRow).join('')}</div>
        <div class="rule"></div>
        <div class="kv"><span>Used by</span><strong>${users.length ? `${fact('jig: the live tasks whose runs use it', users.map(k => `<a class="text-link" href="${href(k)}">${k}</a>`).join(', '))} · ${fact('jig: their runs now', `${users.filter(k => T(k).phase === 'running').length} running`)}` : 'none'}</strong></div>
        <div class="kv"><span>Waiting for it</span><strong>${fact('jig: tasks waiting for the account', 'none')}</strong></div>
      </section>`;
    }).join('');
    const providers = new Set(world.accounts.map(a => a.provider));
    const ways = [providers.has('OpenAI') ? 'Continue with ChatGPT' : '', providers.has('Anthropic') ? 'Use your Claude subscription' : '', 'Use an API key'].filter(Boolean);
    return listPage('accounts', [['Accounts']], `
      <div class="section-heading" style="margin-top:0"><div><h2>Your LLM accounts</h2><p class="muted tiny" style="margin:5px 0 0">Each plan’s windows, read as the account’s facts. The mark on a week is its pace: where usage would be if the week were spent evenly.</p></div></div>
      <div class="stack">${cards}</div>
      <div class="section-heading"><h2>Add an account</h2><div class="filter-row">${ways.map(w => `<button class="button" data-later>${w}</button>`).join('')}</div></div>`);
  };

  // ---- Mount ----------------------------------------------------------

  const render = pages[page] || pages.index;
  const html = render();
  document.body.insertAdjacentHTML('afterbegin', (bare ? '' : mockbar() + stepPop() + stepsPop() + viewPop()) + html);
  bindCasebar();
  bindSources();
  bindTrees();
  bindLaterActs();

  const notes = document.getElementById('notes');
  if (notes) {
    const open = show => { notes.hidden = !show; document.querySelectorAll('[data-notes]').forEach(b => b.setAttribute('aria-expanded', String(show))); };
    document.querySelectorAll('[data-notes]').forEach(b => b.addEventListener('click', () => open(notes.hidden)));
    document.querySelectorAll('[data-notes-close]').forEach(b => b.addEventListener('click', () => open(false)));
    if (location.hash === '#notes') open(true);
  }
  const thread = document.querySelector('.terminal-thread');
  if (thread && page === 'chat') thread.scrollTop = thread.scrollHeight;
  document.title = `${document.title} · ${W.name.toLowerCase()} · ${step.label.toLowerCase()}`;
})();
