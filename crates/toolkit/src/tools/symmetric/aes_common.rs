use aes::cipher::{
    BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, BlockSizeUser,
    KeyInit, KeyIvInit, StreamCipher,
    block_padding::{NoPadding, Pkcs7},
    consts::{U12, U16},
};
use aes_gcm::aead::{Aead, array::Array};
use block_buffer::BlockSizes;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};

/// AES mode of operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AesMode {
    #[serde(rename = "ecb")]
    Ecb,
    #[serde(rename = "cbc")]
    Cbc,
    #[serde(rename = "ctr")]
    Ctr,
    #[serde(rename = "cfb")]
    Cfb,
    #[serde(rename = "ofb")]
    Ofb,
    #[serde(rename = "gcm")]
    Gcm,
}

impl AesMode {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["ecb", "cbc", "ctr", "cfb", "ofb", "gcm"])
    }

    fn label(self) -> &'static str {
        match self {
            AesMode::Ecb => "ecb",
            AesMode::Cbc => "cbc",
            AesMode::Ctr => "ctr",
            AesMode::Cfb => "cfb",
            AesMode::Ofb => "ofb",
            AesMode::Gcm => "gcm",
        }
    }
}

/// Padding for block modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AesPadding {
    #[serde(rename = "pkcs7")]
    Pkcs7,
    #[serde(rename = "none")]
    NoPadding,
}

impl AesPadding {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["pkcs7", "none"])
    }
}

macro_rules! dispatch_aes_key {
    ($key:expr, $func:ident, $($arg:expr),* $(,)?) => {{
        match $key.len() {
            16 => $func::<aes::Aes128>($($arg),*),
            24 => $func::<aes::Aes192>($($arg),*),
            32 => $func::<aes::Aes256>($($arg),*),
            other => Err(CallToolError::from_message(format!(
                "AES key must be 16, 24, or 32 bytes, got {other}"
            ))),
        }
    }};
}

pub fn encrypt(
    data: &[u8],
    key: &[u8],
    mode: AesMode,
    padding: AesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, CallToolError> {
    let iv = validate(mode, padding, key, iv)?;

    match mode {
        AesMode::Ecb => dispatch_aes_key!(key, ecb_encrypt, key, data, padding),
        AesMode::Cbc => dispatch_aes_key!(key, cbc_encrypt, key, iv, data, padding),
        AesMode::Ctr => dispatch_aes_key!(key, ctr_crypt, key, iv, data),
        AesMode::Cfb => dispatch_aes_key!(key, cfb_encrypt, key, iv, data),
        AesMode::Ofb => dispatch_aes_key!(key, ofb_crypt, key, iv, data),
        AesMode::Gcm => gcm_encrypt(key, iv, data),
    }
}

pub fn decrypt(
    data: &[u8],
    key: &[u8],
    mode: AesMode,
    padding: AesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, CallToolError> {
    let iv = validate(mode, padding, key, iv)?;

    match mode {
        AesMode::Ecb => dispatch_aes_key!(key, ecb_decrypt, key, data, padding),
        AesMode::Cbc => dispatch_aes_key!(key, cbc_decrypt, key, iv, data, padding),
        AesMode::Ctr => dispatch_aes_key!(key, ctr_crypt, key, iv, data),
        AesMode::Cfb => dispatch_aes_key!(key, cfb_decrypt, key, iv, data),
        AesMode::Ofb => dispatch_aes_key!(key, ofb_crypt, key, iv, data),
        AesMode::Gcm => gcm_decrypt(key, iv, data),
    }
}

fn validate<'a>(
    mode: AesMode,
    padding: AesPadding,
    key: &[u8],
    iv: Option<&'a [u8]>,
) -> Result<&'a [u8], CallToolError> {
    if !matches!(key.len(), 16 | 24 | 32) {
        return Err(CallToolError::from_message(format!(
            "AES key must be 16, 24, or 32 bytes, got {}",
            key.len()
        )));
    }

    let iv = iv.unwrap_or(&[]);
    match mode {
        AesMode::Ecb => {}
        AesMode::Cbc | AesMode::Ctr | AesMode::Cfb | AesMode::Ofb => {
            if iv.len() != 16 {
                return Err(CallToolError::from_message(format!(
                    "AES-{} requires a 16-byte iv, got {}",
                    mode.label(),
                    iv.len()
                )));
            }
        }
        AesMode::Gcm => {
            if iv.len() != 12 {
                return Err(CallToolError::from_message(format!(
                    "AES-GCM requires a 12-byte nonce (iv), got {}",
                    iv.len()
                )));
            }
        }
    }

    match mode {
        AesMode::Ecb | AesMode::Cbc => {}
        _ => {
            if padding != AesPadding::NoPadding {
                return Err(CallToolError::from_message(format!(
                    "AES-{} requires padding \"none\"",
                    mode.label()
                )));
            }
        }
    }

    Ok(iv)
}

fn ensure_block_aligned(data: &[u8], padding: AesPadding) -> Result<(), CallToolError> {
    if padding == AesPadding::NoPadding && !data.len().is_multiple_of(16) {
        return Err(CallToolError::from_message(format!(
            "AES with padding \"none\" requires a multiple of 16 bytes, got {}",
            data.len()
        )));
    }
    Ok(())
}

fn ensure_ciphertext_block_aligned(data: &[u8]) -> Result<(), CallToolError> {
    if data.is_empty() || !data.len().is_multiple_of(16) {
        return Err(CallToolError::from_message(format!(
            "AES ciphertext must be a non-empty multiple of 16 bytes, got {}",
            data.len()
        )));
    }
    Ok(())
}

