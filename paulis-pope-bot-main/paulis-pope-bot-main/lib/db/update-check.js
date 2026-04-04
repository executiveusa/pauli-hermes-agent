import { randomUUID } from 'crypto';
import { eq, and } from 'drizzle-orm';
import { getDb } from './index.js';
import { settings } from './schema.js';

/**
 * Get the stored available version from the DB.
 * @returns {string|null}
 */
export function getAvailableVersion() {
  const db = getDb();
  const row = db
    .select()
    .from(settings)
    .where(and(eq(settings.type, 'update'), eq(settings.key, 'available_version')))
    .get();

  return row ? row.value : null;
}

/**
 * Set the available version in the DB (delete + insert upsert).
 * @param {string} version
 */
export function setAvailableVersion(version) {
  const db = getDb();
  db.delete(settings)
    .where(and(eq(settings.type, 'update'), eq(settings.key, 'available_version')))
    .run();

  const now = Date.now();
  db.insert(settings).values({
    id: randomUUID(),
    type: 'update',
    key: 'available_version',
    value: version,
    createdAt: now,
    updatedAt: now,
  }).run();
}

/**
 * Clear the available version from the DB.
 */
export function clearAvailableVersion() {
  const db = getDb();
  db.delete(settings)
    .where(and(eq(settings.type, 'update'), eq(settings.key, 'available_version')))
    .run();
}
