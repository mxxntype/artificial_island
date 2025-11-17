use std::fmt::Write as _;

use crate::grading::{CpuUsageGrading, MeasurementGrading, NetUsageRateGrading};
use crate::resource_monitor::{MeasurementType, Metrics};

pub const GRAPH_DENSITY: u8 = 2;

pub const GRAPH_SIGILS: [[char; 4]; 4] = [
    ['⣀', '⣄', '⣆', '⣇'],
    ['⣠', '⣤', '⣦', '⣧'],
    ['⣰', '⣴', '⣶', '⣷'],
    ['⣸', '⣼', '⣾', '⣿'],
];

pub fn render(
    metrics: &Metrics,
    measurement_type: MeasurementType,
) -> Result<String, std::fmt::Error> {
    let measurement_grades: Vec<_> = match measurement_type {
        MeasurementType::Cpu => metrics
            .cpu_usage
            .iter()
            .map(|m| CpuUsageGrading.scale(*m))
            .collect(),
        MeasurementType::Net => metrics
            .net_usage_rate
            .iter()
            .map(|m| NetUsageRateGrading.scale(*m))
            .collect(),
    };

    let graph_length = measurement_grades.len() / 2;
    let mut graph_buffer = String::with_capacity(graph_length);
    for sigil_index in 0..graph_length {
        let index_0 = measurement_grades[sigil_index * 2];
        let index_1 = measurement_grades[sigil_index * 2 + 1];
        let sigil = GRAPH_SIGILS[index_0 as usize][index_1 as usize];
        write!(&mut graph_buffer, "{sigil}",)?;
    }

    Ok(graph_buffer.chars().rev().collect())
}
