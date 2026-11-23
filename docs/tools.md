**English** | [简体中文](tools.zh-CN.md)

# Tool reference

All tools are exposed by both transports (`local` / stdio and `remote` / HTTP).

## Conventions

- **Binary data** is always a base64 string and is marked
  `format: "binary"` in the JSON Schema.
- **Timestamps** are ISO 8601 UTC strings ending in `Z`.
- **Optional parameters** have schema defaults; omitting them uses the default.
- **Errors** are returned as tool results with `isError: true` and a readable
  message (e.g. invalid input, unsupported enum value).
- Constrained values use enums, so clients should only send the listed values.

## Calling a tool over HTTP

```bash
curl -sS -X POST http://127.0.0.1:8080/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2026-07-28' \
  -H 'Mcp-Method: tools/call' \
  -H 'Mcp-Name: <tool-name>' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "_meta": {
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/protocolVersion": "2026-07-28"
      },
      "name": "<tool-name>",
      "arguments": { }
    }
  }'
```

Responses are SSE frames: `data: {"jsonrpc":"2.0", ...}`.

- `Mcp-Name` must match `params.name` and is required for `tools/call`.
- `Accept` must include both `application/json` and `text/event-stream`.
- `tools/list` uses `Mcp-Method: tools/list` and no `Mcp-Name`.


## Time

### current_time

Returns the current UTC time in ISO 8601 format with millisecond precision.

No parameters. Output example: `2026-11-22T04:20:23.384Z`.

---

### unix_to_iso8601

Convert a Unix timestamp to an ISO 8601 UTC string.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `timestamp` | integer | yes | — | Unix timestamp; may be negative |
| `unit` | string (`seconds`, `milliseconds`) | no | `seconds` | Unit of `timestamp` |

Output precision follows `unit`: seconds → `2023-11-14T22:13:20Z`,
milliseconds → `2023-11-14T22:13:20.123Z`.

Out-of-range timestamps return `isError: true`.

---

### iso8601_to_unix

Convert an ISO 8601 datetime to a Unix timestamp.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `datetime` | string (`format: date-time`) | yes | — | RFC 3339 datetime **with a timezone offset** |
| `unit` | string (`seconds`, `milliseconds`) | no | `seconds` | Unit of the returned timestamp |

Accepted: `2023-11-14T22:13:20Z`, `2023-11-14T22:13:20.123Z`,
`2023-11-15T06:13:20+08:00`.

Rejected: naive datetimes without an offset (e.g. `2023-11-14T22:13:20`) and
date-only strings.

---

### solar_to_lunar_date

Convert a Gregorian (solar) date to the Chinese lunar date.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `solar_year` | integer (`1`–`9999`) | yes | Gregorian year |
| `solar_month` | integer (`1`–`12`) | yes | Gregorian month |
| `solar_day` | integer (`1`–`31`) | yes | Gregorian day |

The input must be a real Gregorian date (e.g. `2023-02-29` is rejected). The
lunar date, sexagenary year, and zodiac are computed for that date.

The result is returned both as pretty-printed JSON text and as
`structuredContent`; the tool also declares an `outputSchema` in `tools/list`.

Fields:

| Field | Type | Meaning |
|---|---|---|
| `solar_date` | string | Input Gregorian date, `YYYY-MM-DD` |
| `lunar_year` | integer | Chinese lunar year |
| `lunar_month` | integer | Lunar month (`1`–`12`) |
| `lunar_day` | integer | Lunar day (`1`–`30`) |
| `is_leap_month` | boolean | Whether the month is a leap month |
| `lunar_month_name` | string | Chinese month name (`正月` … `腊月`; `闰` prefix for leap months) |
| `lunar_day_name` | string | Chinese day name (`初一` … `三十`) |
| `lunar_date` | string | Full Chinese date, e.g. `二〇二四年正月初一` |
| `lunar_year_ganzhi` | string | Sexagenary year (天干地支), e.g. `甲辰` |
| `zodiac` | string | Chinese zodiac animal for the lunar year, e.g. `龙` |

Example: `2024-02-10` is the first day of the 2024 lunar year:

```json
{
  "solar_date": "2024-02-10",
  "lunar_year": 2024,
  "lunar_month": 1,
  "lunar_day": 1,
  "is_leap_month": false,
  "lunar_month_name": "正月",
  "lunar_day_name": "初一",
  "lunar_date": "二〇二四年正月初一",
  "lunar_year_ganzhi": "甲辰",
  "zodiac": "龙"
}
```

Leap months are reported with `is_leap_month: true` and a `闰` prefix, e.g.
`2023-03-22` → `二〇二三年闰二月初一`.

The sexagenary year is derived from the lunar year (anchored at
`1984 = 甲子`), not from the LiChun year-pillar boundary. The zodiac is
likewise based on the lunar year.

Gregorian years outside `1..=9999` return `isError: true`.

---

### lunar_to_solar_date

Convert a Chinese lunar date to the corresponding Gregorian date.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `lunar_year` | integer (`1`–`9999`) | yes | — | Chinese lunar year |
| `lunar_month` | integer (`1`–`12`) | yes | — | Lunar month |
| `lunar_day` | integer (`1`–`30`) | yes | — | Lunar day |
| `is_leap_month` | boolean | no | `false` | Whether the month is a leap month |