fn ecb_encrypt<C>(key: &[u8], data: &[u8], padding: AesPadding) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    ecb::Encryptor<C>: KeyInit + BlockModeEncrypt,
{
    ensure_block_aligned(data, padding)?;
    let encryptor = ecb::Encryptor::<C>::new_from_slice(key)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?;

    Ok(match padding {
        AesPadding::Pkcs7 => encryptor.encrypt_padded_vec::<Pkcs7>(data),
        AesPadding::NoPadding => encryptor.encrypt_padded_vec::<NoPadding>(data),
    })
}

fn ecb_decrypt<C>(key: &[u8], data: &[u8], padding: AesPadding) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherDecrypt + KeyInit,
    ecb::Decryptor<C>: KeyInit + BlockModeDecrypt,
{
    ensure_ciphertext_block_aligned(data)?;
    let decryptor = ecb::Decryptor::<C>::new_from_slice(key)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?;

    let plaintext = match padding {
        AesPadding::Pkcs7 => decryptor.decrypt_padded_vec::<Pkcs7>(data),
        AesPadding::NoPadding => decryptor.decrypt_padded_vec::<NoPadding>(data),
    };

    plaintext.map_err(|_| CallToolError::from_message("invalid AES padding".to_string()))
}

fn cbc_encrypt<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    padding: AesPadding,
) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    cbc::Encryptor<C>: KeyIvInit + BlockModeEncrypt,
{
    ensure_block_aligned(data, padding)?;
    let encryptor = cbc::Encryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;

    Ok(match padding {
        AesPadding::Pkcs7 => encryptor.encrypt_padded_vec::<Pkcs7>(data),
        AesPadding::NoPadding => encryptor.encrypt_padded_vec::<NoPadding>(data),
    })
}

fn cbc_decrypt<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    padding: AesPadding,
) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherDecrypt + KeyInit,
    cbc::Decryptor<C>: KeyIvInit + BlockModeDecrypt,
{
    ensure_ciphertext_block_aligned(data)?;
    let decryptor = cbc::Decryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;

    let plaintext = match padding {
        AesPadding::Pkcs7 => decryptor.decrypt_padded_vec::<Pkcs7>(data),
        AesPadding::NoPadding => decryptor.decrypt_padded_vec::<NoPadding>(data),
    };

    plaintext.map_err(|_| CallToolError::from_message("invalid AES padding".to_string()))
}

fn ctr_crypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit + BlockSizeUser<BlockSize = U16>,
    <C as BlockSizeUser>::BlockSize: BlockSizes,
    ctr::Ctr128BE<C>: KeyIvInit + StreamCipher,
{
    let mut cipher = ctr::Ctr128BE::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    Ok(buffer)
}

fn cfb_encrypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    cfb_mode::Encryptor<C>: KeyIvInit,
{
    let encryptor = cfb_mode::Encryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;
    let mut buffer = vec![0u8; data.len()];
    encryptor
        .encrypt_b2b(data, &mut buffer)
        .map_err(|_| CallToolError::from_message("AES-CFB encryption failed".to_string()))?;
    Ok(buffer)
}

fn cfb_decrypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    cfb_mode::Decryptor<C>: KeyIvInit,
{
    let decryptor = cfb_mode::Decryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;
    let mut buffer = vec![0u8; data.len()];
    decryptor
        .decrypt_b2b(data, &mut buffer)
        .map_err(|_| CallToolError::from_message("AES-CFB decryption failed".to_string()))?;
    Ok(buffer)
}

fn ofb_crypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit + BlockSizeUser<BlockSize = U16>,
    <C as BlockSizeUser>::BlockSize: BlockSizes,
    ofb::Ofb<C>: KeyIvInit + StreamCipher,
{
    let mut cipher = ofb::Ofb::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid AES key/iv: {err}")))?;
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    Ok(buffer)
}

type Aes192Gcm = aes_gcm::AesGcm<aes::Aes192, U12>;

fn gcm_nonce(iv: &[u8]) -> Result<Array<u8, U12>, CallToolError> {
    Array::<u8, U12>::try_from(iv)
        .map_err(|_| CallToolError::from_message("AES-GCM nonce must be 12 bytes".to_string()))
}

fn gcm_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError> {
    let nonce = gcm_nonce(iv)?;

    let ciphertext = match key.len() {
        16 => aes_gcm::Aes128Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .encrypt(&nonce, data),
        24 => Aes192Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .encrypt(&nonce, data),
        32 => aes_gcm::Aes256Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .encrypt(&nonce, data),
        other => {
            return Err(CallToolError::from_message(format!(
                "AES key must be 16, 24, or 32 bytes, got {other}"
            )));
        }
    };

    ciphertext.map_err(|_| CallToolError::from_message("AES-GCM encryption failed".to_string()))
}

fn gcm_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError> {
    let nonce = gcm_nonce(iv)?;

    let plaintext = match key.len() {
        16 => aes_gcm::Aes128Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .decrypt(&nonce, data),
        24 => Aes192Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .decrypt(&nonce, data),
        32 => aes_gcm::Aes256Gcm::new_from_slice(key)
            .map_err(|err| CallToolError::from_message(format!("invalid AES key: {err}")))?
            .decrypt(&nonce, data),
        other => {
            return Err(CallToolError::from_message(format!(
                "AES key must be 16, 24, or 32 bytes, got {other}"
            )));
        }
    };

    plaintext.map_err(|_| {
        CallToolError::from_message("AES-GCM authentication failed (wrong key, nonce, or tag)")
    })
}
