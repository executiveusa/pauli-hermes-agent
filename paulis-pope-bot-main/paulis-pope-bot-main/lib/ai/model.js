import { ChatAnthropic } from '@langchain/anthropic';

const DEFAULT_MODELS = {
  anthropic: 'claude-sonnet-4-20250514',
  openai: 'gpt-4o',
  google: 'gemini-2.5-pro',
};

/**
 * Create a LangChain chat model based on environment configuration.
 *
 * Config env vars:
 *   LLM_PROVIDER    — "anthropic" (default), "openai", "google"
 *   LLM_MODEL       — Model name override (e.g. "claude-sonnet-4-20250514")
 *   ANTHROPIC_API_KEY — Required for anthropic provider
 *   OPENAI_API_KEY   — Required for openai provider (optional with OPENAI_BASE_URL)
 *   OPENAI_BASE_URL  — Custom OpenAI-compatible base URL (e.g. http://localhost:11434/v1 for Ollama)
 *   GOOGLE_API_KEY   — Required for google provider
 *
 * @param {object} [options]
 * @param {number} [options.maxTokens=4096] - Max tokens for the response
 * @returns {import('@langchain/core/language_models/chat_models').BaseChatModel}
 */
export async function createModel(options = {}) {
  const provider = process.env.LLM_PROVIDER || 'anthropic';
  const modelName = process.env.LLM_MODEL || DEFAULT_MODELS[provider] || DEFAULT_MODELS.anthropic;
  const maxTokens = options.maxTokens || Number(process.env.LLM_MAX_TOKENS) || 4096;

  switch (provider) {
    case 'anthropic': {
      const apiKey = process.env.ANTHROPIC_API_KEY;
      if (!apiKey) {
        throw new Error('ANTHROPIC_API_KEY environment variable is required');
      }
      return new ChatAnthropic({
        modelName,
        maxTokens,
        anthropicApiKey: apiKey,
      });
    }
    case 'custom':
    case 'openai': {
      const { ChatOpenAI } = await import('@langchain/openai');
      const apiKey = provider === 'custom'
        ? (process.env.CUSTOM_API_KEY || 'not-needed')
        : process.env.OPENAI_API_KEY;
      const baseURL = process.env.OPENAI_BASE_URL;
      if (!apiKey && !baseURL) {
        throw new Error('OPENAI_API_KEY environment variable is required (or set OPENAI_BASE_URL for local models)');
      }
      const config = { modelName, maxTokens };
      config.apiKey = apiKey || 'not-needed';
      if (baseURL) {
        config.configuration = { baseURL };
      }
      return new ChatOpenAI(config);
    }
    case 'google': {
      const { ChatGoogleGenerativeAI } = await import('@langchain/google-genai');
      const apiKey = process.env.GOOGLE_API_KEY;
      if (!apiKey) {
        throw new Error('GOOGLE_API_KEY environment variable is required');
      }
      return new ChatGoogleGenerativeAI({
        modelName,
        maxOutputTokens: maxTokens,
        apiKey,
      });
    }
    default:
      throw new Error(`Unknown LLM provider: ${provider}`);
  }
}