The date is validated exactly as supplied. An out-of-range year/month/day, a
day longer than the actual lunar month, or a leap-month flag for a month that
has no leap instance that year returns `isError: true`.

The result is returned both as pretty-printed JSON text and as
`structuredContent`; the tool also declares an `outputSchema` in `tools/list`.

| Field | Type | Meaning |
|---|---|---|
| `solar_date` | string | Gregorian date, `YYYY-MM-DD` |
| `lunar_date` | string | Lunar date in Chinese, e.g. `二〇二三年闰二月初一` |

Example: lunar `2024-01-01` (not leap) is `2024-02-10`:

```json
{
  "solar_date": "2024-02-10",
  "lunar_date": "二〇二四年正月初一"
}
```

Leap-month example: year `2023`, leap month `2`, day `1` → `2023-03-22`.

---

### cron_validate

Validate a cron expression.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `expression` | string | yes | Cron expression (5–7 fields, with optional seconds and year) |

Invalid expressions are a normal result (`valid: false`), not an `isError`.
The result is returned as pretty-printed JSON text and `structuredContent`, and
the tool declares an `outputSchema`.

| Field | Type | Meaning |
|---|---|---|
| `valid` | boolean | Whether the expression parses |
| `error` | string \| null | Parser error message, or `null` when valid |
| `description` | string \| null | English description of the schedule, or `null` when invalid |

Example: `*/5 * * * *` → `valid: true` with a description such as
"Every 5 minutes".

---

### cron_next

Compute the next N execution times for a cron expression.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `expression` | string | yes | — | Cron expression (5–7 fields) |
| `from` | string (`date-time`) | no | now | RFC 3339 start time; results are strictly after it |
| `count` | integer (`1`–`50`) | no | `5` | Number of occurrences |
| `timezone` | string (`±HH:MM` or `Z`) | no | `Z` | Fixed UTC offset |

- Supports 5 fields (`minute hour day month weekday`), 6 fields (leading
  seconds), and 7 fields (seconds plus year).
- `timezone` accepts any fixed offset from `-12:00` to `+14:00`, for example
  `+08:00`, `-05:30`, or `Z`. DST is not applied.
- Invalid expressions or an expression with no next occurrence return
  `isError: true`.

| Field | Type | Meaning |
|---|---|---|
| `timezone` | string | Applied UTC offset (`Z` or `±HH:MM`) |
| `count` | integer | Number of occurrences returned |
| `times` | array | RFC 3339 timestamps with the applied offset |

Example: expression `0 * * * *`, `from: "2026-01-01T00:00:00Z"`, `count: 3`:

```json
{
  "timezone": "Z",
  "count": 3,
  "times": [
    "2026-01-01T01:00:00Z",
    "2026-01-01T02:00:00Z",
    "2026-01-01T03:00:00Z"
  ]
}
```

---

### cron_prev

Compute the previous N execution times for a cron expression.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `expression` | string | yes | — | Cron expression (5–7 fields) |
| `from` | string (`date-time`) | no | now | RFC 3339 start time; results are strictly before it |
| `count` | integer (`1`–`50`) | no | `5` | Number of occurrences |
| `timezone` | string (`±HH:MM` or `Z`) | no | `Z` | Fixed UTC offset |

- Results are returned most recent first.
- Invalid expressions return `isError: true`.

| Field | Type | Meaning |
|---|---|---|
| `timezone` | string | Applied UTC offset |
| `count` | integer | Number of occurrences returned |
| `times` | array | RFC 3339 timestamps, most recent first |

Example: expression `0 * * * *`, `from: "2026-01-01T03:00:00Z"`, `count: 3` →

```json
{
  "timezone": "Z",
  "count": 3,
  "times": [
    "2026-01-01T02:00:00Z",
    "2026-01-01T01:00:00Z",
    "2026-01-01T00:00:00Z"
  ]
}
```

---

### cron_generate

Build a cron expression from per-field values.

Each field accepts one of:

- a raw cron string, e.g. `"*/5"`, `"1-5"`, `"1,3,5"`, or `"*"`
- a number, e.g. `5`
- a structured term:
  - `{ "type": "wildcard" }` → `*`
  - `{ "type": "value", "value": 5 }`
  - `{ "type": "range", "start": 1, "end": 5 }`
  - `{ "type": "step", "step": 5 }` → `*/5`
  - `{ "type": "step", "start": 1, "end": 10, "step": 2 }` → `1-10/2`
  - `{ "type": "step", "start": 5, "step": 10 }` → `5/10`
- an array of terms (enumeration), joined with commas, e.g.
  `[{ "type": "range", "start": 0, "end": 30 }, { "type": "value", "value": 45 }]`
  → `0-30,45`

Values are numbers only; named values such as `MON` or `JAN` are rejected.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `second` | CronField | no | Seconds field (0–59); include it for a 6- or 7-field expression |
| `minute` | CronField | yes | Minutes field (0–59) |
| `hour` | CronField | yes | Hours field (0–23) |
| `day_of_month` | CronField | yes | Day of month (1–31) |
| `month` | CronField | yes | Month (1–12) |
| `day_of_week` | CronField | yes | Day of week (0–7) |
| `year` | CronField | no | Year field; requires `second` |

