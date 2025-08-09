// tests/cron_parser_tests.rs

use croner::parser::CronParser;

fn bits_set(mut mask: u64, max_bit: u8) -> Vec<u8> {
    let mut out = Vec::new();
    for b in 0..=max_bit {
        if (mask & 1) == 1 {
            out.push(b);
        }
        mask >>= 1;
    }
    out
}

fn assert_has_bits(mask: u64, expected: &[u8], max_bit: u8) {
    let mut got = bits_set(mask, max_bit);
    got.sort_unstable();
    let mut want = expected.to_vec();
    want.sort_unstable();
    assert_eq!(got, want, "bitmask mismatch");
}

#[test]
fn parses_wildcards_for_all_fields() {
    let p = CronParser::new();
    let s = p.parse("* * * * *").unwrap();

    // minute: 0..59
    assert_eq!(bits_set(s.minute, 59).len(), 60);
    // hour: 0..23
    assert_eq!(bits_set(s.hour as u64, 23).len(), 24);
    // day: 1..31
    assert_eq!(bits_set(s.day as u64, 31).len(), 31);
    // month: 1..12
    assert_eq!(bits_set(s.month as u64, 12).len(), 12);
    // weekday: 0..6
    assert_eq!(bits_set(s.weekday as u64, 6).len(), 7);
}

#[test]
fn parses_simple_values_and_boundaries() {
    let p = CronParser::new();
    let s = p.parse("59 23 31 12 6").unwrap();

    assert_has_bits(s.minute, &[59], 59);
    assert_has_bits(s.hour as u64, &[23], 23);
    assert_has_bits(s.day as u64, &[31], 31);
    assert_has_bits(s.month as u64, &[12], 12);
    assert_has_bits(s.weekday as u64, &[6], 6);
}

#[test]
fn parses_comma_lists_and_dedupes() {
    let p = CronParser::new();
    let s = p.parse("0,15,15,30,45 1,2,2 1,2,3 1,2 0,6").unwrap();

    assert_has_bits(s.minute, &[0, 15, 30, 45], 59);
    assert_has_bits(s.hour as u64, &[1, 2], 23);
    assert_has_bits(s.day as u64, &[1, 2, 3], 31);
    assert_has_bits(s.month as u64, &[1, 2], 12);
    assert_has_bits(s.weekday as u64, &[0, 6], 6);
}

#[test]
fn parses_ranges() {
    let p = CronParser::new();
    let s = p.parse("10-12 8-10 5-7 3-5 2-4").unwrap();

    assert_has_bits(s.minute, &[10, 11, 12], 59);
    assert_has_bits(s.hour as u64, &[8, 9, 10], 23);
    assert_has_bits(s.day as u64, &[5, 6, 7], 31);
    assert_has_bits(s.month as u64, &[3, 4, 5], 12);
    assert_has_bits(s.weekday as u64, &[2, 3, 4], 6);
}

#[test]
fn parses_steps_from_start() {
    let p = CronParser::new();

    // */15 minutes => 0,15,30,45
    let s = p.parse("*/15 * * * *").unwrap();
    assert_has_bits(s.minute, &[0, 15, 30, 45], 59);

    // */1 hour => 0..23
    let s = p.parse("* */1 * * *").unwrap();
    assert_eq!(bits_set(s.hour as u64, 23).len(), 24);
}

#[test]
fn mixes_lists_ranges_wildcards_and_steps() {
    let p = CronParser::new();

    // minute: "1,5-7,*/30" => {0,30} from step + {1} + {5,6,7}
    let s = p.parse("1,5-7,*/30 9-11 * 1,6 0,3,6").unwrap();
    assert_has_bits(s.minute, &[0, 1, 5, 6, 7, 30], 59);
    assert_has_bits(s.hour as u64, &[9, 10, 11], 23);
    assert_eq!(bits_set(s.day as u64, 31).len(), 31); // wildcard
    assert_has_bits(s.month as u64, &[1, 6], 12);
    assert_has_bits(s.weekday as u64, &[0, 3, 6], 6);
}

#[test]
fn trims_and_requires_five_fields() {
    let p = CronParser::new();

    // Trimming leading/trailing spaces should work
    let s = p.parse("   0 0 1 1 0   ").unwrap();
    assert_has_bits(s.minute, &[0], 59);

    // Not enough fields
    let err = p.parse("0 0 1 1").unwrap_err();
    assert!(err.contains("Expected 5 fields"), "got err: {}", err);

    // Too many fields
    let err = p.parse("0 0 1 1 0 extra").unwrap_err();
    assert!(err.contains("Expected 5 fields"), "got err: {}", err);
}

#[test]
fn rejects_out_of_range_values() {
    let p = CronParser::new();

    // minute 60 invalid
    let err = p.parse("60 0 1 1 0").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    // hour 24 invalid
    let err = p.parse("0 24 1 1 0").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    // day 0 invalid (days are 1..31)
    let err = p.parse("0 0 0 1 0").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    // month 13 invalid
    let err = p.parse("0 0 1 13 0").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    // weekday 7 invalid
    let err = p.parse("0 0 1 1 7").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);
}

#[test]
fn rejects_bad_ranges() {
    let p = CronParser::new();

    // reversed
    let err = p.parse("10-5 0 1 1 0").unwrap_err();
    assert!(err.contains("Invalid range"), "got err: {}", err);

    // low below start (day must be >=1)
    let err = p.parse("0 0 0-5 1 0").unwrap_err();
    assert!(
        err.contains("Invalid value") || (err.contains("Invalid range")),
        "got err: {}",
        err
    );

    // high above end (month must be <=12)
    let err = p.parse("0 0 1 10-15 0").unwrap_err();
    assert!(err.contains("Invalid range"), "got err: {}", err);

    // malformed "a-b-c"
    let err = p.parse("0 0 1 a-b-c 0").unwrap_err();
    assert!(
        err.contains("Invalid range") || (err.contains("Invalid value")),
        "got err: {}",
        err
    );
}

#[test]
fn rejects_non_numeric_parts() {
    let p = CronParser::new();

    let err = p.parse("x * * * *").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    let err = p.parse("* y * * *").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    let err = p.parse("* * z * *").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    let err = p.parse("* * * w *").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);

    let err = p.parse("* * * * v").unwrap_err();
    assert!(err.contains("Invalid value"), "got err: {}", err);
}

#[test]
fn step_parsing_basic_sanity() {
    let p = CronParser::new();

    // */5 minutes => 0,5,10,...,55 (12 values)
    let s = p.parse("*/5 * * * *").unwrap();
    let mins = bits_set(s.minute, 59);
    assert_eq!(mins.len(), 12);
    assert!(mins.contains(&0) && mins.contains(&55));

    // */2 weekday => 0,2,4,6
    let s = p.parse("* * * * */2").unwrap();
    assert_has_bits(s.weekday as u64, &[0, 2, 4, 6], 6);
}

#[test]
fn range_edges_and_overlaps() {
    let p = CronParser::new();

    // Overlapping ranges and values should coalesce into unique bits
    let s = p.parse("10-12,12-15,14 5-6,6 1-3,2 2-4,4 1-3,0-2").unwrap();

    assert_has_bits(s.minute, &[10, 11, 12, 13, 14, 15], 59);
    assert_has_bits(s.hour as u64, &[5, 6], 23);
    assert_has_bits(s.day as u64, &[1, 2, 3], 31);
    assert_has_bits(s.month as u64, &[2, 3, 4], 12);
    assert_has_bits(s.weekday as u64, &[0, 1, 2, 3], 6);
}
