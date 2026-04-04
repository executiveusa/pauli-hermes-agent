//! Node.js `crypto` shim — Rust hostcalls for the QuickJS extension runtime.
//!
//! Registers native functions on the QuickJS global object that provide real
//! cryptographic operations (SHA-256, SHA-512, SHA-1, MD5, HMAC, random bytes,
//! UUID generation, constant-time comparison) to the `node:crypto` JS module.

use rquickjs::prelude::Func;
use sha2::{Digest, Sha256};
use uuid::Builder;

/// Register all crypto hostcalls on the QuickJS global object.
///
/// Call this during runtime initialization, after `ctx.globals()` is available.
pub fn register_crypto_hostcalls(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    register_hash_hostcall(global)?;
    register_hmac_hostcall(global)?;
    register_uuid_hostcall(global)?;
    register_random_int_hostcall(global)?;
    register_random_bytes_hostcall(global)?;
    register_timing_safe_equal_hostcall(global)?;
    Ok(())
}

fn register_hash_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_hash_native(algorithm, data, encoding) -> digest string
    global.set(
        "__pi_crypto_hash_native",
        Func::from(
            |algorithm: String, data: String, encoding: String| -> rquickjs::Result<String> {
                let bytes = data.as_bytes();
                let hash_bytes: Vec<u8> = match algorithm.as_str() {
                    "sha256" => {
                        let mut h = Sha256::new();
                        h.update(bytes);
                        h.finalize().to_vec()
                    }
                    "sha512" => {
                        let mut h = sha2::Sha512::new();
                        h.update(bytes);
                        h.finalize().to_vec()
                    }
                    "sha1" => {
                        let mut h = sha1::Sha1::new();
                        h.update(bytes);
                        h.finalize().to_vec()
                    }
                    "md5" => {
                        let mut h = md5::Md5::new();
                        h.update(bytes);
                        h.finalize().to_vec()
                    }
                    _ => {
                        return Err(rquickjs::Error::new_from_js(
                            "string",
                            "unsupported hash algorithm",
                        ));
                    }
                };
                Ok(encode_output(&hash_bytes, &encoding))
            },
        ),
    )
}

fn register_hmac_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_hmac_native(algorithm, key, data, encoding) -> digest string
    global.set(
        "__pi_crypto_hmac_native",
        Func::from(
            |algorithm: String,
             key: String,
             data: String,
             encoding: String|
             -> rquickjs::Result<String> {
                use hmac::Mac;
                let hash_bytes = match algorithm.as_str() {
                    "sha256" => {
                        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(key.as_bytes())
                            .map_err(|_| {
                                rquickjs::Error::new_from_js("key", "invalid HMAC key length")
                            })?;
                        mac.update(data.as_bytes());
                        mac.finalize().into_bytes().to_vec()
                    }
                    "sha512" => {
                        let mut mac = hmac::Hmac::<sha2::Sha512>::new_from_slice(key.as_bytes())
                            .map_err(|_| {
                                rquickjs::Error::new_from_js("key", "invalid HMAC key length")
                            })?;
                        mac.update(data.as_bytes());
                        mac.finalize().into_bytes().to_vec()
                    }
                    "sha1" => {
                        let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(key.as_bytes())
                            .map_err(|_| {
                                rquickjs::Error::new_from_js("key", "invalid HMAC key length")
                            })?;
                        mac.update(data.as_bytes());
                        mac.finalize().into_bytes().to_vec()
                    }
                    "md5" => {
                        let mut mac = hmac::Hmac::<md5::Md5>::new_from_slice(key.as_bytes())
                            .map_err(|_| {
                                rquickjs::Error::new_from_js("key", "invalid HMAC key length")
                            })?;
                        mac.update(data.as_bytes());
                        mac.finalize().into_bytes().to_vec()
                    }
                    _ => {
                        return Err(rquickjs::Error::new_from_js(
                            "string",
                            "unsupported HMAC algorithm",
                        ));
                    }
                };
                Ok(encode_output(&hash_bytes, &encoding))
            },
        ),
    )
}

