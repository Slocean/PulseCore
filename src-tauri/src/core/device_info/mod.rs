use std::process::Command;

use sysinfo::{Disks, System};

use crate::types::HardwareInfo;

pub fn collect_hardware_info() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_model = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let total_gb = (sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0)).max(1.0);
    let ram_spec = format!("{total_gb:.0} GB");

    let disk_models = disk_models().unwrap_or_else(|| {
        let disks = Disks::new_with_refreshed_list();
        disks
            .list()
            .iter()
            .map(|d| d.name().to_string_lossy().to_string())
            .collect::<Vec<_>>()
    });

    HardwareInfo {
        cpu_model,
        gpu_model: first_wmic_value(&["path", "win32_VideoController", "get", "Name", "/value"])
            .unwrap_or_else(|| "N/A".to_string()),
        ram_spec,
        disk_models: if disk_models.is_empty() {
            vec!["Unknown disk".to_string()]
        } else {
            disk_models
        },
        motherboard: first_wmic_value(&["baseboard", "get", "product", "/value"])
            .unwrap_or_else(|| "Unknown motherboard".to_string()),
        device_brand: first_wmic_value(&["computersystem", "get", "manufacturer", "/value"])
            .unwrap_or_else(|| "Unknown vendor".to_string()),
    }
}

fn disk_models() -> Option<Vec<String>> {
    let raw = run_wmic(&["diskdrive", "get", "model", "/value"])?;
    let mut out = Vec::new();
    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("Model=") {
            let item = v.trim();
            if !item.is_empty() {
                out.push(item.to_string());
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn first_wmic_value(args: &[&str]) -> Option<String> {
    let raw = run_wmic(args)?;
    for line in raw.lines() {
        if let Some((_k, v)) = line.split_once('=') {
            let text = v.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn run_wmic(args: &[&str]) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic").args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        return Some(String::from_utf8_lossy(&output.stdout).to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = args;
        None
    }
}
