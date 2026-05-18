declare global {
  interface Window {
    /** Set true by the server only for `hermes dashboard --tui` (or HERMES_DASHBOARD_TUI=1). */
    __HERMES_DASHBOARD_EMBEDDED_CHAT__?: boolean;
    /** @deprecated Older injected name; treated as on when true. */
    __HERMES_DASHBOARD_TUI__?: boolean;
  }
}

/** True only when the dashboard was started with embedded TUI Chat (`hermes dashboard --tui`) or remote connection is active. */
export function isDashboardEmbeddedChatEnabled(): boolean {
  if (typeof window === "undefined") return false;
  if (window.__HERMES_DASHBOARD_EMBEDDED_CHAT__ === true) return true;
  if (window.__HERMES_DASHBOARD_TUI__ === true) return true;
  
  // If remote connection settings exist in localStorage, enable embedded chat
  try {
    if (localStorage.getItem("HERMES_BACKEND_URL") || localStorage.getItem("HERMES_SESSION_TOKEN")) {
      return true;
    }
  } catch (e) {
    // Ignore localStorage failures in sandboxed/SSR environments
  }
  
  return false;
}
