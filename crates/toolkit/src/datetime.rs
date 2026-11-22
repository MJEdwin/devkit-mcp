//! RFC 3339 datetime type used by MCP tool parameters.
//!
//! `UtcDateTime` stores a `chrono::DateTime<Utc>`. Serde accepts an RFC 3339
//! string with a timezone offset and serializes it back as UTC with `Z`. Its
//! JSON schema is `{"type":"string","format":"date-time"}`.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcDateTime(DateTime<Utc>);

impl UtcDateTime {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn json_schema() -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("string".to_string()));
        map.insert("format".to_string(), Value::String("date-time".to_string()));
        map
    }
}

impl Deref for UtcDateTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for UtcDateTime {
    type Err = chrono::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        DateTime::parse_from_rfc3339(value).map(|datetime| Self(datetime.with_timezone(&Utc)))
    }
}

impl TryFrom<&str> for UtcDateTime {
    type Error = chrono::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        DateTime::parse_from_rfc3339(&value)
            .map(|datetime| Self(datetime.with_timezone(&Utc)))
            .map_err(|err| {
                serde::de::Error::custom(format!(
                    "invalid ISO 8601 datetime '{value}': {err}; a timezone offset is required (e.g. 2023-11-14T22:13:20Z)"
                ))
            })
    }
}
