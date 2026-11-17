// Order Repository (arc42 5.3): the only code that writes SQL.
export class OrderRepository {
  constructor(db) { this.db = db; this.inTx = false; }

  transaction(fn) {
    if (this.inTx) return fn();
    this.db.exec('BEGIN IMMEDIATE');
    this.inTx = true;
    try { const r = fn(); this.db.exec('COMMIT'); return r; }
    catch (e) { this.db.exec('ROLLBACK'); throw e; }
    finally { this.inTx = false; }
  }
  #q(sql) { return this.db.prepare(sql); }

  // ---- vendors, menus, stock
  listVendors() {
    const vs = this.#q('SELECT id,name,open FROM vendors ORDER BY name').all();
    const is = this.#q('SELECT id,vendor_id,name,price_cents,stock FROM items ORDER BY name').all();
    return vs.map(v => ({ id: v.id, name: v.name, open: !!v.open,
      items: is.filter(i => i.vendor_id === v.id).map(i => ({ id: i.id, name: i.name, priceCents: i.price_cents, stock: i.stock })) }));
  }
  getVendor(id) { const v = this.#q('SELECT id,name,open FROM vendors WHERE id=?').get(id); return v && { id: v.id, name: v.name, open: !!v.open }; }
  getItem(id) { const i = this.#q('SELECT id,vendor_id,name,price_cents,stock FROM items WHERE id=?').get(id); return i && { id: i.id, vendorId: i.vendor_id, name: i.name, priceCents: i.price_cents, stock: i.stock }; }
  decrementStock(itemId, qty) { return this.#q('UPDATE items SET stock=stock-? WHERE id=? AND stock>=?').run(qty, itemId, qty).changes === 1; }
  releaseStock(orderId) {
    this.#q('UPDATE items SET stock=stock+(SELECT qty FROM order_items WHERE order_id=? AND item_id=items.id) WHERE id IN (SELECT item_id FROM order_items WHERE order_id=?)').run(orderId, orderId);
  }
  setStock(itemId, vendorId, qty) { return this.#q('UPDATE items SET stock=? WHERE id=? AND vendor_id=?').run(qty, itemId, vendorId).changes === 1; }
  setMenuItem(itemId, vendorId, { name, priceCents }) {
    return this.#q('UPDATE items SET name=COALESCE(?,name), price_cents=COALESCE(?,price_cents) WHERE id=? AND vendor_id=?').run(name ?? null, priceCents ?? null, itemId, vendorId).changes === 1;
  }
  setOpen(vendorId, open) { return this.#q('UPDATE vendors SET open=? WHERE id=?').run(open ? 1 : 0, vendorId).changes === 1; }

  // ---- orders
  insertOrder(o) {
    this.#q('INSERT INTO orders(id,vendor_id,customer_name,phone,status,total_cents,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)')
      .run(o.id, o.vendorId, o.customerName, o.phone, 'PendingPayment', o.totalCents, o.now, o.now);
    const li = this.#q('INSERT INTO order_items(order_id,item_id,qty,price_cents) VALUES(?,?,?,?)');
    for (const l of o.lines) li.run(o.id, l.itemId, l.qty, l.priceCents);
  }
  getOrder(id) {
    const o = this.#q('SELECT o.id,o.vendor_id,o.customer_name,o.phone,o.status,o.total_cents,o.created_at,o.updated_at,v.name AS vendor_name FROM orders o JOIN vendors v ON v.id=o.vendor_id WHERE o.id=?').get(id);
    if (!o) return null;
    const items = this.#q('SELECT i.name,oi.qty,oi.price_cents FROM order_items oi JOIN items i ON i.id=oi.item_id WHERE oi.order_id=? ORDER BY i.name').all(id);
    return { id: o.id, vendorId: o.vendor_id, vendorName: o.vendor_name, customerName: o.customer_name, phone: o.phone, status: o.status,
      totalCents: o.total_cents, createdAt: o.created_at, updatedAt: o.updated_at, items: items.map(x => ({ name: x.name, qty: x.qty, priceCents: x.price_cents })) };
  }
  setStatus(orderId, fromStatuses, to, now) {
    const ph = fromStatuses.map(() => '?').join(',');
    return this.#q(`UPDATE orders SET status=?, updated_at=? WHERE id=? AND status IN (${ph})`).run(to, now, orderId, ...fromStatuses).changes === 1;
  }
  ordersForVendor(vendorId) {
    return this.#q('SELECT id,status,total_cents,created_at FROM orders WHERE vendor_id=? ORDER BY created_at DESC LIMIT 100').all(vendorId)
      .map(o => ({ id: o.id, status: o.status, totalCents: o.total_cents, createdAt: o.created_at }));
  }
  payoutFor(vendorId, fromMs, toMs) {
    const r = this.#q("SELECT COUNT(*) AS n, COALESCE(SUM(total_cents),0) AS total FROM orders WHERE vendor_id=? AND status IN ('Paid','Ready','Collected') AND created_at>=? AND created_at<?").get(vendorId, fromMs, toMs);
    return { orders: r.n, totalCents: r.total };
  }
  countOrders() { return this.#q('SELECT COUNT(*) AS n FROM orders').get().n; }

  // ---- idempotency
  getIdempotency(key) { return this.#q('SELECT result FROM idempotency WHERE key=?').get(key); }
  putIdempotency(key, operation, result, now) { this.#q('INSERT INTO idempotency(key,operation,result,created_at) VALUES(?,?,?,?)').run(key, operation, result, now); }

  // ---- payments
  insertPayment({ ref, orderId, payUrl, now }) { this.#q("INSERT OR IGNORE INTO payments(ref,order_id,status,pay_url,created_at) VALUES(?,?,'initiated',?,?)").run(ref, orderId, payUrl, now); }
  getPaymentByRef(ref) { const p = this.#q('SELECT ref,order_id,status,pay_url FROM payments WHERE ref=?').get(ref); return p && { ref: p.ref, orderId: p.order_id, status: p.status, payUrl: p.pay_url }; }
  getPaymentByOrder(orderId) { const p = this.#q('SELECT ref,order_id,status,pay_url FROM payments WHERE order_id=?').get(orderId); return p && { ref: p.ref, orderId: p.order_id, status: p.status, payUrl: p.pay_url }; }
  setPaymentStatus(ref, status) { this.#q('UPDATE payments SET status=? WHERE ref=?').run(status, ref); }

  // ---- jobs (the Confirmation Queue, ADR-0003)
  enqueueJob({ kind, uniqueKey, payload, runAt, now, status = 'queued', error = null }) {
    return this.#q('INSERT OR IGNORE INTO jobs(kind,unique_key,payload,status,attempts,next_run_at,last_error,created_at) VALUES(?,?,?,?,0,?,?,?)')
      .run(kind, uniqueKey, JSON.stringify(payload), status, runAt, error, now).changes === 1;
  }
  nextDueJob(now) { return this.#q("SELECT * FROM jobs WHERE status='queued' AND next_run_at<=? ORDER BY next_run_at, id LIMIT 1").get(now); }
  finishJob(id) { this.#q("UPDATE jobs SET status='done' WHERE id=?").run(id); }
  rescheduleJob(id, attempts, runAt, error) { this.#q('UPDATE jobs SET attempts=?, next_run_at=?, last_error=? WHERE id=?').run(attempts, runAt, error, id); }
  parkJob(id, attempts, error) { this.#q("UPDATE jobs SET status='parked', attempts=?, last_error=? WHERE id=?").run(attempts, error, id); }
  jobCounts() { return this.#q('SELECT kind,status,COUNT(*) AS n FROM jobs GROUP BY kind,status').all().map(r => ({ kind: r.kind, status: r.status, n: r.n })); }
  parkedJobs() { return this.#q("SELECT kind,unique_key,last_error FROM jobs WHERE status='parked'").all().map(r => ({ kind: r.kind, key: r.unique_key, error: r.last_error })); }

  // ---- audit trail
  addEvent({ orderId, type, cid, at, details }) { this.#q('INSERT INTO events(order_id,type,cid,at,details) VALUES(?,?,?,?,?)').run(orderId ?? null, type, cid ?? null, at, details ? JSON.stringify(details) : null); }
  eventsFor(orderId) { return this.#q('SELECT type,cid,at,details FROM events WHERE order_id=? ORDER BY id').all(orderId).map(e => ({ type: e.type, cid: e.cid, at: e.at, details: e.details ? JSON.parse(e.details) : null })); }
}
