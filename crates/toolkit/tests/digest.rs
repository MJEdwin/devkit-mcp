mod common;

use common::{bin, from_hex, text, unbase64};
use toolkit::{DigestAlgorithm, DigestTool, HmacTool};

#[test]
fn digest_known_vectors() {
    let cases = [
        (DigestAlgorithm::Md5, "900150983cd24fb0d6963f7d28e17f72"),
        (
            DigestAlgorithm::Sha1,
            "a9993e364706816aba3e25717850c26c9cd0d89d",
        ),
        (
            DigestAlgorithm::Sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            DigestAlgorithm::Sha384,
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
        ),
        (
            DigestAlgorithm::Sha512,
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        ),
    ];

    for (algorithm, expected_hex) in cases {
        let result = DigestTool {
            data: bin(b"abc"),
            algorithm,
        }
        .call_tool()
        .expect("digest");

        assert_eq!(
            unbase64(text(&result)),
            from_hex(expected_hex),
            "algorithm {algorithm:?}"
        );
    }
}

#[test]
fn binary_data_rejects_invalid_base64() {
    let result: Result<toolkit::BinaryData, _> = serde_json::from_value(serde_json::json!("!!!"));
    assert!(result.is_err());
}

#[test]
fn hmac_known_vectors() {
    let key = bin(&[0x0b; 20]);
    let data = bin(b"Hi There");

    let cases = [
        (DigestAlgorithm::Md5, "5ccec34ea9656392457fa1ac27f08fbc"),
        (
            DigestAlgorithm::Sha1,
            "b617318655057264e28bc0b6fb378c8ef146be00",
        ),
        (
            DigestAlgorithm::Sha256,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
        ),
        (
            DigestAlgorithm::Sha384,
            "afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59cfaea9ea9076ede7f4af152e8b2fa9cb6",
        ),
        (
            DigestAlgorithm::Sha512,
            "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854",
        ),
    ];

    for (algorithm, expected_hex) in cases {
        let result = HmacTool {
            data: data.clone(),
            key: key.clone(),
            algorithm,
        }
        .call_tool()
        .expect("hmac");

        assert_eq!(
            unbase64(text(&result)),
            from_hex(expected_hex),
            "algorithm {algorithm:?}"
        );
    }
}

#[test]
fn hmac_accepts_empty_key_and_message() {
    let result = HmacTool {
        data: bin(&[]),
        key: bin(&[]),
        algorithm: DigestAlgorithm::Sha256,
    }
    .call_tool()
    .expect("hmac");
    assert_eq!(
        unbase64(text(&result)),
        from_hex("b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad")
    );
}
