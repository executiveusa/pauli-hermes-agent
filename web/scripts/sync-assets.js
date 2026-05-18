import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const webRoot = path.resolve(__dirname, '..');

function syncDir(srcName, destName) {
  const src = path.join(webRoot, srcName);
  const dest = path.join(webRoot, destName);

  // 1. Clean destination
  if (fs.existsSync(dest)) {
    try {
      fs.rmSync(dest, { recursive: true, force: true });
    } catch (e) {
      // Ignore cleanup errors
    }
  }

  // 2. Re-create destination
  fs.mkdirSync(dest, { recursive: true });

  // 3. Copy files recursively if source exists
  if (fs.existsSync(src)) {
    try {
      fs.cpSync(src, dest, { recursive: true });
      console.log(`Successfully synced ${srcName} -> ${destName}`);
    } catch (err) {
      console.warn(`Warning copying ${srcName}:`, err.message);
    }
  } else {
    console.log(`Source directory ${srcName} does not exist. Skipping.`);
  }
}

try {
  syncDir('node_modules/@nous-research/ui/dist/fonts', 'public/fonts');
  syncDir('node_modules/@nous-research/ui/dist/assets', 'public/ds-assets');
  console.log('Asset synchronization complete.');
} catch (error) {
  console.error('Failed to sync assets:', error);
  // Do not crash the build if asset copy fails, just proceed
}
