use std::time::Duration;

use derive_more::{Add, Deref, DerefMut, Sub};
use serde::{Deserialize, Serialize};
use uom::si::f32::Ratio;
use uom::si::u64::Information;
use uom::si::{information, ratio};

/// Instant CPU usage measurement.
#[derive(Serialize, Deserialize, Add, Sub, PartialEq, Deref, DerefMut, Clone, Copy, Debug)]
#[must_use]
pub struct CpuUsage(Ratio);

/// Accumulating network utilization measerement.
///
/// Stores the amount of data that has been received and/or transmitted within
/// an arbitrary time period.
#[derive(Serialize, Deserialize, Add, Sub, PartialEq, Eq, Deref, DerefMut, Clone, Copy, Debug)]
#[must_use]
pub struct NetUsage(Information);

/// Network utilization measerement.
///
/// Stores the amount of data that has been received and/or transmitted within a
/// known time period.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[must_use]
pub struct NetUsageRate {
    usage: NetUsage,
    time_period: Duration,
}

impl CpuUsage {
    pub fn from_percentage(percentage: f32) -> Self {
        Self(Ratio::new::<ratio::percent>(percentage))
    }
}

impl NetUsage {
    pub fn from_bytes(bytes: u64) -> Self {
        Self(Information::new::<information::byte>(bytes))
    }
}

impl NetUsageRate {
    pub const fn from_usage_and_time_period(usage: NetUsage, time_period: Duration) -> Self {
        Self { usage, time_period }
    }
}