The field values are joined in cron order and validated. Empty or
whitespace-containing fields, malformed terms (`step` of 0, `end` without
`start`, empty lists), `year` without `second`, and invalid field values return
`isError: true`.

The result is returned as pretty-printed JSON text and `structuredContent`;
the tool declares an `outputSchema`.

| Field | Type | Meaning |
|---|---|---|
| `expression` | string | Generated cron expression |
| `field_count` | integer | `5`, `6`, or `7` |
| `description` | string | English description of the schedule |

Example — every 5 minutes using a structured step:

```json
{
  "minute": { "type": "step", "step": 5 },
  "hour": "*",
  "day_of_month": "*",
  "month": "*",
  "day_of_week": "*"
}
```

→ `{"expression": "*/5 * * * *", "field_count": 5, "description": "..."}`.

Example — enumeration of a range and a value:

```json
{
  "minute": [
    { "type": "range", "start": 0, "end": 30 },
    { "type": "value", "value": 45 }
  ],
  "hour": "*",
  "day_of_month": "*",
  "month": "*",
  "day_of_week": "*"
}
```

→ `0-30,45 * * * *`.

Adding `"second": { "type": "value", "value": 0 }` produces
`0 */5 * * * *` (6 fields); adding `"year": 2026` as well produces
`0 */5 * * * * 2026` (7 fields).

---

### cron_parse

Parse a cron expression into structured fields and an English description.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `expression` | string | yes | Cron expression (5–7 fields) |

The expression is normalized (single spaces) and validated. Invalid expressions
return `isError: true`.

The result is returned as pretty-printed JSON text and `structuredContent`; the
tool declares an `outputSchema`.

| Field | Type | Meaning |
|---|---|---|
| `expression` | string | Normalized cron expression |
| `field_count` | integer | `5`, `6`, or `7` |
| `fields` | object | Structured `CronField` values; missing seconds/year are `null` |
| `raw_fields` | object | Original field strings; missing seconds/year are `null` |
| `description` | string | English description of the schedule |

Numeric fields are parsed into structured terms:

- `*` → `{ "type": "wildcard" }`
- `5` → `{ "type": "value", "value": 5 }`
- `1-5` → `{ "type": "range", "start": 1, "end": 5 }`
- `*/5` → `{ "type": "step", "step": 5 }`
- `1-10/2` → `{ "type": "step", "start": 1, "end": 10, "step": 2 }`
- `1,3,5` → an array of terms

Fields using names or special syntax (e.g. `MON`, `L`) are returned as raw
strings in `fields`.

Example: `0 */5 * * * *` →

```json
{
  "expression": "0 */5 * * * *",
  "field_count": 6,
  "fields": {
    "second": { "type": "value", "value": 0 },
    "minute": { "type": "step", "step": 5 },
    "hour": { "type": "wildcard" },
    "day_of_month": { "type": "wildcard" },
    "month": { "type": "wildcard" },
    "day_of_week": { "type": "wildcard" },
    "year": null
  },
  "raw_fields": {
    "second": "0",
    "minute": "*/5",
    "hour": "*",
    "day_of_month": "*",
    "month": "*",
    "day_of_week": "*",
    "year": null
  },
  "description": "At every 5 minutes."
}
```

## Encoding

### url_encode

Percent-encode a string.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `input` | string | yes | Raw string |
| `standard` | string (`query`, `form`) | yes | Encoding rules |

- `query` (RFC 3986): space → `%20`; `~` is kept.
- `form` (`application/x-www-form-urlencoded`): space → `+`; `~` → `%7E`.

Example: `"a b&c=d/é"` → query `a%20b%26c%3Dd%2F%C3%A9`, form
`a+b%26c%3Dd%2F%C3%A9`.

---

### url_decode

Percent-decode a string.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `input` | string | yes | Encoded string |
| `standard` | string (`query`, `form`) | yes | Decoding rules |

- `query`: `%XX` is decoded; `+` stays as `+`.
- `form`: `+` becomes a space before percent-decoding.

Malformed percent escapes (e.g. `%2`, `%G1`) and non-UTF-8 results return
`isError: true`.

---

### binary_to_hex

Convert binary data to a hex string.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `data` | string (`format: binary`) | yes | — | Base64-encoded binary |
| `uppercase` | boolean | no | `false` | Return uppercase hex |

Example: `"aGVsbG8="` → `68656c6c6f`; with `uppercase: true` → `68656C6C6F`.

---

### hex_to_binary

Convert a hex string to binary data.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `hex` | string | yes | Hex string; case-insensitive, whitespace ignored |

Output: base64-encoded binary (JSON Schema `binary` format).

Examples: `68656c6c6f` → `aGVsbG8=`; `E4BDA0 E5A5BD` → `5L2g5aW9`.

Invalid characters or an odd number of digits return `isError: true`.

---

### binary_to_string

Decode binary data into text.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `data` | string (`format: binary`) | yes | — | Base64-encoded binary |
| `encoding` | string | no | `utf-8` | `utf-8`, `ascii`, `utf-16`, `utf-32`, `gbk`, `gb18030` |

