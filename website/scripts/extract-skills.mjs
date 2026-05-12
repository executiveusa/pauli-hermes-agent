#!/usr/bin/env node
/**
 * Extract skill metadata from SKILL.md files into src/data/skills.json.
 * Node.js equivalent of extract-skills.py — runs without Python dependency.
 */

import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, existsSync } from 'fs';
import { join, dirname, relative, sep } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const REPO_ROOT = join(__dirname, '..', '..');
const LOCAL_SKILL_DIRS = [
  ['skills', 'built-in'],
  ['optional-skills', 'optional'],
];
const OUTPUT = join(REPO_ROOT, 'website', 'src', 'data', 'skills.json');

const CATEGORY_LABELS = {
  'apple': 'Apple',
  'autonomous-ai-agents': 'AI Agents',
  'blockchain': 'Blockchain',
  'communication': 'Communication',
  'creative': 'Creative',
  'data-science': 'Data Science',
  'devops': 'DevOps',
  'dogfood': 'Dogfood',
  'domain': 'Domain',
  'email': 'Email',
  'feeds': 'Feeds',
  'gaming': 'Gaming',
  'gifs': 'GIFs',
  'github': 'GitHub',
  'health': 'Health',
  'inference-sh': 'Inference',
  'leisure': 'Leisure',
  'mcp': 'MCP',
  'media': 'Media',
  'migration': 'Migration',
  'mlops': 'MLOps',
  'note-taking': 'Note-Taking',
  'productivity': 'Productivity',
  'red-teaming': 'Red Teaming',
  'research': 'Research',
  'security': 'Security',
  'smart-home': 'Smart Home',
  'social-media': 'Social Media',
  'software-development': 'Software Dev',
  'translation': 'Translation',
  'other': 'Other',
};

/** Parse simple YAML frontmatter without an external library. */
function parseFrontmatter(content) {
  if (!content.startsWith('---')) return null;
  const end = content.indexOf('\n---', 3);
  if (end === -1) return null;
  const yamlBlock = content.slice(3, end).trim();
  const result = {};

  // Parse line by line — handles scalars, lists, and one level of nesting
  const lines = yamlBlock.split('\n');
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const m = line.match(/^([a-zA-Z_][a-zA-Z0-9_-]*):\s*(.*)/);
    if (!m) { i++; continue; }
    const key = m[1];
    let val = m[2].trim();

    // Block scalar (> or |)
    if (val === '>' || val === '|') {
      const parts = [];
      i++;
      while (i < lines.length && (lines[i].startsWith('  ') || lines[i].trim() === '')) {
        parts.push(lines[i].trim());
        i++;
      }
      result[key] = parts.join(' ').trim();
      continue;
    }

    // Inline list [a, b, c]
    if (val.startsWith('[') && val.endsWith(']')) {
      result[key] = val.slice(1, -1).split(',').map(s => s.trim()).filter(Boolean);
      i++;
      continue;
    }

    // Multi-line block-style list
    if (val === '') {
      const items = [];
      i++;
      while (i < lines.length && lines[i].match(/^\s+-\s+/)) {
        items.push(lines[i].replace(/^\s+-\s+/, '').trim());
        i++;
      }
      if (items.length > 0) { result[key] = items; continue; }
      // Nested mapping — collect as sub-object
      const sub = {};
      while (i < lines.length && lines[i].match(/^\s+\S/)) {
        const sm = lines[i].match(/^\s+([a-zA-Z_][a-zA-Z0-9_-]*):\s*(.*)/);
        if (sm) sub[sm[1]] = sm[2].trim();
        i++;
      }
      if (Object.keys(sub).length) { result[key] = sub; continue; }
      result[key] = null;
      continue;
    }

    // Quoted string
    if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
      val = val.slice(1, -1);
    }

    result[key] = val;
    i++;
  }

  return result;
}

/** Recursively walk dir, yield paths that contain a SKILL.md */
function* walkSkillDirs(baseDir) {
  if (!existsSync(baseDir)) return;
  const entries = readdirSync(baseDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const dir = join(baseDir, entry.name);
    const skillFile = join(dir, 'SKILL.md');
    if (existsSync(skillFile)) {
      yield dir;
    } else {
      yield* walkSkillDirs(dir);
    }
  }
}

