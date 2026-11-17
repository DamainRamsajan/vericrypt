// Database block (arc42 5.1). Only the Order Service may use this module (checked in test/architecture.test.js).
import { DatabaseSync } from 'node:sqlite';

const SCHEMA = `
CREATE TABLE IF NOT EXISTS vendors(id TEXT PRIMARY KEY, name TEXT NOT NULL, open INTEGER NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS items(id TEXT PRIMARY KEY, vendor_id TEXT NOT NULL REFERENCES vendors(id), name TEXT NOT NULL, price_cents INTEGER NOT NULL, stock INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS orders(id TEXT PRIMARY KEY, vendor_id TEXT NOT NULL REFERENCES vendors(id), customer_name TEXT NOT NULL, phone TEXT NOT NULL, status TEXT NOT NULL, total_cents INTEGER NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS order_items(order_id TEXT NOT NULL REFERENCES orders(id), item_id TEXT NOT NULL, qty INTEGER NOT NULL, price_cents INTEGER NOT NULL, PRIMARY KEY(order_id, item_id));
CREATE TABLE IF NOT EXISTS idempotency(key TEXT PRIMARY KEY, operation TEXT NOT NULL, result TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS payments(ref TEXT PRIMARY KEY, order_id TEXT NOT NULL UNIQUE REFERENCES orders(id), status TEXT NOT NULL, pay_url TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS jobs(id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL, unique_key TEXT NOT NULL UNIQUE, payload TEXT NOT NULL, status TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0, next_run_at INTEGER NOT NULL, last_error TEXT, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS events(id INTEGER PRIMARY KEY AUTOINCREMENT, order_id TEXT, type TEXT NOT NULL, cid TEXT, at INTEGER NOT NULL, details TEXT);
`;

export function openDb(file = ':memory:') {
  const db = new DatabaseSync(file);
  if (file !== ':memory:') db.exec('PRAGMA journal_mode=WAL');
  db.exec(SCHEMA);
  return db;
}

// Sample data so the demo has something to order.
export function seed(db) {
  if (db.prepare('SELECT COUNT(*) AS n FROM vendors').get().n > 0) return;
  const v = db.prepare('INSERT INTO vendors(id,name,open) VALUES(?,?,1)');
  const i = db.prepare('INSERT INTO items(id,vendor_id,name,price_cents,stock) VALUES(?,?,?,?,?)');
  v.run('ria', "Ria's Doubles"); v.run('ali', "Ali's Roti and Doubles"); v.run('sunny', "Sunny's Pholourie Stand");
  const menu = [
    ['ria-d1', 'ria', 'Doubles (slight pepper)', 600, 200], ['ria-d2', 'ria', 'Doubles (no pepper)', 600, 100], ['ria-p', 'ria', 'Aloo pie', 700, 60],
    ['ali-r', 'ali', 'Chicken roti', 4500, 40], ['ali-d', 'ali', 'Doubles', 600, 150], ['ali-c', 'ali', 'Corn soup', 1200, 50],
    ['sun-p6', 'sunny', 'Pholourie (6)', 500, 120], ['sun-s', 'sunny', 'Sahina (6)', 500, 80], ['sun-k', 'sunny', 'Kachori (2)', 600, 60],
  ];
  for (const m of menu) i.run(...m);
}
