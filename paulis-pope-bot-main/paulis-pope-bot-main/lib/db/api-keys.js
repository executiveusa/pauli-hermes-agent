import { randomUUID, randomBytes, createHash, timingSafeEqual } from 'crypto';
import { eq } from 'drizzle-orm';
import { getDb } from './index.js';
import { settings } from './schema.js';

const KEY_PREFIX = 'tpb_';

// In-memory cache: { key_hash, id } or null
let _cache = null;

/**
 * Generate a new API key: tpb_ + 64 hex chars (32 random bytes).
 * @returns {string}
 */
export function generateApiKey() {
  return KEY_PREFIX + randomBytes(32).toString('hex');
}

/**
 * Hash an API key using SHA-256.
 * @param {string} key - Raw API key
 * @returns {string} Hex digest
 */
export function hashApiKey(key) {
  return createHash('sha256').update(key).digest('hex');
}

/**
 * Lazy-load the API key hash into the in-memory cache.
 */
function _ensureCache() {
  if (_cache !== null) return _cache;

  const db = getDb();
  const row = db
    .select()
    .from(settings)
    .where(eq(settings.type, 'api_key'))
    .get();

  if (row) {
    const parsed = JSON.parse(row.value);
    _cache = { keyHash: parsed.key_hash, id: row.id };
  } else {
    _cache = false; // no key exists — distinguish from "not loaded yet"
  }
  return _cache;
}

/**
 * Clear the in-memory cache (call after create/delete).
 */
export function invalidateApiKeyCache() {
  _cache = null;
}

/**
 * Create (or replace) the API key. Deletes any existing key first.
 * @param {string} createdBy - User ID
 * @returns {{ key: string, record: object }}
 */
export function createApiKeyRecord(createdBy) {
  const db = getDb();

  // Delete any existing API key
  db.delete(settings).where(eq(settings.type, 'api_key')).run();

  const key = generateApiKey();
  const keyHash = hashApiKey(key);
  const keyPrefix = key.slice(0, 8); // "tpb_" + first 4 hex chars
  const now = Date.now();

  const record = {
    id: randomUUID(),
    type: 'api_key',
    key: 'api_key',
    value: JSON.stringify({ key_prefix: keyPrefix, key_hash: keyHash, last_used_at: null }),
    createdBy,
    createdAt: now,
    updatedAt: now,
  };

  db.insert(settings).values(record).run();
  invalidateApiKeyCache();

  return {
    key,
    record: {
      id: record.id,
      keyPrefix,
      createdAt: now,
      lastUsedAt: null,
    },
  };
}

/**
 * Get the current API key metadata (no hash).
 * @returns {object|null}
 */
export function getApiKey() {
  const db = getDb();
  const row = db
    .select()
    .from(settings)
    .where(eq(settings.type, 'api_key'))
    .get();

  if (!row) return null;

  const parsed = JSON.parse(row.value);
  return {
    id: row.id,
    keyPrefix: parsed.key_prefix,
    createdAt: row.createdAt,
    lastUsedAt: parsed.last_used_at,
  };
}

/**
 * Delete the API key.
 */
export function deleteApiKey() {
  const db = getDb();
  db.delete(settings).where(eq(settings.type, 'api_key')).run();
  invalidateApiKeyCache();
}

/**
 * Verify a raw API key against the cached hash.
 * @param {string} rawKey - Raw API key from request header
 * @returns {object|null} Record if valid, null otherwise
 */
export function verifyApiKey(rawKey) {
  if (!rawKey || !rawKey.startsWith(KEY_PREFIX)) return null;

  const keyHash = hashApiKey(rawKey);
  const cached = _ensureCache();

  if (!cached) return null;
  const a = Buffer.from(cached.keyHash, 'hex');
  const b = Buffer.from(keyHash, 'hex');
  if (a.length !== b.length || !timingSafeEqual(a, b)) return null;

  // Update last_used_at in background (non-blocking)
  try {
    const db = getDb();
    const now = Date.now();
    const row = db.select().from(settings).where(eq(settings.id, cached.id)).get();
    if (row) {
      const parsed = JSON.parse(row.value);
      parsed.last_used_at = now;
      db.update(settings)
        .set({ value: JSON.stringify(parsed), updatedAt: now })
        .where(eq(settings.id, cached.id))
        .run();
    }
  } catch {
    // Non-fatal: last_used_at is informational
  }

  return cached;
}
