use std::time::{Duration, Instant};

use clap::ValueEnum;
use epicentre_diagnostics::tracing;
use itertools::Itertools;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use sysinfo::{CpuRefreshKind, Networks, RefreshKind, System};

use crate::Metrics;
use crate::units::{CpuUsage, NetUsage, NetUsageRate};

#[derive(ValueEnum, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum MeasurementType {
    Cpu,
    Net,
}

#[derive(Debug)]
#[must_use]
pub struct ResourceMonitor {
    system: System,
    networks: Networks,
    last_refresh: Instant,

    cpu_usage: AllocRingBuffer<CpuUsage>,
    net_usage_rate: AllocRingBuffer<NetUsageRate>,
}

impl ResourceMonitor {
    const REMOVE_NOT_LISTED_INTERFACES: bool = true;

    fn refresh_specifics() -> RefreshKind {
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything())
    }

    pub fn new(capacity: usize) -> Self {
        let system = System::new_with_specifics(Self::refresh_specifics());
        let networks = Networks::new_with_refreshed_list();

        let mut cpu_usage = AllocRingBuffer::new(capacity);
        let mut net_usage_rate = AllocRingBuffer::new(capacity);

        let cpu_usage_value = CpuUsage::from_percentage(system.global_cpu_usage());
        let net_usage_rate_shim =
            NetUsageRate::from_usage_and_duration(NetUsage::from_bytes(0), Duration::from_secs(0));

        cpu_usage.enqueue(cpu_usage_value);
        net_usage_rate.enqueue(net_usage_rate_shim);

        Self {
            system,
            cpu_usage,
            last_refresh: Instant::now(),
            networks,
            net_usage_rate,
        }
    }

    pub fn refresh(&mut self) {
        let refreshes = Self::refresh_specifics();

        self.system.refresh_specifics(refreshes);
        self.cpu_usage
            .enqueue(CpuUsage::from_percentage(self.system.global_cpu_usage()));

        let net_usage_combined = self
            .networks
            .values()
            .map(|nd| nd.received() + nd.transmitted())
            .map(NetUsage::from_bytes)
            .sum::<NetUsage>();
        let net_usage_rate_combined =
            NetUsageRate::from_usage_and_duration(net_usage_combined, self.last_refresh.elapsed());
        tracing::debug!(?net_usage_rate_combined);
        self.net_usage_rate.enqueue(net_usage_rate_combined);
        self.networks.refresh(Self::REMOVE_NOT_LISTED_INTERFACES);

        self.last_refresh = Instant::now();
    }

    pub fn build_metrics(&self) -> Metrics {
        let cpu_usage = self
            .cpu_usage
            .iter()
            .copied()
            .rev()
            .pad_using(self.cpu_usage.capacity(), |_| CpuUsage::from_percentage(0.))
            .collect();

        let net_usage_rate = self
            .net_usage_rate
            .iter()
            .copied()
            .rev()
            .pad_using(self.net_usage_rate.capacity(), |_| {
                NetUsageRate::from_usage_and_duration(
                    NetUsage::from_bytes(0),
                    Duration::from_secs(1),
                )
            })
            .collect();

        Metrics {
            cpu_usage,
            net_usage_rate,
        }
    }
}
