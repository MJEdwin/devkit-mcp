mod common;

use common::{bin, rsa_test_keys, structured, text, unbase64};
use toolkit::{
    RsaConvertPrivateKeyTool, RsaDecryptTool, RsaEncryptTool, RsaEncryptionPadding,
    RsaGenerateKeyTool, RsaKeyComponentsTool, RsaPrivateKeyFormat, RsaPrivateToPublicTool,
    RsaSignTool, RsaSignaturePadding, RsaVerifyTool,
};

#[test]
fn rsa_generate_key_returns_both_formats() {
    for format in [RsaPrivateKeyFormat::Pkcs8, RsaPrivateKeyFormat::Pkcs1] {
        let result = RsaGenerateKeyTool {
            bits: Some(1024),
            format: Some(format),
        }
        .call_tool()
        .expect("rsa_generate_key");

        let value = structured(&result);
        assert_eq!(value["bits"].as_u64(), Some(1024));
        assert_eq!(
            value["format"].as_str(),
            Some(match format {
                RsaPrivateKeyFormat::Pkcs1 => "pkcs1",
                RsaPrivateKeyFormat::Pkcs8 => "pkcs8",
            })
        );
        assert!(value["private_key"].as_str().unwrap().starts_with(
            if matches!(format, RsaPrivateKeyFormat::Pkcs1) {
                "-----BEGIN RSA PRIVATE KEY-----"
            } else {
                "-----BEGIN PRIVATE KEY-----"
            }
        ));
        assert!(
            value["public_key"]
                .as_str()
                .unwrap()
                .starts_with("-----BEGIN PUBLIC KEY-----")
        );
    }
}

#[test]
fn rsa_generate_key_rejects_small_size() {
    assert!(
        RsaGenerateKeyTool {
            bits: Some(512),
            format: None,
        }
        .call_tool()
        .is_err()
    );
}

#[tokio::test]
async fn rsa_generate_key_async_works() {
    let result = RsaGenerateKeyTool {
        bits: Some(1024),
        format: Some(RsaPrivateKeyFormat::Pkcs8),
    }
    .call_tool_async()
    .await
    .expect("async rsa_generate_key");

    let value = structured(&result);
    assert_eq!(value["bits"].as_u64(), Some(1024));
    assert!(
        value["private_key"]
            .as_str()
            .unwrap()
            .starts_with("-----BEGIN PRIVATE KEY-----")
    );
}

#[test]
fn rsa_private_to_public_accepts_both_formats() {
    let keys = rsa_test_keys();

    for private_key in [&keys.pkcs8, &keys.pkcs1] {
        let result = RsaPrivateToPublicTool {
            private_key: private_key.clone(),
        }
        .call_tool()
        .expect("rsa_private_to_public");
        assert_eq!(text(&result), keys.public);
    }
}

#[test]
fn rsa_convert_private_key_roundtrips() {
    let keys = rsa_test_keys();

    let to_pkcs1 = RsaConvertPrivateKeyTool {
        private_key: keys.pkcs8.clone(),
        format: RsaPrivateKeyFormat::Pkcs1,
    }
    .call_tool()
    .expect("convert to pkcs1");
    assert_eq!(text(&to_pkcs1), keys.pkcs1);

    let to_pkcs8 = RsaConvertPrivateKeyTool {
        private_key: keys.pkcs1.clone(),
        format: RsaPrivateKeyFormat::Pkcs8,
    }
    .call_tool()
    .expect("convert to pkcs8");
    assert_eq!(text(&to_pkcs8), keys.pkcs8);
}

#[test]
fn rsa_key_components_private_and_public() {
    let keys = rsa_test_keys();

    let private = RsaKeyComponentsTool {
        key: keys.pkcs8.clone(),
    }
    .call_tool()
    .expect("private components");
    let private = structured(&private);
    assert_eq!(private["key_type"], "private");
    for field in ["bits", "n", "e", "d", "p", "q", "dp", "dq", "qinv"] {
        assert!(private.get(field).is_some(), "missing {field}");
    }

    let public = RsaKeyComponentsTool {
        key: keys.public.clone(),
    }
    .call_tool()
    .expect("public components");
    let public = structured(&public);
    assert_eq!(public["key_type"], "public");
    assert_eq!(public["n"], private["n"]);
    assert_eq!(public["e"], private["e"]);
    assert!(public.get("d").is_none());
}

