import { loadConfig } from './config.js';
import { createApp } from './server.js';

const config = loadConfig();
const app = createApp({ config });
const base = await app.start(config.port);
console.log(`Doubles-to-Go is running.\n  Customers:  ${base}/\n  Vendors:    ${base}/vendor/\n  Demo panel: ${base}/demo/\n  Data:       ${config.dbFile}`);
