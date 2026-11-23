use base64::Engine as _;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use chrono::{DateTime, SecondsFormat, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, AlgorithmFamily, DecodingKey, Validation, decode};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Decodes and optionally verifies a JWT.
#[mcp_tool(
    name = "jwt_decode",
    title = "JWT Decode",
    description = "Decodes a JWT header and payload. When `key` is supplied, verifies the signature (HS/RS/PS/ES/EdDSA); issuer, audience, and expiry are reported but not enforced."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JwtDecodeTool {
    /// Compact JWS token.
    pub token: String,

    /// Optional verification key: an HMAC secret, or an RSA/EC/EdDSA PEM public key.
    pub key: Option<String>,
}

impl JwtDecodeTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let parts: Vec<&str> = self.token.split('.').collect();
        if parts.len() != 3 {
            return Err(CallToolError::from_message(format!(
                "expected a compact JWS with 3 segments, got {}",
                parts.len()
            )));
        }

        let header = decode_json_segment(parts[0], "header")?;
        let payload = decode_json_segment(parts[1], "payload")?;

        let algorithm = header
            .get("alg")
            .and_then(Value::as_str)
            .map(str::to_string);

        let signature_valid = match &self.key {
            Some(key) => {
                let algorithm = algorithm
                    .as_deref()
                    .ok_or_else(|| CallToolError::from_message("JWT header has no 'alg' field"))?;
                Some(verify_signature(&self.token, algorithm, key)?)
            }
            None => None,
        };

        let (expired, expires_at) = expiry(&payload);
        let verification_error = match signature_valid {
            Some(false) => Some("invalid signature".to_string()),
            _ => None,
        };

        let mut structured = Map::new();
        structured.insert("header".to_string(), header);
        structured.insert("payload".to_string(), payload);
        structured.insert(
            "algorithm".to_string(),
            algorithm.map_or(Value::Null, Value::String),
        );
        structured.insert(
            "signature_valid".to_string(),
            signature_valid.map_or(Value::Null, Value::Bool),
        );
        structured.insert(
            "verification_error".to_string(),
            verification_error.map_or(Value::Null, Value::String),
        );
        structured.insert(
            "expired".to_string(),
            expired.map_or(Value::Null, Value::Bool),
        );
        structured.insert(
            "expires_at".to_string(),
            expires_at.map_or(Value::Null, Value::String),
        );

        let text =
            serde_json::to_string_pretty(&Value::Object(structured.clone())).map_err(|err| {
                CallToolError::from_message(format!("failed to serialize result: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(text)])
            .with_structured_content(structured))
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "header": {
                    "type": "object",
                    "description": "Decoded JWT header.",
                    "additionalProperties": true
                },
                "payload": {
                    "description": "Decoded JWT payload/claims."
                },
                "algorithm": {
                    "type": ["string", "null"],
                    "description": "Algorithm from the header."
                },
                "signature_valid": {
                    "type": ["boolean", "null"],
                    "description": "Signature check result; null when no key was supplied."
                },
                "verification_error": {
                    "type": ["string", "null"],
                    "description": "Signature failure reason, or null."
                },
                "expired": {
                    "type": ["boolean", "null"],
                    "description": "Whether exp is in the past; null when there is no numeric exp claim."
                },
                "expires_at": {
                    "type": ["string", "null"],
                    "description": "exp as an RFC 3339 UTC timestamp, or null."
                }
            },
            "required": [
                "header",
                "payload",
                "algorithm",
                "signature_valid",
                "verification_error",
                "expired",
                "expires_at"
            ]
        }))
    }
}

fn decode_json_segment(segment: &str, name: &str) -> Result<Value, CallToolError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(segment)
        .or_else(|_| URL_SAFE.decode(segment))
        .map_err(|err| CallToolError::from_message(format!("invalid {name} segment: {err}")))?;

    serde_json::from_slice(&bytes).map_err(|err| {
        CallToolError::from_message(format!("invalid JSON in {name} segment: {err}"))
    })
}

fn verify_signature(token: &str, algorithm: &str, key: &str) -> Result<bool, CallToolError> {
    let algorithm: Algorithm = algorithm.parse().map_err(|_| {
        CallToolError::from_message(format!("unsupported JWT algorithm: {algorithm}"))
    })?;

    let decoding_key = match algorithm.family() {
        AlgorithmFamily::Hmac => {
            if key.contains("-----BEGIN") {
                return Err(CallToolError::from_message(
                    "HMAC verification requires a secret, not a PEM key",
                ));
            }
            DecodingKey::from_secret(key.as_bytes())
        }
        AlgorithmFamily::Rsa => DecodingKey::from_rsa_pem(key.as_bytes())
            .map_err(|err| CallToolError::from_message(format!("invalid RSA PEM key: {err}")))?,
        AlgorithmFamily::Ec => DecodingKey::from_ec_pem(key.as_bytes())
            .map_err(|err| CallToolError::from_message(format!("invalid EC PEM key: {err}")))?,
        AlgorithmFamily::Ed => DecodingKey::from_ed_pem(key.as_bytes())
            .map_err(|err| CallToolError::from_message(format!("invalid EdDSA PEM key: {err}")))?,
    };

    let mut validation = Validation::new(algorithm);
    validation.required_spec_claims.clear();
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.validate_aud = false;

    match decode::<Value>(token, &decoding_key, &validation) {
        Ok(_) => Ok(true),
        Err(err) if matches!(err.kind(), ErrorKind::InvalidSignature) => Ok(false),
        Err(err) => Err(CallToolError::from_message(format!(
            "JWT verification failed: {err}"
        ))),
    }
}

fn expiry(payload: &Value) -> (Option<bool>, Option<String>) {
    let Some(exp) = payload.get("exp").and_then(Value::as_f64) else {
        return (None, None);
    };

    let seconds = exp.trunc() as i64;
    let now = Utc::now().timestamp();
    let expired = exp <= now as f64;
    let expires_at = DateTime::<Utc>::from_timestamp(seconds, 0)
        .map(|datetime| datetime.to_rfc3339_opts(SecondsFormat::Secs, true));

    (Some(expired), expires_at)
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == JwtDecodeTool::tool_name() {
        Some(JwtDecodeTool::output_schema())
    } else {
        None
    }
}
