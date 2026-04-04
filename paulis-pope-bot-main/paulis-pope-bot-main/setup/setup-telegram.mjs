#!/usr/bin/env node

import * as clack from '@clack/prompts';

import { checkPrerequisites } from './lib/prerequisites.mjs';
import { setVariables } from './lib/github.mjs';
import { setTelegramWebhook, validateBotToken, generateVerificationCode } from './lib/telegram.mjs';
import { confirm, generateTelegramWebhookSecret } from './lib/prompts.mjs';
import { updateEnvVariable } from './lib/auth.mjs';
import { runVerificationFlow } from './lib/telegram-verify.mjs';
import { loadEnvFile } from './lib/env.mjs';

async function main() {
  clack.intro('Telegram Webhook Setup');
  clack.log.info('Use this to reconfigure the Telegram webhook.');

  // Check prerequisites
  const prereqs = await checkPrerequisites();

  if (!prereqs.git.remoteInfo) {
    clack.log.error('Could not detect GitHub repository from git remote.');
    process.exit(1);
  }

  const { owner, repo } = prereqs.git.remoteInfo;
  clack.log.info(`Repository: ${owner}/${repo}`);

  // Load existing config
  const env = loadEnvFile();

  // Get APP_URL (verify server is up)
  clack.log.warn('Make sure your server is running and publicly accessible.');

  let appUrl = null;

  // Try to read APP_URL from .env first
  const existingAppUrl = env?.APP_URL;
  if (existingAppUrl) {
    clack.log.info(`Found APP_URL in .env: ${existingAppUrl}`);
    const useExisting = await confirm('Use this URL?');
    if (useExisting) {
      appUrl = existingAppUrl;
    }
  }

  while (!appUrl) {
    const url = await clack.text({
      message: 'Enter your APP_URL (https://...):',
      validate: (input) => {
        if (!input) return 'URL is required';
        if (!input.startsWith('https://')) return 'URL must start with https://';
      },
    });
    if (clack.isCancel(url)) {
      clack.cancel('Setup cancelled.');
      process.exit(0);
    }
    appUrl = url.replace(/\/$/, '');
  }

  // Update APP_URL and APP_HOSTNAME in .env
  const appHostname = new URL(appUrl).hostname;
  updateEnvVariable('APP_URL', appUrl);
  updateEnvVariable('APP_HOSTNAME', appHostname);
  clack.log.success('APP_URL saved to .env');

  // Set APP_URL variable on GitHub
  const s = clack.spinner();
  s.start('Updating APP_URL variable...');
  const urlResult = await setVariables(owner, repo, { APP_URL: appUrl });
  if (urlResult.APP_URL.success) {
    s.stop('APP_URL variable updated');
  } else {
    s.stop(`Failed: ${urlResult.APP_URL.error}`);
  }

  // Get Telegram token - try .env first
  let token = env?.TELEGRAM_BOT_TOKEN;
  if (token) {
    clack.log.info('Using Telegram token from .env');
    const validateSpinner = clack.spinner();
    validateSpinner.start('Validating bot token...');
    const validation = await validateBotToken(token);
    if (validation.valid) {
      validateSpinner.stop(`Bot: @${validation.botInfo.username}`);
    } else {
      validateSpinner.stop(`Invalid token in .env: ${validation.error}`);
      token = null;
    }
  }

  if (!token) {
    const inputToken = await clack.password({
      message: 'Telegram bot token:',
      validate: (input) => {
        if (!input) return 'Token is required';
        if (!/^\d+:[A-Za-z0-9_-]+$/.test(input)) {
          return 'Invalid format. Should be like 123456789:ABC-DEF...';
        }
      },
    });
    if (clack.isCancel(inputToken)) {
      clack.cancel('Setup cancelled.');
      process.exit(0);
    }
    token = inputToken;

    const validateSpinner = clack.spinner();
    validateSpinner.start('Validating bot token...');
    const validation = await validateBotToken(token);
    if (!validation.valid) {
      validateSpinner.stop(`Invalid token: ${validation.error}`);
      process.exit(1);
    }
    validateSpinner.stop(`Bot: @${validation.botInfo.username}`);
  }

  // Handle webhook secret
  let webhookSecret = env?.TELEGRAM_WEBHOOK_SECRET;
  if (webhookSecret) {
    clack.log.info('Using existing webhook secret');
  } else {
    webhookSecret = await generateTelegramWebhookSecret();
    updateEnvVariable('TELEGRAM_WEBHOOK_SECRET', webhookSecret);
    clack.log.success('Generated webhook secret');
  }

  // Register Telegram webhook
  const webhookUrl = `${appUrl}/api/telegram/webhook`;
  const tgSpinner = clack.spinner();
  tgSpinner.start('Registering Telegram webhook...');
  const tgResult = await setTelegramWebhook(token, webhookUrl, webhookSecret);
  if (tgResult.ok) {
    tgSpinner.stop('Telegram webhook registered');
  } else {
    tgSpinner.stop(`Failed: ${tgResult.description}`);
  }

  // Handle chat ID verification
  let telegramChatId = env?.TELEGRAM_CHAT_ID;

  if (telegramChatId) {
    clack.log.info(`Using existing chat ID: ${telegramChatId}`);
  } else {
    const verificationCode = generateVerificationCode();
    updateEnvVariable('TELEGRAM_VERIFICATION', verificationCode);

    clack.log.warn('Waiting for server to restart with new verification code...');
    await new Promise(resolve => setTimeout(resolve, 3000));

    const chatId = await runVerificationFlow(verificationCode, { allowSkip: true });

    if (chatId) {
      updateEnvVariable('TELEGRAM_CHAT_ID', chatId);
      clack.log.success(`Chat ID saved: ${chatId}`);
    } else {
      clack.log.warn('Chat ID is required — the bot will not respond without it.');
      clack.log.info('Run npm run setup-telegram again to complete setup.');
    }
  }

  clack.outro(`Webhook URL: ${webhookUrl}`);
}

main().catch((error) => {
  clack.log.error(`Failed: ${error.message}`);
  process.exit(1);
});
