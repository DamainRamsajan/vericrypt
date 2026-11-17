// Online course client. Progressive enhancement: pages are complete HTML; this adds the lesson player,
// the access-code unlock, recall cards, spaced review and saved progress.
// Paid lessons are AES-256-GCM files. The access code carries the key; nothing about it is sent anywhere.
// Everything is stored in this browser only (localStorage), and every read is wrapped in try/catch.
import { $, $$, el, storage } from '/kit/dom.js';

const CODE_KEY = 'iw-course-code';
const PROGRESS_KEY = 'iw-course-progress';
const REVIEW_KEY = 'iw-course-review';
const GAPS = [1, 3, 7, 21]; // days until a card returns, by how many times it was remembered in a row

const b64u = (s) => Uint8Array.from(atob(s.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0));
const readJSON = (k, d) => { try { return JSON.parse(storage.get(k) || '') ?? d; } catch { return d; } };
const writeJSON = (k, v) => storage.set(k, JSON.stringify(v));

/* ---------- access code ---------- */

export function parseCode(text) {
  try {
    const o = JSON.parse(new TextDecoder().decode(b64u(String(text).trim())));
    if (o && o.v === 1 && typeof o.k === 'string' && o.k.length >= 40) return { name: String(o.b || ''), key: o.k };
  } catch { /* fall through */ }
  return null;
}

async function decryptLesson(url, id, code) {
  const res = await fetch(url);
  if (!res.ok) throw new Error('missing');
  const enc = await res.json();
  const key = await crypto.subtle.importKey('raw', b64u(code.key), 'AES-GCM', false, ['decrypt']);
  const plain = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: b64u(enc.iv), additionalData: new TextEncoder().encode(id) }, key, b64u(enc.ct));
  return JSON.parse(new TextDecoder().decode(plain));
}

async function codeWorks(form, code) {
  try { await decryptLesson(form.dataset.probe, form.dataset.probeId, code); return true; } catch { return false; }
}

/* ---------- unlock forms (index and locked lessons) ---------- */

async function setupUnlock(onUnlocked) {
  const forms = $$('[data-unlock]');
  if (!forms.length) return null;
  let active = null;
  const saved = parseCode(storage.get(CODE_KEY) || '');
  for (const form of forms) {
    form.hidden = false;
    const msg = $('[data-unlock-msg]', form);
    const forget = $('[data-forget]', form);
    if (saved && await codeWorks(form, saved)) {
      active = saved;
      msg.textContent = `Unlocked${saved.name ? ` for ${saved.name}` : ''}.`;
      forget.hidden = false;
      $('input', form).hidden = true; $('label', form).hidden = true; $('[type=submit]', form).hidden = true;
    }
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      const code = parseCode($('input', form).value);
      if (!code) { msg.textContent = 'That does not look like an access code. Paste the whole code from your email.'; return; }
      msg.textContent = 'Checking…';
      if (!(await codeWorks(form, code))) { msg.textContent = 'That code does not unlock this course. Check you pasted all of it, or ask us for a new one.'; return; }
      storage.set(CODE_KEY, $('input', form).value.trim());
      storage.set('iw-course-just', '1');
      msg.textContent = 'Code accepted. Opening your course…';
      location.reload();
    });
    forget.addEventListener('click', () => { try { localStorage.removeItem(CODE_KEY); } catch { /* ignore */ } location.reload(); });
  }
  if (active && onUnlocked) onUnlocked(active);
  return active;
}

/* ---------- progress and review ---------- */

const done = () => readJSON(PROGRESS_KEY, {});
const markDone = (id) => { const d = done(); d[id] = Date.now(); writeJSON(PROGRESS_KEY, d); };
const cardKey = (lesson, i) => `${lesson}:${i}`;

function rate(lesson, i, ok) {
  const r = readJSON(REVIEW_KEY, {});
  const k = cardKey(lesson, i);
  const box = ok ? Math.min((r[k]?.box ?? -1) + 1, GAPS.length - 1) : 0;
  r[k] = { box, due: Date.now() + (ok ? GAPS[box] : 1) * 864e5 };
  writeJSON(REVIEW_KEY, r);
}

/* ---------- the player ---------- */

