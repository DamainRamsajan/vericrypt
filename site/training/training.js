// Progressive enhancement for /training/. The pages are complete HTML without this script;
// it adds the early-bird countdown, saved checklists, copy buttons and the email form.
import { $, $$, storage, copyText } from '/kit/dom.js';

/* early-bird text: stays true after the deadline without anyone editing the page */
for (const node of $$('[data-eb]')) {
  const until = new Date(node.dataset.until);
  if (Number.isNaN(until.getTime())) continue;
  const days = Math.ceil((until - Date.now()) / 864e5);
  if (days > 0) {
    node.textContent = `Early-bird price ${node.dataset.label} until ${node.dataset.untilLabel}. ${days} ${days === 1 ? 'day' : 'days'} left.`;
  } else {
    node.textContent = `The early-bird price has ended. ${node.dataset.after || ''}`.trim();
  }
}

/* guide: ticks are remembered in this browser only */
const checks = $$('input[data-check]');
if (checks.length) {
  let saved = {};
  try { saved = JSON.parse(storage.get('iw-training-checks') || '{}'); } catch { saved = {}; }
  for (const c of checks) {
    c.checked = !!saved[c.dataset.check];
    c.addEventListener('change', () => { saved[c.dataset.check] = c.checked; storage.set('iw-training-checks', JSON.stringify(saved)); });
  }
  const note = $('[data-saved-note]');
  if (note) note.hidden = false;
}

/* guide: copy buttons */
for (const btn of $$('button[data-copy]')) {
  const target = document.getElementById(btn.dataset.copy);
  if (!target) continue;
  btn.hidden = false;
  btn.addEventListener('click', async () => {
    btn.textContent = (await copyText(target.textContent)) ? 'Copied' : 'Copy failed: select the text';
    setTimeout(() => { btn.textContent = 'Copy'; }, 1800);
  });
}

/* register: compose an email in the visitor's own mail app. Nothing is sent from this page. */
const form = $('#reserve');
if (form) {
  form.hidden = false;
  const plain = $('[data-mail]');
  if (plain) plain.hidden = true;
  form.addEventListener('submit', (e) => {
    e.preventDefault();
    const f = form.elements;
    const body = `Name: ${f.name.value}\nEmail: ${f.email.value}\nTeam or organisation: ${f.team.value}\nNotes: ${f.message.value}\n`;
    location.href = `mailto:${form.dataset.to}?subject=${encodeURIComponent(form.dataset.subject)}&body=${encodeURIComponent(body)}`;
    $('#reserve-msg').textContent = 'Your email app should open with your details ready to send. If it does not, use the WhatsApp button or write to us directly.';
  });
}

/* hero: tap one of the twelve sections to see the question it answers */
{
  const cells = $$('.sheet .cell');
  const out = $('[data-sheet-q]');
  for (const c of cells) {
    c.addEventListener('click', (e) => {
      e.preventDefault();
      for (const o of cells) o.classList.remove('on');
      c.classList.add('on');
      if (out) out.textContent = c.dataset.q;
    });
  }
}
