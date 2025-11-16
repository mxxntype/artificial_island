use uom::si;

use crate::units::{CpuUsage, NetUsageRate};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(usize)]
pub enum MeasurementGrade {
    Idle,
    Low,
    Medium,
    High,
}

pub trait MeasurementGrading {
    type Measurement;
    fn scale(&self, measurement: Self::Measurement) -> MeasurementGrade;
}

pub struct CpuUsageGrading;
pub struct NetUsageRateGrading;

impl MeasurementGrading for CpuUsageGrading {
    type Measurement = CpuUsage;

    fn scale(&self, measurement: Self::Measurement) -> MeasurementGrade {
        match measurement.get::<si::ratio::percent>() {
            ..10. => MeasurementGrade::Idle,
            10.0..45. => MeasurementGrade::Low,
            45.0..80. => MeasurementGrade::Medium,
            80.0.. => MeasurementGrade::High,
            _ => unreachable!(),
        }
    }
}

impl MeasurementGrading for NetUsageRateGrading {
    type Measurement = NetUsageRate;

    fn scale(&self, measurement: Self::Measurement) -> MeasurementGrade {
        match measurement
            .as_information_rate()
            .get::<si::information_rate::megabit_per_second>()
        {
            ..10. => MeasurementGrade::Idle,
            10.0..100. => MeasurementGrade::Low,
            100.0..800. => MeasurementGrade::Medium,
            800.0.. => MeasurementGrade::High,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uom::si;

    use super::{CpuUsageGrading, MeasurementGrade, MeasurementGrading, NetUsageRateGrading};
    use crate::units::{CpuUsage, NetUsage, NetUsageRate};

    #[test]
    fn cpu_usage_grading() {
        assert_eq!(
            CpuUsageGrading.scale(CpuUsage::from_percentage(1.)),
            MeasurementGrade::Idle
        );
        assert_eq!(
            CpuUsageGrading.scale(CpuUsage::from_percentage(20.)),
            MeasurementGrade::Low
        );
        assert_eq!(
            CpuUsageGrading.scale(CpuUsage::from_percentage(60.)),
            MeasurementGrade::Medium
        );
        assert_eq!(
            CpuUsageGrading.scale(CpuUsage::from_percentage(90.)),
            MeasurementGrade::High
        );
    }

    #[test]
    fn net_usage_rate_grading() {
        let grader = NetUsageRateGrading;
        assert_eq!(
            grader.scale(NetUsageRate::from_usage_and_duration(
                NetUsage::from(si::u64::Information::new::<si::information::kilobyte>(1)),
                Duration::from_secs(1)
            )),
            MeasurementGrade::Idle
        );
        assert_eq!(
            grader.scale(NetUsageRate::from_usage_and_duration(
                NetUsage::from(si::u64::Information::new::<si::information::megabyte>(2)),
                Duration::from_secs(1)
            )),
            MeasurementGrade::Low
        );
        assert_eq!(
            grader.scale(NetUsageRate::from_usage_and_duration(
                NetUsage::from(si::u64::Information::new::<si::information::megabyte>(20)),
                Duration::from_secs(1)
            )),
            MeasurementGrade::Medium
        );
        assert_eq!(
            grader.scale(NetUsageRate::from_usage_and_duration(
                NetUsage::from(si::u64::Information::new::<si::information::megabit>(950)),
                Duration::from_secs(1)
            )),
            MeasurementGrade::High
        );
    }
}
