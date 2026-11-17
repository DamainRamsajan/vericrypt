#!/usr/bin/env node
// Builds the online course for a workshop from its slide deck.
//   site/training/<id>/course/index.html          course home (lessons, enrol, access code)
//   site/training/<id>/course/<lesson>/index.html one page per lesson (a narrated slide player)
//   site/training/<id>/course/data/<lesson>.json  encrypted lesson (paid lessons only)
//   site/training/<id>/course/course.css          the slide look, scoped to .cstage
//
// Source of truth: the slide deck (site/training/<id>/deck-*/slides.js). Each "Part" is one lesson.
// Which lessons are free is set in training.json (workshop.course.free). Paid lessons are encrypted
// with AES-256-GCM. The key lives in COURSE_KEY (env) or .course-key (git-ignored), never in the site.
// A buyer's access code is made with: npm run course:code -- "Buyer name"
// Safe to run repeatedly: the same key gives the same plaintext, a fresh random IV each run.
//
// Usage: node tools/build-course.mjs [--print-lessons]

import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import crypto from 'node:crypto';

export const SITE = path.resolve('site');
// Always next to package.json, wherever the command is run from, so a key is never silently re-created elsewhere.
const KEYFILE = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '.course-key');

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const b64u = (buf) => Buffer.from(buf).toString('base64url');
const slug = (s) => s.toLowerCase().replace(/[§·]/g, ' ').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

/* ---------- key ---------- */

export function loadKey({ create = true } = {}) {
  if (process.env.COURSE_KEY) return Buffer.from(process.env.COURSE_KEY.trim(), 'base64url');
  if (fs.existsSync(KEYFILE)) return Buffer.from(fs.readFileSync(KEYFILE, 'utf8').trim(), 'base64url');
  if (!create) return null;
  const k = crypto.randomBytes(32);
  fs.writeFileSync(KEYFILE, b64u(k) + '\n', { mode: 0o600 });
  console.log('\n  NEW COURSE KEY created in .course-key');
  console.log('  Back it up now (a password manager is ideal). It is not in the repo. Without it you cannot');
  console.log('  issue access codes that work, and changing it locks out every buyer you already have.\n');
  return k;
}

function encrypt(key, id, obj) {
  const iv = crypto.randomBytes(12);
  const c = crypto.createCipheriv('aes-256-gcm', key, iv);
  c.setAAD(Buffer.from(id));
  const ct = Buffer.concat([c.update(JSON.stringify(obj), 'utf8'), c.final(), c.getAuthTag()]); // WebCrypto expects tag appended
  return { v: 1, id, iv: b64u(iv), ct: b64u(ct) };
}

/* ---------- read the deck ---------- */

function loadDeck(workshopDir) {
  const deck = fs.readdirSync(workshopDir).find((d) => d.startsWith('deck-'));
  if (!deck) throw new Error(`no deck-* folder in ${workshopDir}`);
  const src = fs.readFileSync(path.join(workshopDir, deck, 'slides.js'), 'utf8');
  const cut = src.indexOf('var st=document');
  if (cut < 0) throw new Error('slides.js layout changed: cannot find where the viewer starts');
  const ctx = vm.createContext({});
  vm.runInContext(src.slice(0, cut) + '\n;this.__SL=SL;this.__dgx=dgx;', ctx, { timeout: 5000 });
  const SL = ctx.__SL, dgx = ctx.__dgx;
  let part = '';
  const slides = SL.map((s) => {
    if (s.p) part = s.p;
    return { part, t: s.t, o: !!s.o, e: s.e || 0, n: s.n || '', h: s.h };
  });
  return { slides, dgx, deck, css: fs.readFileSync(path.join(workshopDir, deck, 'slides.css'), 'utf8') };
}

function renderSlide(deck, s, index, total) {
  const body = s.h
    .replace(/<h1>/g, "<div class='h1'>").replace(/<\/h1>/g, '</div>') // the lesson page owns the one <h1>
    .replace(/@@(\w+)@@/g, (m, q) => deck.dgx(q))
    .replace(/\{\{slide:(.+?)\}\}/g, (m, t) => { const k = deck.slides.findIndex((y) => y.t === t); return k < 0 ? '?' : String(k + 1); });
  return `<div class='pt'>${s.part}</div>${body}${s.o ? "<span class='opt'>optional</span>" : ''}`;
}

