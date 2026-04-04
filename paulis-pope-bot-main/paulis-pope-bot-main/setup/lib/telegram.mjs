import { randomBytes } from 'crypto';

/**
 * Generate a verification code for Telegram chat ID capture
 */
export function generateVerificationCode() {
  return 'verify-' + randomBytes(4).toString('hex');
}

/**
 * Register a Telegram webhook
 */
export async function setTelegramWebhook(botToken, webhookUrl, secretToken = null) {
  // Delete first — Telegram ignores secret_token changes if the URL is unchanged
  await deleteTelegramWebhook(botToken);

  const body = { url: webhookUrl };
  if (secretToken) {
    body.secret_token = secretToken;
  }

  const response = await fetch(
    `https://api.telegram.org/bot${botToken}/setWebhook`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }
  );

  const result = await response.json();
  return result;
}

/**
 * Get current webhook info
 */
export async function getTelegramWebhookInfo(botToken) {
  const response = await fetch(`https://api.telegram.org/bot${botToken}/getWebhookInfo`);
  const result = await response.json();
  return result;
}

/**
 * Delete existing webhook
 */
export async function deleteTelegramWebhook(botToken) {
  const response = await fetch(`https://api.telegram.org/bot${botToken}/deleteWebhook`, {
    method: 'POST',
  });
  const result = await response.json();
  return result;
}

/**
 * Validate bot token by calling getMe
 */
export async function validateBotToken(botToken) {
  try {
    const response = await fetch(`https://api.telegram.org/bot${botToken}/getMe`);
    const result = await response.json();
    if (result.ok) {
      return { valid: true, botInfo: result.result };
    }
    return { valid: false, error: result.description };
  } catch (error) {
    return { valid: false, error: error.message };
  }
}

/**
 * Get BotFather URL for creating a new bot
 */
export function getBotFatherURL() {
  return 'https://t.me/BotFather';
}
