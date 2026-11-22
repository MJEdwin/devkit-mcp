mod common;

use common::text;
use toolkit::{GenerateUuidTool, UuidVersion};
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
