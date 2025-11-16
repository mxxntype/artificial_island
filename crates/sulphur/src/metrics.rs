use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::MeasurementType;
use crate::grading::{CpuUsageGrading, MeasurementGrade, MeasurementGrading, NetUsageRateGrading};
use crate::units::{CpuUsage, NetUsageRate};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct Metrics {
    #[serde(default)]
    pub cpu_usage: Vec<CpuUsage>,
    #[serde(default)]
    pub net_usage_rate: Vec<NetUsageRate>,
}

pub const GRAPH_SIGILS: [[char; 4]; 4] = [
    ['⣀', '⣄', '⣆', '⣇'],
    ['⣠', '⣤', '⣦', '⣧'],
    ['⣰', '⣴', '⣶', '⣷'],
    ['⣸', '⣼', '⣾', '⣿'],
];

impl Metrics {
    pub fn render_usage_graph(
        &self,
        measurement_type: MeasurementType,
    ) -> Result<String, std::fmt::Error> {
        let mut graph_buffer = String::new();

        let measurement_grades: Vec<MeasurementGrade> = match measurement_type {
            MeasurementType::Cpu => self
                .cpu_usage
                .iter()
                .map(|m| CpuUsageGrading.scale(*m))
                .collect(),
            MeasurementType::Net => self
                .net_usage_rate
                .iter()
                .map(|m| NetUsageRateGrading.scale(*m))
                .collect(),
        };

        let graph_length = measurement_grades.len() / 2;
        for sigil_index in 0..graph_length {
            let index_0 = measurement_grades[sigil_index * 2];
            let index_1 = measurement_grades[sigil_index * 2 + 1];
            let sigil = GRAPH_SIGILS[index_0 as usize][index_1 as usize];
            write!(&mut graph_buffer, "{sigil}",)?;
        }

        Ok(graph_buffer.chars().rev().collect())
    }
}
