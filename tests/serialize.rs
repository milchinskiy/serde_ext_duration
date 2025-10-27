use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct OutHuman {
    #[serde(with = "serde_ext_duration")]
    t: Duration,
}

#[derive(Serialize)]
struct OutSecs {
    #[serde(with = "serde_ext_duration::secs")]
    t: Duration,
}

#[derive(Serialize)]
struct OutMillis {
    #[serde(with = "serde_ext_duration::millis")]
    t: Duration,
}

#[derive(Serialize)]
struct OutF64 {
    #[serde(with = "serde_ext_duration::secs_f64_ms")]
    t: Duration,
}

#[test]
fn human_string_zero_and_basic() {
    // zero -> "0s"
    let j = serde_json::to_string(&OutHuman { t: Duration::from_millis(0) }).unwrap();
    assert!(j.contains("\"0s\""));

    let j = serde_json::to_string(&OutHuman { t: Duration::from_millis(65_000) }).unwrap();
    assert!(j.contains("\"1m 5s\""));
}

#[test]
fn secs_u64_trunc_subsec() {
    // 1234ms -> 1 second
    let j = serde_json::to_string(&OutSecs { t: Duration::from_millis(1234) }).unwrap();
    assert!(j.contains("\"t\":1"));
}

#[test]
fn millis_u64_rounded_and_boundary() {
    // exact ms
    let j = serde_json::to_string(&OutMillis { t: Duration::from_millis(1234) }).unwrap();
    assert!(j.contains("\"t\":1234"));

    // rounding across second boundary: 1.9995s ≈ 2000ms
    let j = serde_json::to_string(&OutMillis { t: Duration::new(1, 999_500_000) }).unwrap();
    assert!(j.contains("\"t\":2000"));
}

#[test]
fn secs_f64_ms_three_decimals() {
    let j = serde_json::to_string(&OutF64 { t: Duration::from_millis(1234) }).unwrap();
    assert!(j.contains("\"t\":1.234"));
}

#[derive(Serialize, Deserialize)]
struct Wrap {
    t: serde_ext_duration::ExtDuration,
}

#[test]
fn newtype_roundtrip_human() {
    let w = Wrap { t: serde_ext_duration::ExtDuration(Duration::from_millis(65_000)) };
    let j = serde_json::to_string(&w).unwrap();
    assert!(j.contains("\"1m 5s\""));
    let back: Wrap = serde_json::from_str(&j).unwrap();
    assert_eq!(back.t.0, Duration::from_millis(65_000));
}

#[derive(Serialize, Deserialize)]
struct RootWith {
    #[serde(with = "serde_ext_duration")] // human out; flexible in
    t: Duration,
}

#[test]
fn root_with_roundtrip() {
    let src = RootWith { t: Duration::from_millis(3_723_250) }; // 1h 2m 3s 250ms
    let j = serde_json::to_string(&src).unwrap();
    assert!(j.contains("1h 2m 3s 250ms"));
    let dst: RootWith = serde_json::from_str(&j).unwrap();
    assert_eq!(dst.t, src.t);
}
