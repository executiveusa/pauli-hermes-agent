import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

/**
 * Parse .env file and return object, or null if no .env exists
 */
export function loadEnvFile(dir = process.cwd()) {
  const envPath = join(dir, '.env');
  if (!existsSync(envPath)) {
    return null;
  }
  const content = readFileSync(envPath, 'utf-8');
  const env = {};
  for (const line of content.split('\n')) {
    const match = line.match(/^([^#=]+)=(.*)$/);
    if (match) {
      env[match[1].trim()] = match[2].trim();
    }
  }
  return env;
}
