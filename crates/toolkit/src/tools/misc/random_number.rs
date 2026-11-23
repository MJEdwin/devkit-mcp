use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Generates a random integer or floating-point number using the OS RNG.
#[mcp_tool(
    name = "random_number",
    title = "Random Number Generator",
    description = "Generates a random integer when decimals is 0 (inclusive range), or a floating-point number when decimals is greater than 0 (half-open range), from the operating system RNG."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RandomNumberTool {
    /// Lower bound. Integers (decimals = 0): inclusive, default 0. Floats (decimals > 0): inclusive, default 0.0.
    pub min: Option<f64>,

    /// Upper bound. Integers (decimals = 0): inclusive, default 100. Floats (decimals > 0): exclusive, default 1.0.
    pub max: Option<f64>,

    /// Decimal places. 0 generates an integer; 1-15 generates a float with that many decimals. Defaults to 0.
    #[json_schema(default = 0, minimum = 0, maximum = 15)]
    pub decimals: Option<u8>,
}

impl RandomNumberTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let decimals = self.decimals.unwrap_or(0);
        if decimals > 15 {
            return Err(CallToolError::from_message(format!(
                "decimals must be between 0 and 15, got {decimals}"
            )));
        }

        let value = if decimals == 0 {
            let (min, max) = self.integer_bounds()?;
            random_integer(min, max)?.to_string()
        } else {
            let (min, max) = self.float_bounds()?;
            let value = random_float(min, max)?;
            format!("{value:.precision$}", precision = usize::from(decimals))
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(value)]))
    }

    fn integer_bounds(&self) -> Result<(i64, i64), CallToolError> {
        let min = self.min.unwrap_or(0.0);
        let max = self.max.unwrap_or(100.0);

        check_integer_bound("min", min)?;
        check_integer_bound("max", max)?;

        let min = min as i64;
        let max = max as i64;

        if min > max {
            return Err(CallToolError::from_message(format!(
                "min ({min}) must be less than or equal to max ({max})"
            )));
        }

        Ok((min, max))
    }

    fn float_bounds(&self) -> Result<(f64, f64), CallToolError> {
        let min = self.min.unwrap_or(0.0);
        let max = self.max.unwrap_or(1.0);

        if !min.is_finite() || !max.is_finite() {
            return Err(CallToolError::from_message(
                "min and max must be finite numbers",
            ));
        }
        if max < min {
            return Err(CallToolError::from_message(format!(
                "min ({min}) must be less than or equal to max ({max})"
            )));
        }
        if !(max - min).is_finite() {
            return Err(CallToolError::from_message(
                "the range between min and max is too large",
            ));
        }

        Ok((min, max))
    }
}

/// Integers must be exactly representable as `f64` so that the JSON number
/// bounds can be converted to `i64` without precision surprises.
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_992.0; // 2^53

fn check_integer_bound(name: &str, value: f64) -> Result<(), CallToolError> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(CallToolError::from_message(format!(
            "{name} must be a finite whole number for integer generation"
        )));
    }

    if value.abs() > MAX_SAFE_INTEGER {
        return Err(CallToolError::from_message(format!(
            "{name} must be within ±2^53 for integer generation"
        )));
    }

    Ok(())
}

fn random_integer(min: i64, max: i64) -> Result<i64, CallToolError> {
    let span = (i128::from(max) - i128::from(min) + 1) as u128;

    // The full i64 range has 2^64 values, which `u64` represents directly.
    if span == 1u128 << 64 {
        return Ok(getrandom::u64().map_err(rng_error)? as i64);
    }

    let span = span as u64;
    let zone = u64::MAX - (u64::MAX % span);

    loop {
        let value = getrandom::u64().map_err(rng_error)?;
        if value < zone {
            return Ok((i128::from(min) + i128::from(value % span)) as i64);
        }
    }
}

fn random_float(min: f64, max: f64) -> Result<f64, CallToolError> {
    if min == max {
        return Ok(min);
    }

    // 53 random bits produce a uniform value in [0, 1).
    let unit = (getrandom::u64().map_err(rng_error)? >> 11) as f64 / (1u64 << 53) as f64;

    Ok(min + unit * (max - min))
}

fn rng_error(err: getrandom::Error) -> CallToolError {
    CallToolError::from_message(format!("random number generation failed: {err}"))
}
