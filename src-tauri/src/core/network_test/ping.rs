use anyhow::Context;
use regex::Regex;
use tokio::process::Command;

use crate::types::PingResult;

pub async fn run_ping(target: &str, count: u32) -> anyhow::Result<PingResult> {
    let count = count.clamp(1, 20);

    let output = {
        #[cfg(target_os = "windows")]
        {
            Command::new("ping")
                .arg("-n")
                .arg(count.to_string())
                .arg(target)
                .output()
                .await
                .context("failed to run ping")?
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("ping")
                .arg("-c")
                .arg(count.to_string())
                .arg(target)
                .output()
                .await
                .context("failed to run ping")?
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let samples = parse_ping_samples(&stdout)?;
    let loss = parse_packet_loss(&stdout);

    let min_ms = samples.iter().copied().reduce(f64::min);
    let max_ms = samples.iter().copied().reduce(f64::max);
    let avg_ms = if samples.is_empty() {
        None
    } else {
        Some(samples.iter().sum::<f64>() / samples.len() as f64)
    };

    let jitter_ms = if samples.len() <= 1 {
        None
    } else {
        let mut diff_sum = 0.0;
        for w in samples.windows(2) {
            diff_sum += (w[1] - w[0]).abs();
        }
        Some(diff_sum / (samples.len() as f64 - 1.0))
    };

    Ok(PingResult {
        target: target.to_string(),
        min_ms,
        max_ms,
        avg_ms,
        jitter_ms,
        loss_pct: loss,
        samples,
    })
}

fn parse_ping_samples(stdout: &str) -> anyhow::Result<Vec<f64>> {
    let regex = Regex::new(r"time[=<]([0-9]+(?:\.[0-9]+)?)\s*ms")?;
    let mut samples = Vec::new();
    for cap in regex.captures_iter(stdout) {
        if let Ok(ms) = cap[1].parse::<f64>() {
            samples.push(ms);
        }
    }
    Ok(samples)
}

fn parse_packet_loss(stdout: &str) -> Option<f64> {
    let regex = Regex::new(r"([0-9]+)%\s*(?:loss|packet loss)").ok()?;
    let cap = regex.captures(stdout)?;
    cap.get(1)?.as_str().parse::<f64>().ok()
}
