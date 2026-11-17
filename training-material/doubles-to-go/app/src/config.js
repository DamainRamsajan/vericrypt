// Configuration. Every value can be overridden with an environment variable.
export function loadConfig(env = process.env) {
  return {
    port: Number(env.PORT) || 3000,
    dbFile: env.DB_FILE || ':memory:',          // use a file path to keep data between runs
    demo: env.NODE_ENV !== 'production',        // enables the demo panel and the fake provider controls
    webhookSecret: env.WEBHOOK_SECRET || 'dev-webhook-secret',
    providerApiKey: env.PROVIDER_API_KEY || 'dev-provider-key',
    vendorToken: env.VENDOR_TOKEN || 'vendor-demo',
    supportToken: env.SUPPORT_TOKEN || 'support-demo',
    maxAttempts: 5,                              // retry with backoff, then park for review (arc42 section 8)
    backoffBaseMs: Number(env.BACKOFF_BASE_MS) || 1000,   // use about 30000 in production
    reconcileAfterMs: Number(env.RECONCILE_AFTER_MS) || 15000, // when to start checking an unpaid order
    workerIntervalMs: Number(env.WORKER_INTERVAL_MS) || 250,
    requestTimeoutMs: Number(env.REQUEST_TIMEOUT_MS) || 3000,
    providerUrl: env.PROVIDER_URL || null,       // default: the built-in fake provider
    smsUrl: env.SMS_URL || null,                 // default: the built-in fake SMS gateway
  };
}
