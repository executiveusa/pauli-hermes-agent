'use server';

import { createFirstUser } from '../db/users.js';

/**
 * Create the first admin user (setup action).
 * Uses atomic createFirstUser() to prevent race conditions.
 * No session/token is created — the admin must log in through the normal auth flow.
 *
 * @param {string} email
 * @param {string} password
 * @returns {Promise<{ success?: boolean, error?: string }>}
 */
export async function setupAdmin(email, password) {
  if (!email || !password) {
    return { error: 'Email and password are required.' };
  }
  if (password.length < 8) {
    return { error: 'Password must be at least 8 characters.' };
  }

  const created = createFirstUser(email, password);
  if (!created) {
    return { error: 'Setup already completed.' };
  }

  return { success: true };
}
