#!/usr/bin/env node
// Deploys a site to Cloudflare Pages by direct upload (Wrangler), bypassing the GitHub integration.
//   npm run deploy -- home                 one target: home | verichain | vericrypt | erp | omc | agentseed
//   npm run deploy -- all                  every target
//   options: --dry (print what would run), --no-gate (skip sync and check), --branch=main
// Needs two environment variables (Codespaces secrets, see INSTALL.md):
//   CLOUDFLARE_API_TOKEN   token with Account > Cloudflare Pages > Edit
//   CLOUDFLARE_ACCOUNT_ID  your account id
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';

const WRANGLER = 'wrangler@4';
const cfg = JSON.parse(fs.readFileSync('tools/deploy.config.json', 'utf8'));
const args = process.argv.slice(2);
const flags = args.filter((a) => a.startsWith('--'));
const names = args.filter((a) => !a.startsWith('--'));
const dry = flags.includes('--dry');
const branch = (flags.find((f) => f.startsWith('--branch=')) || '--branch=main').split('=')[1];
const targets = names.includes('all') ? Object.keys(cfg).filter((k) => k !== 'note') : names;

if (!targets.length || targets.some((t) => !cfg[t] || t === 'note')) {
  console.error(`Usage: npm run deploy -- <${Object.keys(cfg).filter((k) => k !== 'note').join('|')}|all> [--dry] [--no-gate] [--branch=main]`);
  process.exit(2);
}
if (!dry && (!process.env.CLOUDFLARE_API_TOKEN || !process.env.CLOUDFLARE_ACCOUNT_ID)) {
  console.error('Missing CLOUDFLARE_API_TOKEN or CLOUDFLARE_ACCOUNT_ID. See INSTALL.md, "Deploy from the Codespace".');
  process.exit(2);
}

const run = (cmd, a, opts = {}) => {
  console.log(`\n$ ${cmd} ${a.join(' ')}`);
  if (dry) return { status: 0, stdout: '', stderr: '' };
  return spawnSync(cmd, a, { encoding: 'utf8', stdio: opts.capture ? 'pipe' : 'inherit', env: process.env });
};

if (!flags.includes('--no-gate')) {
  // generated files (metadata, manifest, training pages) must be current before upload
  for (const a of [['run', 'sync'], ['tools/check.mjs', '--all']]) {
    const r = a[0] === 'run' ? run('npm', a) : run('node', a);
    if (r.status !== 0) { console.error('\nThe gate failed, nothing was deployed.'); process.exit(1); }
  }
}

for (const t of targets) {
  const { dir, project } = cfg[t];
  if (!dry && !fs.existsSync(`${dir}/index.html`)) { console.error(`${dir}/index.html not found`); process.exit(1); }
  // create the project the first time; harmless if it exists
  const made = run('npx', ['--yes', WRANGLER, 'pages', 'project', 'create', project, '--production-branch', branch], { capture: true });
  const out = `${made.stdout || ''}${made.stderr || ''}`;
  if (made.status !== 0 && !/already exists|8000002/i.test(out)) { console.error(out); console.error(`Could not create project ${project}.`); process.exit(1); }
  if (made.status === 0 && !dry) console.log(`created project ${project}`);
  const dep = run('npx', ['--yes', WRANGLER, 'pages', 'deploy', dir, '--project-name', project, '--branch', branch, '--commit-dirty=true']);
  if (dep.status !== 0) { console.error(`\nDeploy of ${t} failed.`); process.exit(1); }
  console.log(`\n${t}: https://${project}.pages.dev   (open /verify-site/ on it: it must say all files match)`);
}