Encoding behaviour:

- `utf-8`: strict validation; a leading UTF-8 BOM is stripped.
- `ascii`: rejects bytes above `0x7F`.
- `utf-16`: BOM-aware (`FF FE` / `FE FF`), little-endian when no BOM;
  odd byte length or invalid surrogates are errors.
- `utf-32`: BOM-aware (`FF FE 00 00` / `00 00 FE FF`), little-endian when no
  BOM; length must be a multiple of 4 and code points must be valid.
- `gbk` / `gb18030`: decoded with `encoding_rs`; invalid sequences are errors.

---

### string_to_binary

Encode text into binary data.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `text` | string | yes | — | Text to encode |
| `encoding` | string | no | `utf-8` | `utf-8`, `ascii`, `utf-16`, `utf-32`, `gbk`, `gb18030` |

Output: base64-encoded binary (JSON Schema `binary` format).

- `utf-8`: raw UTF-8 bytes.
- `ascii`: non-ASCII text is an error.
- `utf-16`: little-endian with BOM (`FF FE`).
- `utf-32`: little-endian with BOM (`FF FE 00 00`).
- `gbk` / `gb18030`: characters not representable in the target encoding are
  errors (GB18030 covers more Unicode than GBK, e.g. emoji).

Round-trips with `binary_to_string` for all six encodings.

---

### change_case

Convert text to lowercase, uppercase, or title case.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `text` | string | yes | Text to convert |
| `case` | string (`lowercase`, `uppercase`, `titlecase`) | yes | Target case |

- `lowercase`: Unicode lowercase (`ÄÖÜ` → `äöü`).
- `uppercase`: Unicode uppercase (`straße` → `STRASSE`).
- `titlecase`: uppercases the first character of each whitespace-separated word
  and lowercases the rest; other characters are preserved.

Examples:

```json
{ "text": "Hello WORLD", "case": "lowercase" }
{ "text": "hello WORLD  from rust", "case": "titlecase" }
```

Output: the converted text (`hello world`, `Hello World  From Rust`).

---

### naming_case

Convert text between common programming naming conventions.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `text` | string | yes | Text to convert |
| `case` | string (`pascal`, `camel`, `lower_snake`, `upper_snake`) | yes | Target naming convention |

Word boundaries are detected from separators (`_`, `-`, spaces), camel-case
transitions, acronyms, and digits (e.g. `HTTPRequest` → `http_request`).

| `case` | Example output for `my_var_name` |
|---|---|
| `pascal` | `MyVarName` |
| `camel` | `myVarName` |
| `lower_snake` | `my_var_name` |
| `upper_snake` | `MY_VAR_NAME` |

Examples:

```json
{ "text": "my-var name", "case": "pascal" }
{ "text": "HTTPRequest", "case": "lower_snake" }
```

Output: the converted text (`MyVarName`, `http_request`).

## Regex

### regex_match

Find all non-overlapping regex matches in text.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `pattern` | string | yes | — | Rust `regex` syntax (no backreferences or look-around) |
| `text` | string | yes | — | Text to search |
| `limit` | integer (`1`–`1000`) | no | `100` | Maximum matches to return |
| `case_insensitive` | boolean | no | `false` | Case-insensitive matching |

The result is returned both as pretty-printed JSON text and as
`structuredContent`; the tool also declares an `outputSchema`.

| Field | Type | Meaning |
|---|---|---|
| `count` | integer | Matches returned |
| `truncated` | boolean | Whether more matches existed than `limit` |
| `matches` | array | Each with `text`, `start`, `end`, `groups`, and `named_groups` |

`start` / `end` are UTF-8 byte offsets; `groups` contains capture groups 1..N
(`null` for unmatched optional groups).

Example:

```json
{ "pattern": "(\\w+)-(\\d+)", "text": "abc-123 def-456" }
```

Returns two matches: `abc-123` (groups `["abc", "123"]`) and `def-456`.

---

### regex_replace

Replace regex matches in text.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `pattern` | string | yes | — | Rust `regex` syntax |
| `text` | string | yes | — | Text to transform |
| `replacement` | string | yes | — | Supports `$1`, `${name}`, and `$$` for a literal `$` |
| `limit` | integer (`0`–`10000`) | no | `0` | Maximum replacements; `0` = replace all |
| `case_insensitive` | boolean | no | `false` | Case-insensitive matching |

Result: `{ "result": "...", "replacements": N }`.

Example: pattern `(\w+)-(\d+)`, text `abc-123`, replacement `$2:$1` → `123:abc`.

---

### regex_test

Test whether text matches a regex.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `pattern` | string | yes | — | Rust `regex` syntax |
| `text` | string | yes | — | Text to test |
| `case_insensitive` | boolean | no | `false` | Case-insensitive matching |

Result: `{ "matched": bool, "match": ... | null }`; `match` contains the first
match details (`text`, `start`, `end`, `groups`, `named_groups`).

Invalid regexes return `isError: true` for all three tools.

## JSON

### json_pretty