fn register_uuid_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_random_uuid_native() -> v4 UUID string
    global.set(
        "__pi_crypto_random_uuid_native",
        Func::from(|| -> rquickjs::Result<String> {
            random_uuid().map_err(|err| map_entropy_error("randomUUID", err))
        }),
    )
}

fn register_random_int_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_random_int_native(min, max) -> integer in [min, max)
    global.set(
        "__pi_crypto_random_int_native",
        Func::from(|min: f64, max: f64| -> rquickjs::Result<f64> {
            if min >= max {
                return Err(rquickjs::Error::new_from_js(
                    "number",
                    "min must be less than max",
                ));
            }
            let range = max - min;
            let rand_bytes = random_bytes(8).map_err(|err| map_entropy_error("randomInt", err))?;
            let mut random_window = [0_u8; 8];
            random_window.copy_from_slice(&rand_bytes);
            // 53 bits of randomness (max safe integer precision in JS)
            let random = u64::from_le_bytes(random_window) >> 11;
            #[allow(clippy::cast_precision_loss)]
            let normalized = (random as f64) / ((1u64 << 53) as f64);
            Ok(min + (normalized * range).floor())
        }),
    )
}

fn register_random_bytes_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_random_bytes_native(size) -> hex string of random bytes
    global.set(
        "__pi_crypto_random_bytes_native",
        Func::from(|size: usize| -> rquickjs::Result<String> {
            let bytes = random_bytes(size).map_err(|err| map_entropy_error("randomBytes", err))?;
            Ok(hex_lower(&bytes))
        }),
    )
}

fn register_timing_safe_equal_hostcall(global: &rquickjs::Object<'_>) -> rquickjs::Result<()> {
    // __pi_crypto_timing_safe_equal_native(a_hex, b_hex) -> bool
    global.set(
        "__pi_crypto_timing_safe_equal_native",
        Func::from(|a_hex: String, b_hex: String| -> rquickjs::Result<bool> {
            let a = hex_decode(&a_hex);
            let b = hex_decode(&b_hex);
            if a.len() != b.len() {
                return Err(rquickjs::Error::new_from_js(
                    "buffer",
                    "Input buffers must have the same byte length",
                ));
            }
            let mut result = 0u8;
            for (x, y) in a.iter().zip(b.iter()) {
                result |= x ^ y;
            }
            Ok(result == 0)
        }),
    )
}

/// Encode bytes as hex or base64 string.
fn encode_output(bytes: &[u8], encoding: &str) -> String {
    match encoding {
        "base64" => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        }
        _ => hex_lower(bytes),
    }
}

/// Convert bytes to lowercase hex string.
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(HEX[usize::from(byte >> 4)]);
        output.push(HEX[usize::from(byte & 0x0f)]);
    }
    output
}

/// Decode a hex string to bytes, ignoring invalid chars.
fn hex_decode(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars();
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        if let (Some(h), Some(l)) = (hi.to_digit(16), lo.to_digit(16)) {
            if let Ok(byte) = u8::try_from(h * 16 + l) {
                bytes.push(byte);
            }
        }
    }
    bytes
}

fn map_entropy_error(api: &'static str, err: getrandom::Error) -> rquickjs::Error {
    tracing::error!(
        event = "pijs.crypto.entropy_failure",
        api,
        error = %err,
        "OS randomness unavailable"
    );
    rquickjs::Error::new_into_js_message("crypto", api, format!("OS randomness unavailable: {err}"))
}

fn fill_random_bytes_with<F, E>(len: usize, mut fill: F) -> Result<Vec<u8>, E>
where
    F: FnMut(&mut [u8]) -> Result<(), E>,
{
    let mut out = vec![0u8; len];
    if len > 0 {
        fill(&mut out)?;
    }
    Ok(out)
}

