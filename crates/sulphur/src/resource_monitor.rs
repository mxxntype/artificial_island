use itertools::Itertools;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::Metrics;

#[derive(Debug)]
#[must_use]
pub struct ResourceMonitor {
    system: System,
    cpu_usage: AllocRingBuffer<f32>,
}

impl ResourceMonitor {
    fn refresh_specifics() -> RefreshKind {
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything())
    }

    pub fn new(capacity: usize) -> Self {
        let system = System::new_with_specifics(Self::refresh_specifics());
        let mut cpu_usage = AllocRingBuffer::new(capacity);
        cpu_usage.enqueue(system.global_cpu_usage());
        Self { system, cpu_usage }
    }

    pub fn refresh(&mut self) {
        let refreshes = Self::refresh_specifics();
        self.system.refresh_specifics(refreshes);
        self.cpu_usage.enqueue(self.system.global_cpu_usage());
    }

    pub fn build_metrics(&self) -> Metrics {
        let cpu_usage = self
            .cpu_usage
            .iter()
            .copied()
            .rev()
            .pad_using(self.cpu_usage.capacity(), |_| 0.)
            .collect();
        Metrics { cpu_usage }
    }
}
