// Idempotency Guard (arc42 5.3, ADR-0005): one key, at most one result.
import { DomainError } from '../errors.js';

export class IdempotencyGuard {
  constructor(repo) { this.repo = repo; }
  static validate(key) {
    if (typeof key !== 'string' || !/^[A-Za-z0-9_.:-]{8,128}$/.test(key)) throw new DomainError('InvalidRequest', 'Idempotency-Key header is required (8 to 128 safe characters)', 400);
    return key;
  }
  lookup(key) { const r = this.repo.getIdempotency(key); return r ? JSON.parse(r.result) : null; }
  remember(key, operation, result, now) { this.repo.putIdempotency(key, operation, JSON.stringify(result), now); }
}