const plain = (html) => html.replace(/<svg[\s\S]*?<\/svg>/g, (m) => ' ' + [...m.matchAll(/<text[^>]*>([\s\S]*?)<\/text>/g)].map((x) => x[1].replace(/<[^>]+>/g, '')).join('. ') + ' ')
  .replace(/<br\s*\/?>/g, '. ').replace(/<\/(li|p|h\d|tr|div)>/g, '. ').replace(/<[^>]+>/g, ' ').replace(/&nbsp;/g, ' ')
  .replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/\s+/g, ' ').replace(/\s+([.,?!:;])/g, '$1').replace(/\.\s*\./g, '.').trim();

/* ---------- CSS: the deck look, scoped to .cstage ---------- */

function scopedCss(css) {
  const rules = [];
  let depth = 0, cur = '';
  for (const ch of css.replace(/\/\*[\s\S]*?\*\//g, '')) {
    cur += ch;
    if (ch === '{') depth++;
    if (ch === '}') { depth--; if (depth === 0) { rules.push(cur.trim()); cur = ''; } }
  }
  const out = [];
  for (const r of rules) {
    if (r.startsWith('@')) continue; // print, motion, view-transition rules belong to the live deck viewer
    const i = r.indexOf('{');
    const sels = r.slice(0, i).split(',').map((x) => x.trim());
    const decl = r.slice(i);
    const keep = [];
    for (const sel of sels) {
      if (/^(html|body|\*|#hud|#bar|#notes|#ov|#cd|\.recb|#fsask|body\.|::)/.test(sel)) continue;
      if (sel === ':root') { keep.push('.cstage'); continue; }
      if (sel === '#stage') continue;
      keep.push(`.cstage ${sel}`);
    }
    if (keep.length) out.push(`${keep.join(', ')}${decl}`);
  }
  return out.join('\n');
}

/* ---------- lessons ---------- */

function buildLessons(deck, freeIds, skip = []) {
  const order = [];
  const byPart = new Map();
  const keyOf = (p) => (/^Appendix/.test(p) ? 'Appendix · Quiz and quick card' : p);
  deck.slides.forEach((s, i) => {
    const k = keyOf(s.part);
    if (skip.includes(slug(k))) return; // live-day logistics (breaks, lunch) mean nothing online
    if (!byPart.has(k)) { byPart.set(k, []); order.push(k); }
    byPart.get(k).push(i);
  });
  const total = deck.slides.length;
  return order.map((part, n) => {
    const idx = byPart.get(part);
    const id = slug(part);
    const slides = idx.map((i) => deck.slides[i]);
    const content = slides.filter((s) => /<h2>/.test(s.h) && !s.e);
    const pick = [];
    const want = Math.min(5, content.length);
    for (let k = 0; k < want; k++) pick.push(slides.indexOf(content[Math.floor((k * content.length) / want)]));
    const words = slides.reduce((a, s) => a + plain(s.h).split(' ').length, 0);
    const optional = slides.every((s) => s.o);
    return {
      id, n: n + 1, num: n + 1, part,
      title: part.replace(/^(Part\s+\w+|Appendix)\s*·\s*/, '').replace(/^(§\d+)\s+/, '$1 '),
      free: freeIds.includes(id), optional,
      minutes: Math.max(3, Math.round((words / 150) + slides.filter((s) => s.e).reduce((a, s) => a + s.e, 0))),
      titles: slides.map((s) => s.t),
      payload: {
        slides: idx.map((i, k) => renderSlide(deck, deck.slides[i], i + 1, total)),
        say: slides.map((s) => plain(s.h)),
        notes: slides.map((s) => s.n),
        exercise: slides.map((s) => s.e),
        titles: slides.map((s) => s.t),
        cards: pick,
      },
    };
  });
}

/* ---------- pages ---------- */

let base = '';

function page({ title, description, canonicalPath, body, bodyAttrs = '' }) {
  const url = base + canonicalPath;
  return `<!doctype html>
<!-- Generated by tools/build-course.mjs from the slide deck and training.json. Edit those, not this file. -->
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${esc(title)}</title>
  <meta name="description" content="${esc(description)}">
  <meta name="theme-color" content="#f2b631">
  <link rel="canonical" href="${url}">
  <meta property="og:site_name" content="Intellica">
  <meta property="og:type" content="website">
  <meta property="og:title" content="${esc(title)}">
  <meta property="og:description" content="${esc(description)}">
  <meta property="og:url" content="${url}">
  <meta name="twitter:card" content="summary">
  <link rel="icon" href="/assets/favicon.svg" type="image/svg+xml">
  <script src="/kit/theme-init.js"></script>
  <link rel="stylesheet" href="/kit/tokens.css">
  <link rel="stylesheet" href="/kit/shell.css">
  <link rel="stylesheet" href="/training/training.css">
  <link rel="stylesheet" href="/training/arc42/course/course.css">
</head>
<body data-tab="training" data-generated="training" data-section="training"${bodyAttrs}>
  <a class="skip-link" href="#main">Skip to content</a>
  <div id="shell-header"></div>

  <main id="main" class="container tr">
${body}
  </main>

  <div id="shell-footer"></div>
  <script type="module" src="/kit/shell.js"></script>
  <script type="module" src="/training/arc42/course/course.js"></script>
</body>
</html>
`;
}

function subnav(w, active) {
  const items = [['', 'Workshop'], ['course/', 'Online course'], ['guide/', 'Student guide'], ['register/', 'Register']];
  return `    <nav class="subnav" aria-label="${esc(w.title)}">
      <p class="crumb"><a href="/training/">Training</a><span aria-hidden="true"> / </span>${esc(w.title)}</p>
      <ul>${items.map(([p, l]) => `<li><a href="${w.path}${p}"${p === active ? ' aria-current="page"' : ''}>${l}</a></li>`).join('')}</ul>
    </nav>`;
}

function enrolBlock(w, c, priceLabel) {
  const wa = `https://wa.me/${w.whatsapp}?text=${encodeURIComponent(`Hello, I\u2019d like access to the ${w.title} online course${c.price ? ` (${c.price.label})` : ''}.`)}`;
  const shown = c.price && c.price.regular ? `<s class="was" aria-label="Regular price ${esc(c.price.regular)}">${esc(c.price.regular)}</s> ${priceLabel}` : priceLabel;
  return `<aside class="enrol" aria-label="Price and how to buy">
        <p class="eyebrow">Full course</p>
        <p class="c-price">${shown}</p>
        <p class="muted small">${c.price && c.price.note ? esc(c.price.note) + ' ' : ''}${c.price ? 'Lifetime access to this edition.' : 'Confirmed when you enquire. No payment is taken on this site.'}</p>
        <ol class="steps">
          <li><div><strong>Message sales on WhatsApp.</strong> We reply with how to pay${c.methods && c.methods.length ? ` (${esc(c.methods.join(', '))})` : ''}.</div></li>
          <li><div><strong>Pay.</strong> No account and no card form on this site: payment is arranged with you directly.</div></li>
          <li><div><strong>Receive your access code</strong>, normally within one working day of payment.</div></li>
          <li><div><strong>Paste it below</strong> to unlock every lesson on this device. It is kept in this browser only.</div></li>
        </ol>
        <div class="actions"><a class="btn btn-primary contact-btn" href="${esc(wa)}" rel="noopener">Buy on WhatsApp</a></div>
      </aside>`;
}

function unlockBlock(probe) {
  return `<form class="unlock" data-unlock data-probe="${probe.url}" data-probe-id="${probe.id}" hidden>
        <label for="code">Access code<input id="code" name="code" autocomplete="off" spellcheck="false" placeholder="Paste the code from your email"></label>
        <div class="actions"><button class="btn btn-primary" type="submit">Unlock the course</button><button class="btn btn-ghost" type="button" data-forget hidden>Remove my code from this browser</button></div>
        <p class="muted small" role="status" data-unlock-msg></p>
      </form>`;
}

function indexPage(w, c, lessons) {
  const first = lessons.find((l) => !l.free);
  const probe = { url: `${w.path}course/data/${first.id}.json`, id: first.id };
  const byId = Object.fromEntries(lessons.map((l) => [l.id, l]));
  const real = lessons.filter((l) => !l.optional);
  const hours = Math.round(real.reduce((a, l) => a + l.minutes, 0) / 60);
  const slideCount = lessons.reduce((a, l) => a + l.titles.length, 0);
  const exercises = lessons.reduce((a, l) => a + l.payload.exercise.filter(Boolean).length, 0);
  const firstFree = lessons.find((l) => l.free);
  // a real diagram slide from the deck, shown as a live preview of what the player looks like
  const pl = byId[c.preview.lesson];
  const pi = pl.payload.titles.indexOf(c.preview.slideTitle);
  const previewHtml = pl.payload.slides[pi < 0 ? 1 : pi];
  const modules = c.modules.map((m, mi) => `
        <section class="module" aria-labelledby="m${mi}">
          <header><h3 id="m${mi}">${esc(m.title)}</h3><p>${esc(m.note)}</p></header>
          <ol class="lessons" start="${m.lessons.length ? byId[m.lessons[0]].num : 1}">
${m.lessons.map((id) => { const l = byId[id]; return `            <li data-lesson="${l.id}" class="lesson${l.free ? ' is-free' : ''}">
              <a href="${w.path}course/${l.id}/"><span class="ln">${l.num}</span><span class="lt"><strong>${esc(l.title)}</strong><span class="lm">${l.titles.length} slides, about ${l.minutes} min${l.optional ? ', optional' : ''}</span></span></a>
              <span class="badge ${l.free ? 'badge-free' : 'badge-paid'}" data-badge>${l.free ? 'Free preview' : 'Course'}</span>
            </li>`; }).join('\n')}
          </ol>
        </section>`).join('');
  const price = c.price ? esc(c.price.label) : 'Price on enquiry';
  const body = `
${subnav(w, 'course/')}

    <section class="c-hero" aria-labelledby="c-t">
      <div class="c-hero-copy">
        <p class="eyebrow">Online course</p>
        <h1 id="c-t">${esc(c.headline)}</h1>
        <p class="lead">${esc(c.lead)}</p>
        <div class="actions">
          <a class="btn btn-primary" href="${w.path}course/${firstFree.id}/">Start the free lessons</a>
          <a class="btn btn-ghost" href="#get-t">Get the full course</a>
        </div>
        <p class="fine">${esc(c.honesty)}</p>
      </div>
      <figure class="c-preview" aria-label="A slide from the course" data-preview hidden>
        <div class="cstage-frame" data-frame><div class="cstage" data-stage>${'<section class="s on">' + previewHtml + '</section>'}</div></div>
        <figcaption>A real slide from “${esc(pl.title)}”. Every lesson is slides you can pause, read and replay.</figcaption>
      </figure>
    </section>

    <div class="strip c-strip">
      <div><b>${lessons.length}</b><span>lessons</span></div>
      <div><b>${hours} hours</b><span>of study</span></div>
      <div><b>${slideCount}</b><span>slides with transcripts</span></div>
      <div><b>${exercises}</b><span>timed exercises</span></div>
      <div><b>1</b><span>worked example you can run</span></div>
    </div>

    <section class="t-block" aria-labelledby="how-t">
      <p class="kicker"><span class="kn">01</span>How it teaches</p>
      <h2 id="how-t">Built from how people learn</h2>
      <div class="cols c4">
${c.method.map((m) => `        <div><h3>${esc(m.title)}</h3><p>${esc(m.text)}</p></div>`).join('\n')}
      </div>
      <p class="muted small">${c.evidence.map((e) => esc(e)).join(' ')}</p>
    </section>

    <section class="t-block" aria-labelledby="les-t">
      <p class="kicker"><span class="kn">02</span>Syllabus</p>
      <h2 id="les-t">${lessons.length} lessons in four modules</h2>
      <p class="sub">Study in order or jump to what you need. The first two lessons are free and need no code.</p>
      <p class="progress" data-progress hidden></p>
      <div class="modules">${modules}
      </div>
    </section>

    <section class="t-block" aria-labelledby="due-t" data-due hidden>
      <p class="kicker"><span class="kn">03</span>Review</p>
      <h2 id="due-t">Ready to review today</h2>
      <ul class="due-list" data-due-list></ul>
    </section>

    <section class="t-block" aria-labelledby="get-t">
      <p class="kicker"><span class="kn">04</span>Access</p>
      <h2 id="get-t">Get the full course</h2>
      <div class="c-access">
        <div class="c-included">
          <h3>What you get</h3>
          <ul class="facts">
            <li>All ${lessons.length} lessons, in the player, on any device</li>
            <li>A transcript and read-aloud for every slide</li>
            <li>Recall questions and spaced review</li>
            <li>The Doubles-to-Go arc42 and working app, with its tests</li>
            <li>Lifetime access to this edition</li>
          </ul>
          ${unlockBlock(probe)}
        </div>
        ${enrolBlock(w, c, price)}
      </div>
    </section>

    <section class="t-block" aria-labelledby="by-t">
      <p class="kicker"><span class="kn">05</span>Your instructor</p>
      <h2 id="by-t">${esc(w.facilitator.name)}</h2>
      <p class="sub">${esc(w.facilitator.role)}</p>
      <p>${esc(w.facilitator.bio)}</p>
      <ul class="chips">${w.facilitator.facts.map((f) => `<li>${esc(f)}</li>`).join('')}</ul>
    </section>

    <section class="t-block" aria-labelledby="faq-t">
      <p class="kicker"><span class="kn">06</span>Questions</p>
      <h2 id="faq-t">Before you decide</h2>
      <div class="faqs">${c.faq.map((f) => `<details class="faq-item"><summary>${esc(f.q)}</summary><p>${esc(f.a)}</p></details>`).join('')}</div>
    </section>`;
  return page({
    title: `Online course | ${w.title} | Intellica Training`,
    description: c.lead,
    canonicalPath: w.path + 'course/',
    body,
    bodyAttrs: ' data-course="index"',
  });
}

function lessonPage(w, c, l, lessons, key) {
  const i = lessons.indexOf(l);
  const prev = lessons[i - 1], next = lessons[i + 1];
  const outline = l.titles.map((t) => `<li>${esc(t)}</li>`).join('');
  let dataAttr, inline = '';
  if (l.free) {
    dataAttr = ` data-plain="${w.path}course/data/${l.id}.json"`;
  } else {
    dataAttr = ` data-enc="${w.path}course/data/${l.id}.json"`;
  }
  const body = `
${subnav(w, 'course/')}

    <section class="t-hero t-hero-single lesson-head" aria-labelledby="l-t">
      <p class="eyebrow"><a href="${w.path}course/">Online course</a> · Lesson ${l.num} of ${lessons.length}${l.optional ? ' · optional' : ''}</p>
      <h1 id="l-t">${esc(l.title)}</h1>
      <p class="lead">${l.titles.length} slides, about ${l.minutes} minutes. ${l.free ? 'This lesson is a free preview.' : 'Part of the full course.'}</p>
    </section>

    <div class="player" data-player data-lesson="${l.id}"${dataAttr}>
      <div class="locked" data-locked hidden>
        <h2>This lesson is part of the full course</h2>
        <p>Here is what it covers:</p>
        <ol class="outline">${outline}</ol>
        <p>Paste your access code to unlock it, or read how to get one.</p>
        ${unlockBlock({ url: `${w.path}course/data/${l.id}.json`, id: l.id })}
        <div class="actions"><a class="btn btn-primary" href="${w.path}course/#get-t">How to get access</a></div>
      </div>
      <div class="stage-wrap" data-stage-wrap hidden>
        <div class="c-main">
          <div class="cstage-frame" data-frame><div class="cstage" data-stage role="img" aria-label="Lesson slide"></div></div>
          <div class="cbar" role="progressbar" aria-label="Lesson progress" aria-valuemin="0" aria-valuemax="100" data-bar><i data-fill></i></div>
          <div class="controls" role="group" aria-label="Lesson controls">
            <button class="btn btn-ghost" data-prev>Back</button>
            <span class="pos" data-pos aria-live="polite"></span>
            <button class="btn btn-primary" data-next>Next</button>
            <button class="btn btn-ghost" data-reveal hidden>Reveal</button>
            <button class="btn btn-ghost" data-speak aria-pressed="false">Read aloud</button>
            <label class="rate">Speed<select data-rate><option value="0.85">Slower</option><option value="1" selected>Normal</option><option value="1.2">Faster</option></select></label>
          </div>
          <p class="licence muted small" data-licence></p>
          <details class="transcript" open><summary>Transcript and notes</summary><div data-transcript></div></details>
        </div>
        <aside class="c-side" aria-label="In this lesson">
          <h2>In this lesson</h2>
          <ol class="c-outline" data-outline></ol>
        </aside>
      </div>
      <div class="recall" data-recall hidden>
        <h2>Pause and recall</h2>
        <p class="sub">Close your eyes or look away, and say what you remember before you reveal each answer. Trying to remember is what makes it stick.</p>
        <div data-cards></div>
      </div>
      <noscript><p>The lesson player needs JavaScript. If you cannot use it, write to us and we will send the material another way.</p></noscript>
    </div>

    <nav class="lesson-nav" aria-label="Lessons">
      ${prev ? `<a class="btn btn-ghost" href="${w.path}course/${prev.id}/">&larr; ${esc(prev.title)}</a>` : '<span></span>'}
      ${next ? `<a class="btn btn-ghost" href="${w.path}course/${next.id}/">${esc(next.title)} &rarr;</a>` : ''}
    </nav>
    ${inline}`;
  return page({
    title: `${l.title} | Online course | ${w.title}`,
    description: `Lesson ${l.num} of the ${w.title} online course: ${l.titles.slice(0, 4).join(', ')}.`,
    canonicalPath: `${w.path}course/${l.id}/`,
    body,
    bodyAttrs: ' data-course="lesson"',
  });
}

/* ---------- run ---------- */

if (import.meta.url === `file://${process.argv[1]}`) {
  const data = JSON.parse(fs.readFileSync(path.join(SITE, 'training', 'training.json'), 'utf8'));
  const family = JSON.parse(fs.readFileSync(path.join(SITE, 'family.json'), 'utf8'));
  base = family.home.replace(/\/$/, '');
  const key = loadKey();
  let total = 0;
  for (const w of data.workshops) {
    const c = w.course;
    if (!c || !c.enabled) continue;
    const dir = path.join(SITE, w.path.replace(/^\//, ''));
    const deck = loadDeck(dir);
    const lessons = buildLessons(deck, c.free || [], c.skip || []);
    for (const f of c.free || []) if (!lessons.some((l) => l.id === f)) throw new Error(`course.free names an unknown lesson: ${f}`);
    if (process.argv.includes('--print-lessons')) { for (const l of lessons) console.log(l.id.padEnd(34), String(l.titles.length).padStart(3), 'slides', l.free ? 'FREE' : '    ', l.title); continue; }
    const out = path.join(dir, 'course');
    fs.rmSync(path.join(out, 'data'), { recursive: true, force: true });
    for (const e of fs.existsSync(out) ? fs.readdirSync(out, { withFileTypes: true }) : []) if (e.isDirectory() && e.name !== 'audio') fs.rmSync(path.join(out, e.name), { recursive: true, force: true });
    fs.mkdirSync(path.join(out, 'data'), { recursive: true });
    fs.writeFileSync(path.join(out, 'course.css'), `/* Generated by tools/build-course.mjs: the deck's own look, scoped to .cstage, plus the course layout. */\n${scopedCss(deck.css)}\n${fs.readFileSync(path.join(SITE, 'training', 'course-layout.css'), 'utf8')}`);
    fs.writeFileSync(path.join(out, 'index.html'), indexPage(w, c, lessons));
    for (const l of lessons) {
      // optional recorded narration: course/audio/<lesson>/<nn>.mp3 (nn = slide number within the lesson, 01...)
      const adir = path.join(out, 'audio', l.id);
      l.payload.audio = l.payload.slides.map((_, k) => {
        const n = String(k + 1).padStart(2, '0');
        const f = ['mp3', 'm4a', 'webm', 'ogg'].map((x) => `${n}.${x}`).find((x) => fs.existsSync(path.join(adir, x)));
        return f ? `${w.path}course/audio/${l.id}/${f}` : '';
      });
      fs.mkdirSync(path.join(out, l.id), { recursive: true });
      fs.writeFileSync(path.join(out, l.id, 'index.html'), lessonPage(w, c, l, lessons, key));
      fs.writeFileSync(path.join(out, 'data', `${l.id}.json`), JSON.stringify(l.free ? l.payload : encrypt(key, l.id, l.payload)));
    }
    total += lessons.length;
    console.log(`built course ${w.id}: ${lessons.length} lessons (${lessons.filter((l) => l.free).length} free, ${lessons.filter((l) => !l.free).length} encrypted)`);
  }
}

export { loadDeck, buildLessons, scopedCss, plain, encrypt, slug };