Pretty-print a JSON string.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `json` | string | yes | — | JSON text to format |
| `indent` | integer (`0`–`8`) | no | `2` | Spaces per indentation level; `0` keeps newlines without indentation |
| `unicode` | string (`characters`, `escapes`) | no | `characters` | `characters` decodes `\uXXXX` to real characters; `escapes` emits non-ASCII as `\uXXXX` |

- Input is parsed and re-serialized, so object key order is preserved.
- `unicode: "characters"` (default): decodes `\uXXXX` escape sequences to real
  UTF-8 characters, e.g. `"你好"` → `"你好"`.
- `unicode: "escapes"`: emits non-ASCII characters as lowercase `\uXXXX`
  escapes; code points above U+FFFF use surrogate pairs (e.g. `😀` →
  `😀`).
- Invalid JSON returns `isError: true` with the line and column.

Example input:

```json
{"a":1,"b":[true,null,"你好"]}
```

Output with defaults:

```json
{
  "a": 1,
  "b": [
    true,
    null,
    "你好"
  ]
}
```

---

### json_minify

Minify a JSON string.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `json` | string | yes | — | JSON text to minify |
| `unicode` | string (`characters`, `escapes`) | no | `characters` | `characters` keeps real UTF-8 characters; `escapes` emits non-ASCII as `\uXXXX` |

- Removes all insignificant whitespace.
- Object key order is preserved.
- Invalid JSON returns `isError: true` with the line and column.
- With `unicode: "escapes"`, code points above U+FFFF use surrogate pairs
  (e.g. `😀` → `😀`).

Example input:

```json
{
  "a": 1,
  "b": [true, null, "你好"]
}
```

Output:

```json
{"a":1,"b":[true,null,"你好"]}
```

---

### json_query

Query a JSON document with a JSONPath expression (RFC 9535).

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `json` | string | yes | — | JSON text to query |
| `path` | string | yes | — | JSONPath expression, e.g. `$.store.book[*].title` |
| `limit` | integer (`1`–`1000`) | no | `100` | Maximum number of matches to return |

- Supports RFC 9535 selectors: child (`$.a`), wildcard (`$.*`, `$[*]`),
  recursive descent (`$..price`), slices, unions, and filters
  (`[?(@.price > 10)]`).
- Invalid JSON or an invalid JSONPath returns `isError: true`.
- The result is returned both as pretty-printed JSON text and as
  `structuredContent`; the tool also declares an `outputSchema`.

| Field | Type | Meaning |
|---|---|---|
| `count` | integer | Number of matches returned |
| `truncated` | boolean | Whether more matches existed than `limit` |
| `matches` | array | Matched JSON values |

Example:

```json
{
  "json": "{\"store\":{\"book\":[{\"title\":\"A\"},{\"title\":\"B\"}]}}",
  "path": "$.store.book[*].title"
}
```

Output:

```json
{
  "count": 2,
  "truncated": false,
  "matches": ["A", "B"]
}
```

---

### json_validate

Validate whether a string is legal JSON.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `json` | string | yes | JSON text to validate |

The result is returned both as pretty-printed JSON text and as
`structuredContent`; the tool also declares an `outputSchema` in `tools/list`.
Invalid JSON is a normal result (`valid: false`), not an `isError`.

| Field | Type | Meaning |
|---|---|---|
| `valid` | boolean | Whether the input is legal JSON |
| `line` | integer \| null | 1-based error line, or `null` when valid |
| `column` | integer \| null | 1-based error column, or `null` when valid |
| `error` | string \| null | Parser error message, or `null` when valid |

Example for `{"a":}`:

```json
{
  "valid": false,
  "line": 1,
  "column": 6,
  "error": "expected value"
}
```

---

### json_schema_validate

Validate a JSON document against a JSON Schema.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `json` | string | yes | JSON text to validate |
| `json_schema` | any | yes | JSON Schema (object/boolean, or a JSON-encoded string) |

The schema and the JSON are both parsed first; invalid JSON or an invalid
schema returns `isError: true`. Schema violations are a normal result
(`valid: false`).

The result is returned both as pretty-printed JSON text and as
`structuredContent`; the tool also declares an `outputSchema` in `tools/list`.

| Field | Type | Meaning |
|---|---|---|
| `valid` | boolean | Whether the JSON conforms to the schema |
| `schema_errors` | array | Schema violations (up to 100), each with `path`, `schema_path`, and `message` |

Example: validating `{"age": 42}` against
`{"type":"object","properties":{"age":{"type":"integer"}},"required":["age"]}`
returns `valid: true` and an empty `schema_errors`. `{"age": "old"}` returns a
`schema_errors` entry with `path: "/age"`.

## Digest

### digest

Compute a hash digest of binary data.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Base64-encoded binary |
| `algorithm` | string (`md5`, `sha1`, `sha256`, `sha384`, `sha512`) | yes | Hash algorithm |

Output: the digest as base64-encoded binary (JSON Schema `binary` format).

Digest sizes: MD5 = 16 bytes, SHA-1 = 20 bytes, SHA-256 = 32 bytes,
SHA-384 = 48 bytes, SHA-512 = 64 bytes.

Reference values for the ASCII input `abc` (`data` = `YWJj`):

