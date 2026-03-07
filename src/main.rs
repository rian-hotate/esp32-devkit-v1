mod app;
mod common;
mod config;

use esp_idf_sys::link_patches;

use crate::{app::tasks::TaskManager, common::Result};

fn main() -> Result<()> {
    link_patches();

    // ログシステムの初期化
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Application started");

    let mut manager = TaskManager::new();
    manager.start()?;

    log::info!("All tasks started");

    // mainは生かしておく（タスク側が動き続ける）
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
