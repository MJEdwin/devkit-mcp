use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Ranges at or below this width are scanned so that "no prime" is detected.
const SMALL_RANGE_WIDTH: u128 = 100_000;

/// Random samples attempted for wider ranges.
const MAX_SAMPLES: usize = 10_000;

/// Generates a random prime within an inclusive range.
#[mcp_tool(
    name = "random_prime",
    title = "Random Prime Generator",
    description = "Generates a uniformly random prime number within an inclusive range using the operating system RNG."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RandomPrimeTool {
    /// Lower bound (inclusive), at least 2. Defaults to 2.
    #[json_schema(default = 2, minimum = 2)]
    pub min: Option<u64>,

    /// Upper bound (inclusive), at least `min`. Defaults to 1000.
    #[json_schema(default = 1000, minimum = 2)]
    pub max: Option<u64>,
}

impl RandomPrimeTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let min = self.min.unwrap_or(2);
        let max = self.max.unwrap_or(1000);

        if min < 2 {
            return Err(CallToolError::from_message(format!(
                "min must be at least 2, got {min}"
            )));
        }
        if max < min {
            return Err(CallToolError::from_message(format!(
                "min ({min}) must be less than or equal to max ({max})"
            )));
        }

        let width = u128::from(max) - u128::from(min) + 1;
        let prime = if width <= SMALL_RANGE_WIDTH {
            let primes: Vec<u64> = (min..=max).filter(|value| is_prime(*value)).collect();
            if primes.is_empty() {
                return Err(CallToolError::from_message(format!(
                    "no prime exists in range {min}..={max}"
                )));
            }

            let index = random_in_range(0, (primes.len() - 1) as u64)? as usize;
            primes[index]
        } else {
            let mut found = None;
            for _ in 0..MAX_SAMPLES {
                let candidate = random_in_range(min, max)?;
                if is_prime(candidate) {
                    found = Some(candidate);
                    break;
                }
            }

            found.ok_or_else(|| {
                CallToolError::from_message(format!("no prime found in range {min}..={max}"))
            })?
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            prime.to_string(),
        )]))
    }
}

fn random_in_range(min: u64, max: u64) -> Result<u64, CallToolError> {
    let span = u128::from(max) - u128::from(min) + 1;

    if span == 1u128 << 64 {
        return getrandom::u64().map_err(rng_error);
    }

    let span = span as u64;
    let zone = u64::MAX - (u64::MAX % span);

    loop {
        let value = getrandom::u64().map_err(rng_error)?;
        if value < zone {
            return Ok((u128::from(min) + u128::from(value % span)) as u64);
        }
    }
}

fn rng_error(err: getrandom::Error) -> CallToolError {
    CallToolError::from_message(format!("random generation failed: {err}"))
}

/// Deterministic Miller-Rabin for all `u64` values.
fn is_prime(value: u64) -> bool {
    const BASES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    if value < 2 {
        return false;
    }

    for base in BASES {
        if value == base {
            return true;
        }
        if value.is_multiple_of(base) {
            return false;
        }
    }

    let mut odd_part = value - 1;
    let mut power_of_two = 0u32;
    while odd_part.is_multiple_of(2) {
        odd_part >>= 1;
        power_of_two += 1;
    }

    'witness: for base in BASES {
        let mut x = mod_pow(base, odd_part, value);
        if x == 1 || x == value - 1 {
            continue;
        }

        for _ in 1..power_of_two {
            x = mul_mod(x, x, value);
            if x == value - 1 {
                continue 'witness;
            }
        }

        return false;
    }

    true
}

fn mod_pow(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut result = 1u64;

    while exponent > 0 {
        if !exponent.is_multiple_of(2) {
            result = mul_mod(result, base, modulus);
        }
        base = mul_mod(base, base, modulus);
        exponent >>= 1;
    }

    result
}

fn mul_mod(left: u64, right: u64, modulus: u64) -> u64 {
    ((u128::from(left) * u128::from(right)) % u128::from(modulus)) as u64
}
