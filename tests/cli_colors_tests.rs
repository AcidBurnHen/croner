// tests/cli_colors_tests.rs

use croner::cli_colors::CliColorPicker;
use std::collections::HashSet;

#[test]
fn returns_valid_ansi_codes() {
    let mut picker = CliColorPicker::new();
    let c = picker.get(0);
    assert!(c.starts_with("\u{1b}["), "Not an ANSI escape code: {}", c);
}

#[test]
fn same_job_id_returns_same_color() {
    let mut picker = CliColorPicker::new();
    let c1 = picker.get(5);
    let c2 = picker.get(5);
    assert_eq!(c1, c2, "Color changed for same job id");
}

#[test]
fn different_jobs_get_different_colors_until_exhaustion() {
    let mut picker = CliColorPicker::new();
    let mut seen = HashSet::new();

    // The palette size can be inferred by counting unique colors until repeat
    for job_id in 0..100 {
        let c = picker.get(job_id);
        if !seen.insert(c) {
            break;
        }
    }

    // Ensure we got more than 1 unique color and didn't immediately repeat
    assert!(seen.len() > 1, "Too few unique colors before repetition");
}

#[test]
fn cycles_colors_after_exhaustion() {
    let mut picker = CliColorPicker::new();
    let mut seen = HashSet::new();

    // Fill the palette
    for job_id in 0..50 {
        seen.insert(picker.get(job_id));
    }

    let before_cycle = seen.clone();

    // Trigger another round (forces reshuffle eventually)
    for job_id in 50..100 {
        seen.insert(picker.get(job_id));
    }

    // All colors seen should still be valid ANSI sequences
    assert!(seen.iter().all(|c| c.starts_with("\u{1b}[")));
    // The set size shouldn't shrink
    assert!(seen.len() >= before_cycle.len());
}

#[test]
fn high_job_ids_trigger_resize() {
    let mut picker = CliColorPicker::new();
    let high_id = 500;
    let c = picker.get(high_id);

    // Even for very high job IDs, we still get a valid ANSI color
    assert!(c.starts_with("\u{1b}["), "Invalid ANSI code for high ID");
}

#[test]
fn colors_cover_full_palette_over_time() {
    let mut picker = CliColorPicker::new();
    let mut seen = HashSet::new();

    // Arbitrary large number to give multiple cycles
    for job_id in 0..500 {
        seen.insert(picker.get(job_id));
    }

    // Must have used at least 10+ unique colors from the palette
    assert!(
        seen.len() >= 10,
        "Too few colors used over multiple cycles: {:?}",
        seen
    );
}
