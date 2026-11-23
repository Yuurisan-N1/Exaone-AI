use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rsa::pkcs8::DecodePublicKey;
use rsa::{Oaep, RsaPublicKey};
use sha2::Sha256;

use crate::support::error::{AppError, AppResult};

pub fn seal(pubkey_b64: &str, payload: &serde_json::Value) -> AppResult<String> {
    seal_raw(
        pubkey_b64,
        serde_json::to_string(payload)
            .unwrap_or_default()
            .as_bytes(),
    )
}

pub fn seal_raw(pubkey_b64: &str, payload: &[u8]) -> AppResult<String> {
    let key = parse(pubkey_b64)?;
    let sealed = key
        .encrypt(&mut rand::rngs::OsRng, Oaep::new::<Sha256>(), payload)
        .map_err(|_| AppError::api("the password could not be sealed for the server"))?;
    Ok(STANDARD.encode(sealed))
}

fn parse(pubkey_b64: &str) -> AppResult<RsaPublicKey> {
    let bare: String = pubkey_b64
        .replace("-----BEGIN PUBLIC KEY-----", "")
        .replace("-----END PUBLIC KEY-----", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = STANDARD
        .decode(bare)
        .map_err(|_| AppError::api("the server handed over an unreadable public key"))?;
    RsaPublicKey::from_public_key_der(&der)
        .map_err(|_| AppError::api("the server handed over an unreadable public key"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD as B64;
    use rsa::pkcs8::EncodePublicKey;
    use rsa::traits::PublicKeyParts;
    use rsa::RsaPrivateKey;

    fn sample() -> (RsaPrivateKey, String) {
        let key = RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048).unwrap();
        let pem = key
            .to_public_key()
            .to_public_key_pem(Default::default())
            .unwrap();
        (key, pem)
    }

    #[test]
    fn the_pem_wrapper_is_tolerated() {
        let (key, pem) = sample();
        assert!(parse(&pem).is_ok());
        assert!(parse(
            &pem.replace("-----BEGIN PUBLIC KEY-----", "")
                .replace("-----END PUBLIC KEY-----", "")
        )
        .is_ok());
        assert_eq!(key.to_public_key().size(), 256);
    }

    #[test]
    fn a_junk_key_is_refused_in_words() {
        let err = parse("not-a-key").unwrap_err().to_string();
        assert!(err.contains("unreadable public key"));
    }

    #[test]
    fn the_sealed_blob_is_base64_of_a_full_block() {
        let (_, pem) = sample();
        let sealed = seal(&pem, &serde_json::json!({"email": "a@b.test"})).unwrap();
        let raw = B64.decode(&sealed).unwrap();
        assert_eq!(raw.len(), 256);
        assert_ne!(
            sealed,
            seal(&pem, &serde_json::json!({"email": "a@b.test"})).unwrap()
        );
    }

    #[test]
    fn a_bare_string_seals_without_the_json_quotes() {
        let (_, pem) = sample();
        let text = seal_raw(&pem, b"Secret-123!").unwrap();
        let object = seal(&pem, &serde_json::json!("Secret-123!")).unwrap();
        assert_ne!(text, object);
    }
}
