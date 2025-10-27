// SPDX-License-Identifier: MIT OR Apache-2.0
//! serde_ext_duration — flexible Duration deserializer + multiple serializers.
//!
//! Use the concise `#[serde(with = "...")]` attribute:
//! - `#[serde(with = "serde_ext_duration")]`          → human output (root alias)
//! - `#[serde(with = "serde_ext_duration::human")]`   → human output
//! - `#[serde(with = "serde_ext_duration::secs")]`    → u64 seconds
//! - `#[serde(with = "serde_ext_duration::millis")]`  → u64 milliseconds
//! - `#[serde(with = "serde_ext_duration::secs_f64_ms")]` → f64 seconds (3 decimals)
//!
//! Deserialization accepts **int / float / string** (units: d, h, m, s, ms).

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::{fmt, time::Duration};

/// Flexible deserializer: int (secs), float (secs.millis, rounded), or string tokens (d/h/m/s/ms).
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurVisitor;
    impl Visitor<'_> for DurVisitor {
        type Value = Duration;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("integer seconds, float seconds.millis, or a string like '1h 23m 45s' / '123s' / '250ms'")
        }
        fn visit_u64<E>(self, v: u64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs(v))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            if v < 0 {
                return Err(E::custom("negative duration not allowed"));
            }
            Ok(Duration::from_secs(v as u64))
        }
        fn visit_f64<E>(self, v: f64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            if !v.is_finite() {
                return Err(E::custom("non-finite float"));
            }
            if v < 0.0 {
                return Err(E::custom("negative duration not allowed"));
            }
            let secs_trunc = v.trunc() as u64;
            let frac = v - (secs_trunc as f64);
            let mut millis = (frac * 1000.0).round() as u64;
            let mut secs = secs_trunc;
            if millis == 1000 {
                secs = secs.checked_add(1).ok_or_else(|| E::custom("duration overflow"))?;
                millis = 0;
            }
            Duration::from_secs(secs)
                .checked_add(Duration::from_millis(millis))
                .ok_or_else(|| E::custom("duration overflow"))
        }
        fn visit_str<E>(self, s: &str) -> Result<Duration, E>
        where
            E: de::Error,
        {
            parse_str(s).map_err(E::custom)
        }
        fn visit_string<E>(self, s: String) -> Result<Duration, E>
        where
            E: de::Error,
        {
            self.visit_str(&s)
        }
    }
    deserializer.deserialize_any(DurVisitor)
}

/// Root `serialize`: human format (so `#[serde(with = "serde_ext_duration")]` works).
pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serialize_human(dur, serializer)
}

pub fn serialize_human<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&to_human_string(dur))
}

pub fn serialize_secs<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(dur.as_secs())
}

pub fn serialize_millis<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ms_total = (dur.as_secs() as u128) * 1000 + ((dur.subsec_nanos() as u128 + 500_000) / 1_000_000);
    if ms_total > u64::MAX as u128 {
        return Err(serde::ser::Error::custom("duration too large"));
    }
    serializer.serialize_u64(ms_total as u64)
}

pub fn serialize_secs_f64_ms<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let f = dur.as_secs() as f64 + (dur.subsec_millis() as f64) / 1000.0;
    let f = (f * 1000.0).round() / 1000.0; // 3 decimals
    serializer.serialize_f64(f)
}

/// Build a canonical human string out of a `Duration` with units d/h/m/s/ms.
fn to_human_string(dur: &Duration) -> String {
    // Round to nearest millisecond, then decompose.
    let mut ms_total: u128 = (dur.as_secs() as u128) * 1000 + ((dur.subsec_nanos() as u128 + 500_000) / 1_000_000);

    if ms_total == 0 {
        return "0s".to_string();
    }

    let day = 86_400_000u128;
    let hour = 3_600_000u128;
    let minute = 60_000u128;
    let second = 1_000u128;

    let mut parts = Vec::new();

    let d = ms_total / day;
    ms_total %= day;
    if d > 0 {
        parts.push(format!("{d}d"));
    }
    let h = ms_total / hour;
    ms_total %= hour;
    if h > 0 {
        parts.push(format!("{h}h"));
    }
    let m = ms_total / minute;
    ms_total %= minute;
    if m > 0 {
        parts.push(format!("{m}m"));
    }
    let s = ms_total / second;
    ms_total %= second;
    if s > 0 {
        parts.push(format!("{s}s"));
    }
    let ms = ms_total;
    if ms > 0 {
        parts.push(format!("{ms}ms"));
    }

    parts.join(" ")
}

