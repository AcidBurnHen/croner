use croner::parser::CronSchedule;
use croner::scheduler::{compute_next_run, hash_id};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Returns (minute, hour, weekday) like `compute_next_run` logic uses.
fn now_components() -> (u8, u8, u8) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let minutes_total = secs / 60;
    let minute = (minutes_total % 60) as u8;
    let hour = ((minutes_total / 60) % 24) as u8;
    let days_since_epoch = minutes_total / (60 * 24);
    let weekday = ((days_since_epoch + 4) % 7) as u8;
    (minute, hour, weekday)
}

/// Bitmask helpers for the actual CronSchedule field types.
fn bit_u64(v: u8) -> u64 {
    1u64 << v
}
fn bit_u32(v: u8) -> u32 {
    1u32 << v
}
fn bit_u16(v: u8) -> u16 {
    1u16 << v
}
fn bit_u8(v: u8) -> u8 {
    1u8 << v
}

/// Helper to build a valid CronSchedule with only minute/hour/weekday constraints.
fn make_schedule(minute_mask: u64, hour_mask: u32, weekday_mask: u8) -> CronSchedule {
    CronSchedule {
        minute: minute_mask,
        hour: hour_mask,
        day: u32::MAX,   // match any day-of-month
        month: u16::MAX, // match any month
        weekday: weekday_mask,
        // fill in any extra fields your CronSchedule actually has here:
        // these names are from your error message
        minutes: Vec::new(),
        hours: Vec::new(),
        days: Vec::new(),
        months: Vec::new(),
        weekdays: Vec::new(),
    }
}

#[test]
fn hash_id_is_stable_and_distinguishes() {
    assert_eq!(hash_id("alpha"), hash_id("alpha"));
    assert_eq!(hash_id(""), hash_id(""));

    let a = hash_id("alpha");
    let b = hash_id("beta");
    assert_ne!(a, b);
    assert_ne!(a, hash_id("alphA"));
    assert_ne!(b, hash_id("alphA"));
}

#[test]
fn compute_next_run_exact_next_minute() {
    let (now_min, now_hour, now_wd) = now_components();

    let next_min = (now_min + 1) % 60;
    let carry_hour = if next_min == 0 { 1 } else { 0 };
    let next_hour = (now_hour + carry_hour) % 24;
    let carry_day = if carry_hour == 1 && next_hour == 0 {
        1
    } else {
        0
    };
    let next_wd = (now_wd + carry_day) % 7;

    let schedule = make_schedule(bit_u64(next_min), bit_u32(next_hour), bit_u8(next_wd));

    let start = Instant::now();
    let when = compute_next_run(&schedule);
    let dur = when.duration_since(start);

    assert!(
        dur >= Duration::from_secs(60) && dur <= Duration::from_secs(62),
        "expected ~60s ahead, got {:?}",
        dur
    );
}

#[test]
fn compute_next_run_respects_longer_gap() {
    let (now_min, now_hour, now_wd) = now_components();

    let target_min = (now_min + 2) % 60;
    let carry_hour = if target_min < now_min { 1 } else { 0 };
    let target_hour = (now_hour + carry_hour) % 24;
    let carry_day = if carry_hour == 1 && target_hour == 0 {
        1
    } else {
        0
    };
    let target_wd = (now_wd + carry_day) % 7;

    let schedule = make_schedule(bit_u64(target_min), bit_u32(target_hour), bit_u8(target_wd));

    let start = Instant::now();
    let when = compute_next_run(&schedule);
    let dur = when.duration_since(start);

    assert!(
        dur >= Duration::from_secs(120) && dur <= Duration::from_secs(122),
        "expected ~120s ahead, got {:?}",
        dur
    );
}
