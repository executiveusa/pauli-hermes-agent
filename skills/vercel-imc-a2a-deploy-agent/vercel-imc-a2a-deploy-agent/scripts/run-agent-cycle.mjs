#!/usr/bin/env node
import path from 'node:path';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { parseArgs, nowId, slugify, run, writeJson, extractUrl } from './lib/util.mjs';

const args = parseArgs(process.argv.slice(2));
const repo = args.repo;
const branch = args.branch || process.env.PRODUCTION_BRANCH || 'main';
const dryRun = !!args['dry-run'];
const prod = !!args.prod || !dryRun;
const cwdArg = args.cwd;
const runId = args['run-id'] || `${nowId()}-${slugify(repo || cwdArg || 'manual')}`;
const runDir = path.resolve(args['run-dir'] || `runs/${runId}`);
await mkdir(runDir, { recursive: true });

const policy = JSON.parse(await readFile('config/production-policy.json', 'utf8'));
const event = { repo, branch, dryRun, prod, cwd: cwdArg || null, runId, runDir, receivedAt: new Date().toISOString() };
await writeJson(path.join(runDir, 'event.json'), event);

function fail(reason, extra = {}) {
  return { ok: false, visible: false, runId, runDir, reason, ...extra, finishedAt: new Date().toISOString() };
}

let result;
try {
  if (prod && !policy.allowedProductionBranches.includes(branch)) {
    result = fail('branch-not-approved-for-production', { branch, allowed: policy.allowedProductionBranches });
    await writeJson(path.join(runDir, 'summary.json'), result);
    console.log(JSON.stringify(result, null, 2));
    process.exit(1);
  }
  if (prod && !process.env.VERCEL_TOKEN) throw new Error('VERCEL_TOKEN is required for production deployment');

  let workdir = cwdArg ? path.resolve(cwdArg) : path.join(runDir, 'checkout');
  if (!cwdArg) {
    if (!repo) throw new Error('Provide --repo owner/repo or --cwd path');
    const cloneUrl = process.env.GITHUB_TOKEN
      ? `https://x-access-token:${process.env.GITHUB_TOKEN}@github.com/${repo}.git`
      : `https://github.com/${repo}.git`;
    const clone = await run('git', ['clone', '--depth=1', '--branch', branch, cloneUrl, workdir]);
    await writeJson(path.join(runDir, 'clone.json'), clone);
    if (!clone.ok) throw new Error(`git clone failed: ${clone.stderr || clone.stdout}`);
  }

  const repoState = {
    packageJson: existsSync(path.join(workdir, 'package.json')),
    vercelDir: existsSync(path.join(workdir, '.vercel')),
    workdir
  };
  await writeJson(path.join(runDir, 'repo-state.json'), repoState);

  if (dryRun) {
    result = { ok: true, dryRun: true, visible: false, runId, runDir, next: 'Run without --dry-run to deploy.', repoState };
    await writeJson(path.join(runDir, 'summary.json'), result);
    console.log(JSON.stringify(result, null, 2));
    process.exit(0);
  }

  const vercelEnv = { VERCEL_TOKEN: process.env.VERCEL_TOKEN };
  const scopeArgs = [];
  if (process.env.VERCEL_TEAM_SLUG) scopeArgs.push('--scope', process.env.VERCEL_TEAM_SLUG);

  const whoami = await run('vercel', ['whoami', ...scopeArgs], { cwd: workdir, env: vercelEnv });
  await writeJson(path.join(runDir, 'vercel-whoami.json'), whoami);

  if (!repoState.vercelDir) {
    const linkArgs = ['link', '--repo', '--yes', ...scopeArgs];
    const link = await run('vercel', linkArgs, { cwd: workdir, env: vercelEnv });
    await writeJson(path.join(runDir, 'vercel-link.json'), link);
    if (!link.ok) throw new Error(`vercel link failed: ${link.stderr || link.stdout}`);
  }

  const deployArgs = ['deploy', '--prod', '--yes', '--no-wait', '--meta', `imcAgentRun=${runId}`, ...scopeArgs];
  const deploy = await run('vercel', deployArgs, { cwd: workdir, env: vercelEnv });
  const deploymentUrl = extractUrl(deploy.stdout + '\n' + deploy.stderr);
  await writeJson(path.join(runDir, 'deploy.json'), { ...deploy, deploymentUrl });
  if (!deploy.ok || !deploymentUrl) throw new Error(`vercel deploy failed or returned no URL: ${deploy.stderr || deploy.stdout}`);

  const inspect = await run('vercel', ['inspect', deploymentUrl, '--wait', '--logs', '--timeout=10m', ...scopeArgs], { cwd: workdir, env: vercelEnv });
  await writeJson(path.join(runDir, 'inspect.json'), inspect);
  await writeFile(path.join(runDir, 'build.log'), `${inspect.stdout}\n${inspect.stderr}\n`, 'utf8');

  const verify = await run('node', ['scripts/verify-url.mjs', deploymentUrl, '--out', path.join(runDir, 'browser-check.json')]);
  await writeJson(path.join(runDir, 'verify-command.json'), verify);
  let browserCheck = null;
  try { browserCheck = JSON.parse(await readFile(path.join(runDir, 'browser-check.json'), 'utf8')); } catch {}

  const report = [
    `# Vercel IMC Agent Report`,
    ``,
    `- Run: \`${runId}\``,
    `- Repo: \`${repo || workdir}\``,
    `- Branch: \`${branch}\``,
    `- Deployment: ${deploymentUrl}`,
    `- Deploy command OK: ${deploy.ok}`,
    `- Inspect command OK: ${inspect.ok}`,
    `- Visible: ${browserCheck?.visible === true}`,
    ``,
    `## Browser evidence`,
    ``,
    '```json',
    JSON.stringify(browserCheck, null, 2),
    '```'
  ].join('\n');
  await writeFile(path.join(runDir, 'report.md'), report + '\n', 'utf8');

  result = { ok: true, runId, runDir, repo, branch, deploymentUrl, inspectOk: inspect.ok, visible: browserCheck?.visible === true, browserCheck, report: path.join(runDir, 'report.md') };
} catch (e) {
  result = fail('cycle-failed', { error: e.message });
}

await writeJson(path.join(runDir, 'summary.json'), result);
console.log(JSON.stringify(result, null, 2));
process.exit(result.ok && (dryRun || result.visible) ? 0 : 1);
