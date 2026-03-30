#!/usr/bin/env node
/**
 * Build script — assembles static deploy artifacts for Vercel.
 *
 * Layout:
 *   dist/              → landing page (root)
 *   dist/dashboard/    → Archon X cockpit
 *   dist/repos.json    → repo catalog (optional)
 */

const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const DIST = path.join(ROOT, 'dist');
const LANDING = path.join(ROOT, 'landingpage');
const DASHBOARD = path.join(ROOT, 'web-ui');
const REPOS_JSON = path.join(ROOT, 'workspace', 'github', 'repos.json');

function copyDir(src, dest) {
  if (!fs.existsSync(src)) {
    console.warn(`⚠  Source not found: ${src}`);
    return;
  }
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

// Clean
fs.rmSync(DIST, { recursive: true, force: true });
fs.mkdirSync(DIST, { recursive: true });

// 1. Landing page → dist/
console.log('📦 Copying landing page → dist/');
copyDir(LANDING, DIST);

// 2. Dashboard → dist/dashboard/
console.log('📦 Copying dashboard  → dist/dashboard/');
copyDir(DASHBOARD, path.join(DIST, 'dashboard'));

// 3. repos.json → dist/repos.json (if exists)
if (fs.existsSync(REPOS_JSON)) {
  console.log('📦 Copying repos.json → dist/repos.json');
  fs.copyFileSync(REPOS_JSON, path.join(DIST, 'repos.json'));
}

// Summary
const count = (dir) => {
  let n = 0;
  if (!fs.existsSync(dir)) return 0;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    n += e.isDirectory() ? count(path.join(dir, e.name)) : 1;
  }
  return n;
};

console.log(`\n✅ Build complete — ${count(DIST)} files → dist/`);
console.log('   /            → Landing page');
console.log('   /dashboard   → Archon X cockpit');
console.log('   /repos.json  → Repository catalog');
