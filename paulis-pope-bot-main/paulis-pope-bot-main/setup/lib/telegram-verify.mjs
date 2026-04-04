import * as clack from '@clack/prompts';

/**
 * Run the chat ID verification flow
 * @param {string} verificationCode - The code user should send to bot
 * @param {object} [options] - Options
 * @param {boolean} [options.allowSkip=false] - Allow pressing Enter to skip
 * @returns {Promise<string|null>} - The chat ID or null if skipped
 */
export async function runVerificationFlow(verificationCode, { allowSkip = false } = {}) {
  clack.log.warn('Chat ID Verification');
  clack.log.info(
    'To lock the bot to your chat, send the verification code.\n' +
    `  Send this message to your bot: ${verificationCode}\n` +
    '  The bot will reply with your chat ID. Paste it below.'
  );

  const message = allowSkip
    ? 'Paste your chat ID from the bot (or press Enter to skip):'
    : 'Paste your chat ID from the bot:';

  const chatId = await clack.text({
    message,
    defaultValue: allowSkip ? '' : undefined,
    validate: (input) => {
      if (!input) return allowSkip ? undefined : 'Chat ID is required';
      if (!/^-?\d+$/.test(input.trim())) {
        return 'Chat ID should be a number (can be negative for groups)';
      }
    },
  });

  if (clack.isCancel(chatId)) {
    clack.cancel('Setup cancelled.');
    process.exit(0);
  }

  return chatId.trim() || null;
}

/**
 * Wait for server to pick up .env changes and verify it's running
 * @param {string} ngrokUrl - The ngrok URL
 * @returns {Promise<boolean>} - True if verified successfully
 */
export async function verifyRestart(ngrokUrl) {
  const s = clack.spinner();
  s.start('Waiting for server to pick up changes...');
  await new Promise(resolve => setTimeout(resolve, 3000));

  try {
    await fetch(`${ngrokUrl}/api/ping`, {
      method: 'GET',
      signal: AbortSignal.timeout(10000)
    });
  } catch (err) {
    s.stop(`Server not reachable: ${err.message}`);
    return false;
  }

  s.stop('Server is running');
  return true;
}