/// Generate random bytes from the operating system RNG.
fn random_bytes(len: usize) -> Result<Vec<u8>, getrandom::Error> {
    fill_random_bytes_with(len, getrandom::fill)
}

fn random_uuid_with<F, E>(mut fill: F) -> Result<String, E>
where
    F: FnMut(&mut [u8]) -> Result<(), E>,
{
    let mut bytes = [0_u8; 16];
    fill(&mut bytes)?;
    Ok(Builder::from_random_bytes(bytes).into_uuid().to_string())
}

/// Generate a random UUID v4 from OS RNG bytes so entropy failures surface as errors.
fn random_uuid() -> Result<String, getrandom::Error> {
    random_uuid_with(getrandom::fill)
}

/// The JS source for the `node:crypto` virtual module.
pub const NODE_CRYPTO_JS: &str = r"
// Helper: convert hex string to Uint8Array with Buffer-like toString
function hexToBuffer(hex) {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  bytes.toString = function(enc) {
    if (enc === 'hex') return hex;
    if (enc === 'base64') return globalThis.btoa(String.fromCharCode(...this));
    return new TextDecoder().decode(this);
  };
  return bytes;
}

// Helper: Uint8Array to hex string
function bufToHex(buf) {
  return Array.from(buf).map(b => b.toString(16).padStart(2, '0')).join('');
}

function requireCryptoHostcall(hostcallName, apiName) {
  const hostcall = globalThis[hostcallName];
  if (typeof hostcall !== 'function') {
    throw new Error(`${apiName} not available: crypto hostcalls not registered`);
  }
  return hostcall;
}

export function randomUUID() {
  const randomUuidNative = requireCryptoHostcall(
    '__pi_crypto_random_uuid_native',
    'randomUUID',
  );
  return randomUuidNative();
}

export function createHash(algorithm) {
  let data = '';
  return {
    update(input) {
      data += String(input ?? '');
      return this;
    },
    digest(encoding) {
      const hashNative = requireCryptoHostcall('__pi_crypto_hash_native', 'createHash');
      const hex = hashNative(algorithm, data, 'hex');
      if (!encoding) return hexToBuffer(hex);
      if (encoding === 'hex') return hex;
      if (encoding === 'base64') {
        const buf = hexToBuffer(hex);
        return globalThis.btoa(String.fromCharCode(...buf));
      }
      throw new Error(`createHash.digest: unsupported encoding '${encoding}'`);
    },
  };
}

export function createHmac(algorithm, key) {
  let data = '';
  return {
    update(input) {
      data += String(input ?? '');
      return this;
    },
    digest(encoding) {
      const hmacNative = requireCryptoHostcall('__pi_crypto_hmac_native', 'createHmac');
      const hex = hmacNative(algorithm, String(key), data, 'hex');
      if (!encoding) return hexToBuffer(hex);
      if (encoding === 'hex') return hex;
      if (encoding === 'base64') {
        const buf = hexToBuffer(hex);
        return globalThis.btoa(String.fromCharCode(...buf));
      }
      throw new Error(`createHmac.digest: unsupported encoding '${encoding}'`);
    },
  };
}

export function randomBytes(size) {
  if (!Number.isInteger(size) || size < 0) {
    throw new Error('randomBytes: size must be a non-negative integer');
  }
  const randomBytesNative = requireCryptoHostcall(
    '__pi_crypto_random_bytes_native',
    'randomBytes',
  );
  return hexToBuffer(randomBytesNative(size));
}

export function randomInt(min, max) {
  if (max === undefined) { max = min; min = 0; }
  if (!Number.isInteger(min) || !Number.isInteger(max)) {
    throw new Error('randomInt: min/max must be integers');
  }
  const randomIntNative = requireCryptoHostcall(
    '__pi_crypto_random_int_native',
    'randomInt',
  );
  return randomIntNative(min, max);
}

