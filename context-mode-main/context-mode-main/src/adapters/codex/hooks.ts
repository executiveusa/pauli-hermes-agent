/**
 * adapters/codex/hooks — Codex CLI hook definitions (stub).
 *
 * Codex CLI does NOT support hooks (PRs #2904, #9796 were closed without merge).
 * Only MCP integration is available. This module exports empty/stub constants
 * for interface consistency with other adapters.
 *
 * Config: ~/.codex/config.toml (TOML format, not JSON)
 * MCP: full support via [mcp_servers] in config.toml
 */

// ─────────────────────────────────────────────────────────
// Hook type constants (empty — no hook support)
// ─────────────────────────────────────────────────────────

/**
 * Codex CLI hook types — empty object.
 * Codex CLI has no hook support; only MCP integration is available.
 */
export const HOOK_TYPES = {} as const;

// ─────────────────────────────────────────────────────────
// Routing instructions
// ─────────────────────────────────────────────────────────

/**
 * Path to the routing instructions file appended to the system prompt
 * when Codex CLI initializes the MCP server. This is the only integration
 * point since hooks are not supported.
 */
export const ROUTING_INSTRUCTIONS_PATH = "configs/codex/AGENTS.md";
