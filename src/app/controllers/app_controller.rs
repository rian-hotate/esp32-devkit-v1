use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use esp_idf_hal::delay::FreeRtos;

use crate::app::button::event::ButtonEvent;
use crate::app::events::app_event::AppEvent;
use crate::app::events::ble_ctrl_cmd::BleCtrlCommand;
use crate::app::events::ui_cmd::UiCommand;
use crate::common::{Error, Result};

/// アプリ全体の意思決定を担うコントローラ
/// - ButtonEvent を受信して BleController に指示を出す
/// - BleController からの AppEvent を受信して UiController に表示を指示する
pub struct AppController {
    _handle: JoinHandle<()>,
}

impl AppController {
    pub fn start(
        button_rx: mpsc::Receiver<ButtonEvent>,
        app_event_rx: mpsc::Receiver<AppEvent>,
        ble_ctrl_tx: mpsc::Sender<BleCtrlCommand>,
        ui_cmd_tx: mpsc::Sender<UiCommand>,
    ) -> Result<Self> {
        let handle = thread::Builder::new()
            .name("app_controller".into())
            .stack_size(4096)
            .spawn(move || {
                log::info!("AppController started");

                loop {
                    // ボタンイベント処理
                    while let Ok(event) = button_rx.try_recv() {
                        log::debug!("AppController: button event {:?}", event);
                        match event {
                            ButtonEvent::LongPress => {
                                log::info!("AppController: long press -> StartPairing");
                                let _ = ble_ctrl_tx
                                    .send(BleCtrlCommand::StartPairing { timeout_ms: 60000 });
                            }
                            ButtonEvent::ShortPress => {
                                log::debug!("AppController: short press (未実装)");
                            }
                        }
                    }

                    // BLE コントローラからのアプリイベント処理
                    while let Ok(event) = app_event_rx.try_recv() {
                        log::debug!("AppController: app event {:?}", event);
                        let cmd = match event {
                            AppEvent::PairingStarted => UiCommand::ShowPairing,
                            AppEvent::DeviceConnected => UiCommand::ShowConnected,
                            AppEvent::DeviceDisconnected | AppEvent::PairingStopped => {
                                UiCommand::ShowIdle
                            }
                            AppEvent::BleError => UiCommand::ShowError,
                        };
                        let _ = ui_cmd_tx.send(cmd);
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| {
                Error::new_unexpected(&format!("failed to spawn app_controller: {e}"))
            })?;

        Ok(Self { _handle: handle })
    }
}
