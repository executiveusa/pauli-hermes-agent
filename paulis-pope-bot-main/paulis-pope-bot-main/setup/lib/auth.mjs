import { writeFileSync, readFileSync, existsSync, mkdirSync } from 'fs';
import { join } from 'path';

const ROOT_DIR = process.cwd();

/**
 * Validate Anthropic API key by making a minimal test call
 */
export async function validateAnthropicKey(key) {
  try {
    const response = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': key,
        'anthropic-version': '2023-06-01',
      },
      body: JSON.stringify({
        model: 'claude-3-haiku-20240307',
        max_tokens: 1,
        messages: [{ role: 'user', content: 'Hi' }],
      }),
    });

    if (response.status === 401) {
      return { valid: false, error: 'Invalid API key' };
    }
    if (response.status === 400) {
      // Bad request but key is valid (e.g., rate limit, model error)
      return { valid: true };
    }
    if (response.ok) {
      return { valid: true };
    }
    return { valid: false, error: `HTTP ${response.status}` };
  } catch (error) {
    return { valid: false, error: error.message };
  }
}

/**
 * Generate .pi/agent/models.json for providers not built into PI.
 * PI resolves the apiKey field as an env var name at runtime ($ENV_VAR).
 */
export function writeModelsJson(providerName, { baseUrl, apiKey, api, models }) {
  const config = {
    providers: {
      [providerName]: {
        baseUrl,
        apiKey,
        api: api || 'openai-completions',
        models: models.map((m) => ({ id: m })),
      },
    },
  };
  const dir = join(ROOT_DIR, '.pi', 'agent');
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'models.json'), JSON.stringify(config, null, 2));
}

/**
 * Update a single variable in an existing .env file
 */
export function updateEnvVariable(key, value) {
  const envPath = join(ROOT_DIR, '.env');
  if (!existsSync(envPath)) {
    throw new Error('.env file not found. Run npm run setup first.');
  }

  let content = readFileSync(envPath, 'utf-8');
  const regex = new RegExp(`^${key}=.*$`, 'm');

  if (regex.test(content)) {
    content = content.replace(regex, `${key}=${value}`);
  } else {
    content = content.trimEnd() + `\n${key}=${value}\n`;
  }

  writeFileSync(envPath, content);
  return envPath;
}
