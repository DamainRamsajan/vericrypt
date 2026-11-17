// Fitness functions: the rules in arc42 sections 5, 8 and 9, checked on every build.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const SRC = path.join(ROOT, 'src');
const walk = (d) => fs.readdirSync(d, { withFileTypes: true }).flatMap((e) => e.isDirectory() ? walk(path.join(d, e.name)) : e.name.endsWith('.js') ? [path.join(d, e.name)] : []);
const files = walk(SRC);
const rel = (f) => path.relative(SRC, f).split(path.sep).join('/');
const importsOf = (f) => [...fs.readFileSync(f, 'utf8').matchAll(/^import .*? from '(.+?)';/gm)].map((m) => m[1]);
const resolved = (f, spec) => spec.startsWith('.') ? rel(path.resolve(path.dirname(f), spec)) : spec;
const inDir = (dir) => files.filter((f) => rel(f).startsWith(dir + '/'));

test('Blocks_only_call_allowed_neighbours', () => {
  // arc42 5.1: adapters know nothing about the Database or the Order Service; the fakes know only the router.
  const rules = {
    'payment-adapter': () => false,                                 // may import nothing from this project
    'notification-adapter': () => false,
    'fake-provider': (r) => r === 'router.js',
    'fake-sms': (r) => r === 'router.js',
    'order-service': (r) => !/^(payment-adapter|notification-adapter|fake-)/.test(r) && r !== 'server.js',
  };
  for (const [dir, allowed] of Object.entries(rules))
    for (const f of inDir(dir))
      for (const spec of importsOf(f).filter((s) => s.startsWith('.')))
        assert.ok(allowed(resolved(f, spec)), `${rel(f)} must not import ${spec}`);
});

test('Only_the_Order_Service_uses_the_Database', () => {
  for (const f of files) {
    const text = fs.readFileSync(f, 'utf8'), r = rel(f);
    if (r !== 'db.js') assert.ok(!text.includes('node:sqlite'), `${r} must not import the database driver`);
    if (!['db.js', 'order-service/order-repository.js'].includes(r)) assert.ok(!/\b(SELECT|INSERT INTO|UPDATE \w+ SET|DELETE FROM)\b/.test(text), `${r} must not contain SQL`);
    if (!['server.js', 'order-service/order-repository.js'].some((x) => r === x) && r !== 'db.js' && !r.startsWith('order-service/'))
      assert.ok(!importsOf(f).some((s) => s.endsWith('db.js')), `${r} must not import db.js`);
  }
});

test('No_card_data_in_the_schema_or_the_code', () => {
  const schema = fs.readFileSync(path.join(SRC, 'db.js'), 'utf8').split('const SCHEMA')[1].split('`;')[0];
  assert.ok(!/card|pan\b|cvv|cvc/i.test(schema), 'the schema must have no card columns');   // C1, Q3
  for (const f of files.filter((x) => !rel(x).startsWith('fake-provider/')))
    assert.ok(!/cardNumber|card_number|cvv|cvc/i.test(fs.readFileSync(f, 'utf8')), `${rel(f)} mentions card data`);
});

test('Lightweight_pages_customer_pages_stay_within_the_transfer_budget', () => {
  const pub = path.join(ROOT, 'public');
  const bytes = ['index.html', 'app.js', 'style.css', 'order.html'].reduce((s, f) => s + fs.statSync(path.join(pub, f)).size, 0);
  assert.ok(bytes <= 20_000, `customer pages weigh ${bytes} bytes; the budget is 20000`);   // ADR-0001, Q2, R3
  for (const f of ['index.html', 'order.html']) assert.ok(!/<script[^>]+src="https?:/.test(fs.readFileSync(path.join(pub, f), 'utf8')), 'no third-party scripts');
});

test('Every_write_endpoint_requires_a_role_or_an_idempotency_key', () => {
  const api = fs.readFileSync(path.join(SRC, 'order-service/order-api.js'), 'utf8');
  for (const m of api.matchAll(/router\.post\('(\/api\/[^']+)'/g)) {
    const line = api.split('\n').find((l) => l.includes(`'${m[1]}'`)) + (api.split(m[0])[1] || '').split('router.')[0];
    const ok = /vendorAuth|idempotency-key|coordinator\.startPayment/.test(line);
    assert.ok(ok, `POST ${m[1]} has neither a role check nor an idempotency key`);
  }
});

test('Documentation_arc42_passes_its_own_checks', (t) => {
  const check = path.join(ROOT, '..', 'check-arc42.py'), doc = path.join(ROOT, '..', 'arc42.md');
  if (!fs.existsSync(check) || !fs.existsSync(doc)) return t.skip('arc42 files not found next to the app');
  const r = spawnSync('python3', [check, doc], { encoding: 'utf8' });
  if (r.error) return t.skip('python3 not available');
  assert.equal(r.status, 0, r.stdout + r.stderr);
});

test('Documentation_every_test_named_in_arc42_exists', (t) => {
  const doc = path.join(ROOT, '..', 'arc42.md');
  if (!fs.existsSync(doc)) return t.skip('arc42.md not found next to the app');
  const named = [...new Set([...fs.readFileSync(doc, 'utf8').matchAll(/`([A-Z][A-Za-z0-9]*_[A-Za-z0-9_]+)`/g)].map((m) => m[1]))];
  const suite = fs.readdirSync(path.join(ROOT, 'test')).map((f) => fs.readFileSync(path.join(ROOT, 'test', f), 'utf8')).join('\n');
  assert.ok(named.length >= 8, 'expected test names in the contracts and in section 10.3');
  for (const name of named) assert.ok(suite.includes(`'${name}'`), `arc42 names the test ${name}, but no such test exists`);
});
