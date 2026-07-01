import { spawn } from 'node:child_process';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import path from 'node:path';

export function nowId() {
  return new Date().toISOString().replace(/[-:]/g, '').replace(/\.\d+Z$/, 'Z');
}

export function slugify(input = 'manual') {
  return String(input).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 80) || 'manual';
}

export function redact(value) {
  if (value == null) return value;
  let s = String(value);
  s = s.replace(/(ghp_|github_pat_|gho_|ghu_)[A-Za-z0-9_]+/g, '$1[REDACTED]');
  s = s.replace(/(vercel_)[A-Za-z0-9_]+/g, '$1[REDACTED]');
  s = s.replace(/(sk-)[A-Za-z0-9_-]+/g, '$1[REDACTED]');
  s = s.replace(/(xoxb-|xapp-)[A-Za-z0-9_-]+/g, '$1[REDACTED]');
  return s;
}

export async function ensureDir(dir) {
  await mkdir(dir, { recursive: true });
}

export async function writeJson(file, data) {
  await ensureDir(path.dirname(file));
  await writeFile(file, JSON.stringify(data, null, 2) + '\n', 'utf8');
}

export async function readJson(file) {
  return JSON.parse(await readFile(file, 'utf8'));
}

export async function run(cmd, args = [], options = {}) {
  const startedAt = new Date().toISOString();
  const child = spawn(cmd, args, {
    cwd: options.cwd || process.cwd(),
    env: { ...process.env, ...(options.env || {}) },
    shell: false
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', d => stdout += d.toString());
  child.stderr.on('data', d => stderr += d.toString());
  const code = await new Promise(resolve => child.on('close', resolve));
  return {
    cmd,
    args,
    cwd: options.cwd || process.cwd(),
    code,
    stdout: redact(stdout),
    stderr: redact(stderr),
    startedAt,
    finishedAt: new Date().toISOString(),
    ok: code === 0
  };
}

export function parseArgs(argv) {
  const out = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (!a.startsWith('--')) { out._.push(a); continue; }
    const key = a.slice(2);
    const next = argv[i + 1];
    if (!next || next.startsWith('--')) out[key] = true;
    else { out[key] = next; i++; }
  }
  return out;
}

export function extractUrl(text) {
  const matches = String(text).match(/https?:\/\/[^\s)]+/g) || [];
  return matches[matches.length - 1] || null;
}
