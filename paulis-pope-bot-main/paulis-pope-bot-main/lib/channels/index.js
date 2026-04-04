import { TelegramAdapter } from './telegram.js';

let _telegramAdapter = null;

/**
 * Get the Telegram channel adapter (lazy singleton).
 * @param {string} botToken - Telegram bot token
 * @returns {TelegramAdapter}
 */
export function getTelegramAdapter(botToken) {
  if (!_telegramAdapter || _telegramAdapter.botToken !== botToken) {
    _telegramAdapter = new TelegramAdapter(botToken);
  }
  return _telegramAdapter;
}
