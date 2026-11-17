// Minimal, safe Markdown renderer for the Docs and Verify tabs.
// Everything is HTML-escaped first; the only tags produced are the ones below.
// Supported: headings, paragraphs, bullet/ordered lists (nested by indentation),
// tables, fenced code, blockquotes, rules, links, images, bold, italic, inline code.

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

const BAD_SCHEME = /^\s*(javascript|data|vbscript):/i;

export function slugify(text) {
  return text.toLowerCase().replace(/<[^>]+>/g, '').replace(/[^\p{L}\p{N}\s-]/gu, '').trim().replace(/\s+/g, '-') || 'section';
}

function inline(raw) {
  const codes = [];
  let s = raw.replace(/`([^`]+)`/g, (_, c) => { codes.push(c); return `\u0000${codes.length - 1}\u0000`; });
  s = esc(s);
  s = s.replace(/!\[([^\]]*)\]\(([^)\s]+)\)/g, (_, alt, url) =>
    (/^(https?:|\/\/)/i.test(url) || BAD_SCHEME.test(url)) ? alt : `<img src="${url}" alt="${alt}" loading="lazy">`);
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, text, url) => {
    if (BAD_SCHEME.test(url)) return text;
    const external = /^https?:\/\//i.test(url);
    return `<a href="${url}"${external ? ' rel="noopener noreferrer"' : ''}>${text}</a>`;
  });
  s = s.replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/(^|[^*\w])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>');
  s = s.replace(/\u0000(\d+)\u0000/g, (_, n) => `<code>${esc(codes[Number(n)])}</code>`);
  return s;
}

const LIST_ITEM = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;
const TABLE_SEP = /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/;

function splitRow(line) {
  let t = line.trim();
  if (t.startsWith('|')) t = t.slice(1);
  if (t.endsWith('|') && !t.endsWith('\\|')) t = t.slice(0, -1);
  const cells = [];
  let cur = '';
  for (let i = 0; i < t.length; i++) {
    if (t[i] === '\\' && t[i + 1] === '|') { cur += '|'; i++; continue; }
    if (t[i] === '|') { cells.push(cur.trim()); cur = ''; continue; }
    cur += t[i];
  }
  cells.push(cur.trim());
  return cells;
}

function isBlockStart(line, next) {
  return /^\s*(```|#{1,6}\s|>|(-{3,}|\*{3,})\s*$)/.test(line) || LIST_ITEM.test(line) ||
    (line.includes('|') && next !== undefined && TABLE_SEP.test(next));
}

function renderList(items) {
  // items: [{indent, ordered, text}]; build nested lists by indentation
  let html = '';
  const stack = []; // {indent, ordered}
  const closeTo = (indent) => {
    while (stack.length && stack[stack.length - 1].indent > indent) {
      const top = stack.pop();
      html += `</li></${top.ordered ? 'ol' : 'ul'}>`;
    }
  };
  for (const item of items) {
    closeTo(item.indent);
    const top = stack[stack.length - 1];
    if (!top || item.indent > top.indent) {
      html += `<${item.ordered ? 'ol' : 'ul'}><li>`;
      stack.push({ indent: item.indent, ordered: item.ordered });
    } else {
      html += '</li><li>';
    }
    html += inline(item.text);
  }
  closeTo(-1);
  return html;
}

// opts.demote: shift heading levels down (the page already has its own <h1>).
export function renderMarkdown(source, opts = {}) {
  const demote = opts.demote || 0;
  const lines = String(source).replace(/\r\n?/g, '\n').split('\n');
  const out = [];
  const seen = new Map();
  let i = 0;

  const headingId = (text) => {
    const base = slugify(text);
    const n = seen.get(base) || 0;
    seen.set(base, n + 1);
    return n ? `${base}-${n}` : base;
  };

  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) { i++; continue; }

    const fence = line.match(/^\s*```\s*([\w-]*)\s*$/);
    if (fence) {
      const buf = [];
      i++;
      while (i < lines.length && !/^\s*```\s*$/.test(lines[i])) buf.push(lines[i++]);
      i++;
      out.push(`<pre><code${fence[1] ? ` class="lang-${esc(fence[1])}"` : ''}>${esc(buf.join('\n'))}</code></pre>`);
      continue;
    }

    const h = line.match(/^(#{1,6})\s+(.*?)\s*#*\s*$/);
    if (h) {
      const level = Math.min(6, h[1].length + demote);
      out.push(`<h${level} id="${headingId(h[2])}">${inline(h[2])}</h${level}>`);
      i++;
      continue;
    }

    if (/^\s*(-{3,}|\*{3,})\s*$/.test(line)) { out.push('<hr>'); i++; continue; }

    if (/^\s*>/.test(line)) {
      const buf = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) buf.push(lines[i++].replace(/^\s*>\s?/, ''));
      out.push(`<blockquote>${renderMarkdown(buf.join('\n'), opts)}</blockquote>`);
      continue;
    }

    if (line.includes('|') && i + 1 < lines.length && TABLE_SEP.test(lines[i + 1])) {
      const head = splitRow(line);
      i += 2;
      const rows = [];
      while (i < lines.length && lines[i].trim() && lines[i].includes('|')) rows.push(splitRow(lines[i++]));
      out.push('<div class="table-wrap"><table><thead><tr>' + head.map((c) => `<th>${inline(c)}</th>`).join('') +
        '</tr></thead><tbody>' + rows.map((r) => '<tr>' + head.map((_, k) => `<td>${inline(r[k] ?? '')}</td>`).join('') + '</tr>').join('') +
        '</tbody></table></div>');
      continue;
    }

    if (LIST_ITEM.test(line)) {
      const items = [];
      while (i < lines.length) {
        const m = lines[i].match(LIST_ITEM);
        if (m) {
          items.push({ indent: m[1].replace(/\t/g, '    ').length, ordered: /\d/.test(m[2]), text: m[3] });
          i++;
        } else if (lines[i].trim() && /^\s+\S/.test(lines[i]) && !isBlockStart(lines[i], lines[i + 1])) {
          items[items.length - 1].text += ' ' + lines[i].trim();
          i++;
        } else break;
      }
      out.push(renderList(items));
      continue;
    }

    const buf = [];
    while (i < lines.length && lines[i].trim() && !(buf.length && isBlockStart(lines[i], lines[i + 1]))) buf.push(lines[i++].trim());
    out.push(`<p>${inline(buf.join(' '))}</p>`);
  }
  return out.join('\n');
}
