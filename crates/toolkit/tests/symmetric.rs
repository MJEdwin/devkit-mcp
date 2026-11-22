mod common;

use common::{bin, from_hex, text, unbase64};
use toolkit::{
    AesDecryptTool, AesEncryptTool, AesGenerateIvTool, AesGenerateKeyTool, AesIvMode, AesKeySize,
    AesMode, AesPadding, DesDecryptTool, DesEncryptTool, DesGenerateIvTool, DesGenerateKeyTool,
    DesIvMode, DesKeySize, DesMode, DesPadding,
};

fn key_of(len: usize) -> Vec<u8> {
    (0..len).map(|value| value as u8).collect()
}

fn iv16() -> Vec<u8> {
    (0..16).map(|value| value as u8).collect()
}

fn nonce12() -> Vec<u8> {
    (0..12).map(|value| value as u8).collect()
}

fn encrypt(
    data: &[u8],
    key: &[u8],
    mode: AesMode,
    padding: AesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let result = AesEncryptTool {
        data: bin(data),
        key: bin(key),
        mode,
        padding,
        iv: iv.map(bin),
    }
    .call_tool();

    match result {
        Ok(result) => Ok(unbase64(text(&result))),
        Err(error) => Err(error.to_string()),
    }
}

fn decrypt(
    data: &[u8],
    key: &[u8],
    mode: AesMode,
    padding: AesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    AesDecryptTool {
        data: bin(data),
        key: bin(key),
        mode,
        padding,
        iv: iv.map(bin),
    }
    .call_tool()
    .map(|result| unbase64(text(&result)))
    .map_err(|error| error.to_string())
}

#[test]
fn aes_generate_key_sizes() {
    for (bits, expected) in [
        (None, 32),
        (Some(AesKeySize::Bits128), 16),
        (Some(AesKeySize::Bits192), 24),
        (Some(AesKeySize::Bits256), 32),
    ] {
        let result = AesGenerateKeyTool { bits }.call_tool().expect("aes key");
        assert_eq!(unbase64(text(&result)).len(), expected);
    }

    let first = AesGenerateKeyTool { bits: None }
        .call_tool()
        .expect("aes key");
    let second = AesGenerateKeyTool { bits: None }
        .call_tool()
        .expect("aes key");
    assert_ne!(text(&first), text(&second));
}

fn des_do_encrypt(
    data: &[u8],
    key: &[u8],
    mode: DesMode,
    padding: DesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    DesEncryptTool {
        data: bin(data),
        key: bin(key),
        mode,
        padding,
        iv: iv.map(bin),
    }
    .call_tool()
    .map(|result| unbase64(text(&result)))
    .map_err(|error| error.to_string())
}

fn des_do_decrypt(
    data: &[u8],
    key: &[u8],
    mode: DesMode,
    padding: DesPadding,
    iv: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    DesDecryptTool {
        data: bin(data),
        key: bin(key),
        mode,
        padding,
        iv: iv.map(bin),
    }
    .call_tool()
    .map(|result| unbase64(text(&result)))
    .map_err(|error| error.to_string())
}

#[test]
fn des_generate_key_sizes() {
    for (key_size, expected) in [
        (None, 24),
        (Some(DesKeySize::Des), 8),
        (Some(DesKeySize::Tdes2Key), 16),
        (Some(DesKeySize::Tdes3Key), 24),
    ] {
        let result = DesGenerateKeyTool { key_size }
            .call_tool()
            .expect("des key");
        assert_eq!(unbase64(text(&result)).len(), expected);
    }

    let first = DesGenerateKeyTool { key_size: None }
        .call_tool()
        .expect("des key");
    let second = DesGenerateKeyTool { key_size: None }
        .call_tool()
        .expect("des key");
    assert_ne!(text(&first), text(&second));
}

#[test]
fn des_generate_iv_sizes_and_uniqueness() {
    for mode in [
        DesIvMode::Cbc,
        DesIvMode::Ctr,
        DesIvMode::Cfb,
        DesIvMode::Ofb,
    ] {
        let first = DesGenerateIvTool { mode }.call_tool().expect("des iv");
        let second = DesGenerateIvTool { mode }.call_tool().expect("des iv");

        assert_eq!(unbase64(text(&first)).len(), 8, "{mode:?}");
        assert_ne!(text(&first), text(&second), "{mode:?}");
    }
}

