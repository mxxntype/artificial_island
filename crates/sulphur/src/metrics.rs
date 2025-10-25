use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct Metrics {
    pub cpu_usage: Vec<f32>,
}

pub const GRAPH_SIGILS: [[char; 4]; 4] = [
    ['⣀', '⣄', '⣆', '⣇'],
    ['⣠', '⣤', '⣦', '⣧'],
    ['⣰', '⣴', '⣶', '⣷'],
    ['⣸', '⣼', '⣾', '⣿'],
];

impl Metrics {
    pub fn render_cpu_usage_graph(&self) -> Result<String, std::fmt::Error> {
        let mut graph_buffer = String::new();

        let graph_length = self.cpu_usage.len() / 2;
        for sigil_index in 0..graph_length {
            let cpu_usage_percent_0 = self.cpu_usage[sigil_index * 2];
            let cpu_usage_percent_1 = self.cpu_usage[sigil_index * 2 + 1];

            let index_0 = Self::scale_usage_to_symbol_index(cpu_usage_percent_0);
            let index_1 = Self::scale_usage_to_symbol_index(cpu_usage_percent_1);

            write!(&mut graph_buffer, "{}", GRAPH_SIGILS[index_0][index_1])?;
        }

        Ok(graph_buffer.chars().rev().collect())
    }

    /// Scale an [`f32`] value in the range [0.0; 100.0] to a [`usize`] that
    /// represents an index within [`GRAPH_SIGILS`]. The scaling is not linear.
    #[must_use]
    pub fn scale_usage_to_symbol_index(value: f32) -> usize {
        match value {
            ..10.0 => 0,
            10.0..45.0 => 1,
            45.0..80.0 => 2,
            80.0.. => 3,
            _ => unreachable!(),
        }
    }
}