function initPlayer(root, data, code) {
  const stage = $('[data-stage]', root), frame = $('[data-frame]', root);
  const pos = $('[data-pos]', root), prev = $('[data-prev]', root), next = $('[data-next]', root);
  const reveal = $('[data-reveal]', root), speak = $('[data-speak]', root), rateSel = $('[data-rate]', root);
  const transcript = $('[data-transcript]', root);
  const outline = $('[data-outline]', root), bar = $('[data-bar]', root), fill = $('[data-fill]', root);
  const lesson = root.dataset.lesson;
  const n = data.slides.length;
  let i = 0, narrating = false, audio = null;

  $('[data-stage-wrap]', root).hidden = false;
  if (code && code.name) $('[data-licence]', root).textContent = `Licensed to ${code.name}. Please do not share this course.`;
  const canSpeak = 'speechSynthesis' in window;
  if (!canSpeak && !data.audio.some(Boolean)) speak.hidden = true;

  const fit = () => {
    const s = frame.clientWidth / 1280;
    stage.style.transform = `scale(${s})`;
    frame.style.height = `${Math.round(720 * s)}px`;
  };
  new ResizeObserver(fit).observe(frame);
  const items = data.titles.map((t, k) => {
    const li = document.createElement('li'), b = document.createElement('button');
    b.type = 'button'; b.textContent = t; b.addEventListener('click', () => show(k));
    li.append(b); outline.append(li); return li;
  });

  function show(k) {
    i = Math.max(0, Math.min(n - 1, k));
    const sec = document.createElement('section');
    sec.className = 's on';
    sec.innerHTML = data.slides[i]; // trusted: built by tools/build-course.mjs from our own deck; paid lessons are authenticated by AES-GCM
    stage.replaceChildren(sec);
    stage.setAttribute('aria-label', `Slide ${i + 1} of ${n}: ${data.titles[i]}`);
    pos.textContent = `${i + 1} / ${n}`;
    items.forEach((li, k) => { li.classList.toggle('on', k === i); li.classList.toggle('seen', k < i); const b = li.firstChild; if (k === i) b.setAttribute('aria-current', 'step'); else b.removeAttribute('aria-current'); });
    const pct = Math.round(((i + 1) / n) * 100);
    fill.style.width = pct + '%'; bar.setAttribute('aria-valuenow', String(pct));
    const cur = items[i]; if (cur && cur.scrollIntoView && outline.scrollHeight > outline.clientHeight) outline.scrollTop = cur.offsetTop - outline.offsetTop - 40;
    prev.disabled = i === 0;
    next.textContent = i === n - 1 ? 'Finish' : 'Next';
    const frs = $$('.fr', sec);
    reveal.hidden = frs.length === 0;
    reveal.onclick = () => { const f = frs.find((x) => !x.classList.contains('on')); if (f) f.classList.add('on'); reveal.hidden = frs.every((x) => x.classList.contains('on')); };
    const say = document.createElement('p'); say.textContent = data.say[i];
    transcript.replaceChildren(say);
    if (data.notes[i]) { const f = document.createElement('p'); f.className = 'fnote'; f.textContent = `Facilitator note: ${data.notes[i]}`; transcript.append(f); }
    fit();
    if (narrating) narrate();
  }

  function stopNarration() {
    if (canSpeak) speechSynthesis.cancel();
    if (audio) { audio.pause(); audio = null; }
  }
  function narrate() {
    stopNarration();
    const advance = () => { if (narrating && i < n - 1) show(i + 1); else { narrating = false; speak.setAttribute('aria-pressed', 'false'); speak.textContent = 'Read aloud'; } };
    if (data.audio[i]) { audio = new Audio(data.audio[i]); audio.playbackRate = Number(rateSel.value); audio.onended = advance; audio.play().catch(advance); return; }
    if (!canSpeak) return;
    const u = new SpeechSynthesisUtterance(data.say[i]);
    u.rate = Number(rateSel.value);
    u.onend = advance;
    speechSynthesis.speak(u);
  }
  speak.addEventListener('click', () => {
    narrating = !narrating;
    speak.setAttribute('aria-pressed', String(narrating));
    speak.textContent = narrating ? 'Stop reading' : 'Read aloud';
    if (narrating) narrate(); else stopNarration();
  });
  rateSel.addEventListener('change', () => { if (narrating) narrate(); });

  prev.addEventListener('click', () => show(i - 1));
  next.addEventListener('click', () => { if (i === n - 1) finish(); else show(i + 1); });
  document.addEventListener('keydown', (e) => {
    if (/^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName) || e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 'ArrowRight') { e.preventDefault(); next.click(); }
    if (e.key === 'ArrowLeft') { e.preventDefault(); prev.click(); }
  });

  function finish() {
    narrating = false; stopNarration();
    markDone(lesson);
    next.textContent = 'Finished';
    const rec = $('[data-recall]', root);
    const box = $('[data-cards]', root);
    box.replaceChildren();
    for (const k of data.cards) {
      const card = document.createElement('div'); card.className = 'rcard';
      const q = document.createElement('p'); q.textContent = `Without looking back: what do you remember about “${data.titles[k]}”?`;
      const b = document.createElement('button'); b.className = 'btn btn-ghost'; b.textContent = 'Show the answer';
      const ans = document.createElement('div'); ans.className = 'ans'; ans.hidden = true;
      const t = document.createElement('p'); t.textContent = data.say[k];
      const row = document.createElement('div'); row.className = 'actions';
      const yes = document.createElement('button'); yes.className = 'btn btn-primary'; yes.textContent = 'I remembered it';
      const no = document.createElement('button'); no.className = 'btn btn-ghost'; no.textContent = 'Not yet';
      const out = document.createElement('p'); out.className = 'muted small'; out.setAttribute('role', 'status');
      const answer = (ok) => { rate(lesson, k, ok); out.textContent = ok ? 'Good. This comes back for review later.' : 'No problem. This comes back tomorrow.'; yes.disabled = no.disabled = true; };
      yes.onclick = () => answer(true); no.onclick = () => answer(false);
      row.append(yes, no);
      ans.append(t, row, out);
      b.onclick = () => { ans.hidden = false; b.hidden = true; };
      card.append(q, b, ans);
      box.append(card);
    }
    rec.hidden = data.cards.length === 0;
    if (!rec.hidden) rec.scrollIntoView({ block: 'start' });
  }

  show(0);
}

