#!/usr/bin/env node
import { access, readFile } from 'node:fs/promises';
import { constants } from 'node:fs';

const required = [
  'AGENTS.md',
  'CONTEXT.md',
  'a2a/.well-known/agent-card.json',
  'guardrails/production-safety.md',
  'stages/00_handshake/CONTEXT.md',
  'stages/08_report_and_handoff/CONTEXT.md',
  'scripts/run-agent-cycle.mjs',
  'scripts/verify-url.mjs'
];

const results = [];
for (const f of required) {
  try { await access(f, constants.R_OK); results.push({ file: f, ok: true }); }
  catch (e) { results.push({ file: f, ok: false, error: e.message }); }
}

let cardOk = false;
try {
  JSON.parse(await readFile('a2a/.well-known/agent-card.json', 'utf8'));
  cardOk = true;
} catch {}

const ok = results.every(r => r.ok) && cardOk;
console.log(JSON.stringify({ ok, cardOk, results }, null, 2));
process.exit(ok ? 0 : 1);
