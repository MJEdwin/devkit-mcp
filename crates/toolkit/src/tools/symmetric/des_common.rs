use block_buffer::BlockSizes;
use des::cipher::{
    BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, BlockSizeUser,
    KeyInit, KeyIvInit, StreamCipher,
    block_padding::{NoPadding, Pkcs7},
    consts::U8,
};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};

/// DES/3DES mode of operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DesMode {
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
}

impl DesMode {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["ecb", "cbc", "ctr", "cfb", "ofb"])
    }

    fn label(self) -> &'static str {
        match self {
            DesMode::Ecb => "ecb",
            DesMode::Cbc => "cbc",
            DesMode::Ctr => "ctr",
            DesMode::Cfb => "cfb",
            DesMode::Ofb => "ofb",
        }
    }
}

/// Padding for DES/3DES block modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DesPadding {
    #[serde(rename = "pkcs7")]
    Pkcs7,
    #[serde(rename = "none")]
    NoPadding,
}

impl DesPadding {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["pkcs7", "none"])
    }
}

macro_rules! dispatch_des_key {
    ($key:expr, $func:ident, $($arg:expr),* $(,)?) => {{
        match $key.len() {
            8 => $func::<des::Des>($($arg),*),
            16 => $func::<des::TdesEde2>($($arg),*),
            24 => $func::<des::TdesEde3>($($arg),*),
            other => Err(CallToolError::from_message(format!(
                "DES key must be 8 (DES), 16 (2-key 3DES), or 24 (3-key 3DES) bytes, got {other}"
            ))),
        }
    }};
}

pub fn encrypt(
    data: &[u8],
    key: &[u8],
    mode: DesMode,
    padding: DesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, CallToolError> {
    let iv = validate(mode, padding, key, iv)?;

    match mode {
        DesMode::Ecb => dispatch_des_key!(key, ecb_encrypt, key, data, padding),
        DesMode::Cbc => dispatch_des_key!(key, cbc_encrypt, key, iv, data, padding),
        DesMode::Ctr => dispatch_des_key!(key, ctr_crypt, key, iv, data),
        DesMode::Cfb => dispatch_des_key!(key, cfb_encrypt, key, iv, data),
        DesMode::Ofb => dispatch_des_key!(key, ofb_crypt, key, iv, data),
    }
}

pub fn decrypt(
    data: &[u8],
    key: &[u8],
    mode: DesMode,
    padding: DesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, CallToolError> {
    let iv = validate(mode, padding, key, iv)?;

    match mode {
        DesMode::Ecb => dispatch_des_key!(key, ecb_decrypt, key, data, padding),
        DesMode::Cbc => dispatch_des_key!(key, cbc_decrypt, key, iv, data, padding),
        DesMode::Ctr => dispatch_des_key!(key, ctr_crypt, key, iv, data),
        DesMode::Cfb => dispatch_des_key!(key, cfb_decrypt, key, iv, data),
        DesMode::Ofb => dispatch_des_key!(key, ofb_crypt, key, iv, data),
    }
}

fn validate<'a>(
    mode: DesMode,
    padding: DesPadding,
    key: &[u8],
    iv: Option<&'a [u8]>,
) -> Result<&'a [u8], CallToolError> {
    if !matches!(key.len(), 8 | 16 | 24) {
        return Err(CallToolError::from_message(format!(
            "DES key must be 8, 16, or 24 bytes, got {}",
            key.len()
        )));
    }

    let iv = iv.unwrap_or(&[]);
    match mode {
        DesMode::Ecb => {}
        DesMode::Cbc | DesMode::Ctr | DesMode::Cfb | DesMode::Ofb => {
            if iv.len() != 8 {
                return Err(CallToolError::from_message(format!(
                    "DES-{} requires an 8-byte iv, got {}",
                    mode.label(),
                    iv.len()
                )));
            }
        }
    }

    match mode {
        DesMode::Ecb | DesMode::Cbc => {}
        _ => {
            if padding != DesPadding::NoPadding {
                return Err(CallToolError::from_message(format!(
                    "DES-{} requires padding \"none\"",
                    mode.label()
                )));
            }
        }
    }

    Ok(iv)
}

fn ensure_block_aligned(data: &[u8], padding: DesPadding) -> Result<(), CallToolError> {
    if padding == DesPadding::NoPadding && !data.len().is_multiple_of(8) {
        return Err(CallToolError::from_message(format!(
            "DES with padding \"none\" requires a multiple of 8 bytes, got {}",
            data.len()
        )));
    }
    Ok(())
}