/* ---------- pages ---------- */

async function lessonPage() {
  const root = $('[data-player]');
  if (root.dataset.plain) {
    try { initPlayer(root, await (await fetch(root.dataset.plain)).json(), null); }
    catch { $('[data-stage-wrap]', root).replaceChildren('The lesson could not be loaded. Check your connection and refresh.'); $('[data-stage-wrap]', root).hidden = false; }
    return;
  }
  const locked = $('[data-locked]', root);
  const url = root.dataset.enc;
  const code = await setupUnlock();
  if (!code) { locked.hidden = false; return; }
  try {
    const data = await decryptLesson(url, root.dataset.lesson, code);
    locked.hidden = true;
    initPlayer(root, data, code);
  } catch {
    locked.hidden = false;
    const msg = $('[data-unlock-msg]', locked);
    if (msg) msg.textContent = 'Your saved code does not open this lesson. Ask us for a new code.';
  }
}

function fitPreview() {
  const fig = $('[data-preview]');
  if (!fig) return;
  const frame = $('[data-frame]', fig), stage = $('[data-stage]', fig);
  fig.hidden = false;
  const fit = () => { const s = frame.clientWidth / 1280; stage.style.transform = `scale(${s})`; frame.style.height = `${Math.round(720 * s)}px`; };
  new ResizeObserver(fit).observe(frame); fit();
}

async function indexPage() {
  fitPreview();
  const code = await setupUnlock();
  const d = done();
  const rows = $$('li.lesson');
  let count = 0;
  for (const li of rows) {
    const id = li.dataset.lesson;
    const badge = $('[data-badge]', li);
    if (d[id]) { count++; badge.textContent = 'Done'; badge.classList.add('badge-done'); }
    else if (code && !li.classList.contains('is-free')) badge.textContent = 'Unlocked';
  }
  if (code) {
    const next = rows.find((li) => !d[li.dataset.lesson]) || rows[0];
    const link = $('a', next);
    const just = storage.get('iw-course-just') === '1';
    const banner = el('div', { class: 'unlock-banner', role: 'status', tabindex: '-1' },
      el('div', {}, el('strong', {}, `You are in${code.name ? `, ${code.name}` : ''}. All ${rows.length} lessons are open.`),
        el('span', {}, count ? ` ${count} of ${rows.length} finished on this device.` : ' Your progress is saved in this browser.')),
      el('a', { class: 'btn btn-primary', href: link.getAttribute('href') }, count ? 'Continue' : 'Start lesson 1'));
    $('main').prepend(banner);
    if (just) { storage.set('iw-course-just', ''); window.scrollTo(0, 0); banner.focus(); }
  }
  const prog = $('[data-progress]');
  if (prog && count) { prog.hidden = false; prog.textContent = `${count} of ${rows.length} lessons finished on this device.`; }
  // review: lessons with cards due today
  const r = readJSON(REVIEW_KEY, {});
  const due = {};
  for (const [k, v] of Object.entries(r)) if (v.due <= Date.now()) { const id = k.split(':')[0]; due[id] = (due[id] || 0) + 1; }
  const list = $('[data-due-list]');
  const ids = Object.keys(due).filter((id) => rows.some((li) => li.dataset.lesson === id));
  if (list && ids.length) {
    $('[data-due]').hidden = false;
    for (const id of ids) {
      const li = document.createElement('li');
      const a = document.createElement('a');
      const src = rows.find((x) => x.dataset.lesson === id);
      a.href = $('a', src).getAttribute('href');
      a.textContent = `${$('strong', src).textContent}: ${due[id]} ${due[id] === 1 ? 'question' : 'questions'} to review`;
      li.append(a); list.append(li);
    }
  }
}

const kind = document.body.dataset.course;
if (kind === 'lesson') lessonPage(); else if (kind === 'index') indexPage();
