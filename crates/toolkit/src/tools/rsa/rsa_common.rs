use getrandom::SysRng;
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{BoxedUint, PrivateKey, PublicKey};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};

/// Padding scheme for RSA encryption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RsaEncryptionPadding {
    /// PKCS#1 v1.5 encryption padding (legacy).
    #[serde(rename = "pkcs1v15")]
    Pkcs1v15,
    /// OAEP with SHA-256 (recommended).
    #[serde(rename = "oaep-sha256")]
    OaepSha256,
    /// Raw RSA with no padding. The input must be a full modulus-sized block.
    #[serde(rename = "raw")]
    Raw,
}

impl RsaEncryptionPadding {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["pkcs1v15", "oaep-sha256", "raw"])
    }
}

/// Padding scheme for RSA signatures. Standard schemes hash with SHA-256.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RsaSignaturePadding {
    /// PKCS#1 v1.5 with SHA-256.
    #[serde(rename = "pkcs1v15-sha256")]
    Pkcs1v15Sha256,
    /// PSS with SHA-256 (recommended).
    #[serde(rename = "pss-sha256")]
    PssSha256,
    /// Raw RSA with no padding. The input must be a full modulus-sized block.
    #[serde(rename = "raw")]
    Raw,
}

impl RsaSignaturePadding {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["pkcs1v15-sha256", "pss-sha256", "raw"])
    }
}

pub fn parse_private_key(pem: &str) -> Result<PrivateKey, CallToolError> {
    let pem = pem.trim();

    if let Ok(key) = PrivateKey::from_pkcs8_pem(pem) {
        return Ok(key);
    }

    PrivateKey::from_pkcs1_pem(pem)
        .map_err(|err| CallToolError::from_message(format!("invalid RSA private key PEM: {err}")))
}

pub fn parse_public_key(pem: &str) -> Result<PublicKey, CallToolError> {
    let pem = pem.trim();

    if let Ok(key) = PublicKey::from_public_key_pem(pem) {
        return Ok(key);
    }

    PublicKey::from_pkcs1_pem(pem)
        .map_err(|err| CallToolError::from_message(format!("invalid RSA public key PEM: {err}")))
}

/// Raw RSA public-key operation: `m^e mod n`.
pub fn raw_encrypt(public_key: &PublicKey, data: &[u8]) -> Result<Vec<u8>, CallToolError> {
    let size = public_key.size();
    let message = decode_representative("data", data, public_key.n_bits_precision(), size)?;

    if message >= public_key.n().as_ref() {
        return Err(CallToolError::from_message(
            "raw RSA input must be smaller than the modulus".to_string(),
        ));
    }

    let ciphertext = rsa::hazmat::rsa_encrypt(public_key, &message)
        .map_err(|err| CallToolError::from_message(format!("raw RSA encryption failed: {err}")))?;

    encode_representative(&ciphertext, size)
}

/// Raw RSA private-key operation: `c^d mod n` (RSADP / RSASP1).
pub fn raw_decrypt(private_key: &PrivateKey, data: &[u8]) -> Result<Vec<u8>, CallToolError> {
    raw_private_op(private_key, data, "decryption")
}

/// Raw RSA private-key operation used as RSASP1.
pub fn raw_sign(private_key: &PrivateKey, data: &[u8]) -> Result<Vec<u8>, CallToolError> {
    raw_private_op(private_key, data, "signing")
}

/// Raw RSA verification: compare `s^e mod n` with the supplied block.
pub fn raw_verify(
    public_key: &PublicKey,
    data: &[u8],
    signature: &[u8],
) -> Result<bool, CallToolError> {
    let size = public_key.size();
    let expected = decode_representative("data", data, public_key.n_bits_precision(), size)?;
    let signature =
        decode_representative("signature", signature, public_key.n_bits_precision(), size)?;

    if signature >= public_key.n().as_ref() {
        return Ok(false);
    }

    let recovered = rsa::hazmat::rsa_encrypt(public_key, &signature).map_err(|err| {
        CallToolError::from_message(format!("raw RSA verification failed: {err}"))
    })?;

    Ok(recovered == expected)
}

/// Maps a signature-verification result to `true`/`false`.
pub fn verification_result(result: rsa::Result<()>) -> Result<bool, CallToolError> {
    match result {
        Ok(()) => Ok(true),
        Err(rsa::Error::Verification) => Ok(false),
        Err(err) => Err(CallToolError::from_message(format!(
            "RSA verification failed: {err}"
        ))),
    }
}

fn raw_private_op(
    private_key: &PrivateKey,
    data: &[u8],
    operation: &str,
) -> Result<Vec<u8>, CallToolError> {
    let size = private_key.size();
    let input = decode_representative("data", data, private_key.n_bits_precision(), size)?;

    let mut rng = SysRng;
    let output = rsa::hazmat::rsa_decrypt(Some(&mut rng), private_key, &input)
        .map_err(|err| CallToolError::from_message(format!("raw RSA {operation} failed: {err}")))?;

    encode_representative(&output, size)
}

fn decode_representative(
    field: &str,
    bytes: &[u8],
    bits_precision: u32,
    size: usize,
) -> Result<BoxedUint, CallToolError> {
    if bytes.len() != size {
        return Err(CallToolError::from_message(format!(
            "raw RSA {field} must be exactly {size} bytes (modulus size), got {}",
            bytes.len()
        )));
    }

    BoxedUint::from_be_slice(bytes, bits_precision)
        .map_err(|err| CallToolError::from_message(format!("invalid raw RSA {field}: {err}")))
}

fn encode_representative(value: &BoxedUint, size: usize) -> Result<Vec<u8>, CallToolError> {
    let trimmed = value.to_be_bytes_trimmed_vartime();

    if trimmed.len() > size {
        return Err(CallToolError::from_message(
            "raw RSA result is larger than the modulus size".to_string(),
        ));
    }

    let mut bytes = vec![0u8; size];
    bytes[size - trimmed.len()..].copy_from_slice(&trimmed);
    Ok(bytes)
}
