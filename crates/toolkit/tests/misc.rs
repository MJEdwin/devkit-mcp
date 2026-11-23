mod common;

use common::text;
use toolkit::{GenerateUuidTool, RandomCharset, RandomNumberTool, RandomStringTool, UuidVersion};
use uuid::Uuid;

#[test]
fn uuid_v4_and_v7_versions() {
    for (version, expected) in [(UuidVersion::V4, 4), (UuidVersion::V7, 7)] {
        let result = GenerateUuidTool {
            version,
            uppercase: None,
            no_hyphens: None,
        }
        .call_tool()
        .expect("generate_uuid");

        let parsed = Uuid::parse_str(text(&result)).expect("valid UUID");
        assert_eq!(parsed.get_version_num(), expected);
    }
}

#[test]
fn uuid_formatting_options() {
    let default = GenerateUuidTool {
        version: UuidVersion::V4,
        uppercase: None,
        no_hyphens: None,
    }
    .call_tool()
    .expect("generate_uuid");
    assert!(text(&default).contains('-'));

    let uppercase = GenerateUuidTool {
        version: UuidVersion::V4,
        uppercase: Some(true),
        no_hyphens: None,
    }
    .call_tool()
    .expect("generate_uuid");
    let upper_value = text(&uppercase).to_string();
    assert_eq!(upper_value, upper_value.to_uppercase());
    assert!(Uuid::parse_str(&upper_value).is_ok());

    let no_hyphens = GenerateUuidTool {
        version: UuidVersion::V7,
        uppercase: None,
        no_hyphens: Some(true),
    }
    .call_tool()
    .expect("generate_uuid");
    assert!(!text(&no_hyphens).contains('-'));
    assert_eq!(text(&no_hyphens).len(), 32);
    assert!(Uuid::parse_str(text(&no_hyphens)).is_ok());
}

#[test]
fn uuid_values_are_unique() {
    let first = GenerateUuidTool {
        version: UuidVersion::V4,
        uppercase: None,
        no_hyphens: None,
    }
    .call_tool()
    .expect("generate_uuid");
    let second = GenerateUuidTool {
        version: UuidVersion::V4,
        uppercase: None,
        no_hyphens: None,
    }
    .call_tool()
    .expect("generate_uuid");
    assert_ne!(text(&first), text(&second));
}

#[test]
fn random_integer_respects_bounds() {
    for _ in 0..100 {
        let result = RandomNumberTool {
            min: Some(-5.0),
            max: Some(5.0),
            decimals: None,
        }
        .call_tool()
        .expect("random_number");

        let value: i64 = text(&result).parse().expect("integer");
        assert!((-5..=5).contains(&value));
    }
}

#[test]
fn random_integer_defaults_and_exact_bounds() {
    let default = RandomNumberTool {
        min: None,
        max: None,
        decimals: None,
    }
    .call_tool()
    .expect("random_number");
    let value: i64 = text(&default).parse().expect("integer");
    assert!((0..=100).contains(&value));

    let exact = RandomNumberTool {
        min: Some(7.0),
        max: Some(7.0),
        decimals: None,
    }
    .call_tool()
    .expect("random_number");
    assert_eq!(text(&exact), "7");
}

#[test]
fn random_integer_rejects_invalid_bounds() {
    let reversed = RandomNumberTool {
        min: Some(10.0),
        max: Some(1.0),
        decimals: None,
    }
    .call_tool();
    assert!(reversed.is_err());

    let fractional = RandomNumberTool {
        min: Some(1.5),
        max: Some(2.0),
        decimals: None,
    }
    .call_tool();
    assert!(fractional.is_err());
}

#[test]
fn random_float_respects_bounds_and_decimals() {
    for _ in 0..100 {
        let result = RandomNumberTool {
            min: Some(1.0),
            max: Some(2.0),
            decimals: Some(6),
        }
        .call_tool()
        .expect("random_number");

        let rendered = text(&result);
        let value: f64 = rendered.parse().expect("float");
        assert!((1.0..=2.0).contains(&value));
        assert_eq!(rendered.split('.').nth(1).map(str::len), Some(6));
    }
}

#[test]
fn random_float_equal_bounds() {
    let result = RandomNumberTool {
        min: Some(1.5),
        max: Some(1.5),
        decimals: Some(2),
    }
    .call_tool()
    .expect("random_number");

    assert_eq!(text(&result), "1.50");
}

#[test]
fn random_float_rejects_invalid_bounds() {
    let reversed = RandomNumberTool {
        min: Some(2.0),
        max: Some(1.0),
        decimals: Some(6),
    }
    .call_tool();
    assert!(reversed.is_err());

    let not_finite = RandomNumberTool {
        min: Some(f64::NAN),
        max: Some(1.0),
        decimals: Some(6),
    }
    .call_tool();
    assert!(not_finite.is_err());
}

#[test]
fn random_string_respects_length_and_charset() {
    type CharsetCheck = fn(char) -> bool;

    let cases: [(RandomCharset, CharsetCheck); 4] = [
        (RandomCharset::Alphanumeric, |c| c.is_ascii_alphanumeric()),
        (RandomCharset::Alphabetic, |c| c.is_ascii_alphabetic()),
        (RandomCharset::Numeric, |c| c.is_ascii_digit()),
        (RandomCharset::Hex, |c| {
            c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
        }),
    ];

    for (charset, allowed) in cases {
        let result = RandomStringTool {
            length: Some(64),
            charset: Some(charset),
        }
        .call_tool()
        .expect("random_string");

        let value = text(&result);
        assert_eq!(value.len(), 64);
        assert!(
            value.chars().all(allowed),
            "unexpected character in {value}"
        );
    }
}

#[test]
fn random_string_default_length() {
    let result = RandomStringTool {
        length: None,
        charset: None,
    }
    .call_tool()
    .expect("random_string");

    let value = text(&result);
    assert_eq!(value.len(), 16);
    assert!(value.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn random_string_rejects_invalid_length() {
    for length in [0, 1025] {
        let result = RandomStringTool {
            length: Some(length),
            charset: None,
        }
        .call_tool();

        assert!(result.is_err(), "length {length} should be rejected");
    }
}
