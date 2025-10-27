# `serde_ext_duration`

Flexible Serde (de)serializers for [`std::time::Duration`].

- **Input** (any of these accepted on *deserialization*):
  - **integer** → seconds
  - **float** → `seconds + fractional·1000ms` (rounded to nearest millisecond)
  - **string** → human tokens with units `d`, `h`, `m`, `s`, `ms` (case‑insensitive, order‑free, whitespace optional), e.g. `"1h 23m 45s"`, `"30m 1h"`, `"1m250ms"`, `"250ms"`.
- **Output** (choose one *serialization* shape via `#[serde(with = ...)]`):
  - `human` → canonical human string, e.g. `"1h 2m 3s 250ms"`
  - `secs` → integer seconds (`u64`)
  - `millis` → integer milliseconds (`u64`, ms‑rounded)
  - `secs_f64_ms` → `f64` seconds with millisecond precision (3 decimals)

[`std::time::Duration`]: https://doc.rust-lang.org/std/time/struct.Duration.html

---

## Quick start

**Human output (default) with flexible input**

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Cfg {
    #[serde(with = "serde_ext_duration")] // human out; flexible in
    timeout: Duration,
}
```

**Pick a specific output shape**

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Cfg {
    #[serde(with = "serde_ext_duration::human")]      a: Duration, // "1h 2m 3s 250ms"
    #[serde(with = "serde_ext_duration::secs")]       b: Duration, // 3723
    #[serde(with = "serde_ext_duration::millis")]     c: Duration, // 3723250
    #[serde(with = "serde_ext_duration::secs_f64_ms")] d: Duration, // 3723.250
}
```

**Optional newtype**

```rust
use serde::{Deserialize, Serialize};
use serde_ext_duration::ExtDuration; // Serialize → human; Deserialize ← flexible
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Cfg { t: ExtDuration }

let cfg = Cfg { t: ExtDuration(Duration::from_millis(65_000)) };
let json = serde_json::to_string(&cfg)?; // {"t":"1m 5s"}
let back: Cfg = serde_json::from_str(&json)?; // round‑trips
```

---

## `Option<Duration>` support (`opt` module)

Serde does **not** automatically apply `with = "…"` to the inner type of an `Option<T>`. The provided `opt` modules target the field type `Option<Duration>` directly.

> **Important:** when using `#[serde(with = ...)]` on an `Option<…>` field and you want missing fields to become `None`, add `#[serde(default)]`.

### Human output (root)

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Cfg {
  // missing or null -> None; otherwise parse int/float/string -> Some(Duration)
  #[serde(default, with = "serde_ext_duration::opt")]
  timeout: Option<std::time::Duration>,
}
```

### Choose specific output shapes for `Option<Duration>`

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Cfg {
  #[serde(default, with = "serde_ext_duration::opt::human")]      a: Option<std::time::Duration>,
  #[serde(default, with = "serde_ext_duration::opt::secs")]       b: Option<std::time::Duration>,
  #[serde(default, with = "serde_ext_duration::opt::millis")]     c: Option<std::time::Duration>,
  #[serde(default, with = "serde_ext_duration::opt::secs_f64_ms")] d: Option<std::time::Duration>,
}
```

### Elide `None` at serialization

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Cfg {
  #[serde(default, with = "serde_ext_duration::opt")]
  #[serde(skip_serializing_if = "Option::is_none")]
  timeout: Option<std::time::Duration>,
}
```

### Alternative: let Serde handle missing/`None` with a newtype

If you prefer no `with` attributes at all:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Cfg {
  timeout: Option<serde_ext_duration::ExtDuration>, // ExtDuration implements Serde itself
}
```

Access the inner duration via `timeout.map(|d| d.0)`.

---

## String grammar

- Grammar is a sequence of `<unsigned-integer><unit>` tokens, separated by optional ASCII whitespace.
- Units (case‑insensitive): `d` (days), `h` (hours), `m` (minutes), `s` (seconds), `ms` (milliseconds).
- Order is free: `"30m 1h"` equals `"1h 30m"`.
- Empty strings, unknown units, and negative numbers are rejected.

Examples:

```
"1h 23m 45s"
"90m"          # 1h 30m
"1m250ms"
"250ms"
```

---

## Behavior

- **Deserialization**
  - **Integers** are seconds.
  - **Floats** are seconds; the fractional part is interpreted as **milliseconds** and rounded to the nearest ms. `1.9996` → `2.000s`.
  - **Strings** follow the grammar above. Mixed units accumulate; overflow is detected and reported.
  - **Negatives** (ints/floats) and **non‑finite floats** are rejected.

- **Serialization**
  - `human` produces a minimal canonical sequence `d h m s ms`, omitting zero parts; zero duration renders as `"0s"`.
  - `secs` truncates sub‑second parts (same as `Duration::as_secs`).
  - `millis` rounds to nearest millisecond and returns a `u64` count.
  - `secs_f64_ms` rounds to 3 decimals (millisecond precision) to avoid implying higher precision.

---

## Examples

**JSON (human)**

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Job { #[serde(with = "serde_ext_duration")] timeout: std::time::Duration }

let src = Job { timeout: std::time::Duration::from_millis(3_723_250) };
let json = serde_json::to_string(&src)?;       // {"timeout":"1h 2m 3s 250ms"}
let dst: Job = serde_json::from_str(&json)?;   // round‑trip OK
```

**YAML (mixed inputs)**

```rust
#[derive(serde::Deserialize, Debug)]
struct Mixed {
    #[serde(with = "serde_ext_duration")] a: std::time::Duration, // string
    #[serde(with = "serde_ext_duration")] b: std::time::Duration, // float
    #[serde(with = "serde_ext_duration")] c: std::time::Duration, // int
}

let doc = r#"
---
a: "1h 2m 3s"
b: 2.5
c: 42
"#;
let val: Mixed = serde_yaml::from_str(doc)?;
```

---

## MSRV & features

- **MSRV**: aims to work on stable Rust 1.70+ (no special features). If you rely on an older compiler, adjust as needed.
- **no_std**: not supported (uses `std::time::Duration`).

---

## License

Dual‑licensed under either:

- MIT — see `LICENSE-MIT`
- Apache‑2.0 — see `LICENSE-APACHE`

at your option.

