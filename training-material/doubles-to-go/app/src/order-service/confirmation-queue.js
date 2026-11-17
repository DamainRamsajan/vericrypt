// Confirmation Queue (arc42 5.3, ADR-0003): a database-backed queue inside the same deployable.
// Jobs run one by one. Failures retry with backoff, then are parked for review (arc42 section 8).
// It carries three kinds of job: 'confirm' (payment confirmations), 'check' (reconciliation) and 'sms' (ready notices).
export class ConfirmationQueue {
  constructor({ repo, clock, config, logger }) { this.repo = repo; this.clock = clock; this.config = config; this.logger = logger; }

  enqueue(kind, uniqueKey, payload, delayMs = 0) {
    const now = this.clock.now();
    return this.repo.enqueueJob({ kind, uniqueKey, payload, runAt: now + delayMs, now });   // false when the key already exists (a replay)
  }
  park(kind, uniqueKey, payload, reason) {
    const now = this.clock.now();
    this.logger.log(null, 'job.parked', { kind, key: uniqueKey, reason });
    return this.repo.enqueueJob({ kind, uniqueKey, payload, runAt: now, now, status: 'parked', error: reason });
  }
  backoff(attempts) { return this.config.backoffBaseMs * 2 ** (attempts - 1); }

  async runDue(handlers, { limit = 500 } = {}) {
    let n = 0;
    while (n < limit) {
      const job = this.repo.nextDueJob(this.clock.now());
      if (!job) break;
      n++;
      await this.#process(job, handlers);
    }
    return n;
  }
  async #process(job, handlers) {
    const handler = handlers[job.kind];
    try {
      const r = handler ? await handler(JSON.parse(job.payload), job) : { retry: true, error: 'no handler for ' + job.kind };
      if (r && r.retry) this.#retry(job, r.error || 'retry'); else this.repo.finishJob(job.id);
    } catch (e) { this.#retry(job, e.message); }
  }
  #retry(job, error) {
    const attempts = job.attempts + 1;
    if (attempts >= this.config.maxAttempts) {
      this.repo.parkJob(job.id, attempts, error);
      this.logger.log(`job-${job.id}`, 'job.parked', { kind: job.kind, key: job.unique_key, attempts, error });
    } else {
      this.repo.rescheduleJob(job.id, attempts, this.clock.now() + Math.max(1, this.backoff(attempts)), error);
    }
  }
}
