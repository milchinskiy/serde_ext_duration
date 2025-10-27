use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(with = "serde_ext_duration")] // human out by default; flexible in
    t: Duration,
}

#[derive(Deserialize)]
struct HumanOnly {
    #[serde(with = "serde_ext_duration::human")] // still flexible on input
    t: Duration,
}

#[test]
fn int_seconds() {
    let v: Root = serde_json::from_str(r#"{ "t": 5 }"#).unwrap();
    assert_eq!(v.t, Duration::from_secs(5));
}

#[test]
fn float_ms_rounding() {
    let v: Root = serde_json::from_str(r#"{ "t": 1.234 }"#).unwrap();
    assert_eq!(v.t, Duration::from_secs(1) + Duration::from_millis(234));

    // rounding carry (1.9996s -> 2s)
    let v: Root = serde_json::from_str(r#"{ "t": 1.9996 }"#).unwrap();
    assert_eq!(v.t, Duration::from_secs(2));
}

#[test]
fn string_units_hms() {
    let v: HumanOnly = serde_json::from_str(r#"{ "t": "1h 23m 45s" }"#).unwrap();
    assert_eq!(v.t, Duration::from_secs(3600 + 23 * 60 + 45));
}

#[test]
fn string_units_mix_and_order_free() {
    let a: Root = serde_json::from_str(r#"{ "t": "1m250ms" }"#).unwrap();
    let b: Root = serde_json::from_str(r#"{ "t": "30m 1h" }"#).unwrap(); // order shouldn't matter
    assert_eq!(a.t, Duration::from_secs(60) + Duration::from_millis(250));
    assert_eq!(b.t, Duration::from_secs(90 * 60));
}

#[test]
fn string_days_hours() {
    let v: Root = serde_yaml::from_str("t: '1d 2h'").unwrap();
    assert_eq!(v.t, Duration::from_secs(24 * 3600 + 2 * 3600));
}

#[test]
fn error_negative_int() {
    let err = serde_json::from_str::<Root>(r#"{ "t": -1 }"#).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("negative"));
}

#[test]
fn error_negative_float() {
    let err = serde_json::from_str::<Root>(r#"{ "t": -0.1 }"#).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("negative"));
}

#[test]
fn error_empty_string() {
    let err = serde_json::from_str::<Root>(r#"{ "t": "  \t  " }"#).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("empty duration"));
}

#[test]
fn error_unknown_unit() {
    let err = serde_json::from_str::<Root>(r#"{ "t": "5q" }"#).unwrap_err();
    assert!(err.to_string().contains("unknown unit"));
}
