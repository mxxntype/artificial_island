use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::ValueEnum;
use epicentre_diagnostics::tracing;
use itertools::Itertools;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Networks, RefreshKind, System};
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

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

    update_intervals: UpdateIntervals,
    last_update: Instant,

    cpu_usage: AllocRingBuffer<CpuUsage>,
    net_usage_rate: AllocRingBuffer<NetUsageRate>,
}

#[derive(Debug)]
pub struct UpdateIntervals {
    pub realtime: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct Metrics {
    #[serde(default)]
    pub cpu_usage: Vec<CpuUsage>,
    #[serde(default)]
    pub net_usage_rate: Vec<NetUsageRate>,
}

impl ResourceMonitor {
    const REMOVE_NOT_LISTED_INTERFACES: bool = true;

    fn system_refresh_specifics() -> RefreshKind {
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything())
    }

    pub fn new(capacity: usize, refresh_intervals: UpdateIntervals) -> Self {
        let system = System::new_with_specifics(Self::system_refresh_specifics());
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
            networks,
            update_intervals: refresh_intervals,
            last_update: Instant::now(),
            cpu_usage,
            net_usage_rate,
        }
    }

    pub fn refresh_realtime(&mut self) {
        self.system
            .refresh_specifics(Self::system_refresh_specifics());
        self.cpu_usage
            .enqueue(CpuUsage::from_percentage(self.system.global_cpu_usage()));

        let combined_net_usage = self
            .networks
            .values()
            .map(|nd| nd.received() + nd.transmitted())
            .map(NetUsage::from_bytes)
            .sum::<NetUsage>();
        let combined_net_usage_rate =
            NetUsageRate::from_usage_and_duration(combined_net_usage, self.last_update.elapsed());
        self.net_usage_rate.enqueue(combined_net_usage_rate);
        self.networks.refresh(Self::REMOVE_NOT_LISTED_INTERFACES);

        self.last_update = Instant::now();
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
            .pad_using(self.net_usage_rate.capacity(), |_| NetUsageRate::idle())
            .collect();

        Metrics {
            cpu_usage,
            net_usage_rate,
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn realtime_update_thread(
    resource_monitor: Arc<AsyncMutex<ResourceMonitor>>,
    cancellation_token: CancellationToken,
) {
    let update_interval = resource_monitor.lock().await.update_intervals.realtime;
    let update_loop = async move {
        loop {
            tokio::time::sleep(update_interval).await;
            resource_monitor.lock().await.refresh_realtime();
        }
    };

    tokio::select! {
        () = update_loop => {}
        () = cancellation_token.cancelled() => {}
    }
}
