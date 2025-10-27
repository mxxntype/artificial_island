use std::fmt::Write as _;

use epicentre_diagnostics::color_eyre::owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct Metrics {
    pub cpu_usage: Vec<f32>,
}

pub const HISTOGRAM_RANGE: usize = 4;
pub const HISTOGRAM_SYMBOLS: [[char; HISTOGRAM_RANGE]; HISTOGRAM_RANGE] = [
    ['⣀', '⣄', '⣆', '⣇'],
    ['⣠', '⣤', '⣦', '⣧'],
    ['⣰', '⣴', '⣶', '⣷'],
    ['⣸', '⣼', '⣾', '⣿'],
];

impl Metrics {
    pub fn render_cpu_usage_histogram(&self) -> Result<String, std::fmt::Error> {
        let mut histogram = String::new();

        let histogram_length_chars = self.cpu_usage.len() / 2;
        for histogram_char_index in (0..histogram_length_chars).rev() {
            let cpu_usage_percent_0 = self.cpu_usage[histogram_char_index * 2];
            let cpu_usage_percent_1 = self.cpu_usage[histogram_char_index * 2 + 1];

            let histogram_table_index_v = Self::scale_usage_to_symbol_index(cpu_usage_percent_0);
            let histogram_table_index_h = Self::scale_usage_to_symbol_index(cpu_usage_percent_1);
            let histogram_combined_grade = histogram_table_index_v + histogram_table_index_h + 2;

            let histogram_symbol = {
                let symbol = HISTOGRAM_SYMBOLS[histogram_table_index_v][histogram_table_index_h];
                match histogram_combined_grade {
                    ..=4 => symbol.green().to_string(),
                    5..=6 => symbol.yellow().to_string(),
                    7..=8 => symbol.red().to_string(),
                    _ => unreachable!(),
                }
            };

            write!(&mut histogram, "{histogram_symbol}")?;
        }

        Ok(histogram)
    }

    /// Scale an [`f32`] value in the range [0.0; 100.0] to a [`usize`] that
    /// represents an index in [`HISTOGRAM_SYMBOLS`]. The scaling is not linear.
    #[must_use]
    pub fn scale_usage_to_symbol_index(usage_percent: f32) -> usize {
        let index = match usage_percent {
            ..10.0 => 0,
            10.0..45.0 => 1,
            45.0..80.0 => 2,
            80.0.. => 3,
            _ => unreachable!(),
        };

        debug_assert!((0..HISTOGRAM_RANGE).contains(&index));

        index
    }
}