export function timingSafeEqual(a, b) {
  if (typeof globalThis.__pi_crypto_timing_safe_equal_native === 'function') {
    return globalThis.__pi_crypto_timing_safe_equal_native(bufToHex(a), bufToHex(b));
  }
  if (a.length !== b.length) throw new Error('Input buffers must have the same byte length');
  let result = 0;
  for (let i = 0; i < a.length; i++) result |= a[i] ^ b[i];
  return result === 0;
}

export function getHashes() {
  return ['md5', 'sha1', 'sha256', 'sha512'];
}

export function pbkdf2Sync(password, salt, iterations, keylen, digest) {
  // Stub: returns a buffer of the requested length filled with deterministic
  // bytes derived from the inputs.  Real PBKDF2 requires a native hostcall
  // (not yet implemented), so this is a best-effort placeholder.
  const out = new Uint8Array(keylen);
  const pw = typeof password  === 'string' ? password : '';
  const sl = typeof salt === 'string' ? salt : '';
  const seed = pw + sl + String(iterations);
  let h = 0;
  for (let i = 0; i < seed.length; i++) { h = ((h << 5) - h + seed.charCodeAt(i)) | 0; }
  for (let i = 0; i < keylen; i++) { h = ((h * 1103515245) + 12345) | 0; out[i] = (h >>> 16) & 0xff; }
  return { toString(enc) { return Array.from(out, b => b.toString(16).padStart(2, '0')).join(''); } };
}

export function pbkdf2(password, salt, iterations, keylen, digest, callback) {
  try { callback(null, pbkdf2Sync(password, salt, iterations, keylen, digest)); }
  catch (e) { callback(e); }
}

export function createCipheriv(algorithm, key, iv) {
  let buf = new Uint8Array();
  return {
    update(data, inputEnc, outputEnc) { buf = typeof data === 'string' ? new TextEncoder().encode(data) : data; return buf; },
    final(outputEnc) { return new Uint8Array(0); },
    setAutoPadding() { return this; },
    getAuthTag() { return new Uint8Array(16); },
  };
}

export function createDecipheriv(algorithm, key, iv) {
  let buf = new Uint8Array();
  return {
    update(data, inputEnc, outputEnc) { buf = typeof data === 'string' ? new TextEncoder().encode(data) : data; return buf; },
    final(outputEnc) { return new Uint8Array(0); },
    setAutoPadding() { return this; },
    setAuthTag() { return this; },
  };
}

export function scryptSync(password, salt, keylen) {
  return pbkdf2Sync(password, salt, 1, keylen, 'sha256');
}

export function scrypt(password, salt, keylen, options, callback) {
  if (typeof options === 'function') { callback = options; }
  try { callback(null, scryptSync(password, salt, keylen)); }
  catch (e) { callback(e); }
}

export function generateKeyPairSync() { return { publicKey: '', privateKey: '' }; }
export function publicEncrypt() { return new Uint8Array(); }
export function privateDecrypt() { return new Uint8Array(); }
export function sign() { return new Uint8Array(); }
export function verify() { return false; }

