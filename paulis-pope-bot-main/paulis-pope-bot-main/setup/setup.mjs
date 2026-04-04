#!/usr/bin/env node

import { execSync } from 'child_process';
import path from 'path';
import chalk from 'chalk';
import open from 'open';
import * as clack from '@clack/prompts';

import {
  checkPrerequisites,
  runGhAuth,
} from './lib/prerequisites.mjs';
import {
  promptForPAT,
  promptForProvider,
  promptForModel,
  promptForApiKey,
  promptForOptionalKey,
  promptForCustomProvider,
  promptForBraveKey,
  confirm,
  pressEnter,
  maskSecret,
  keepOrReconfigure,
} from './lib/prompts.mjs';
import { PROVIDERS } from './lib/providers.mjs';
import {
  validatePAT,
  checkPATScopes,
  generateWebhookSecret,
  getPATCreationURL,
} from './lib/github.mjs';
import { writeModelsJson } from './lib/auth.mjs';
import { loadEnvFile } from './lib/env.mjs';
import { syncConfig } from './lib/sync.mjs';

const logo = `
 _____ _          ____                  ____        _
|_   _| |__   ___|  _ \\ ___  _ __   ___| __ )  ___ | |_
  | | | '_ \\ / _ \\ |_) / _ \\| '_ \\ / _ \\  _ \\ / _ \\| __|
  | | | | | |  __/  __/ (_) | |_) |  __/ |_) | (_) | |_
  |_| |_| |_|\\___|_|   \\___/| .__/ \\___|____/ \\___/ \\__|
                            |_|
`;

