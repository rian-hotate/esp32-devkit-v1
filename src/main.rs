mod app;
mod common;
mod config;

use esp_idf_sys::link_patches;

use crate::{app::tasks::TaskManager, common::Result};

/// タスク死活監視の間隔（秒）
const TASK_MONITOR_INTERVAL_SECS: u64 = 5;

fn main() -> Result<()> {
    link_patches();

    // ログシステムの初期化
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Application started");

    let manager = TaskManager::start()?;

    log::info!("All tasks started");

    // タスク死活監視ループ
    // いずれかのタスクが異常終了した場合、デバイスを再起動する
    loop {
        std::thread::sleep(std::time::Duration::from_secs(TASK_MONITOR_INTERVAL_SECS));

        if let Some(task_name) = manager.terminated_task_name() {
            log::error!(
                "タスク「{}」が異常終了しました。デバイスを再起動します。",
                task_name
            );
            // esp_restart() は即時リセットのため、ログが UART バッファに
            // フラッシュされるよう少し待つ
            std::thread::sleep(std::time::Duration::from_millis(100));
            unsafe { esp_idf_sys::esp_restart() }
        }
    }
}
