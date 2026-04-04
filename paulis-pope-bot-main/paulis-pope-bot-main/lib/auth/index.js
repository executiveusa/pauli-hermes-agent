import { handlers, auth } from './config.js';

// Re-export Auth.js route handlers (GET + POST for [...nextauth])
export const { GET, POST } = handlers;

// Re-export auth for session checking
export { auth };

/**
 * Get the auth state for the main page (server component).
 * Returns both the session and whether setup is needed, in one call.
 * DB import is dynamic so it doesn't get pulled in at module level.
 *
 * @returns {Promise<{ session: object|null, needsSetup: boolean }>}
 */
export async function getPageAuthState() {
  const { getUserCount } = await import('../db/users.js');
  const [session, userCount] = await Promise.all([
    auth(),
    Promise.resolve(getUserCount()),
  ]);

  return {
    session,
    needsSetup: userCount === 0,
  };
}
