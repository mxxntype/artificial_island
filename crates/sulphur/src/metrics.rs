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
    #[must_use]
    pub fn render_cpu_usage_graph(&self) -> String {
        let mut graph = String::new();

        let graph_length = self.cpu_usage.len() / 2;
        for sigil_index in 0..graph_length {
            let cpu_usage_percent_0 = self.cpu_usage[sigil_index * 2];
            let cpu_usage_percent_1 = self.cpu_usage[sigil_index * 2 + 1];

            let grade_0 = Self::make_grade(cpu_usage_percent_0);
            let grade_1 = Self::make_grade(cpu_usage_percent_1);

            write!(&mut graph, "{}", GRAPH_SIGILS[grade_0][grade_1]).unwrap();
        }

        graph.chars().rev().collect()
    }

    fn make_grade(value: f32) -> usize {
        match value {
            0.0..25.0 => 0,
            25.0..50.0 => 1,
            50.0..75.0 => 2,
            75.0.. => 3,
            _ => unreachable!(),
        }
    }
}
