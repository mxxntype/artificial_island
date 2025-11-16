use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::MeasurementType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct Metrics {
    #[serde(default)]
    pub cpu_usage: Vec<f32>,
}

pub const GRAPH_SIGILS: [[char; 4]; 4] = [
    ['⣀', '⣄', '⣆', '⣇'],
    ['⣠', '⣤', '⣦', '⣧'],
    ['⣰', '⣴', '⣶', '⣷'],
    ['⣸', '⣼', '⣾', '⣿'],
];

pub trait MeasurementGrading {
    fn scale_f32(&self, value: f32) -> usize;
}

pub struct CpuGrading;

impl MeasurementGrading for CpuGrading {
    fn scale_f32(&self, value: f32) -> usize {
        match value {
            ..10.0 => 0,
            10.0..45.0 => 1,
            45.0..80.0 => 2,
            80.0.. => 3,
            _ => unreachable!(),
        }
    }
}

impl Metrics {
    pub fn render_usage_graph(
        &self,
        measurement_type: MeasurementType,
    ) -> Result<String, std::fmt::Error> {
        let mut graph_buffer = String::new();
        let measurements = match measurement_type {
            MeasurementType::Cpu => &self.cpu_usage,
        };

        let graph_length = measurements.len() / 2;
        for sigil_index in 0..graph_length {
            let cpu_usage_percent_0 = measurements[sigil_index * 2];
            let cpu_usage_percent_1 = measurements[sigil_index * 2 + 1];

            let index_0 = CpuGrading.scale_f32(cpu_usage_percent_0);
            let index_1 = CpuGrading.scale_f32(cpu_usage_percent_1);

            write!(&mut graph_buffer, "{}", GRAPH_SIGILS[index_0][index_1])?;
        }

        Ok(graph_buffer.chars().rev().collect())
    }
}
