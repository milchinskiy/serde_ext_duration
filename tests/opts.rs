use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Debug, PartialEq)]
struct InHumanRoot {
    #[serde(default, with = "serde_ext_duration::opt")] // human out; flexible in
    timeout: Option<Duration>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct InSecs {
    #[serde(with = "serde_ext_duration::opt::secs")] // output shape irrelevant for input
    timeout: Option<Duration>,
}

#[test]
fn opt_missing_and_null() {
    // missing -> None
    let a: InHumanRoot = serde_json::from_str(r#"{}"#).unwrap();
    assert_eq!(a.timeout, None);

    // null -> None
    let b: InHumanRoot = serde_json::from_str(r#"{"timeout": null}"#).unwrap();
    assert_eq!(b.timeout, None);
}

#[test]
fn opt_some_from_string_float_int() {
    // string units
    let v: InHumanRoot = serde_json::from_str(r#"{"timeout":"1m 250ms"}"#).unwrap();
    assert_eq!(v.timeout, Some(Duration::from_secs(60) + Duration::from_millis(250)));

    // float seconds (ms precision)
    let v: InHumanRoot = serde_json::from_str(r#"{"timeout": 1.234}"#).unwrap();
    assert_eq!(v.timeout, Some(Duration::from_secs(1) + Duration::from_millis(234)));

    // int seconds
    let v: InSecs = serde_json::from_str(r#"{"timeout": 42}"#).unwrap();
    assert_eq!(v.timeout, Some(Duration::from_secs(42)));
}

#[test]
fn opt_errors_propagate() {
    // unknown unit
    let err = serde_json::from_str::<InHumanRoot>(r#"{"timeout":"3q"}"#).unwrap_err();
    assert!(err.to_string().contains("unknown unit"));

    // negative
    let err = serde_json::from_str::<InHumanRoot>(r#"{"timeout":-1}"#).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("negative"));
}

#[derive(Serialize)]
struct OutHumanRoot {
    #[serde(with = "serde_ext_duration::opt")] // human
    timeout: Option<Duration>,
}

#[derive(Serialize)]
struct OutSecs {
    #[serde(with = "serde_ext_duration::opt::secs")] // u64 seconds
    timeout: Option<Duration>,
}

#[derive(Serialize)]
struct OutMillis {
    #[serde(with = "serde_ext_duration::opt::millis")] // u64 milliseconds
    timeout: Option<Duration>,
}

#[derive(Serialize)]
struct OutF64 {
    #[serde(with = "serde_ext_duration::opt::secs_f64_ms")] // f64 seconds
    timeout: Option<Duration>,
}

#[test]
fn opt_serialize_human_some_and_none() {
    let some = OutHumanRoot { timeout: Some(Duration::from_millis(65_000)) };
    let j = serde_json::to_string(&some).unwrap();
    assert!(j.contains("\"1m 5s\""));

    let none = OutHumanRoot { timeout: None };
    let j = serde_json::to_string(&none).unwrap();
    assert!(j.contains("null"));
}

#[test]
fn opt_serialize_secs_truncates_subsec() {
    let v = OutSecs { timeout: Some(Duration::from_millis(1234)) };
    let j = serde_json::to_string(&v).unwrap();
    assert!(j.contains("\"timeout\":1")); // 1.234s -> 1
}

#[test]
fn opt_serialize_millis_rounding() {
    let v = OutMillis { timeout: Some(Duration::new(1, 999_500_000)) }; // 1.9995s
    let j = serde_json::to_string(&v).unwrap();
    assert!(j.contains("\"timeout\":2000"));
}

#[test]
fn opt_serialize_secs_f64_ms_precision() {
    let v = OutF64 { timeout: Some(Duration::from_millis(1234)) };
    let j = serde_json::to_string(&v).unwrap();
    assert!(j.contains("\"timeout\":1.234"));
}

#[derive(Serialize)]
struct OutSkipNone {
    #[serde(with = "serde_ext_duration::opt")]
    #[serde(skip_serializing_if = "Option::is_none")] // elide if None
    timeout: Option<Duration>,
}

#[test]
fn opt_skip_serializing_if_none() {
    let v = OutSkipNone { timeout: None };
    let j = serde_json::to_string(&v).unwrap();
    assert_eq!(j, "{}");
}
