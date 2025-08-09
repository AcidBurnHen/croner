use std::time::{SystemTime, UNIX_EPOCH};

pub struct CliColorPicker {
    ansi_colors: &'static [&'static str; 14],
    color_order: Vec<usize>,
    job_colors: Vec<Option<usize>>,
    next_color_idx: usize,
}

impl CliColorPicker {
    pub fn new() -> Self {
        const COLORS: [&str; 14] = [
            "\u{1b}[30m", // black
            "\u{1b}[31m", // red
            "\u{1b}[32m", // green
            "\u{1b}[33m", // yellow
            "\u{1b}[34m", // blue
            "\u{1b}[35m", // magenta
            "\u{1b}[36m", // cyan
            "\u{1b}[90m", // bright black
            "\u{1b}[91m", // bright red
            "\u{1b}[92m", // bright green
            "\u{1b}[93m", // bright yellow
            "\u{1b}[94m", // bright blue
            "\u{1b}[95m", // bright magenta
            "\u{1b}[96m", // bright cyan
        ];

        let mut picker = Self {
            ansi_colors: &COLORS,
            color_order: (0..COLORS.len()).collect(),
            job_colors: Vec::new(),
            next_color_idx: 0,
        };

        picker.shuffle_order();
        picker
    }

    #[inline]
    fn shuffle_order(&mut self) {
        // Fisher-Yates shuffle with time-based pseudo-randomness
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as usize;

        for i in (1..self.color_order.len()).rev() {
            let j = nanos.wrapping_add(i * 31) % (i + 1);
            self.color_order.swap(i, j);
        }

        self.next_color_idx = 0;
    }

    #[inline]
    pub fn get(&mut self, job_id: usize) -> &'static str {
        // Expand storage if needed
        if job_id >= self.job_colors.len() {
            self.job_colors.resize(job_id + 1, None);
        }

        // If already assigned, return immediately
        if let Some(color_idx) = self.job_colors[job_id] {
            return self.ansi_colors[color_idx];
        }

        // If we've used all colors in this cycle, reshuffle
        if self.next_color_idx >= self.color_order.len() {
            self.shuffle_order();
        }

        let color_idx = self.color_order[self.next_color_idx];
        self.next_color_idx += 1;
        self.job_colors[job_id] = Some(color_idx);

        self.ansi_colors[color_idx]
    }
}
