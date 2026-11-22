mod common;

use common::{base64, bin, text, unbase64};
use toolkit::{
    BinaryToHexTool, BinaryToStringTool, EncodingStandard, HexToBinaryTool, StringToBinaryTool,
    TextEncoding, UrlDecodeTool, UrlEncodeTool,
};

#[test]
fn binary_to_hex_lowercase_and_uppercase() {
    let data = bin(b"hello");
    let lower = BinaryToHexTool {
        data: data.clone(),
        uppercase: None,
    }
    .call_tool()
    .expect("binary_to_hex");
    assert_eq!(text(&lower), "68656c6c6f");

    let upper = BinaryToHexTool {
        data,
        uppercase: Some(true),
    }
    .call_tool()
    .expect("binary_to_hex");
    assert_eq!(text(&upper), "68656C6C6F");
}

#[test]
fn hex_to_binary_roundtrip() {
    let result = HexToBinaryTool {
        hex: "68656c6c6f".to_string(),
    }
    .call_tool()
    .expect("hex_to_binary");
    assert_eq!(text(&result), base64(b"hello"));

    let uppercase_with_space = HexToBinaryTool {
        hex: "E4BDA0 E5A5BD".to_string(),
    }
    .call_tool()
    .expect("hex_to_binary");
    assert_eq!(text(&uppercase_with_space), base64("你好".as_bytes()));
}

#[test]
fn hex_to_binary_rejects_invalid_input() {
    assert!(
        HexToBinaryTool {
            hex: "zz".to_string()
        }
        .call_tool()
        .is_err()
    );
    assert!(
        HexToBinaryTool {
            hex: "abc".to_string()
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn url_encode_query_and_form() {
    let query = UrlEncodeTool {
        input: "a b&c=d/é".to_string(),
        standard: EncodingStandard::Query,
    }
    .call_tool()
    .expect("url_encode");
    assert_eq!(text(&query), "a%20b%26c%3Dd%2F%C3%A9");

    let form = UrlEncodeTool {
        input: "a b&c=d/é".to_string(),
        standard: EncodingStandard::Form,
    }
    .call_tool()
    .expect("url_encode");
    assert_eq!(text(&form), "a+b%26c%3Dd%2F%C3%A9");
}

#[test]
fn url_decode_query_and_form() {
    let query = UrlDecodeTool {
        input: "a+b%26c%3Dd".to_string(),
        standard: EncodingStandard::Query,
    }
    .call_tool()
    .expect("url_decode");
    assert_eq!(text(&query), "a+b&c=d");

    let form = UrlDecodeTool {
        input: "a+b%26c%3Dd".to_string(),
        standard: EncodingStandard::Form,
    }
    .call_tool()
    .expect("url_decode");
    assert_eq!(text(&form), "a b&c=d");
}

#[test]
fn url_decode_rejects_malformed_input() {
    let malformed = UrlDecodeTool {
        input: "bad%2".to_string(),
        standard: EncodingStandard::Query,
    }
    .call_tool();
    assert!(malformed.is_err());

    let invalid_utf8 = UrlDecodeTool {
        input: "%FF".to_string(),
        standard: EncodingStandard::Query,
    }
    .call_tool();
    assert!(invalid_utf8.is_err());
}

#[test]
fn binary_to_string_utf8_and_ascii() {
    let utf8 = BinaryToStringTool {
        data: bin("hello, 世界".as_bytes()),
        encoding: None,
    }
    .call_tool()
    .expect("binary_to_string");
    assert_eq!(text(&utf8), "hello, 世界");

    let ascii = BinaryToStringTool {
        data: bin(b"hello"),
        encoding: Some(TextEncoding::Ascii),
    }
    .call_tool()
    .expect("binary_to_string");
    assert_eq!(text(&ascii), "hello");

    let bad_ascii = BinaryToStringTool {
        data: bin(&[0x80]),
        encoding: Some(TextEncoding::Ascii),
    }
    .call_tool();
    assert!(bad_ascii.is_err());
}

#[test]
fn string_to_binary_roundtrips() {
    for (encoding, text_value) in [
        (TextEncoding::Utf8, "hello, 世界"),
        (TextEncoding::Ascii, "hello"),
        (TextEncoding::Utf16, "hello, 世界"),
        (TextEncoding::Utf32, "hello, 世界"),
        (TextEncoding::Gbk, "你好"),
        (TextEncoding::Gb18030, "你好"),
    ] {
        let encoded = StringToBinaryTool {
            text: text_value.to_string(),
            encoding: Some(encoding),
        }
        .call_tool()
        .expect("string_to_binary");

        let decoded = BinaryToStringTool {
            data: bin(&unbase64(text(&encoded))),
            encoding: Some(encoding),
        }
        .call_tool()
        .expect("binary_to_string");

        assert_eq!(text(&decoded), text_value, "encoding {encoding:?}");
    }
}

#[test]
fn string_to_binary_rejects_unrepresentable_text() {
    let ascii = StringToBinaryTool {
        text: "héllo".to_string(),
        encoding: Some(TextEncoding::Ascii),
    }
    .call_tool();
    assert!(ascii.is_err());

    let gbk = StringToBinaryTool {
        text: "😀".to_string(),
        encoding: Some(TextEncoding::Gbk),
    }
    .call_tool();
    assert!(gbk.is_err());
}

#[test]
fn hex_and_binary_tools_roundtrip_bytes() {
    let bytes = [0x00, 0x01, 0xff, 0x10];
    let hex = BinaryToHexTool {
        data: bin(&bytes),
        uppercase: None,
    }
    .call_tool()
    .expect("binary_to_hex");
    let binary = HexToBinaryTool {
        hex: text(&hex).to_string(),
    }
    .call_tool()
    .expect("hex_to_binary");
    assert_eq!(unbase64(text(&binary)), bytes);
}
