/// security.rs — enterprise-grade secrets management
///
/// - API keys stored in the OS system keyring (macOS Keychain, Windows Credential Vault, Linux Secret Service)
/// - In-memory encryption via AES-256-GCM for short-lived session tokens
/// - Keys are zeroized from memory on drop

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "ai.hermesagent.hermes";
const NONCE_SIZE: usize = 12; // 96-bit nonce for AES-GCM

/// Save an API key to the OS system keyring
///
/// # Security
/// - Key never touches disk as plaintext
/// - Protected by OS credential store (user-space encryption)
pub fn save_secret(key_name: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key_name)
        .context("Failed to create keyring entry")?;
    entry.set_password(value).context("Failed to save secret to keyring")?;
    Ok(())
}

/// Retrieve an API key from the OS system keyring
pub fn get_secret(key_name: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key_name)
        .context("Failed to create keyring entry")?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
    }
}

/// Delete a secret from the OS keyring
pub fn delete_secret(key_name: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key_name)
        .context("Failed to create keyring entry")?;
    entry.delete_credential().context("Failed to delete secret")?;
    Ok(())
}

/// Encrypt a payload in memory using AES-256-GCM
///
/// Returns base64-encoded `nonce || ciphertext`
pub fn encrypt_secret(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut payload = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    Ok(B64.encode(&payload))
}

/// Decrypt a payload encrypted by `encrypt_secret`
pub fn decrypt_secret(encoded: &str, key: &[u8; 32]) -> Result<Zeroizing<String>> {
    let payload = B64.decode(encoded).context("Invalid base64 payload")?;
    anyhow::ensure!(payload.len() > NONCE_SIZE, "Payload too short");

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    let s = String::from_utf8(plaintext).context("Invalid UTF-8 plaintext")?;
    Ok(Zeroizing::new(s))
}

/// Mask a secret for display — shows only the last 4 characters
pub fn mask_secret(value: &str) -> String {
    if value.len() <= 4 {
        return "••••".to_string();
    }
    let visible = &value[value.len() - 4..];
    format!("••••••••{}", visible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let original = "sk-test-secret-value-1234";
        let encrypted = encrypt_secret(original, &key).unwrap();
        let decrypted = decrypt_secret(&encrypted, &key).unwrap();
        assert_eq!(original, decrypted.as_str());
    }

    #[test]
    fn test_mask_secret() {
        assert_eq!(mask_secret("sk-abc-1234"), "••••••••1234");
        assert_eq!(mask_secret("ab"), "••••");
    }
}