function extractLocalSkills() {
  const skills = [];

  for (const [baseDir, sourceLabel] of LOCAL_SKILL_DIRS) {
    const basePath = join(REPO_ROOT, baseDir);

    for (const skillDir of walkSkillDirs(basePath)) {
      const skillPath = join(skillDir, 'SKILL.md');
      let content;
      try {
        content = readFileSync(skillPath, 'utf8');
      } catch { continue; }

      const fm = parseFrontmatter(content);
      if (!fm || typeof fm !== 'object') continue;

      const rel = relative(basePath, skillDir);
      const category = rel.split(sep)[0] || rel;

      let tags = [];
      const metadata = fm['metadata'];
      if (metadata && typeof metadata === 'object') {
        const hermesMeta = metadata['hermes'];
        if (hermesMeta && typeof hermesMeta === 'object') {
          tags = hermesMeta['tags'] || [];
        }
      }
      if (!tags.length) tags = fm['tags'] || [];
      if (typeof tags === 'string') tags = [tags];

      skills.push({
        name: fm['name'] || category,
        description: String(fm['description'] || '').replace(/\s+/g, ' ').trim(),
        category,
        categoryLabel: CATEGORY_LABELS[category] || category.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
        source: sourceLabel,
        tags: Array.isArray(tags) ? tags : [],
        platforms: fm['platforms'] || [],
        author: fm['author'] || '',
        version: String(fm['version'] || ''),
      });
    }
  }

  return skills;
}

const MIN_CATEGORY_SIZE = 4;

function consolidateSmallCategories(skills) {
  const counts = {};
  for (const s of skills) {
    if (!s.category || s.category === 'uncategorized') {
      s.category = 'other';
      s.categoryLabel = 'Other';
    }
    counts[s.category] = (counts[s.category] || 0) + 1;
  }
  const smallCats = new Set(Object.entries(counts).filter(([, n]) => n < MIN_CATEGORY_SIZE).map(([c]) => c));
  for (const s of skills) {
    if (smallCats.has(s.category)) {
      s.category = 'other';
      s.categoryLabel = 'Other';
    }
  }
  return skills;
}

function main() {
  try {
    console.log(`[extract-skills] Starting skill extraction...`);
    console.log(`[extract-skills] REPO_ROOT: ${REPO_ROOT}`);
    console.log(`[extract-skills] OUTPUT: ${OUTPUT}`);
    console.log(`[extract-skills] Node version: ${process.version}`);

    const local = extractLocalSkills();
    const allSkills = consolidateSmallCategories(local);

    const sourceOrder = { 'built-in': 0, 'optional': 1 };
    allSkills.sort((a, b) => {
      const so = (sourceOrder[a.source] ?? 2) - (sourceOrder[b.source] ?? 2);
      if (so !== 0) return so;
      const ao = a.category === 'other' ? 1 : 0;
      const bo = b.category === 'other' ? 1 : 0;
      if (ao !== bo) return ao - bo;
      if (a.category < b.category) return -1;
      if (a.category > b.category) return 1;
      if (a.name < b.name) return -1;
      if (a.name > b.name) return 1;
      return 0;
    });

    mkdirSync(dirname(OUTPUT), { recursive: true });
    writeFileSync(OUTPUT, JSON.stringify(allSkills, null, 2));

    const builtIn = allSkills.filter(s => s.source === 'built-in').length;
    const optional = allSkills.filter(s => s.source === 'optional').length;
    console.log(`Extracted ${allSkills.length} skills to ${OUTPUT}`);
    console.log(`  ${local.length} local (${builtIn} built-in, ${optional} optional)`);
  } catch (err) {
    // Fallback: create empty skills.json so Docusaurus build doesn't crash
    console.error('[WARN] Failed to extract skills:', err.message);
    console.error('[WARN] Stack:', err.stack);
    try {
      console.log(`[FALLBACK] Attempting to create empty skills.json at ${OUTPUT}`);
      mkdirSync(dirname(OUTPUT), { recursive: true });
      writeFileSync(OUTPUT, JSON.stringify([], null, 2));
      console.log('[FALLBACK] ✓ Created empty skills.json as fallback');
      process.exit(0);
    } catch (err2) {
      console.error('[ERROR] Could not create fallback skills.json:', err2.message);
      console.error('[ERROR] Stack:', err2.stack);
      process.exit(1);
    }
  }
}

main();
