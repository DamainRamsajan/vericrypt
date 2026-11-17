// Domain errors map directly to the error modes in the contracts (arc42 section 5).
export class DomainError extends Error {
  constructor(code, message, status = 409) { super(message || code); this.code = code; this.status = status; }
}
