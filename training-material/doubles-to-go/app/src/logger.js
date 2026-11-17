// Logging with a correlation id on every line (arc42 section 8).
// Personal data is never logged: these keys are always dropped (C4, privacy by minimising).
const NEVER_LOG = ['phone', 'customerName', 'card', 'to', 'message'];
export function createLogger(sink = (line) => console.log(line)) {
  return {
    log(cid, event, fields = {}) {
      const safe = { ...fields };
      for (const k of NEVER_LOG) delete safe[k];
      sink(JSON.stringify({ t: new Date().toISOString(), cid, event, ...safe }));
    },
  };
}