#[test]
fn generated_des_key_and_iv_work() {
    let generated_key = DesGenerateKeyTool {
        key_size: Some(DesKeySize::Tdes3Key),
    }
    .call_tool()
    .expect("des key");
    let key = unbase64(text(&generated_key));

    let generated_iv = DesGenerateIvTool {
        mode: DesIvMode::Cbc,
    }
    .call_tool()
    .expect("des iv");
    let iv = unbase64(text(&generated_iv));

    let plaintext = b"generated DES key and IV";
    let ciphertext = des_do_encrypt(plaintext, &key, DesMode::Cbc, DesPadding::Pkcs7, Some(&iv))
        .expect("encrypt");
    let decrypted = des_do_decrypt(
        &ciphertext,
        &key,
        DesMode::Cbc,
        DesPadding::Pkcs7,
        Some(&iv),
    )
    .expect("decrypt");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn des_ecb_known_vector() {
    let key = from_hex("133457799bbcdff1");
    let plaintext = from_hex("0123456789abcdef");
    let expected = from_hex("85e813540f0ab405");

    let ciphertext = des_do_encrypt(&plaintext, &key, DesMode::Ecb, DesPadding::NoPadding, None)
        .expect("DES-ECB encrypt");
    assert_eq!(ciphertext, expected);

    let decrypted = des_do_decrypt(&expected, &key, DesMode::Ecb, DesPadding::NoPadding, None)
        .expect("DES-ECB decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn des_roundtrips_all_modes_and_key_lengths() {
    let block_data = b"0123456789abcdef";
    let pkcs7_data = b"hello DES";
    let stream_data = b"hello DES stream!!";
    let iv = [0x24u8; 8];

    for key_len in [8usize, 16, 24] {
        let key: Vec<u8> = (0..key_len).map(|value| value as u8).collect();

        for (mode, padding, data, v) in [
            (DesMode::Ecb, DesPadding::Pkcs7, &pkcs7_data[..], None),
            (DesMode::Ecb, DesPadding::NoPadding, &block_data[..], None),
            (
                DesMode::Cbc,
                DesPadding::Pkcs7,
                &pkcs7_data[..],
                Some(&iv[..]),
            ),
            (
                DesMode::Cbc,
                DesPadding::NoPadding,
                &block_data[..],
                Some(&iv[..]),
            ),
            (
                DesMode::Ctr,
                DesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
            (
                DesMode::Cfb,
                DesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
            (
                DesMode::Ofb,
                DesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
        ] {
            let ciphertext = des_do_encrypt(data, &key, mode, padding, v)
                .unwrap_or_else(|error| panic!("{mode:?}/{key_len}: {error}"));
            let plaintext = des_do_decrypt(&ciphertext, &key, mode, padding, v)
                .unwrap_or_else(|error| panic!("{mode:?}/{key_len}: {error}"));
            assert_eq!(plaintext, data, "{mode:?}/{key_len}");
        }
    }
}

#[test]
fn des_rejects_invalid_parameters() {
    let data = bin(b"0123456789abcdef");
    let key = bin(&[0x11u8; 8]);
    let iv = bin(&[0x22u8; 8]);

    assert!(
        DesEncryptTool {
            data: data.clone(),
            key: bin(&[0x11u8; 7]),
            mode: DesMode::Cbc,
            padding: DesPadding::Pkcs7,
            iv: Some(iv.clone()),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        DesEncryptTool {
            data: data.clone(),
            key: key.clone(),
            mode: DesMode::Cbc,
            padding: DesPadding::Pkcs7,
            iv: None,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        DesEncryptTool {
            data: data.clone(),
            key: key.clone(),
            mode: DesMode::Cbc,
            padding: DesPadding::Pkcs7,
            iv: Some(bin(&[0x22u8; 7])),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        DesEncryptTool {
            data: data.clone(),
            key: key.clone(),
            mode: DesMode::Ctr,
            padding: DesPadding::Pkcs7,
            iv: Some(iv.clone()),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        DesEncryptTool {
            data: bin(b"0123456789abcdef0"),
            key,
            mode: DesMode::Ecb,
            padding: DesPadding::NoPadding,
            iv: None,
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn aes_generate_iv_sizes_and_uniqueness() {
    for (mode, expected) in [
        (AesIvMode::Cbc, 16),
        (AesIvMode::Ctr, 16),
        (AesIvMode::Cfb, 16),
        (AesIvMode::Ofb, 16),
        (AesIvMode::Gcm, 12),
    ] {
        let first = AesGenerateIvTool { mode }.call_tool().expect("aes iv");
        let second = AesGenerateIvTool { mode }.call_tool().expect("aes iv");

        assert_eq!(unbase64(text(&first)).len(), expected, "{mode:?}");
        assert_ne!(text(&first), text(&second), "{mode:?}");
    }
}

#[test]
fn generated_iv_works_with_aes_cbc() {
    let key = key_of(32);
    let generated = AesGenerateIvTool {
        mode: AesIvMode::Cbc,
    }
    .call_tool()
    .expect("aes iv");
    let iv = unbase64(text(&generated));
    let plaintext = b"generated iv";

    let ciphertext =
        encrypt(plaintext, &key, AesMode::Cbc, AesPadding::Pkcs7, Some(&iv)).expect("encrypt");
    let decrypted = decrypt(
        &ciphertext,
        &key,
        AesMode::Cbc,
        AesPadding::Pkcs7,
        Some(&iv),
    )
    .expect("decrypt");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn aes_128_cbc_known_vector() {
    let key = [0x42u8; 16];
    let iv = [0x24u8; 16];
    let plaintext = b"hello world! this is my plaintext.";
    let expected = from_hex(
        "c7fe247ef97b21f07cbdd26cb5d346bfd27867cb00d9486723e159978fb9a5f914cfb228a710de4171e396e7b6cf859e",
    );

    let ciphertext = encrypt(plaintext, &key, AesMode::Cbc, AesPadding::Pkcs7, Some(&iv))
        .expect("AES-128-CBC encrypt");

    assert_eq!(ciphertext, expected);
    assert_eq!(
        decrypt(
            &ciphertext,
            &key,
            AesMode::Cbc,
            AesPadding::Pkcs7,
            Some(&iv)
        )
        .expect("AES-128-CBC decrypt"),
        plaintext
    );
}

#[test]
fn aes_128_gcm_known_vector() {
    let key = from_hex("11754cd72aec309bf52f7687212e8957");
    let nonce = from_hex("3c819d9a9bed087615030b65");
    let tag = from_hex("250327c674aaf477aef2675748cf6971");

    let ciphertext = encrypt(&[], &key, AesMode::Gcm, AesPadding::NoPadding, Some(&nonce))
        .expect("AES-128-GCM encrypt");
    assert_eq!(ciphertext, tag);

    let plaintext = decrypt(
        &tag,
        &key,
        AesMode::Gcm,
        AesPadding::NoPadding,
        Some(&nonce),
    )
    .expect("AES-128-GCM decrypt");
    assert!(plaintext.is_empty());
}

#[test]
fn aes_gcm_rejects_tampered_tag() {
    let key = key_of(32);
    let nonce = nonce12();
    let plaintext = b"authenticated data";

    let mut ciphertext = encrypt(
        plaintext,
        &key,
        AesMode::Gcm,
        AesPadding::NoPadding,
        Some(&nonce),
    )
    .expect("AES-GCM encrypt");
    *ciphertext.last_mut().unwrap() ^= 1;

    assert!(
        decrypt(
            &ciphertext,
            &key,
            AesMode::Gcm,
            AesPadding::NoPadding,
            Some(&nonce)
        )
        .is_err()
    );
}

#[test]
fn aes_roundtrips_all_modes_and_key_sizes() {
    let block_data = b"0123456789abcdef0123456789abcdef";
    let stream_data = b"hello AES stream mode!";
    let iv = iv16();
    let nonce = nonce12();

    for key_len in [16usize, 24, 32] {
        let key = key_of(key_len);

        for (mode, padding, data, v) in [
            (AesMode::Ecb, AesPadding::Pkcs7, &block_data[..], None),
            (AesMode::Ecb, AesPadding::NoPadding, &block_data[..], None),
            (
                AesMode::Cbc,
                AesPadding::Pkcs7,
                &block_data[..],
                Some(&iv[..]),
            ),
            (
                AesMode::Cbc,
                AesPadding::NoPadding,
                &block_data[..],
                Some(&iv[..]),
            ),
            (
                AesMode::Ctr,
                AesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
            (
                AesMode::Cfb,
                AesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
            (
                AesMode::Ofb,
                AesPadding::NoPadding,
                &stream_data[..],
                Some(&iv[..]),
            ),
            (
                AesMode::Gcm,
                AesPadding::NoPadding,
                &stream_data[..],
                Some(&nonce[..]),
            ),
        ] {
            let ciphertext = encrypt(data, &key, mode, padding, v)
                .unwrap_or_else(|error| panic!("{mode:?}/{key_len}: {error}"));
            let plaintext = decrypt(&ciphertext, &key, mode, padding, v)
                .unwrap_or_else(|error| panic!("{mode:?}/{key_len}: {error}"));
            assert_eq!(plaintext, data, "{mode:?}/{key_len}");
        }
    }
}

#[test]
fn aes_rejects_invalid_parameters() {
    let data = bin(b"0123456789abcdef");
    let key = bin(&key_of(32));
    let iv = bin(&iv16());

    let wrong_key = AesEncryptTool {
        data: data.clone(),
        key: bin(&key_of(15)),
        mode: AesMode::Cbc,
        padding: AesPadding::Pkcs7,
        iv: Some(iv.clone()),
    }
    .call_tool();
    assert!(wrong_key.is_err());

    let missing_iv = AesEncryptTool {
        data: data.clone(),
        key: key.clone(),
        mode: AesMode::Cbc,
        padding: AesPadding::Pkcs7,
        iv: None,
    }
    .call_tool();
    assert!(missing_iv.is_err());

    let stream_with_padding = AesEncryptTool {
        data: data.clone(),
        key: key.clone(),
        mode: AesMode::Ctr,
        padding: AesPadding::Pkcs7,
        iv: Some(iv.clone()),
    }
    .call_tool();
    assert!(stream_with_padding.is_err());

    let unaligned_no_padding = AesEncryptTool {
        data: bin(b"0123456789abcdef0"),
        key: key.clone(),
        mode: AesMode::Ecb,
        padding: AesPadding::NoPadding,
        iv: None,
    }
    .call_tool();
    assert!(unaligned_no_padding.is_err());

    let bad_nonce = AesEncryptTool {
        data,
        key,
        mode: AesMode::Gcm,
        padding: AesPadding::NoPadding,
        iv: Some(bin(&[0u8; 11])),
    }
    .call_tool();
    assert!(bad_nonce.is_err());
}
