use std::time::Duration;

use derive_more::{Add, Deref, From, Sub, Sum};
use serde::{Deserialize, Serialize};
use uom::si;

/// Instant CPU usage measurement.
#[derive(Serialize, Deserialize, PartialEq, Deref, Clone, Copy, Debug)]
#[must_use]
pub struct CpuUsage(si::f32::Ratio);

/// Accumulating network utilization measerement.
///
/// Stores the amount of data that has been received and/or transmitted within
/// an arbitrary time period.
#[derive(Serialize, Deserialize, Add, Sub, Sum, From, PartialEq, Eq, Deref, Clone, Copy, Debug)]
#[must_use]
pub struct NetUsage(si::u64::Information);

/// Network utilization measerement.
///
/// Stores the amount of data that has been received and/or transmitted within a
/// known time period.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[must_use]
pub struct NetUsageRate {
    net_usage: NetUsage,
    duration: Duration,
}

impl CpuUsage {
    pub fn from_percentage(percentage: f32) -> Self {
        Self(si::f32::Ratio::new::<si::ratio::percent>(percentage))
    }
}

impl NetUsage {
    pub fn from_bytes(bytes: u64) -> Self {
        Self(si::u64::Information::new::<si::information::byte>(bytes))
    }
}

impl NetUsageRate {
    pub const fn from_usage_and_duration(usage: NetUsage, duration: Duration) -> Self {
        Self {
            net_usage: usage,
            duration,
        }
    }

    pub const fn net_usage(&self) -> NetUsage {
        self.net_usage
    }

    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn as_information_rate(&self) -> si::f32::InformationRate {
        let seconds = self.duration.as_secs_f32();
        let bytes = self.net_usage.get::<si::information::byte>();
        let bytes_per_second = (bytes as f32) / seconds;
        si::f32::InformationRate::new::<si::information_rate::byte_per_second>(bytes_per_second)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uom::si;

    use super::NetUsage;
    use crate::units::NetUsageRate;

    #[test]
    fn net_usage_addition() {
        let net_usage_1 = NetUsage::from_bytes(100);
        let net_usage_2 = NetUsage::from_bytes(100);
        let net_usage_3 = NetUsage::from_bytes(100);

        let total_net_usage = net_usage_1 + net_usage_2 + net_usage_3;
        let total_net_usage_bytes = total_net_usage.get::<si::information::byte>();

        assert_eq!(total_net_usage_bytes, 300);
    }

    #[test]
    fn net_usage_rate_collection() {
        let collected_net_usage = [128, 128, 256, 512]
            .map(NetUsage::from_bytes)
            .into_iter()
            .sum::<NetUsage>();

        let time_period = Duration::from_secs(10);
        let net_usage_rate =
            NetUsageRate::from_usage_and_duration(collected_net_usage, time_period);

        let bytes_per_second = net_usage_rate
            .as_information_rate()
            .get::<si::information_rate::byte_per_second>();

        assert!((bytes_per_second - 102.4).abs() <= 10e-6);
    }
}
