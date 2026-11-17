// Cart Validator (arc42 5.3): the pre-conditions of PlaceOrder.
import { DomainError } from '../errors.js';

export class CartValidator {
  constructor(repo) { this.repo = repo; }
  validate(input) {
    const bad = (m) => new DomainError('InvalidRequest', m, 400);
    if (!input || typeof input !== 'object') throw bad('Request body is required');
    const { vendorId, items, customerName, phone } = input;
    if (typeof customerName !== 'string' || !customerName.trim() || customerName.length > 60) throw bad('customerName is required (60 characters at most)');
    if (typeof phone !== 'string' || !/^[0-9+()\-\s]{7,20}$/.test(phone)) throw bad('phone must look like a phone number');
    if (!Array.isArray(items) || items.length === 0) throw bad('Cart must not be empty');
    const vendor = this.repo.getVendor(vendorId);
    if (!vendor) throw bad('Unknown vendor');
    if (!vendor.open) throw new DomainError('VendorClosed', `${vendor.name} is closed right now`);
    const lines = []; let total = 0; const seen = new Set();
    for (const it of items) {
      if (!it || !Number.isInteger(it.qty) || it.qty < 1 || it.qty > 20) throw bad('Each quantity must be a whole number from 1 to 20');
      if (seen.has(it.itemId)) throw bad('Each item may appear once');
      seen.add(it.itemId);
      const item = this.repo.getItem(it.itemId);
      if (!item || item.vendorId !== vendor.id) throw bad('Item does not belong to this vendor');
      if (item.stock < it.qty) throw new DomainError('ItemUnavailable', `${item.name} is not available in that quantity`);
      lines.push({ itemId: item.id, qty: it.qty, priceCents: item.priceCents });
      total += item.priceCents * it.qty;
    }
    return { vendor, lines, totalCents: total, customerName: customerName.trim(), phone: phone.trim() };
  }
}