| algorithm | hex digest |
|---|---|
| `md5` | `900150983cd24fb0d6963f7d28e17f72` |
| `sha1` | `a9993e364706816aba3e25717850c26c9cd0d89d` |
| `sha256` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |
| `sha384` | `cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7` |
| `sha512` | `ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f` |

Arguments:

```json
{ "data": "YWJj", "algorithm": "sha256" }
```

---

### hmac

Compute a keyed-hash message authentication code.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Message as base64-encoded binary |
| `key` | string (`format: binary`) | yes | Secret key as base64-encoded binary; any length |
| `algorithm` | string (`md5`, `sha1`, `sha256`, `sha384`, `sha512`) | yes | Hash algorithm |

Output: the MAC as base64-encoded binary (JSON Schema `binary` format).
MAC sizes match the digest sizes: 16 / 20 / 32 / 48 / 64 bytes.

Reference values for `key` = twenty `0x0b` bytes
(`CwsLCwsLCwsLCwsLCwsLCwsLCws=`) and `data` = `Hi There`
(`SGkgVGhlcmU=`):

| algorithm | hex MAC |
|---|---|
| `md5` | `5ccec34ea9656392457fa1ac27f08fbc` |
| `sha1` | `b617318655057264e28bc0b6fb378c8ef146be00` |
| `sha256` | `b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7` |
| `sha384` | `afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59cfaea9ea9076ede7f4af152e8b2fa9cb6` |
| `sha512` | `87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854` |

Arguments:

```json
{
  "data": "SGkgVGhlcmU=",
  "key": "CwsLCwsLCwsLCwsLCwsLCwsLCws=",
  "algorithm": "sha256"
}
```

This tool covers the HMAC family for the same hash algorithms as `digest`.
For text keys or messages, encode them first with `string_to_binary`.

## Symmetric

### aes_generate_key

Generate a random AES key using the operating system RNG.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `bits` | integer (`128`, `192`, `256`) | no | `256` | Key size in bits |

Output: base64-encoded binary key.

```json
{ "bits": 256 }
```

---

### aes_generate_iv

Generate a random AES IV/nonce using the operating system RNG.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `mode` | string (`cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | yes | AES mode the IV will be used with |

Output: base64-encoded binary.

- 16 bytes for `cbc`, `ctr`, `cfb`, `ofb`
- 12 bytes for `gcm`

```json
{ "mode": "gcm" }
```

---

### aes_encrypt

Encrypt binary data with AES.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Plaintext |
| `key` | string (`format: binary`) | yes | AES key: 16, 24, or 32 bytes |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | yes | AES mode |
| `padding` | string (`pkcs7`, `none`) | yes | Padding mode |
| `iv` | string (`format: binary`) | no | 16-byte IV for cbc/ctr/cfb/ofb, 12-byte nonce for gcm; ignored for ecb |

Output: ciphertext as base64 binary.

- Key size selects AES-128 (16 bytes), AES-192 (24 bytes), or AES-256 (32 bytes).
- `pkcs7` is supported only for `ecb` and `cbc`; other modes require `none`.
- With `none` padding, `ecb`/`cbc` input must be a multiple of 16 bytes.
- GCM output is `ciphertext || tag` (16-byte tag appended).

```json
{
  "data": "aGVsbG8=",
  "key": "MDEyMzQ1Njc4OWFiY2RlZg==",
  "mode": "gcm",
  "padding": "none",
  "iv": "MDEyMzQ1Njc4OWFi"
}
```

---

### aes_decrypt

Decrypt AES ciphertext.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Ciphertext |
| `key` | string (`format: binary`) | yes | AES key: 16, 24, or 32 bytes |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | yes | AES mode |
| `padding` | string (`pkcs7`, `none`) | yes | Padding mode |
| `iv` | string (`format: binary`) | no | IV/nonce (see `aes_encrypt`) |

Output: plaintext as base64 binary.

For GCM, `data` must be `ciphertext || tag`; authentication failure returns
`isError: true`. Invalid PKCS#7 padding also returns `isError: true`.

---

### des_generate_key

Generate a random DES or Triple DES key using the operating system RNG.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `key_size` | string (`des`, `3des-2key`, `3des-3key`) | no | `3des-3key` | Key variant |

Output: base64-encoded binary key.

- `des` → 8 bytes
- `3des-2key` → 16 bytes
- `3des-3key` → 24 bytes

```json
{ "key_size": "3des-3key" }
```

---

### des_generate_iv

Generate a random 8-byte DES/3DES IV using the operating system RNG.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `mode` | string (`cbc`, `ctr`, `cfb`, `ofb`) | yes | DES mode the IV will be used with |

Output: base64-encoded binary (8 bytes).

```json
{ "mode": "cbc" }
```

---

### des_encrypt

Encrypt binary data with DES or Triple DES.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Plaintext |
| `key` | string (`format: binary`) | yes | 8 bytes (DES), 16 bytes (2-key 3DES), or 24 bytes (3-key 3DES) |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`) | yes | DES mode |
| `padding` | string (`pkcs7`, `none`) | yes | Padding mode |
| `iv` | string (`format: binary`) | no | 8-byte IV for cbc/ctr/cfb/ofb; ignored for ecb |

