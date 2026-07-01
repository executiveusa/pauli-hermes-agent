#!/usr/bin/env node
import { parseArgs, writeJson } from './lib/util.mjs';

const args = parseArgs(process.argv.slice(2));
const owner = args['github-owner'] || process.env.DEFAULT_GITHUB_OWNER;
const out = args.out;

async function github(path) {
  if (!process.env.GITHUB_TOKEN) throw new Error('GITHUB_TOKEN is required');
  const res = await fetch(`https://api.github.com${path}`, {
    headers: {
      authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
      accept: 'application/vnd.github+json',
      'user-agent': 'vercel-imc-a2a-deploy-agent/0.1'
    }
  });
  if (!res.ok) throw new Error(`GitHub ${res.status} for ${path}: ${await res.text()}`);
  return res.json();
}

async function vercel(path) {
  if (!process.env.VERCEL_TOKEN) throw new Error('VERCEL_TOKEN is required');
  const url = new URL(`https://api.vercel.com${path}`);
  if (process.env.VERCEL_TEAM_ID) url.searchParams.set('teamId', process.env.VERCEL_TEAM_ID);
  const res = await fetch(url, {
    headers: {
      authorization: `Bearer ${process.env.VERCEL_TOKEN}`,
      'user-agent': 'vercel-imc-a2a-deploy-agent/0.1'
    }
  });
  if (!res.ok) throw new Error(`Vercel ${res.status} for ${path}: ${await res.text()}`);
  return res.json();
}

async function listRepos() {
  if (!owner) return [];
  try { return await github(`/orgs/${owner}/repos?per_page=100&type=all&sort=pushed`); }
  catch (e) { return await github(`/users/${owner}/repos?per_page=100&type=all&sort=pushed`); }
}

function repoKeyFromProject(project) {
  const link = project.link || project.gitRepository || {};
  const org = link.org || link.owner || link.repoOwner || link.account || link.githubOrg;
  const repo = link.repo || link.name || link.repoName || link.githubRepo;
  if (org && repo) return `${org}/${repo}`.toLowerCase();
  if (typeof link === 'string') return link.toLowerCase();
  return null;
}

const startedAt = new Date().toISOString();
let result;
try {
  const githubRepos = await listRepos();
  const projectsPayload = await vercel('/v9/projects?limit=100');
  const vercelProjects = projectsPayload.projects || projectsPayload;
  const repoMap = new Map(githubRepos.map(r => [String(r.full_name).toLowerCase(), r]));
  const matches = [];
  for (const p of vercelProjects) {
    const key = repoKeyFromProject(p);
    matches.push({
      vercelProjectId: p.id,
      vercelProjectName: p.name,
      repoKey: key,
      githubRepoFound: key ? repoMap.has(key) : false,
      framework: p.framework || null,
      latestDeployments: p.latestDeployments || []
    });
  }
  result = { ok: true, startedAt, finishedAt: new Date().toISOString(), githubOwner: owner, githubRepos, vercelProjects, matches };
} catch (e) {
  result = { ok: false, startedAt, finishedAt: new Date().toISOString(), error: e.message };
}

if (out) await writeJson(out, result);
console.log(JSON.stringify(result, null, 2));
process.exit(result.ok ? 0 : 1);