async function main() {
  console.log(chalk.cyan(logo));
  clack.intro('Interactive Setup Wizard');

  const TOTAL_STEPS = 7;
  let currentStep = 0;

  // Load existing .env (always exists after init — seed .env has AUTH_SECRET etc.)
  const env = loadEnvFile();

  if (env) {
    clack.log.info('Existing .env detected — previously configured values can be skipped.');
  }

  // Flat object collecting all config values for sync
  const collected = {};
  let owner = null;
  let repo = null;

  // ─── Step 1: Prerequisites ───────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] Checking prerequisites`);

  const s = clack.spinner();
  s.start('Checking system requirements...');
  const prereqs = await checkPrerequisites();
  s.stop('Prerequisites checked');

  // Node.js
  if (prereqs.node.ok) {
    clack.log.success(`Node.js ${prereqs.node.version}`);
  } else if (prereqs.node.installed) {
    clack.log.error(`Node.js ${prereqs.node.version} (need >= 18)`);
    clack.cancel('Please upgrade Node.js to version 18 or higher.');
    process.exit(1);
  } else {
    clack.log.error('Node.js not found');
    clack.cancel('Please install Node.js 18+: https://nodejs.org');
    process.exit(1);
  }

  // Package manager
  if (prereqs.packageManager.installed) {
    clack.log.success(`Package manager: ${prereqs.packageManager.name}`);
  } else {
    clack.log.error('No package manager found (need pnpm or npm)');
    process.exit(1);
  }

  // Git
  if (!prereqs.git.installed) {
    clack.log.error('Git not found');
    process.exit(1);
  }
  clack.log.success('Git installed');

  // gh CLI
  if (prereqs.gh.installed) {
    if (prereqs.gh.authenticated) {
      clack.log.success('GitHub CLI authenticated');
    } else {
      clack.log.warn('GitHub CLI installed but not authenticated');
      const shouldAuth = await confirm('Run gh auth login now?');
      if (shouldAuth) {
        try {
          runGhAuth();
          clack.log.success('GitHub CLI authenticated');
        } catch {
          clack.log.error('Failed to authenticate gh CLI');
          process.exit(1);
        }
      } else {
        clack.log.error('GitHub CLI authentication required');
        process.exit(1);
      }
    }
  } else {
    clack.log.error('GitHub CLI (gh) not found');
    clack.log.info('Install with: brew install gh');
    const shouldInstall = await confirm('Try to install gh with homebrew?');
    if (shouldInstall) {
      const installSpinner = clack.spinner();
      installSpinner.start('Installing gh CLI...');
      try {
        execSync('brew install gh', { stdio: 'inherit' });
        installSpinner.stop('gh CLI installed');
        runGhAuth();
      } catch {
        installSpinner.stop('Failed to install gh CLI');
        process.exit(1);
      }
    } else {
      process.exit(1);
    }
  }

  // Initialize git repo if needed
  if (!prereqs.git.initialized) {
    const initSpinner = clack.spinner();
    initSpinner.start('Initializing git repo...');
    execSync('git init', { stdio: 'ignore' });
    initSpinner.stop('Git repo initialized');
  }

  if (prereqs.git.remoteInfo) {
    owner = prereqs.git.remoteInfo.owner;
    repo = prereqs.git.remoteInfo.repo;
    clack.log.success(`Repository: ${owner}/${repo}`);
  } else {
    clack.log.warn('No GitHub remote detected. We\'ll set one up.');

    // Stage and commit
    execSync('git add .', { stdio: 'ignore' });
    try {
      execSync('git diff --cached --quiet', { stdio: 'ignore' });
      clack.log.success('Nothing new to commit');
    } catch {
      const commitSpinner = clack.spinner();
      commitSpinner.start('Creating initial commit...');
      execSync('git commit -m "initial commit [skip ci]"', { stdio: 'ignore' });
      commitSpinner.stop('Created initial commit');
    }

    // Ask for project name
    const dirName = path.basename(process.cwd());
    const projectName = await clack.text({
      message: 'Name your project:',
      initialValue: dirName,
      validate: (input) => {
        if (!input) return 'Name is required';
      },
    });
    if (clack.isCancel(projectName)) {
      clack.cancel('Setup cancelled.');
      process.exit(0);
    }

    clack.log.info('Create a GitHub repo:');
    clack.log.info('  1. Create a new private repository');
    clack.log.info('  2. Do NOT initialize with a README');
    clack.log.info('  3. Copy the HTTPS URL');

    const openGitHub = await confirm('Open GitHub repo creation page in browser?');
    if (openGitHub) {
      await open(`https://github.com/new?name=${encodeURIComponent(projectName)}&visibility=private`);
      clack.log.info('Opened in browser (name and private pre-filled).');
    }

    // Ask for the remote URL and add it
    let remoteAdded = false;
    while (!remoteAdded) {
      const remoteUrl = await clack.text({
        message: 'Paste the HTTPS repository URL:',
        validate: (input) => {
          if (!input) return 'URL is required';
          if (!input.startsWith('https://github.com/')) return 'Must be an HTTPS GitHub URL (https://github.com/...)';
        },
      });
      if (clack.isCancel(remoteUrl)) {
        clack.cancel('Setup cancelled.');
        process.exit(0);
      }

      try {
        const url = remoteUrl.replace(/\/$/, '').replace(/\.git$/, '') + '.git';
        execSync(`git remote add origin ${url}`, { stdio: 'ignore' });
        remoteAdded = true;
      } catch {
        try {
          const url = remoteUrl.replace(/\/$/, '').replace(/\.git$/, '') + '.git';
          execSync(`git remote set-url origin ${url}`, { stdio: 'ignore' });
          remoteAdded = true;
        } catch {
          clack.log.error('Failed to set remote. Try again.');
        }
      }
    }

    const { getGitRemoteInfo } = await import('./lib/prerequisites.mjs');
    const remoteInfo = getGitRemoteInfo();
    if (remoteInfo) {
      owner = remoteInfo.owner;
      repo = remoteInfo.repo;
      clack.log.success(`Repository: ${owner}/${repo}`);
    } else {
      clack.log.error('Could not detect repository from remote.');
      process.exit(1);
    }
  }

  // Add owner/repo to collected
  collected.GH_OWNER = owner;
  collected.GH_REPO = repo;

  // Track whether we need to push after getting the PAT
  let needsPush = false;
  try {
    execSync('git rev-parse --verify origin/main', { stdio: 'ignore' });
  } catch {
    needsPush = true;
  }

  // ngrok check (informational only)
  if (prereqs.ngrok.installed) {
    clack.log.success('ngrok installed');
  } else {
    clack.log.warn('ngrok not installed (needed to expose local server)');
    clack.log.info('Install with: brew install ngrok/ngrok/ngrok');
  }

  // ─── Step 2: GitHub PAT ──────────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] GitHub Personal Access Token`);

  let pat = null;
  if (await keepOrReconfigure('GitHub PAT', env?.GH_TOKEN ? maskSecret(env.GH_TOKEN) : null)) {
    pat = env.GH_TOKEN;
  }

  if (!pat) {
    clack.log.info(
      `Create a fine-grained PAT scoped to ${owner}/${repo} only:\n` +
      `  Repository access: Only select repositories > ${owner}/${repo}\n` +
      '  Actions: Read and write\n' +
      '  Administration: Read and write (required for self-hosted runners)\n' +
      '  Contents: Read and write\n' +
      '  Metadata: Read-only (required, auto-selected)\n' +
      '  Pull requests: Read and write\n' +
      '  Workflows: Read and write'
    );

    const openPATPage = await confirm('Open GitHub PAT creation page in browser?');
    if (openPATPage) {
      await open(getPATCreationURL());
      clack.log.info(`Opened in browser. Scope it to ${owner}/${repo} only.`);
    }

    let patValid = false;
    while (!patValid) {
      pat = await promptForPAT();

      const validateSpinner = clack.spinner();
      validateSpinner.start('Validating PAT...');
      const validation = await validatePAT(pat);

      if (!validation.valid) {
        validateSpinner.stop(`Invalid PAT: ${validation.error}`);
        continue;
      }

      const scopes = await checkPATScopes(pat);
      if (!scopes.hasRepo || !scopes.hasWorkflow) {
        validateSpinner.stop('PAT missing required scopes');
        clack.log.info(`Found scopes: ${scopes.scopes.join(', ') || 'none'}`);
        continue;
      }

      if (scopes.isFineGrained) {
        validateSpinner.stop(`Fine-grained PAT valid for user: ${validation.user}`);
      } else {
        validateSpinner.stop(`PAT valid for user: ${validation.user}`);
      }
      patValid = true;
    }
  }

  collected.GH_TOKEN = pat;

  // Push to GitHub now that we have the PAT
  if (needsPush) {
    const remote = execSync('git remote get-url origin', { encoding: 'utf-8' }).trim();

    let pushed = false;
    while (!pushed) {
      const authedUrl = remote.replace('https://github.com/', `https://x-access-token:${pat}@github.com/`);
      execSync(`git remote set-url origin ${authedUrl}`, { stdio: 'ignore' });

      const pushSpinner = clack.spinner();
      pushSpinner.start('Pushing to GitHub...');
      try {
        execSync('git branch -M main', { stdio: 'ignore' });
        execSync('git push -u origin main 2>&1', { encoding: 'utf-8' });
        pushSpinner.stop('Pushed to GitHub');
        pushed = true;
      } catch (err) {
        pushSpinner.stop('Failed to push');
        const output = (err.stdout || '') + (err.stderr || '');
        if (output) clack.log.error(output.trim());
        execSync(`git remote set-url origin ${remote}`, { stdio: 'ignore' });
        clack.log.info('Your PAT may not have write access to this repository.');
        pat = await promptForPAT();
        collected.GH_TOKEN = pat;
        continue;
      }

      // Reset remote URL back to clean HTTPS (no token embedded)
      execSync(`git remote set-url origin ${remote}`, { stdio: 'ignore' });
    }
  }

  // ─── Step 3: API Keys ────────────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] API Keys`);

  // Step 3a: Agent LLM
  let agentProvider = null;
  let agentModel = null;
  let openaiBaseUrl = null;

  // Build display string for existing LLM config
  let llmDisplay = null;
  if (env?.LLM_PROVIDER && env?.LLM_MODEL) {
    const existingEnvKey = env.LLM_PROVIDER === 'custom'
      ? 'CUSTOM_API_KEY'
      : PROVIDERS[env.LLM_PROVIDER]?.envKey;

    if (existingEnvKey) {
      const existingKey = env[existingEnvKey];
      const providerLabel = env.LLM_PROVIDER === 'custom'
        ? 'Local (OpenAI Compatible API)'
        : (PROVIDERS[env.LLM_PROVIDER]?.label || env.LLM_PROVIDER);
      llmDisplay = existingKey
        ? `${providerLabel} / ${env.LLM_MODEL} (${maskSecret(existingKey)})`
        : `${providerLabel} / ${env.LLM_MODEL}`;
      if ((env.LLM_PROVIDER === 'openai' || env.LLM_PROVIDER === 'custom') && env.OPENAI_BASE_URL) {
        llmDisplay += ` @ ${env.OPENAI_BASE_URL}`;
      }
    }
  }

  if (llmDisplay && await keepOrReconfigure('LLM', llmDisplay)) {
    // Keep existing LLM config
    agentProvider = env.LLM_PROVIDER;
    agentModel = env.LLM_MODEL;
    const existingEnvKey = agentProvider === 'custom'
      ? 'CUSTOM_API_KEY'
      : PROVIDERS[agentProvider].envKey;
    collected.LLM_PROVIDER = agentProvider;
    collected.LLM_MODEL = agentModel;
    collected[existingEnvKey] = env[existingEnvKey] || '';
    if (env.OPENAI_BASE_URL) {
      openaiBaseUrl = env.OPENAI_BASE_URL;
      collected.OPENAI_BASE_URL = openaiBaseUrl;
    }
  } else {
    // Prompt for new LLM config
    clack.log.info('Choose the LLM provider for your agent.');

    agentProvider = await promptForProvider();

    if (agentProvider === 'custom') {
      clack.log.info('If the model runs on this machine, use http://host.docker.internal:<port>/v1');
      clack.log.info('instead of localhost (localhost won\'t work from inside Docker)');
      clack.log.info('Ollama example: http://host.docker.internal:11434/v1');
      const custom = await promptForCustomProvider();
      agentModel = custom.model;
      openaiBaseUrl = custom.baseUrl;
      writeModelsJson('custom', {
        baseUrl: custom.baseUrl,
        apiKey: 'CUSTOM_API_KEY',
        api: 'openai-completions',
        models: [custom.model],
      });
      collected.CUSTOM_API_KEY = custom.apiKey || '';
      collected.OPENAI_BASE_URL = openaiBaseUrl;
      clack.log.success(`Custom provider configured: ${custom.model} @ ${custom.baseUrl}`);
      if (custom.apiKey) {
        clack.log.success(`API key added (${maskSecret(custom.apiKey)})`);
      }
    } else {
      const providerConfig = PROVIDERS[agentProvider];
      agentModel = await promptForModel(agentProvider);
      const agentApiKey = await promptForApiKey(agentProvider);
      collected[providerConfig.envKey] = agentApiKey;

      // Non-builtin providers need models.json
      if (!providerConfig.builtin) {
        writeModelsJson(agentProvider, {
          baseUrl: providerConfig.baseUrl,
          apiKey: providerConfig.envKey,
          api: providerConfig.api,
          models: providerConfig.models.map((m) => m.id),
        });
        clack.log.success(`Generated .pi/agent/models.json for ${providerConfig.name}`);
      }

      clack.log.success(`${providerConfig.name} key added (${maskSecret(agentApiKey)})`);
    }

    collected.LLM_PROVIDER = agentProvider;
    collected.LLM_MODEL = agentModel;

    if (agentProvider === 'custom') {
      collected.RUNS_ON = 'self-hosted';
    }
  }

  // Re-run: reconfigure existing OPENAI_BASE_URL if provider was kept
  if ((agentProvider === 'openai' || agentProvider === 'custom') && env?.OPENAI_BASE_URL && !collected.OPENAI_BASE_URL) {
    if (!await keepOrReconfigure('Custom LLM URL', env.OPENAI_BASE_URL)) {
      clack.log.info('If the model runs on this machine, use http://host.docker.internal:<port>/v1');
      clack.log.info('instead of localhost (localhost won\'t work from inside Docker)');
      clack.log.info('Ollama example: http://host.docker.internal:11434/v1');
      const baseUrl = await clack.text({
        message: 'API base URL:',
        validate: (input) => {
          if (!input) return 'URL is required';
          if (!input.startsWith('http://') && !input.startsWith('https://')) {
            return 'URL must start with http:// or https://';
          }
        },
      });
      if (clack.isCancel(baseUrl)) {
        clack.cancel('Setup cancelled.');
        process.exit(0);
      }
      openaiBaseUrl = baseUrl;
      collected.OPENAI_BASE_URL = openaiBaseUrl;
      clack.log.success(`Custom base URL: ${openaiBaseUrl}`);
    } else {
      openaiBaseUrl = env.OPENAI_BASE_URL;
      collected.OPENAI_BASE_URL = openaiBaseUrl;
    }
  }

  // Step 3b: Voice Messages (OpenAI optional)
  if (collected['OPENAI_API_KEY']) {
    clack.log.success('Your OpenAI key can also power voice messages.');
  } else if (env?.OPENAI_API_KEY && await keepOrReconfigure('OpenAI key for voice', maskSecret(env.OPENAI_API_KEY))) {
    collected['OPENAI_API_KEY'] = env.OPENAI_API_KEY;
  } else if (!collected['OPENAI_API_KEY']) {
    const result = await promptForOptionalKey('openai', 'voice messages');
    if (result) {
      collected['OPENAI_API_KEY'] = result;
      clack.log.success(`OpenAI key added (${maskSecret(result)})`);
    }
  }

  // Step 3c: Brave Search (optional — not in .env, always ask)
  const braveKey = await promptForBraveKey();
  if (braveKey) {
    collected.BRAVE_API_KEY = braveKey;
    clack.log.success(`Brave Search key added (${maskSecret(braveKey)})`);
  }

  // ─── Step 4: App URL ─────────────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] App URL`);

  let appUrl = null;

  if (await keepOrReconfigure('APP_URL', env?.APP_URL || null)) {
    appUrl = env.APP_URL;
  }

  if (!appUrl) {
    clack.log.info(
      'Your app needs a public URL for GitHub webhooks and Traefik routing.\n' +
      '  Examples:\n' +
      '    ngrok: https://abc123.ngrok.io\n' +
      '    VPS:   https://mybot.example.com\n' +
      '    PaaS:  https://mybot.vercel.app'
    );

    while (!appUrl) {
      const urlInput = await clack.text({
        message: 'Enter your APP_URL (https://...):',
        validate: (input) => {
          if (!input) return 'URL is required';
          if (!input.startsWith('https://')) return 'URL must start with https://';
        },
      });
      if (clack.isCancel(urlInput)) {
        clack.cancel('Setup cancelled.');
        process.exit(0);
      }
      appUrl = urlInput.replace(/\/$/, '');
    }
  }

  collected.APP_URL = appUrl;
  collected.APP_HOSTNAME = new URL(appUrl).hostname;

  // Generate GH_WEBHOOK_SECRET if missing
  collected.GH_WEBHOOK_SECRET = env?.GH_WEBHOOK_SECRET || generateWebhookSecret();

  // ─── Step 5: Sync Config ─────────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] Sync config`);

  if (!owner || !repo) {
    clack.log.warn('Could not detect repository. Please enter manually.');
    const ownerInput = await clack.text({ message: 'GitHub owner/org:' });
    if (clack.isCancel(ownerInput)) { clack.cancel('Setup cancelled.'); process.exit(0); }
    owner = ownerInput;
    const repoInput = await clack.text({ message: 'Repository name:' });
    if (clack.isCancel(repoInput)) { clack.cancel('Setup cancelled.'); process.exit(0); }
    repo = repoInput;
    collected.GH_OWNER = owner;
    collected.GH_REPO = repo;
  }

  const report = await syncConfig(env, collected, { owner, repo });

  clack.log.info('Your agent includes a web chat interface at your APP_URL.');

  // ─── Step 6: Build & Start Server ────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] Build & Start Server`);

  // Check if server is already running
  let serverAlreadyRunning = false;
  try {
    await fetch('http://localhost:80/api/ping', {
      method: 'GET',
      signal: AbortSignal.timeout(3000),
    });
    serverAlreadyRunning = true;
  } catch {
    // Server not reachable
  }

  if (serverAlreadyRunning) {
    clack.log.success('Server is already running');
    if (await confirm('Rebuild and restart anyway?', false)) {
      clack.log.info('Rebuilding Next.js...');
      try {
        execSync('npm run build', { stdio: 'inherit' });
        clack.log.success('Build complete');
      } catch {
        clack.log.error('Build failed — run npm run build manually');
      }
    }
  } else {
    clack.log.info('Building Next.js...');
    try {
      execSync('npm run build', { stdio: 'inherit' });
      clack.log.success('Build complete');
    } catch {
      clack.log.error('Build failed — run npm run build manually');
    }

    clack.log.info('Start Docker in a new terminal window:\n\n     docker compose up -d');

    let serverReachable = false;
    while (!serverReachable) {
      await pressEnter('Press enter once Docker is running');
      const serverSpinner = clack.spinner();
      serverSpinner.start('Checking server...');
      try {
        await fetch('http://localhost:80/api/ping', {
          method: 'GET',
          signal: AbortSignal.timeout(5000),
        });
        serverSpinner.stop('Server is running');
        serverReachable = true;
      } catch {
        serverSpinner.stop('Could not reach server on localhost:80');
      }
    }
  }

  // ─── Step 7: Summary ─────────────────────────────────────────────────
  clack.log.step(`[${++currentStep}/${TOTAL_STEPS}] Setup Complete!`);

  const providerLabel = agentProvider === 'custom' ? 'Local (OpenAI Compatible API)' : PROVIDERS[agentProvider].label;

  let summary = '';
  summary += `Repository:   ${owner}/${repo}\n`;
  summary += `App URL:      ${appUrl}\n`;
  summary += `Agent LLM:    ${providerLabel} (${agentModel})\n`;
  summary += `GitHub PAT:   ${maskSecret(pat)}`;

  clack.note(summary, 'Configuration');

  if (report.secrets.length > 0) {
    clack.log.info(`GitHub secrets set: ${report.secrets.join(', ')}`);
  }
  if (report.variables.length > 0) {
    clack.log.info(`GitHub variables set: ${report.variables.join(', ')}`);
  }

  clack.outro(
    `Start your server:\n\n` +
    `  docker compose up -d\n\n` +
    `Then chat with your agent at ${appUrl}`
  );
}

main().catch((error) => {
  clack.log.error(`Setup failed: ${error.message}`);
  process.exit(1);
});
