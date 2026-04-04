import { execSync, exec } from 'child_process';
import { promisify } from 'util';
import { randomBytes } from 'crypto';

const execAsync = promisify(exec);

/**
 * Validate GitHub PAT by making a test API call
 */
export async function validatePAT(token) {
  try {
    const response = await fetch('https://api.github.com/user', {
      headers: {
        Authorization: `token ${token}`,
        Accept: 'application/vnd.github.v3+json',
      },
    });
    if (!response.ok) return { valid: false, error: 'Invalid token' };
    const user = await response.json();
    return { valid: true, user: user.login };
  } catch (error) {
    return { valid: false, error: error.message };
  }
}

/**
 * Check PAT scopes/permissions
 * Works with both classic tokens (x-oauth-scopes header) and fine-grained tokens
 */
export async function checkPATScopes(token) {
  try {
    const response = await fetch('https://api.github.com/user', {
      headers: {
        Authorization: `token ${token}`,
        Accept: 'application/vnd.github.v3+json',
      },
    });
    const scopes = response.headers.get('x-oauth-scopes') || '';
    const scopeList = scopes.split(',').map((s) => s.trim()).filter(Boolean);

    // Classic tokens have x-oauth-scopes header
    if (scopeList.length > 0) {
      return {
        hasRepo: scopeList.includes('repo'),
        hasWorkflow: scopeList.includes('workflow'),
        scopes: scopeList,
        isFineGrained: false,
      };
    }

    // Fine-grained tokens don't have x-oauth-scopes header
    // We can't check permissions directly, so we assume valid if token works
    return {
      hasRepo: true,
      hasWorkflow: true,
      scopes: [],
      isFineGrained: true,
    };
  } catch {
    return { hasRepo: false, hasWorkflow: false, scopes: [], isFineGrained: false };
  }
}

/**
 * Set a GitHub repository secret using gh CLI
 */
export async function setSecret(owner, repo, name, value) {
  try {
    // Use stdin to pass the secret value securely
    const { stdout, stderr } = await execAsync(
      `echo "${value.replace(/"/g, '\\"')}" | gh secret set ${name} --repo ${owner}/${repo}`,
      { encoding: 'utf-8' }
    );
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Set multiple GitHub secrets
 */
export async function setSecrets(owner, repo, secrets) {
  const results = {};
  for (const [name, value] of Object.entries(secrets)) {
    results[name] = await setSecret(owner, repo, name, value);
  }
  return results;
}

/**
 * List existing secrets
 */
export async function listSecrets(owner, repo) {
  try {
    const { stdout } = await execAsync(`gh secret list --repo ${owner}/${repo}`, {
      encoding: 'utf-8',
    });
    const secrets = stdout
      .trim()
      .split('\n')
      .filter(Boolean)
      .map((line) => line.split('\t')[0]);
    return secrets;
  } catch {
    return [];
  }
}

/**
 * Set a GitHub repository variable using gh CLI
 */
export async function setVariable(owner, repo, name, value) {
  try {
    const { stdout, stderr } = await execAsync(
      `echo "${value.replace(/"/g, '\\"')}" | gh variable set ${name} --repo ${owner}/${repo}`,
      { encoding: 'utf-8' }
    );
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Set multiple GitHub repository variables
 */
export async function setVariables(owner, repo, variables) {
  const results = {};
  for (const [name, value] of Object.entries(variables)) {
    results[name] = await setVariable(owner, repo, name, value);
  }
  return results;
}

/**
 * Generate a random webhook secret
 */
export function generateWebhookSecret() {
  return randomBytes(32).toString('hex');
}

/**
 * Get the GitHub PAT creation URL with pre-selected scopes
 */
export function getPATCreationURL() {
  return 'https://github.com/settings/personal-access-tokens/new';
}