fn ensure_ciphertext_block_aligned(data: &[u8]) -> Result<(), CallToolError> {
    if data.is_empty() || !data.len().is_multiple_of(8) {
        return Err(CallToolError::from_message(format!(
            "DES ciphertext must be a non-empty multiple of 8 bytes, got {}",
            data.len()
        )));
    }
    Ok(())
}

fn ecb_encrypt<C>(key: &[u8], data: &[u8], padding: DesPadding) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    ecb::Encryptor<C>: KeyInit + BlockModeEncrypt,
{
    ensure_block_aligned(data, padding)?;
    let encryptor = ecb::Encryptor::<C>::new_from_slice(key)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key: {err}")))?;

    Ok(match padding {
        DesPadding::Pkcs7 => encryptor.encrypt_padded_vec::<Pkcs7>(data),
        DesPadding::NoPadding => encryptor.encrypt_padded_vec::<NoPadding>(data),
    })
}

fn ecb_decrypt<C>(key: &[u8], data: &[u8], padding: DesPadding) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherDecrypt + KeyInit,
    ecb::Decryptor<C>: KeyInit + BlockModeDecrypt,
{
    ensure_ciphertext_block_aligned(data)?;
    let decryptor = ecb::Decryptor::<C>::new_from_slice(key)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key: {err}")))?;

    let plaintext = match padding {
        DesPadding::Pkcs7 => decryptor.decrypt_padded_vec::<Pkcs7>(data),
        DesPadding::NoPadding => decryptor.decrypt_padded_vec::<NoPadding>(data),
    };

    plaintext.map_err(|_| CallToolError::from_message("invalid DES padding".to_string()))
}

fn cbc_encrypt<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    padding: DesPadding,
) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    cbc::Encryptor<C>: KeyIvInit + BlockModeEncrypt,
{
    ensure_block_aligned(data, padding)?;
    let encryptor = cbc::Encryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;

    Ok(match padding {
        DesPadding::Pkcs7 => encryptor.encrypt_padded_vec::<Pkcs7>(data),
        DesPadding::NoPadding => encryptor.encrypt_padded_vec::<NoPadding>(data),
    })
}

fn cbc_decrypt<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    padding: DesPadding,
) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherDecrypt + KeyInit,
    cbc::Decryptor<C>: KeyIvInit + BlockModeDecrypt,
{
    ensure_ciphertext_block_aligned(data)?;
    let decryptor = cbc::Decryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;

    let plaintext = match padding {
        DesPadding::Pkcs7 => decryptor.decrypt_padded_vec::<Pkcs7>(data),
        DesPadding::NoPadding => decryptor.decrypt_padded_vec::<NoPadding>(data),
    };

    plaintext.map_err(|_| CallToolError::from_message("invalid DES padding".to_string()))
}

fn ctr_crypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit + BlockSizeUser<BlockSize = U8>,
    <C as BlockSizeUser>::BlockSize: BlockSizes,
    ctr::Ctr64BE<C>: KeyIvInit + StreamCipher,
{
    let mut cipher = ctr::Ctr64BE::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;
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
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;
    let mut buffer = vec![0u8; data.len()];
    encryptor
        .encrypt_b2b(data, &mut buffer)
        .map_err(|_| CallToolError::from_message("DES-CFB encryption failed".to_string()))?;
    Ok(buffer)
}

fn cfb_decrypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit,
    cfb_mode::Decryptor<C>: KeyIvInit,
{
    let decryptor = cfb_mode::Decryptor::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;
    let mut buffer = vec![0u8; data.len()];
    decryptor
        .decrypt_b2b(data, &mut buffer)
        .map_err(|_| CallToolError::from_message("DES-CFB decryption failed".to_string()))?;
    Ok(buffer)
}

fn ofb_crypt<C>(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CallToolError>
where
    C: BlockCipherEncrypt + KeyInit + BlockSizeUser<BlockSize = U8>,
    <C as BlockSizeUser>::BlockSize: BlockSizes,
    ofb::Ofb<C>: KeyIvInit + StreamCipher,
{
    let mut cipher = ofb::Ofb::<C>::new_from_slices(key, iv)
        .map_err(|err| CallToolError::from_message(format!("invalid DES key/iv: {err}")))?;
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    Ok(buffer)
}
