use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use esp_idf_hal::delay::FreeRtos;

use crate::app::ble::{ble_command::BleCommand, ble_event::BleEvent, ble_handle::BleHandle};
use crate::app::events::app_event::AppEvent;
use crate::app::events::ble_ctrl_cmd::BleCtrlCommand;
use crate::common::{Error, Result};

/// BLE ライフサイクルを管理するコントローラ
/// - AppController からの BleCtrlCommand を受信して BleTask に BleCommand を発行する
/// - BleTask からの BleEvent を受信して AppEvent に変換し AppController へ報告する
pub struct BleController {
    _handle: JoinHandle<()>,
}

impl BleController {
    pub fn start(
        ble_ctrl_rx: mpsc::Receiver<BleCtrlCommand>,
        ble_event_rx: mpsc::Receiver<BleEvent>,
        ble_handle: BleHandle,
        app_event_tx: mpsc::Sender<AppEvent>,
    ) -> Result<Self> {
        let handle = thread::Builder::new()
            .name("ble_controller".into())
            .stack_size(4096)
            .spawn(move || {
                log::info!("BleController started");

                loop {
                    // AppController からのコマンド処理
                    while let Ok(cmd) = ble_ctrl_rx.try_recv() {
                        log::debug!("BleController: ble ctrl command {:?}", cmd);
                        match cmd {
                            BleCtrlCommand::StartPairing { timeout_ms } => {
                                let _ = ble_handle
                                    .tx
                                    .send(BleCommand::StartAdvertise { timeout_ms });
                            }
                            BleCtrlCommand::StopPairing => {
                                let _ = ble_handle.tx.send(BleCommand::StopAdvertise);
                            }
                        }
                    }

                    // BleTask からのイベントを AppEvent に変換して報告
                    while let Ok(event) = ble_event_rx.try_recv() {
                        log::debug!("BleController: ble event {:?}", event);
                        let app_event = match event {
                            BleEvent::AdvertisingStarted => Some(AppEvent::PairingStarted),
                            BleEvent::AdvertisingStopped => Some(AppEvent::PairingStopped),
                            BleEvent::Connected => Some(AppEvent::DeviceConnected),
                            BleEvent::Disconnected => Some(AppEvent::DeviceDisconnected),
                            BleEvent::Error => Some(AppEvent::BleError),
                            // GetState レスポンスはコントローラレベルでは使用しない
                            BleEvent::StateResponse(_) => None,
                        };
                        if let Some(e) = app_event {
                            let _ = app_event_tx.send(e);
                        }
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| {
                Error::new_unexpected(&format!("failed to spawn ble_controller: {e}"))
            })?;

        Ok(Self { _handle: handle })
    }

    /// スレッドが予期せず終了しているかを返す（シャットダウン手段がないため終了は常に異常）
    pub fn is_abnormally_terminated(&self) -> bool {
        self._handle.is_finished()
    }
}