/// Human: `serialize` + flexible `deserialize`.
pub mod human {
    use super::*;
    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_human(d, s)
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize(d)
    }
}

/// Seconds (u64) on output; flexible input on deserialize.
pub mod secs {
    use super::*;
    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_secs(d, s)
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize(d)
    }
}

/// Milliseconds (u64) on output; flexible input on deserialize.
pub mod millis {
    use super::*;
    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_millis(d, s)
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize(d)
    }
}

/// Seconds as f64 (ms precision) on output; flexible input on deserialize.
pub mod secs_f64_ms {
    use super::*;
    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_secs_f64_ms(d, s)
    }
    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize(d)
    }
}

pub fn parse_str(s: &str) -> Result<Duration, String> {
    let mut total_ms: u128 = 0;
    let mut token_count: u32 = 0;
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let is_alpha = |b: u8| (b as char).is_ascii_alphabetic();

    while i < len {
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }
        let start_num = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == start_num {
            return Err(format!("expected number at position {start_num}"));
        }
        let n: u128 = s[start_num..i].parse().map_err(|_| format!("invalid number at position {start_num}"))?;
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let start_unit = i;
        while i < len && is_alpha(bytes[i]) {
            i += 1;
        }
        if start_unit == i {
            return Err(format!("expected unit after number at position {}", start_num));
        }
        let unit = s[start_unit..i].to_ascii_lowercase();
        let ms_per_unit: u128 = match unit.as_str() {
            "d" => 86_400_000,
            "h" => 3_600_000,
            "ms" => 1,
            "m" => 60_000,
            "s" => 1_000,
            _ => return Err(format!("unknown unit '{unit}' (use d, h, m, s, ms)")),
        };
        let inc = n.checked_mul(ms_per_unit).ok_or_else(|| "duration overflow".to_string())?;
        total_ms = total_ms.checked_add(inc).ok_or_else(|| "duration overflow".to_string())?;
        token_count += 1;
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
    }
    if token_count == 0 {
        return Err("empty duration string".into());
    }
    if total_ms > u64::MAX as u128 {
        return Err("duration too large".into());
    }
    Ok(Duration::from_millis(total_ms as u64))
}

// ===== Optional newtype (defaults to human on Serialize) =====
#[derive(Debug, Clone, Copy)]
pub struct ExtDuration(pub Duration);

impl<'de> Deserialize<'de> for ExtDuration {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(d).map(ExtDuration)
    }
}
impl Serialize for ExtDuration {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_human(&self.0, s)
    }
}

pub mod opt {
    use super::*;

    struct De(Duration);
    impl<'de> Deserialize<'de> for De {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(d).map(De)
        }
    }

    /// Root: human on serialize; flexible on deserialize.
    pub fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(d) => super::serialize_human(d, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Option::<De>::deserialize(d)?;
        Ok(inner.map(|De(d)| d))
    }

    /// Human variant
    pub mod human {
        use super::*;
        pub fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match v {
                Some(d) => super::super::serialize_human(d, s),
                None => s.serialize_none(),
            }
        }
        pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(d)
        }
    }

    /// Seconds (u64)
    pub mod secs {
        use super::*;
        pub fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match v {
                Some(d) => super::super::serialize_secs(d, s),
                None => s.serialize_none(),
            }
        }
        pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(d)
        }
    }

    /// Milliseconds (u64)
    pub mod millis {
        use super::*;
        pub fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match v {
                Some(d) => super::super::serialize_millis(d, s),
                None => s.serialize_none(),
            }
        }
        pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(d)
        }
    }

    /// Seconds as f64 (ms precision)
    pub mod secs_f64_ms {
        use super::*;
        pub fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match v {
                Some(d) => super::super::serialize_secs_f64_ms(d, s),
                None => s.serialize_none(),
            }
        }
        pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(d)
        }
    }
}
