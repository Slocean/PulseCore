use std::path::PathBuf;

use anyhow::Context;

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    ipc::commands,
    state::SharedState,
    types::{AppSettings, Mode},
};

pub fn start_telemetry_loop(app: AppHandle, state: SharedState) {
    tauri::async_runtime::spawn(async move {
        let mut tick_count: u64 = 0;

        loop {
            let snapshot = state.collect_snapshot().await;
            state.record_snapshot(snapshot.clone()).await;

            if let Err(e) = app.emit("telemetry://snapshot", snapshot) {
                tracing::warn!("failed to emit telemetry snapshot: {e}");
            }

            tick_count = tick_count.saturating_add(1);
            if tick_count % 180 == 0 {
                if let Err(e) = state.prune_history().await {
                    let _ = app.emit(
                        "system://warning",
                        crate::types::WarningEvent {
                            message: e.to_string(),
                            source: "history_prune".to_string(),
                        },
                    );
                }
            }

            let mode = *state.mode.read().await;
            let settings: AppSettings = state.settings.read().await.clone();
            let interval_ms = match mode {
                Mode::Normal => settings.refresh_rate_ms,
                Mode::LowPower => settings.low_power_rate_ms,
            }
            .clamp(100, 10_000);

            tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
        }
    });
}

pub fn ensure_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("overlay").is_some() {
        return Ok(());
    }

    // Create lazily. The initial position/size is controlled by tauri.conf.json, but
    // we keep a fallback here for runtime-created window.
    tauri::WebviewWindowBuilder::new(app, "overlay", tauri::WebviewUrl::App("index.html#/overlay".into()))
        .title("PulseCore Overlay")
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .inner_size(340.0, 260.0)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    Ok(())
}

pub fn app_data_paths(app: &tauri::App) -> anyhow::Result<(PathBuf, PathBuf)> {
    let app_data = app.path().app_data_dir().context("failed to resolve app data dir")?;
    Ok((app_data.join("pulsecore.db"), app_data.join("exports")))
}

pub fn register_invoke_handler(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        commands::get_initial_state,
        commands::get_hardware_info,
        commands::get_settings,
        commands::update_settings,
        commands::start_speed_test,
        commands::cancel_speed_test,
        commands::run_ping_test,
        commands::query_history,
        commands::export_history_csv,
        commands::toggle_overlay,
        commands::set_low_power_mode
    ])
}

