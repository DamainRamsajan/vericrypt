// A stand-in for the external SMS gateway (arc42 3.2), with an outbox you can show on screen and an outage switch.
import { sendJson, parseJson } from '../router.js';

export function registerFakeSms(router, { logger }) {
  const state = { up: true, outbox: [] };
  router.post('/fake-sms/send', ({ res, raw }) => {
    if (!state.up) return sendJson(res, 503, { error: 'gateway down' });
    const { to, message } = parseJson(raw);
    state.outbox.push({ to: '***' + String(to || '').replace(/\D/g, '').slice(-4), message, at: new Date().toISOString() });  // masked, like a real delivery report
    sendJson(res, 200, { accepted: true });
  });
  router.get('/fake-sms/outbox', ({ res }) => sendJson(res, 200, { up: state.up, messages: state.outbox.slice(-50) }));
  router.post('/fake-sms/admin/mode', ({ res, raw }) => { const { up } = parseJson(raw); state.up = !!up; sendJson(res, 200, { up: state.up }); });
  return state;
}