Output: ciphertext as base64 binary.

- Key length selects DES (8), 2-key 3DES (16), or 3-key 3DES (24).
- `pkcs7` is supported only for `ecb` and `cbc`; other modes require `none`.
- With `none` padding, `ecb`/`cbc` input must be a multiple of 8 bytes.

> **Security warning:** DES (56-bit) is cryptographically broken and 3DES is
> legacy. Use AES for new designs.

```json
{
  "data": "aGVsbG8=",
  "key": "MTIzNDU2Nzg=",
  "mode": "cbc",
  "padding": "pkcs7",
  "iv": "MDEyMzQ1Njc="
}
```

---

### des_decrypt

Decrypt DES or Triple DES ciphertext.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Ciphertext |
| `key` | string (`format: binary`) | yes | 8, 16, or 24 bytes |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`) | yes | DES mode |
| `padding` | string (`pkcs7`, `none`) | yes | Padding mode |
| `iv` | string (`format: binary`) | no | IV (see `des_encrypt`) |

Output: plaintext as base64 binary. Invalid PKCS#7 padding returns
`isError: true`.

## RSA

### RSA tools

Shared conventions:

- **Private keys**: unencrypted PEM, PKCS#8 (`BEGIN PRIVATE KEY`) or PKCS#1
  (`BEGIN RSA PRIVATE KEY`).
- **Public keys**: unencrypted PEM, SPKI (`BEGIN PUBLIC KEY`) or PKCS#1
  (`BEGIN RSA PUBLIC KEY`).
- **Binary values** (`data`, `signature`) are base64 strings with
  `format: "binary"`.
- 2048-bit keys or larger are recommended.
- OAEP and PSS use **SHA-256**; PKCS#1 v1.5 signatures also use SHA-256.
- **Raw mode** (`raw`) is available for debugging and interop. It performs RSA
  with no padding and requires input blocks exactly `k` bytes long (the modulus
  length). Raw RSA is deterministic and malleable; do not use it for real
  encryption or signatures without a reviewed scheme.

---

### rsa_encrypt

Encrypt binary data with an RSA public key.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Plaintext |
| `public_key` | string | yes | RSA public key PEM |
| `padding` | string (`pkcs1v15`, `oaep-sha256`, `raw`) | yes | Padding scheme |

Output: ciphertext as base64 binary.

Prefer `oaep-sha256`. The maximum plaintext size depends on the key size and
padding (`k - 11` for PKCS#1 v1.5, `k - 2*32 - 2` for OAEP-SHA256 with a
2048-bit key).

`raw` performs `m^e mod n` directly. `data` must be exactly `k` bytes
(modulus length, big-endian) and numerically smaller than the modulus;
output is exactly `k` bytes.

```json
{ "data": "aGVsbG8=", "public_key": "-----BEGIN PUBLIC KEY-----\n...", "padding": "oaep-sha256" }
```

---

### rsa_decrypt

Decrypt RSA ciphertext with a private key.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Ciphertext |
| `private_key` | string | yes | RSA private key PEM |
| `padding` | string (`pkcs1v15`, `oaep-sha256`, `raw`) | yes | Padding scheme |

Output: plaintext as base64 binary.

`raw` performs `c^d mod n` directly and returns the full `k`-byte block; the
ciphertext must be exactly `k` bytes.

> **PKCS#1 v1.5 note:** decryption uses implicit rejection. Invalid padding
> returns a deterministic synthetic message instead of an error; the enclosing
> protocol must authenticate the result. OAEP reports invalid ciphertext as
> `isError: true`.

---

### rsa_sign

Sign binary data with an RSA private key.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Message |
| `private_key` | string | yes | RSA private key PEM |
| `padding` | string (`pkcs1v15-sha256`, `pss-sha256`, `raw`) | yes | Signature scheme |

Output: signature as base64 binary. Prefer `pss-sha256`.

`raw` applies the RSA private-key operation (`RSASP1`) to `data` directly.
`data` must be the fully encoded EMSA block, exactly `k` bytes and smaller
than the modulus. The caller is responsible for building and validating the
encoding (e.g. DigestInfo or EMSA-PSS).

```json
{ "data": "aGVsbG8=", "private_key": "-----BEGIN PRIVATE KEY-----\n...", "padding": "pss-sha256" }
```

---

### rsa_verify

Verify an RSA signature.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `data` | string (`format: binary`) | yes | Message |
| `signature` | string (`format: binary`) | yes | Signature |
| `public_key` | string | yes | RSA public key PEM |
| `padding` | string (`pkcs1v15-sha256`, `pss-sha256`, `raw`) | yes | Signature scheme |

Output: the text `"true"` or `"false"`.

An invalid signature returns `"false"`; malformed keys, base64, or other
operational errors return `isError: true`.

`raw` computes `s^e mod n` from `signature` and compares it with `data`.
Both `data` and `signature` must be exactly `k` bytes; an out-of-range
signature returns `"false"`.

---

### rsa_generate_key

Generate a new RSA private/public key pair.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `bits` | integer | no | `2048` | Modulus size in bits (1024–8192) |
| `format` | string (`pkcs1`, `pkcs8`) | no | `pkcs8` | Private key output format |

The result is returned both ways:

- `content`: pretty-printed JSON text
- `structuredContent`: the same object

The tool also declares an `outputSchema` in `tools/list` matching this object
(`bits`, `format`, `private_key`, `public_key` all required).

```json
{
  "bits": 2048,
  "format": "pkcs8",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...",
  "public_key": "-----BEGIN PUBLIC KEY-----\n..."
}
```

- `format`: the selected private key format
- `private_key`: the private key in that format (unencrypted PEM)
  - `pkcs8` → `BEGIN PRIVATE KEY`
  - `pkcs1` → `BEGIN RSA PRIVATE KEY`
- `public_key`: **SPKI PEM** (`BEGIN PUBLIC KEY`)

3072 or 4096 bits are recommended for stronger security. Key generation for
4096+ bits can take noticeably longer. `rsa_private_to_public` can derive a
public key from an existing private key, and `rsa_convert_private_key` can
convert between the two private key formats.

```json
{ "bits": 3072 }
```

---

### rsa_private_to_public

Derive an RSA public key from a private key.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `private_key` | string | yes | Unencrypted private key PEM (PKCS#8 or PKCS#1) |

Output: a **SPKI public key PEM** (`BEGIN PUBLIC KEY`).

```json
{ "private_key": "-----BEGIN PRIVATE KEY-----\n..." }
```

---

### rsa_convert_private_key

Convert an RSA private key between PEM formats.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `private_key` | string | yes | Unencrypted private key PEM (PKCS#1 or PKCS#8) |
| `format` | string (`pkcs1`, `pkcs8`) | yes | Target format |

Output: the converted private key PEM.

- `pkcs8` → `BEGIN PRIVATE KEY`
- `pkcs1` → `BEGIN RSA PRIVATE KEY`

The key material is unchanged; only the container format is re-encoded.

```json
{ "private_key": "-----BEGIN PRIVATE KEY-----\n...", "format": "pkcs1" }
```

---

### rsa_key_components

Extract the numeric components of an RSA key.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `key` | string | yes | Unencrypted private or public key PEM |

The result is returned both as pretty-printed JSON text and as
`structuredContent`. The tool declares an `outputSchema` in `tools/list`:
`key_type`, `bits`, `n`, `e` are required; the private components are
optional in the schema.

Fields:

| Field | Present for | Meaning |
|---|---|---|
| `key_type` | always | `"private"` or `"public"` |
| `bits` | always | Modulus bit length |
| `n` | always | Modulus |
| `e` | always | Public exponent |
| `d` | private | Private exponent |
| `p`, `q` | private | Prime factors |
| `dp` | private | `d mod (p-1)` (OpenSSL `dmp1`) |
| `dq` | private | `d mod (q-1)` (OpenSSL `dmq1`) |
| `qinv` | private | `q^-1 mod p` (OpenSSL `iqmp`) |

All values are lowercase hex, minimal big-endian (no leading zero bytes).

Example:

```json
{
  "key_type": "private",
  "bits": 2048,
  "n": "c3a1...",
  "e": "010001",
  "d": "1f2e...",
  "p": "...",
  "q": "...",
  "dp": "...",
  "dq": "...",
  "qinv": "..."
}
```

> **Security warning:** for private keys, `d`, `p`, `q`, `dp`, `dq`, and
> `qinv` are secret key material. Do not log or transmit them.

## Misc

### generate_uuid

Generate a UUID/GUID.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `version` | integer (`4`, `7`) | yes | — | 4 = random, 7 = time-ordered |
| `uppercase` | boolean | no | `false` | Return uppercase |
| `no_hyphens` | boolean | no | `false` | `true` removes `-`; `false` keeps it |

Arguments:

```json
{ "version": 7, "uppercase": true, "no_hyphens": false }
```

Output: UUID text, e.g. `01A20175-C53C-7128-A588-7FE1080D1C8D`.

---

### random_number

Generate a random integer or floating-point number from the operating system
RNG. `decimals` selects the mode.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `min` | number | no | `0` (integer) / `0.0` (float) | Lower bound (inclusive) |
| `max` | number | no | `100` (integer) / `1.0` (float) | Upper bound (inclusive for integers, exclusive for floats) |
| `decimals` | integer (`0`–`15`) | no | `0` | `0` = integer; `1`–`15` = float with that many decimals |

- `decimals: 0`: returns an integer uniform in `[min, max]`. Bounds default to
  `0` and `100` and must be finite whole numbers within ±2^53.
- `decimals: 1`–`15`: returns a float uniform in `[min, max)`, rounded to
  `decimals` places. Bounds default to `0.0` and `1.0`; if `min == max`, that
  value is returned.

Examples:

```json
{ "min": 1, "max": 6 }
{ "min": 0, "max": 1, "decimals": 4 }
```

Output: the generated number as text.

---

### random_string

Generate a random string from the operating system RNG.

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `length` | integer (`1`–`1024`) | no | `16` | String length |
| `charset` | string (`alphanumeric`, `alphabetic`, `numeric`, `hex`) | no | `alphanumeric` | Character set |

Example:

```json
{ "length": 32, "charset": "hex" }
```

Output: the generated string.