#[test]
fn rsa_encrypt_decrypt_roundtrips() {
    let keys = rsa_test_keys();
    let message = b"rsa message";
    let data = bin(message);

    for padding in [
        RsaEncryptionPadding::Pkcs1v15,
        RsaEncryptionPadding::OaepSha256,
    ] {
        let encrypted = RsaEncryptTool {
            data: data.clone(),
            public_key: keys.public.clone(),
            padding,
        }
        .call_tool()
        .expect("rsa_encrypt");

        let decrypted = RsaDecryptTool {
            data: bin(&unbase64(text(&encrypted))),
            private_key: keys.pkcs8.clone(),
            padding,
        }
        .call_tool()
        .expect("rsa_decrypt");

        assert_eq!(unbase64(text(&decrypted)), message, "{padding:?}");
    }
}

#[test]
fn rsa_raw_roundtrips() {
    let keys = rsa_test_keys();
    let block: Vec<u8> = std::iter::once(0u8)
        .chain(std::iter::repeat_n(1u8, 127))
        .collect();
    let data = bin(&block);

    let encrypted = RsaEncryptTool {
        data: data.clone(),
        public_key: keys.public.clone(),
        padding: RsaEncryptionPadding::Raw,
    }
    .call_tool()
    .expect("raw encrypt");
    let decrypted = RsaDecryptTool {
        data: bin(&unbase64(text(&encrypted))),
        private_key: keys.pkcs8.clone(),
        padding: RsaEncryptionPadding::Raw,
    }
    .call_tool()
    .expect("raw decrypt");
    assert_eq!(unbase64(text(&decrypted)), block);

    let signature = RsaSignTool {
        data: data.clone(),
        private_key: keys.pkcs8.clone(),
        padding: RsaSignaturePadding::Raw,
    }
    .call_tool()
    .expect("raw sign");
    let verified = RsaVerifyTool {
        data,
        signature: bin(&unbase64(text(&signature))),
        public_key: keys.public.clone(),
        padding: RsaSignaturePadding::Raw,
    }
    .call_tool()
    .expect("raw verify");
    assert_eq!(text(&verified), "true");
}

#[test]
fn rsa_sign_verify_roundtrips() {
    let keys = rsa_test_keys();
    let message = bin(b"rsa signature message");

    for padding in [
        RsaSignaturePadding::Pkcs1v15Sha256,
        RsaSignaturePadding::PssSha256,
    ] {
        let signature = RsaSignTool {
            data: message.clone(),
            private_key: keys.pkcs8.clone(),
            padding,
        }
        .call_tool()
        .expect("rsa_sign");

        let valid = RsaVerifyTool {
            data: message.clone(),
            signature: bin(&unbase64(text(&signature))),
            public_key: keys.public.clone(),
            padding,
        }
        .call_tool()
        .expect("rsa_verify");
        assert_eq!(text(&valid), "true", "{padding:?}");

        let tampered = RsaVerifyTool {
            data: bin(b"tampered"),
            signature: bin(&unbase64(text(&signature))),
            public_key: keys.public.clone(),
            padding,
        }
        .call_tool()
        .expect("rsa_verify");
        assert_eq!(text(&tampered), "false", "{padding:?}");
    }
}

#[test]
fn rsa_rejects_invalid_inputs() {
    let keys = rsa_test_keys();

    assert!(
        RsaEncryptTool {
            data: bin(b"x"),
            public_key: "not a key".to_string(),
            padding: RsaEncryptionPadding::OaepSha256,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        RsaDecryptTool {
            data: bin(&[0xff; 128]),
            private_key: keys.pkcs8.clone(),
            padding: RsaEncryptionPadding::OaepSha256,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        RsaEncryptTool {
            data: bin(b"x"),
            public_key: keys.public.clone(),
            padding: RsaEncryptionPadding::Raw,
        }
        .call_tool()
        .is_err()
    );
}
