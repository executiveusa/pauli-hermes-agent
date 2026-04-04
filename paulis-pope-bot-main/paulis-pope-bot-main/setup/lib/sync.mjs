import * as clack from '@clack/prompts';
import { updateEnvVariable } from './auth.mjs';
import { setSecret, setVariable } from './github.mjs';
import { CONFIG_TARGETS } from './targets.mjs';

/**
 * Sync collected config values to .env and GitHub.
 *
 * Only keys present in `collected` are considered.
 * Only values that actually changed (vs `env`) are written.
 *
 * @param {object|null} env - Current .env values (null if no .env)
 * @param {object} collected - All collected config values
 * @param {object} options
 * @param {string} options.owner - GitHub owner
 * @param {string} options.repo - GitHub repo
 * @returns {Promise<object>} Sync report
 */
export async function syncConfig(env, collected, { owner, repo }) {
  const envUpdates = [];
  const secretUpdates = [];
  const variableUpdates = [];
  const isFirstRun = !env?.GH_TOKEN;

  for (const [key, value] of Object.entries(collected)) {
    const target = CONFIG_TARGETS[key];
    if (!target) continue;

    const oldValue = env?.[key] ?? '';
    const changed = value !== oldValue;

    // .env — write if changed and target has env: true
    if (target.env && changed) {
      envUpdates.push({ key, value });
    }

    // GitHub secret — only if changed AND value is non-empty
    if (target.secret && changed && value) {
      const secretName = target.secret === true ? key : target.secret;
      secretUpdates.push({ key, secretName, value });
    }

    // GitHub variable — only if changed
    if (target.variable && changed) {
      // Skip firstRunOnly variables unless this is the first setup
      if (target.firstRunOnly && !isFirstRun) continue;
      variableUpdates.push({ key, value });
    }
  }

  // Also handle firstRunOnly defaults that aren't in collected
  if (isFirstRun) {
    for (const [key, target] of Object.entries(CONFIG_TARGETS)) {
      if (target.firstRunOnly && target.default && !(key in collected)) {
        variableUpdates.push({ key, value: target.default });
      }
    }
  }

  const report = { env: [], secrets: [], variables: [], errors: [] };

  if (envUpdates.length === 0 && secretUpdates.length === 0 && variableUpdates.length === 0) {
    clack.log.info('Config unchanged');
    return report;
  }

  const s = clack.spinner();

  // Write .env updates
  if (envUpdates.length > 0) {
    for (const { key, value } of envUpdates) {
      updateEnvVariable(key, value);
      report.env.push(key);
    }
    clack.log.success(`Updated .env (${report.env.join(', ')})`);
  }

  // Set GitHub secrets
  if (secretUpdates.length > 0) {
    s.start('Setting GitHub secrets...');
    let allOk = true;
    for (const { key, secretName, value } of secretUpdates) {
      const result = await setSecret(owner, repo, secretName, value);
      if (result.success) {
        report.secrets.push(secretName);
      } else {
        report.errors.push(`Failed to set secret ${secretName}: ${result.error}`);
        allOk = false;
      }
    }
    if (allOk) {
      s.stop('GitHub secrets set');
    } else {
      s.stop('Some secrets failed');
      for (const err of report.errors) {
        clack.log.error(err);
      }
    }
  }

  // Set GitHub variables
  if (variableUpdates.length > 0) {
    s.start('Setting GitHub variables...');
    let allOk = true;
    for (const { key, value } of variableUpdates) {
      const result = await setVariable(owner, repo, key, value);
      if (result.success) {
        report.variables.push(key);
      } else {
        report.errors.push(`Failed to set variable ${key}: ${result.error}`);
        allOk = false;
      }
    }
    if (allOk) {
      s.stop('GitHub variables set');
    } else {
      s.stop('Some variables failed');
      for (const err of report.errors) {
        clack.log.error(err);
      }
    }
  }

  return report;
}
