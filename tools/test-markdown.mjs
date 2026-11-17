#!/usr/bin/env node
// Tests for the Markdown renderer (the only place HTML is generated from content).
import assert from 'node:assert/strict';
import { renderMarkdown } from '../site/kit/markdown.js';

const t = (name, fn) => { try { fn(); console.log(`ok   ${name}`); } catch (e) { console.log(`FAIL ${name}\n     ${e.message}`); process.exitCode = 1; } };

t('escapes raw HTML', () => {
  const html = renderMarkdown('<script>alert(1)</script> and <img src=x onerror=alert(1)>');
  assert.ok(!html.includes('<script>')); assert.ok(!html.includes('<img'));
});
t('blocks javascript: links', () => {
  const html = renderMarkdown('[x](javascript:alert(1))');
  assert.ok(!html.includes('href="javascript'));
});
t('blocks external images', () => {
  assert.ok(!renderMarkdown('![a](https://evil.example/x.png)').includes('<img'));
});
t('headings get ids', () => {
  assert.match(renderMarkdown('## Hello World'), /<h2 id="hello-world">Hello World<\/h2>/);
});
t('duplicate headings get distinct ids', () => {
  const html = renderMarkdown('## A\n\n## A');
  assert.match(html, /id="a"/); assert.match(html, /id="a-1"/);
});
t('tables', () => {
  const html = renderMarkdown('| a | b |\n| --- | --- |\n| 1 | `x|y` |');
  assert.match(html, /<th>a<\/th>/); assert.match(html, /<td>1<\/td>/);
});
t('escaped pipe in table cell', () => {
  assert.match(renderMarkdown('| a | b |\n| - | - |\n| x \\| y | z |'), /<td>x \| y<\/td>/);
});
t('fenced code is escaped and not formatted', () => {
  const html = renderMarkdown('```js\nconst a = "<b>**x**</b>";\n```');
  assert.match(html, /&lt;b&gt;\*\*x\*\*&lt;\/b&gt;/);
});
t('nested lists', () => {
  const html = renderMarkdown('- a\n  - b\n- c');
  assert.match(html, /<ul><li>a<ul><li>b<\/li><\/ul><\/li><li>c<\/li><\/ul>/);
});
t('ordered list', () => {
  assert.match(renderMarkdown('1. one\n2. two'), /<ol><li>one<\/li><li>two<\/li><\/ol>/);
});
t('inline formatting', () => {
  const html = renderMarkdown('**bold** and *em* and `code` and [link](https://example.com)');
  assert.match(html, /<strong>bold<\/strong>/); assert.match(html, /<em>em<\/em>/);
  assert.match(html, /<code>code<\/code>/); assert.match(html, /<a href="https:\/\/example.com" rel="noopener noreferrer">link<\/a>/);
});
t('snake_case is not italic', () => {
  assert.ok(!renderMarkdown('some_snake_case_name').includes('<em>'));
});
t('blockquote and hr', () => {
  const html = renderMarkdown('> quoted\n\n---\n\nafter');
  assert.match(html, /<blockquote><p>quoted<\/p>\n<\/blockquote>|<blockquote>/); assert.match(html, /<hr>/);
});
t('demote shifts heading levels', () => {
  assert.match(renderMarkdown('# A\n\n## B', { demote: 1 }), /<h2 id="a">A<\/h2>[\s\S]*<h3 id="b">B<\/h3>/);
  assert.match(renderMarkdown('###### Z', { demote: 1 }), /<h6/);
});
