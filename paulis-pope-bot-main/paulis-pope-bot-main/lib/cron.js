import cron from 'node-cron';
import fs from 'fs';
import path from 'path';
import { cronsFile, cronDir } from './paths.js';
import { executeAction } from './actions.js';

function getInstalledVersion() {
  const pkgPath = path.join(process.cwd(), 'node_modules', 'thepopebot', 'package.json');
  return JSON.parse(fs.readFileSync(pkgPath, 'utf8')).version;
}

// In-memory flag for available update (read by sidebar, written by cron)
let _updateAvailable = null;

/**
 * Get the in-memory update-available version (or null).
 * @returns {string|null}
 */
function getUpdateAvailable() {
  return _updateAvailable;
}

/**
 * Set the in-memory update-available version.
 * @param {string|null} v
 */
function setUpdateAvailable(v) {
  _updateAvailable = v;
}

/**
 * Compare two semver strings numerically.
 * @param {string} candidate - e.g. "1.2.40"
 * @param {string} baseline  - e.g. "1.2.39"
 * @returns {boolean} true if candidate > baseline
 */
function isVersionNewer(candidate, baseline) {
  // Pre-release candidate is never "newer" for upgrade purposes
  if (candidate.includes('-')) return false;

  const a = candidate.split('.').map(Number);
  const b = baseline.replace(/-.*$/, '').split('.').map(Number);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const av = a[i] || 0;
    const bv = b[i] || 0;
    if (av > bv) return true;
    if (av < bv) return false;
  }
  return false;
}

/**
 * Check npm registry for a newer version of thepopebot.
 */
async function runVersionCheck() {
  try {
    const res = await fetch('https://registry.npmjs.org/thepopebot/latest');
    if (!res.ok) {
      console.warn(`[version check] npm registry returned ${res.status}`);
      return;
    }
    const data = await res.json();
    const latest = data.version;

    const installed = getInstalledVersion();
    if (isVersionNewer(latest, installed)) {
      console.log(`[version check] update available: ${installed} → ${latest}`);
      setUpdateAvailable(latest);
      // Persist to DB
      const { setAvailableVersion } = await import('./db/update-check.js');
      setAvailableVersion(latest);
    } else {
      setUpdateAvailable(null);
      // Clear DB
      const { clearAvailableVersion } = await import('./db/update-check.js');
      clearAvailableVersion();
    }
  } catch (err) {
    console.warn(`[version check] failed: ${err.message}`);
    // Leave existing flag untouched on error
  }
}

/**
 * Start built-in crons (version check). Called from instrumentation.
 */
function startBuiltinCrons() {
  // Schedule hourly
  cron.schedule('0 * * * *', runVersionCheck);
  // Run once immediately
  runVersionCheck();
}

/**
 * Load and schedule crons from CRONS.json
 * @returns {Array} - Array of scheduled cron tasks
 */
function loadCrons() {
  const cronFile = cronsFile;

  console.log('\n--- Cron Jobs ---');

  if (!fs.existsSync(cronFile)) {
    console.log('No CRONS.json found');
    console.log('-----------------\n');
    return [];
  }

  const crons = JSON.parse(fs.readFileSync(cronFile, 'utf8'));
  const tasks = [];

  for (const cronEntry of crons) {
    const { name, schedule, type = 'agent', enabled } = cronEntry;
    if (enabled === false) continue;

    if (!cron.validate(schedule)) {
      console.error(`Invalid schedule for "${name}": ${schedule}`);
      continue;
    }

    const task = cron.schedule(schedule, async () => {
      try {
        const result = await executeAction(cronEntry, { cwd: cronDir });
        console.log(`[CRON] ${name}: ${result || 'ran'}`);
        console.log(`[CRON] ${name}: completed!`);
      } catch (err) {
        console.error(`[CRON] ${name}: error - ${err.message}`);
      }
    });

    tasks.push({ name, schedule, type, task });
  }

  if (tasks.length === 0) {
    console.log('No active cron jobs');
  } else {
    for (const { name, schedule, type } of tasks) {
      console.log(`  ${name}: ${schedule} (${type})`);
    }
  }

  console.log('-----------------\n');

  return tasks;
}

export { loadCrons, startBuiltinCrons, getUpdateAvailable, setUpdateAvailable, getInstalledVersion };
