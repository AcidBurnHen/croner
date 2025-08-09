use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CronSchedule {
    pub minute: HashSet<u8>,
    pub hour: HashSet<u8>,
    pub day: HashSet<u8>,
    pub month: HashSet<u8>,
    pub weekday: HashSet<u8>,
}

type FieldRanges = HashMap<&'static str, (u8, u8)>;
pub struct CronParser {
    field_names: [&'static str; 5],
    field_ranges: FieldRanges,
}

impl CronParser {
    pub fn new() -> Self {
        let mut field_ranges: FieldRanges = HashMap::new();

        field_ranges.insert("minute", (0, 59));
        field_ranges.insert("hour", (0, 23));
        field_ranges.insert("day", (1, 31));
        field_ranges.insert("month", (1, 12));
        field_ranges.insert("weekday", (0, 6));

        Self {
            field_names: ["minute", "hour", "day", "month", "weekday"],
            field_ranges,
        }
    }
}
