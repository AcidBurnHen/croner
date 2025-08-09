#[derive(Debug, Clone, Copy)]
pub struct CronSchedule {
    pub minute: u64, // bits 0..59
    pub hour: u32,   // bits 0..23
    pub day: u32,    // bits 1..31 (bit 0 unused)
    pub month: u16,  // bits 1..12 (bit 0 unused)
    pub weekday: u8, // bits 0..6
}

pub struct CronParser {
    field_ranges: [(u8, u8); 5],
}

impl CronParser {
    pub fn new() -> Self {
        Self {
            field_ranges: [
                (0, 59), // minute
                (0, 23), // hour
                (1, 31), // day
                (1, 12), // month
                (0, 6),  // weekday
            ],
        }
    }

    pub fn parse(&self, expr: &str) -> Result<CronSchedule, String> {
        let parts: Vec<&str> = expr.trim().split_whitespace().collect();

        if parts.len() != 5 {
            return Err(format!(
                "Expected 5 fields in cron expression, got: {}",
                parts.len()
            ));
        }

        let [minute_str, hour_str, day_str, month_str, weekday_str]: [&str; 5] =
            parts.try_into().unwrap();

        Ok(CronSchedule {
            minute: self.parse_field(minute_str, self.field_ranges[0])?,
            hour: self.parse_field(hour_str, self.field_ranges[1])? as u32,
            day: self.parse_field(day_str, self.field_ranges[2])? as u32,
            month: self.parse_field(month_str, self.field_ranges[3])? as u16,
            weekday: self.parse_field(weekday_str, self.field_ranges[4])? as u8,
        })
    }

    fn parse_u8(&self, value: &str, err_msg: &str) -> Result<u8, String> {
        match value.parse::<u8>() {
            Ok(v) => Ok(v),
            Err(_) => Err(err_msg.to_string()),
        }
    }

    fn set_bit(mask: &mut u64, bit: u8) {
        *mask |= 1 << bit;
    }

    fn parse_field(&self, part: &str, field_range: (u8, u8)) -> Result<u64, String> {
        let mut mask: u64 = 0;
        let (start, end) = field_range;

        for expr_part in part.split(',') {
            if expr_part == "*" {
                for v in start..=end {
                    Self::set_bit(&mut mask, v);
                }
            } else if let Some(step_str) = expr_part.strip_prefix("*/") {
                let step = self.parse_u8(step_str, &format!("Invalid step: {}", expr_part))?;
                let mut v = start;
                while v <= end {
                    Self::set_bit(&mut mask, v);
                    v += step;
                }
            } else if expr_part.contains('-') {
                let parts: Vec<&str> = expr_part.split('-').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid range: {}", expr_part));
                }

                let a = self.parse_u8(parts[0], &format!("Invalid range: {}", expr_part))?;
                let b = self.parse_u8(parts[1], &format!("Invalid range: {}", expr_part))?;

                if a > b || a < start || b > end {
                    return Err(format!("Invalid range: {}", expr_part));
                }

                for v in a..=b {
                    Self::set_bit(&mut mask, v);
                }
            } else {
                let val = self.parse_u8(expr_part, &format!("Invalid value: {}", expr_part))?;
                if val < start || val > end {
                    return Err(format!("Invalid value: {}", expr_part));
                }
                Self::set_bit(&mut mask, val);
            }
        }

        Ok(mask)
    }
}
