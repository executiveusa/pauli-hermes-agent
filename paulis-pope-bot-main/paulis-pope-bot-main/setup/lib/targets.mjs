/**
 * Config target mapping — single source of truth for where each config key goes.
 *
 * `env: true` — written to .env
 * `secret: true` — GitHub secret name === env key name
 * `secret: 'NAME'` — GitHub secret uses a different name
 * `variable: true` — GitHub repository variable
 * `default` — default value for firstRunOnly variables
 * `firstRunOnly: true` — only set on GitHub during first-time setup
 *
 * Only keys present in `collected` are synced — absent keys are untouched.
 * Empty string values write to .env but do NOT set as GitHub secrets.
 */
export const CONFIG_TARGETS = {
  GH_TOKEN:              { env: true, secret: 'AGENT_GH_TOKEN' },
  GH_OWNER:              { env: true },
  GH_REPO:               { env: true },

  LLM_PROVIDER:          { env: true, variable: true },
  LLM_MODEL:             { env: true, variable: true },
  ANTHROPIC_API_KEY:     { env: true, secret: 'AGENT_ANTHROPIC_API_KEY' },
  OPENAI_API_KEY:        { env: true, secret: 'AGENT_OPENAI_API_KEY' },
  GOOGLE_API_KEY:        { env: true, secret: 'AGENT_GOOGLE_API_KEY' },
  CUSTOM_API_KEY:        { env: true, secret: 'AGENT_CUSTOM_API_KEY' },
  OPENAI_BASE_URL:       { env: true, variable: true },

  BRAVE_API_KEY:         { secret: 'AGENT_LLM_BRAVE_API_KEY' },

  GH_WEBHOOK_SECRET:     { env: true, secret: true },

  APP_URL:               { env: true, variable: true },
  APP_HOSTNAME:          { env: true },

  AUTO_MERGE:            { variable: true, default: 'true', firstRunOnly: true },
  ALLOWED_PATHS:         { variable: true, default: '/logs', firstRunOnly: true },
  RUNS_ON:               { variable: true },

  TELEGRAM_BOT_TOKEN:    { env: true },
  TELEGRAM_WEBHOOK_SECRET: { env: true },
  TELEGRAM_CHAT_ID:      { env: true },
  TELEGRAM_VERIFICATION: { env: true },
};