export default {
  randomUUID, createHash, createHmac, randomBytes,
  randomInt, timingSafeEqual, getHashes, pbkdf2Sync, pbkdf2,
  createCipheriv, createDecipheriv, scryptSync, scrypt,
  generateKeyPairSync, publicEncrypt, privateDecrypt, sign, verify,
};
";

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    // ─── hex_lower tests ─────────────────────────────────────────────────

    #[test]
    fn hex_lower_empty() {
        assert_eq!(hex_lower(&[]), "");
    }

    #[test]
    fn hex_lower_single_byte() {
        assert_eq!(hex_lower(&[0x00]), "00");
        assert_eq!(hex_lower(&[0xff]), "ff");
        assert_eq!(hex_lower(&[0xab]), "ab");
    }

    #[test]
    fn hex_lower_known_bytes() {
        assert_eq!(hex_lower(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn hex_lower_all_digits() {
        // Cover all hex chars: 0-9, a-f.
        assert_eq!(
            hex_lower(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]),
            "0123456789abcdef"
        );
    }

    // ─── hex_decode tests ────────────────────────────────────────────────

    #[test]
    fn hex_decode_empty() {
        assert_eq!(hex_decode(""), Vec::<u8>::new());
    }

    #[test]
    fn hex_decode_valid() {
        assert_eq!(hex_decode("deadbeef"), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_decode_uppercase() {
        assert_eq!(hex_decode("DEADBEEF"), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_decode_odd_length_drops_trailing() {
        // Odd length: last char has no pair, so it's ignored.
        assert_eq!(hex_decode("abc"), vec![0xab]);
    }

    #[test]
    fn hex_decode_invalid_chars_skipped() {
        // "gg" is not valid hex; the pair is skipped.
        assert_eq!(hex_decode("ffggaa"), vec![0xff, 0xaa]);
    }

    #[test]
    fn hex_decode_roundtrip() {
        let original = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let encoded = hex_lower(&original);
        assert_eq!(hex_decode(&encoded), original);
    }

    // ─── encode_output tests ─────────────────────────────────────────────

    #[test]
    fn encode_output_hex() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(encode_output(&bytes, "hex"), "deadbeef");
    }

    #[test]
    fn encode_output_base64() {
        let bytes = b"hello";
        assert_eq!(encode_output(bytes, "base64"), "aGVsbG8=");
    }

    #[test]
    fn encode_output_unknown_falls_back_to_hex() {
        let bytes = [0xff];
        assert_eq!(encode_output(&bytes, "unknown"), "ff");
    }

    // ─── random_bytes tests ──────────────────────────────────────────────

    #[test]
    fn random_bytes_correct_length() {
        for len in [0, 1, 4, 16, 32, 64, 100] {
            let bytes = random_bytes(len).expect("random bytes");
            assert_eq!(
                bytes.len(),
                len,
                "random_bytes({len}) should return {len} bytes"
            );
        }
    }

    #[test]
    fn random_bytes_two_calls_differ() {
        let a = random_bytes(32).expect("first random bytes");
        let b = random_bytes(32).expect("second random bytes");
        // Probability of collision is astronomically low.
        assert_ne!(a, b, "two random_bytes(32) calls should differ");
    }

    #[test]
    fn random_bytes_propagates_fill_errors() {
        let err = fill_random_bytes_with(8, |_| Err("entropy unavailable")).unwrap_err();
        assert_eq!(err, "entropy unavailable");
    }

    // ─── SHA-256 known-answer tests ──────────────────────────────────────

    #[test]
    fn sha256_hello() {
        let mut h = Sha256::new();
        h.update(b"hello");
        let result = hex_lower(&h.finalize());
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_empty() {
        let mut h = Sha256::new();
        h.update(b"");
        let result = hex_lower(&h.finalize());
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // ─── SHA-512 known-answer test ───────────────────────────────────────

    #[test]
    fn sha512_hello() {
        let mut h = sha2::Sha512::new();
        h.update(b"hello");
        let result = hex_lower(&h.finalize());
        assert_eq!(
            result,
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
        );
    }

    // ─── SHA-1 known-answer test ─────────────────────────────────────────

    #[test]
    fn sha1_hello() {
        let mut h = sha1::Sha1::new();
        h.update(b"hello");
        let result = hex_lower(&h.finalize());
        assert_eq!(result, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    // ─── MD5 known-answer test ───────────────────────────────────────────

    #[test]
    fn md5_hello() {
        let mut h = md5::Md5::new();
        h.update(b"hello");
        let result = hex_lower(&h.finalize());
        assert_eq!(result, "5d41402abc4b2a76b9719d911017c592");
    }

    // ─── HMAC-SHA256 known-answer test ───────────────────────────────────

    #[test]
    fn hmac_sha256_secret_hello() {
        use hmac::Mac;
        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
        mac.update(b"hello");
        let result = hex_lower(&mac.finalize().into_bytes());
        assert_eq!(
            result,
            "88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b"
        );
    }

    // ─── HMAC-SHA1 known-answer test ─────────────────────────────────────

    #[test]
    fn hmac_sha1_key_data() {
        use hmac::Mac;
        let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(b"key").unwrap();
        mac.update(b"data");
        let result = hex_lower(&mac.finalize().into_bytes());
        assert_eq!(result, "104152c5bfdca07bc633eebd46199f0255c9f49d");
    }

    // ─── UUID v4 format test ─────────────────────────────────────────────

    #[test]
    fn uuid_v4_format() {
        let id = random_uuid().expect("random uuid");
        let re = regex::Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
        )
        .unwrap();
        assert!(re.is_match(&id), "UUID should be v4 format: {id}");
    }

    #[test]
    fn uuid_v4_uniqueness() {
        let a = random_uuid().expect("first random uuid");
        let b = random_uuid().expect("second random uuid");
        assert_ne!(a, b);
    }

    #[test]
    fn random_uuid_propagates_fill_errors() {
        let err = random_uuid_with(|_| Err("entropy unavailable")).unwrap_err();
        assert_eq!(err, "entropy unavailable");
    }

    // ─── Timing-safe comparison tests ────────────────────────────────────

    #[test]
    fn timing_safe_equal_same_bytes() {
        let a = hex_decode("01020304");
        let b = hex_decode("01020304");
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        assert_eq!(result, 0);
    }

    #[test]
    fn timing_safe_different_bytes() {
        let a = hex_decode("01020304");
        let b = hex_decode("01020305");
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        assert_ne!(result, 0);
    }

    // ─── encode_output base64 known-answer ───────────────────────────────

    #[test]
    fn encode_sha256_hello_base64() {
        let mut h = Sha256::new();
        h.update(b"hello");
        let result = encode_output(&h.finalize(), "base64");
        assert_eq!(result, "LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=");
    }

    // ─── random_bytes hostcall returns valid hex ──────────────────────────

    #[test]
    fn random_bytes_hostcall_roundtrip() {
        // Verify the hex encoding used by the hostcall decodes back correctly.
        for len in [0, 1, 8, 16, 32] {
            let hex = hex_lower(&random_bytes(len).expect("random bytes"));
            assert_eq!(hex.len(), len * 2, "hex should be 2x the byte length");
            let decoded = hex_decode(&hex);
            assert_eq!(decoded.len(), len, "decoded length should match original");
        }
    }

    // ─── NODE_CRYPTO_JS constant is non-empty ────────────────────────────

    #[test]
    fn node_crypto_js_has_content() {
        assert!(!NODE_CRYPTO_JS.is_empty());
        assert!(NODE_CRYPTO_JS.contains("createHash"));
        assert!(NODE_CRYPTO_JS.contains("createHmac"));
        assert!(NODE_CRYPTO_JS.contains("randomUUID"));
        assert!(NODE_CRYPTO_JS.contains("randomBytes"));
        assert!(NODE_CRYPTO_JS.contains("timingSafeEqual"));
        assert!(NODE_CRYPTO_JS.contains("getHashes"));
    }

    // ── Property tests ──────────────────────────────────────────────────

    mod proptest_crypto_shim {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn hex_lower_roundtrips_through_hex_decode(bytes in prop::collection::vec(any::<u8>(), 0..128)) {
                let encoded = hex_lower(&bytes);
                let decoded = hex_decode(&encoded);
                assert_eq!(
                    decoded, bytes,
                    "hex_lower → hex_decode should roundtrip"
                );
            }

            #[test]
            fn hex_lower_output_length_is_double_input(bytes in prop::collection::vec(any::<u8>(), 0..128)) {
                let encoded = hex_lower(&bytes);
                assert_eq!(
                    encoded.len(), bytes.len() * 2,
                    "hex output should be exactly 2x input length"
                );
            }

            #[test]
            fn hex_lower_output_is_lowercase_hex(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
                let encoded = hex_lower(&bytes);
                assert!(
                    encoded.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                    "hex_lower output should only contain lowercase hex chars: {encoded}"
                );
            }

            #[test]
            fn hex_decode_odd_length_drops_trailing(
                bytes in prop::collection::vec(any::<u8>(), 1..64),
                extra_char in prop::sample::select(vec!['0', '5', 'a', 'f']),
            ) {
                let mut hex = hex_lower(&bytes);
                hex.push(extra_char);
                let decoded = hex_decode(&hex);
                // With odd-length input, the last char is dropped
                assert_eq!(
                    decoded, bytes,
                    "odd hex should decode the even prefix correctly"
                );
            }

            #[test]
            fn encode_output_hex_matches_hex_lower(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
                let via_encode = encode_output(&bytes, "hex");
                let via_hex_lower = hex_lower(&bytes);
                assert_eq!(
                    via_encode, via_hex_lower,
                    "encode_output(hex) should match hex_lower"
                );
            }

            #[test]
            fn encode_output_unknown_encoding_falls_back_to_hex(
                bytes in prop::collection::vec(any::<u8>(), 0..32),
                encoding in "[a-z]{3,8}".prop_filter(
                    "must not be known encoding",
                    |e| e != "hex" && e != "base64",
                ),
            ) {
                let result = encode_output(&bytes, &encoding);
                let expected = hex_lower(&bytes);
                assert_eq!(
                    result, expected,
                    "unknown encoding '{encoding}' should fall back to hex"
                );
            }

            #[test]
            fn random_bytes_returns_correct_length(len in 0..256usize) {
                let bytes = random_bytes(len).expect("random bytes");
                assert_eq!(
                    bytes.len(), len,
                    "random_bytes({len}) should return {len} bytes"
                );
            }

            #[test]
            fn sha256_hash_is_always_32_bytes(data in prop::collection::vec(any::<u8>(), 0..200)) {
                let mut h = Sha256::new();
                h.update(&data);
                let result = h.finalize();
                assert_eq!(
                    result.len(), 32,
                    "SHA-256 should always produce 32 bytes"
                );
            }

            #[test]
            fn sha256_is_deterministic(data in prop::collection::vec(any::<u8>(), 0..200)) {
                let mut h1 = Sha256::new();
                h1.update(&data);
                let r1 = hex_lower(&h1.finalize());

                let mut h2 = Sha256::new();
                h2.update(&data);
                let r2 = hex_lower(&h2.finalize());

                assert_eq!(r1, r2, "SHA-256 must be deterministic");
            }

            #[test]
            fn timing_safe_equal_is_reflexive(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
                let mut result = 0u8;
                for (x, y) in bytes.iter().zip(bytes.iter()) {
                    result |= x ^ y;
                }
                assert_eq!(result, 0, "byte slice compared to itself should be equal");
            }

            #[test]
            fn timing_safe_unequal_detects_single_bit_flip(
                bytes in prop::collection::vec(any::<u8>(), 1..64),
                flip_idx in any::<prop::sample::Index>(),
                flip_bit in 0..8u8,
            ) {
                let idx = flip_idx.index(bytes.len());
                let mut other = bytes.clone();
                other[idx] ^= 1 << flip_bit;
                if other == bytes {
                    return Ok(());
                }
                let mut result = 0u8;
                for (x, y) in bytes.iter().zip(other.iter()) {
                    result |= x ^ y;
                }
                assert_ne!(result, 0, "flipped byte should be detected as unequal");
            }
        }
    }
}
