#!/usr/bin/env node
/**
 * build-deploy.js — Static asset builder for Vercel deploy
 *
 * Produces:
 *   dist/index.html          (landing page, copied from landingpage/)
 *   dist/dashboard/index.html (dashboard,    copied from web-ui/)
 */

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const DIST = path.join(ROOT, "dist");

function copyDir(src, dest) {
  if (!fs.existsSync(src)) {
    console.error(`❌  Source directory not found: ${src}`);
    process.exit(1);
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

// --- landing page → dist/ ---
const landingDir = path.join(ROOT, "landingpage");
console.log(`📄  Copying landing page from ${landingDir} …`);
copyDir(landingDir, DIST);

// --- dashboard → dist/dashboard/ ---
const webUiDir = path.join(ROOT, "web-ui");
console.log(`📊  Copying dashboard from ${webUiDir} …`);
copyDir(webUiDir, path.join(DIST, "dashboard"));

// --- sanity checks ---
const required = ["dist/index.html", "dist/dashboard/index.html"];
let ok = true;
for (const rel of required) {
  const full = path.join(ROOT, rel);
  if (fs.existsSync(full)) {
    console.log(`✅  ${rel} (${fs.statSync(full).size} bytes)`);
  } else {
    console.error(`❌  Missing: ${rel}`);
    ok = false;
  }
}

if (!ok) process.exit(1);

const size = execSync(`du -sh ${DIST}`).toString().split("\t")[0];
console.log(`\n📦  Build complete — dist/ size: ${size}`);

function execSync(cmd) {
  try {
    return require("child_process").execSync(cmd, { encoding: "utf8" });
  } catch {
    return "";
  }
}
