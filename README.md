# PulseCore

PulseCore is a Windows-first desktop performance monitor built with Rust + Tauri + Vue3.

## Implemented MVP Scope

- Real-time telemetry event stream (`telemetry://snapshot`)
- Core metrics cards and live charts (CPU/GPU best-effort, memory, disk, network)
- HTTP speed test + ping diagnostics (`network://speedtest_progress`, `network://speedtest_done`, `network://ping_done`)
- Settings persistence in SQLite
- History query and CSV export commands
- Compact overlay window and low-power sampling mode toggle
- Bilingual UI (`zh-CN`, `en-US`)

## Project Layout

- `src/` - Vue3 renderer
- `src-tauri/` - Rust backend, telemetry collectors, IPC commands, SQLite persistence
- `design/` - PRD and UI reference files

## Notes

- GPU and temperature collection are best-effort by design. Missing sensors are surfaced as `N/A`.
- This implementation targets Windows first and keeps interfaces ready for phase-2 expansion.
