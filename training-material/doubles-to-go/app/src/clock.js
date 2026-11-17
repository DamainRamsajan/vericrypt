// A clock we can control in tests, so that backoff and reconciliation can be tested without waiting.
export const systemClock = { now: () => Date.now() };
export function testClock(start = 1_800_000_000_000) {
  let t = start;
  return { now: () => t, advance: (ms) => { t += ms; } };
}
