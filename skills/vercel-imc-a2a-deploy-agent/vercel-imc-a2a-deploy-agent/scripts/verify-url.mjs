#!/usr/bin/env node
import { parseArgs, writeJson } from './lib/util.mjs';

const args = parseArgs(process.argv.slice(2));
const url = args._[0] || args.url;
if (!url) {
  console.error('Usage: node scripts/verify-url.mjs <url> [--out file]');
  process.exit(2);
}

const headers = { 'user-agent': 'vercel-imc-a2a-deploy-agent/0.1' };
if (process.env.VERCEL_BYPASS_TOKEN) {
  headers['x-vercel-protection-bypass'] = process.env.VERCEL_BYPASS_TOKEN;
}

let result;
try {
  const res = await fetch(url, { redirect: 'follow', headers });
  const text = await res.text();
  const lower = text.toLowerCase();
  const title = (text.match(/<title[^>]*>([\s\S]*?)<\/title>/i)?.[1] || '').replace(/\s+/g, ' ').trim();
  const h1 = (text.match(/<h1[^>]*>([\s\S]*?)<\/h1>/i)?.[1] || '').replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim();
  const blockers = [];
  const signals = [];
  if (res.status >= 200 && res.status < 400) signals.push('http-ok'); else blockers.push(`http-${res.status}`);
  if (title) signals.push(`title:${title.slice(0, 120)}`);
  if (h1) signals.push(`h1:${h1.slice(0, 120)}`);
  if (text.length > 500) signals.push(`body-length:${text.length}`); else blockers.push('body-too-short');
  if (lower.includes('404: this page could not be found') || lower.includes('not_found') || title === '404: This page could not be found') blockers.push('generic-404');
  if (lower.includes('deployment protection') || lower.includes('authentication required')) blockers.push('auth-or-protection-wall');
  if (lower.includes('application error') || lower.includes('unhandled runtime error')) blockers.push('runtime-error-page');
  if (lower.includes('<main') || lower.includes('id="__next"') || lower.includes('id="root"') || h1) signals.push('app-shell-or-main-detected');
  const visible = blockers.length === 0 && signals.includes('http-ok') && text.length > 500;
  result = { ok: true, visible, url, finalUrl: res.url, status: res.status, title, h1, bodyLength: text.length, signals, blockers, checkedAt: new Date().toISOString() };
} catch (e) {
  result = { ok: false, visible: false, url, error: e.message, blockers: ['fetch-failed'], checkedAt: new Date().toISOString() };
}

if (args.out) await writeJson(args.out, result);
console.log(JSON.stringify(result, null, 2));
process.exit(result.visible ? 0 : 1);
