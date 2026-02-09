use std::time::Instant;

use chrono::Utc;
use sysinfo::{Disks, Networks, System};

use crate::types::{
    CpuMetrics, DiskMetrics, GpuMetrics, MemoryMetrics, NetworkMetrics, TelemetrySnapshot,
};

pub struct SystemCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    prev_rx: u64,
    prev_tx: u64,
    prev_tick: Instant,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let disks = Disks::new_with_refreshed_list();
        let (prev_rx, prev_tx) = Self::network_totals(&networks);

        Self {
            system,
            networks,
            disks,
            prev_rx,
            prev_tx,
            prev_tick: Instant::now(),
        }
    }

    pub fn collect(&mut self) -> TelemetrySnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);

        let cpu_usage = self.system.global_cpu_usage() as f64;
        let frequency = if self.system.cpus().is_empty() {
            None
        } else {
            let total: u64 = self.system.cpus().iter().map(|cpu| cpu.frequency()).sum();
            Some(total / self.system.cpus().len() as u64)
        };

        let memory_total_mb = self.system.total_memory() as f64 / (1024.0 * 1024.0);
        let memory_used_mb = self.system.used_memory() as f64 / (1024.0 * 1024.0);
        let memory_usage_pct = if memory_total_mb > 0.0 {
            memory_used_mb / memory_total_mb * 100.0
        } else {
            0.0
        };

        let mut total_disk = 0.0;
        let mut used_disk = 0.0;
        for disk in self.disks.list() {
            let total = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let avail = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            total_disk += total;
            used_disk += (total - avail).max(0.0);
        }
        let disk_usage_pct = if total_disk > 0.0 {
            used_disk / total_disk * 100.0
        } else {
            0.0
        };

        let (rx_total, tx_total) = Self::network_totals(&self.networks);
        let elapsed = self.prev_tick.elapsed().as_secs_f64().max(0.001);
        let rx_rate = rx_total.saturating_sub(self.prev_rx) as f64 / elapsed;
        let tx_rate = tx_total.saturating_sub(self.prev_tx) as f64 / elapsed;

        self.prev_rx = rx_total;
        self.prev_tx = tx_total;
        self.prev_tick = Instant::now();

        TelemetrySnapshot {
            timestamp: Utc::now(),
            cpu: CpuMetrics {
                usage_pct: cpu_usage,
                frequency_mhz: frequency,
                temperature_c: None,
            },
            gpu: GpuMetrics {
                usage_pct: None,
                temperature_c: None,
                memory_used_mb: None,
                memory_total_mb: None,
            },
            memory: MemoryMetrics {
                used_mb: memory_used_mb,
                total_mb: memory_total_mb,
                usage_pct: memory_usage_pct,
            },
            disk: DiskMetrics {
                used_gb: used_disk,
                total_gb: total_disk,
                usage_pct: disk_usage_pct,
                read_bytes_per_sec: None,
                write_bytes_per_sec: None,
            },
            network: NetworkMetrics {
                download_bytes_per_sec: rx_rate,
                upload_bytes_per_sec: tx_rate,
                latency_ms: None,
            },
            power_watts: None,
        }
    }

    fn network_totals(networks: &Networks) -> (u64, u64) {
        let mut rx = 0_u64;
        let mut tx = 0_u64;
        for (_name, data) in networks {
            rx = rx.saturating_add(data.total_received());
            tx = tx.saturating_add(data.total_transmitted());
        }
        (rx, tx)
    }
}
