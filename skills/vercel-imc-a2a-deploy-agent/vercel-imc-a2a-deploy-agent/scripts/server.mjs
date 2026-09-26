#!/usr/bin/env node
import http from 'node:http';
import crypto from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import path from 'node:path';
import { nowId, slugify, writeJson } from './lib/util.mjs';

const port = Number(process.env.PORT || 8787);

function send(res, status, data, headers = {}) {
  const body = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
  res.writeHead(status, { 'content-type': typeof data === 'string' ? 'text/plain' : 'application/json', ...headers });
  res.end(body);
}

async function readBody(req) {
  const chunks = [];
  for await (const c of req) chunks.push(c);
  return Buffer.concat(chunks);
}

function verifyGithubSignature(raw, sig) {
  if (!process.env.GITHUB_WEBHOOK_SECRET) return true;
  if (!sig || !sig.startsWith('sha256=')) return false;
  const expected = 'sha256=' + crypto.createHmac('sha256', process.env.GITHUB_WEBHOOK_SECRET).update(raw).digest('hex');
  return crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(sig));
}

function requireBearer(req) {
  const expected = process.env.A2A_SHARED_SECRET;
  if (!expected) return true;
  const got = req.headers.authorization || '';
  return got === `Bearer ${expected}`;
}

function startCycle({ repo, branch, dryRun, prod, runId }) {
  const args = ['scripts/run-agent-cycle.mjs', '--repo', repo, '--branch', branch || 'main', '--run-id', runId];
  if (dryRun) args.push('--dry-run');
  if (prod) args.push('--prod');
  const child = spawn('node', args, { stdio: 'ignore', detached: true });
  child.unref();
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === 'GET' && (req.url === '/.well-known/agent-card.json' || req.url === '/a2a/.well-known/agent-card.json')) {
      const card = await readFile('a2a/.well-known/agent-card.json', 'utf8');
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(card);
      return;
    }

    if (req.method === 'GET' && req.url === '/healthz') {
      send(res, 200, { ok: true, service: 'vercel-imc-a2a-deploy-agent', time: new Date().toISOString() });
      return;
    }

    if (req.method === 'POST' && req.url === '/a2a/tasks') {
      if (!requireBearer(req)) return send(res, 401, { error: 'unauthorized' });
      const raw = await readBody(req);
      const payload = JSON.parse(raw.toString('utf8'));
      const params = payload.params || {};
      const repo = params.repo;
      const branch = params.branch || 'main';
      const runId = `${nowId()}-${slugify(repo || 'a2a-task')}`;
      const runDir = path.join('runs', runId);
      await mkdir(runDir, { recursive: true });
      await writeJson(path.join(runDir, 'a2a-request.json'), payload);
      if (repo) startCycle({ repo, branch, dryRun: !!params.dryRun, prod: !params.dryRun, runId });
      return send(res, 202, { jsonrpc: '2.0', id: payload.id || null, result: { taskId: runId, state: repo ? 'accepted' : 'input-required', runDir, required: repo ? [] : ['params.repo'] } });
    }

    if (req.method === 'POST' && req.url === '/webhooks/github') {
      const raw = await readBody(req);
      if (!verifyGithubSignature(raw, req.headers['x-hub-signature-256'])) return send(res, 401, { error: 'invalid-signature' });
      const event = req.headers['x-github-event'];
      const delivery = req.headers['x-github-delivery'];
      const payload = JSON.parse(raw.toString('utf8'));
      const repo = payload.repository?.full_name;
      const ref = payload.ref || '';
      const branch = ref.startsWith('refs/heads/') ? ref.slice('refs/heads/'.length) : (payload.workflow_run?.head_branch || 'main');
      const sha = payload.after || payload.workflow_run?.head_sha || payload.deployment_status?.sha || 'event';
      const runId = `${nowId()}-${slugify(repo || 'github')}-${String(sha).slice(0, 7)}`;
      const runDir = path.join('runs', runId);
      await mkdir(runDir, { recursive: true });
      await writeJson(path.join(runDir, 'github-webhook.json'), { event, delivery, repo, branch, sha, payload });
      if (event === 'push' && repo && branch === (process.env.PRODUCTION_BRANCH || 'main')) {
        startCycle({ repo, branch, dryRun: false, prod: true, runId });
        return send(res, 202, { accepted: true, runId, runDir, repo, branch });
      }
      return send(res, 202, { accepted: true, started: false, reason: 'event-not-production-push', event, repo, branch, runId, runDir });
    }

    send(res, 404, { error: 'not-found' });
  } catch (e) {
    send(res, 500, { error: e.message });
  }
});

server.listen(port, () => {
  console.error(`Vercel IMC A2A Deploy Agent listening on :${port}`);
});
