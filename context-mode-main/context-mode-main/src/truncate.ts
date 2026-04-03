/**
 * truncate — Pure string and output truncation utilities for context-mode.
 *
 * These helpers are used by both the core ContentStore (chunking) and the
 * PolyglotExecutor (smart output truncation). They are extracted here so
 * SessionDB and any other future consumer can import them without pulling
 * in the full store or executor.
 */

// ─────────────────────────────────────────────────────────
// String truncation
// ─────────────────────────────────────────────────────────

/**
 * Truncate a string to at most `maxChars` characters, appending an ellipsis
 * when truncation occurs.
 *
 * @param str     - Input string.
 * @param maxChars - Maximum character count (inclusive). Must be >= 3.
 * @returns The original string if short enough, otherwise a truncated string
 *          ending with "...".
 */
export function truncateString(str: string, maxChars: number): string {
  if (str.length <= maxChars) return str;
  return str.slice(0, Math.max(0, maxChars - 3)) + "...";
}

// ─────────────────────────────────────────────────────────
// Byte-aware smart truncation (head + tail)
// ─────────────────────────────────────────────────────────

/**
 * Smart truncation that keeps the head (60%) and tail (40%) of output,
 * preserving both initial context and final error messages.
 * Snaps to line boundaries and handles UTF-8 safely via `Buffer.byteLength`.
 *
 * Used by PolyglotExecutor to cap stdout/stderr before returning to context.
 *
 * @param raw - Raw output string.
 * @param maxBytes - Soft cap in bytes. Output below this threshold is returned as-is.
 * @returns The original string if within budget, otherwise head + separator + tail.
 */
export function smartTruncate(raw: string, maxBytes: number): string {
  if (Buffer.byteLength(raw) <= maxBytes) return raw;

  const lines = raw.split("\n");

  // Budget: 60% head, 40% tail (errors/results are usually at the end)
  const headBudget = Math.floor(maxBytes * 0.6);
  const tailBudget = maxBytes - headBudget;

  // Collect head lines
  const headLines: string[] = [];
  let headBytes = 0;
  for (const line of lines) {
    const lineBytes = Buffer.byteLength(line) + 1; // +1 for \n
    if (headBytes + lineBytes > headBudget) break;
    headLines.push(line);
    headBytes += lineBytes;
  }

  // Collect tail lines (from end)
  const tailLines: string[] = [];
  let tailBytes = 0;
  for (let i = lines.length - 1; i >= headLines.length; i--) {
    const lineBytes = Buffer.byteLength(lines[i]) + 1;
    if (tailBytes + lineBytes > tailBudget) break;
    tailLines.unshift(lines[i]);
    tailBytes += lineBytes;
  }

  const skippedLines = lines.length - headLines.length - tailLines.length;
  const skippedBytes = Buffer.byteLength(raw) - headBytes - tailBytes;

  const separator =
    `\n\n... [${skippedLines} lines / ${(skippedBytes / 1024).toFixed(1)}KB truncated` +
    ` — showing first ${headLines.length} + last ${tailLines.length} lines] ...\n\n`;

  return headLines.join("\n") + separator + tailLines.join("\n");
}

// ─────────────────────────────────────────────────────────
// JSON truncation
// ─────────────────────────────────────────────────────────

/**
 * Serialize a value to JSON, then truncate the result to `maxBytes` bytes.
 * If truncation occurs, the string is cut at a UTF-8-safe boundary and
 * "... [truncated]" is appended. The result is NOT guaranteed to be valid
 * JSON after truncation — it is suitable only for display/logging.
 *
 * @param value    - Any JSON-serializable value.
 * @param maxBytes - Maximum byte length of the returned string.
 * @param indent   - JSON indentation spaces (default 2). Pass 0 for compact.
 */
export function truncateJSON(
  value: unknown,
  maxBytes: number,
  indent: number = 2,
): string {
  const serialized = JSON.stringify(value, null, indent) ?? "null";
  if (Buffer.byteLength(serialized) <= maxBytes) return serialized;

  // Find the largest character slice that stays within maxBytes once encoded.
  // Buffer.byteLength is O(n) but we only call it once per truncation.
  const marker = "... [truncated]";
  const markerBytes = Buffer.byteLength(marker);
  const budget = maxBytes - markerBytes;

  // Binary-search for the right character count — avoids O(n²) scanning.
  let lo = 0;
  let hi = serialized.length;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (Buffer.byteLength(serialized.slice(0, mid)) <= budget) {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }

  return serialized.slice(0, lo) + marker;
}

// ─────────────────────────────────────────────────────────
// XML / HTML escaping
// ─────────────────────────────────────────────────────────

/**
 * Escape a string for safe embedding in an XML or HTML attribute or text node.
 * Replaces the five XML-reserved characters: `&`, `<`, `>`, `"`, `'`.
 *
 * Used by the resume snapshot template builder to embed user content in
 * `<tool_response>` and `<user_message>` XML tags without breaking the
 * structured prompt format.
 */
export function escapeXML(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

// ─────────────────────────────────────────────────────────
// maxBytes guard
// ─────────────────────────────────────────────────────────

/**
 * Return `str` unchanged if it fits within `maxBytes`, otherwise return a
 * byte-safe slice with an ellipsis appended. Useful for single-value fields
 * (e.g., tool response strings) where head+tail splitting is not needed.
 *
 * @param str      - Input string.
 * @param maxBytes - Hard byte cap.
 */
export function capBytes(str: string, maxBytes: number): string {
  if (Buffer.byteLength(str) <= maxBytes) return str;
  const marker = "...";
  const markerBytes = Buffer.byteLength(marker);
  const budget = maxBytes - markerBytes;

  let lo = 0;
  let hi = str.length;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (Buffer.byteLength(str.slice(0, mid)) <= budget) {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }

  return str.slice(0, lo) + marker;
}
