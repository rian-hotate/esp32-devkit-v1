use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use esp_idf_hal::delay::FreeRtos;

use super::Tasks;
use crate::app::ble::{ble_command::BleCommand, ble_event::BleEvent};
use crate::app::button::event::ButtonEvent;
use crate::app::led::led_command::LedCommand;
use crate::common::{Error, Result};

/// イベント集約・制御タスク
/// 各タスクからのイベントを受信し、システム全体を調整する
pub struct EventCoordinator {
    _handle: JoinHandle<()>,
}

impl EventCoordinator {
    pub fn start(
        tasks: Arc<Tasks>,
    ) -> Result<(Self, mpsc::Sender<ButtonEvent>, mpsc::Sender<BleEvent>)> {
        let (button_tx, button_rx) = mpsc::channel::<ButtonEvent>();
        let (ble_tx, ble_rx) = mpsc::channel::<BleEvent>();

        let h = thread::Builder::new()
            .name("event_coordinator".into())
            .stack_size(4096)
            .spawn(move || {
                log::info!("Event coordinator started");

                loop {
                    // ボタンイベント処理
                    while let Ok(event) = button_rx.try_recv() {
                        log::debug!("Button event received: {:?}", event);
                        match event {
                            ButtonEvent::LongPress => {
                                // 長押し：BLEアドバタイズ開始
                                log::info!("Button: Long press detected, starting BLE advertising");
                                tasks.send_ble_command(BleCommand::StartAdvertise {
                                    timeout_ms: 60000,
                                });
                            }
                            ButtonEvent::ShortPress => {
                                // 短押し：将来の拡張用
                                log::debug!("Button: Short press detected (not yet implemented)");
                            }
                        }
                    }

                    // BLEイベント処理
                    while let Ok(event) = ble_rx.try_recv() {
                        log::debug!("BLE event received: {:?}", event);
                        match event {
                            BleEvent::AdvertisingStarted => {
                                log::info!("BLE: Advertising started");
                                let cmd = LedCommand::Blink { interval_ms: 500 };
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                            BleEvent::AdvertisingStopped => {
                                log::debug!("BLE: Advertising stopped");
                                let cmd = LedCommand::Off;
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                            BleEvent::Connected => {
                                log::info!("BLE: Connected");
                                let cmd = LedCommand::On;
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                            BleEvent::Disconnected => {
                                log::info!("BLE: Disconnected");
                                let cmd = LedCommand::Off;
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                            BleEvent::Error => {
                                log::warn!("BLE: Error detected");
                                let cmd = LedCommand::Blink { interval_ms: 100 };
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                            BleEvent::StateResponse(state) => {
                                let cmd = if state.error {
                                    log::warn!("BLE: Error state, LED blinking (100ms)");
                                    LedCommand::Blink { interval_ms: 100 }
                                } else if state.connected {
                                    log::info!("BLE: Connected, LED ON");
                                    LedCommand::On
                                } else if state.advertising {
                                    log::info!("BLE: Advertising, LED blinking (500ms)");
                                    LedCommand::Blink { interval_ms: 500 }
                                } else {
                                    log::info!("BLE: Not connected, LED off");
                                    LedCommand::Off
                                };
                                log::debug!("Sending LED command: {:?}", cmd);
                                tasks.send_led_command(cmd);
                            }
                        }
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| {
                Error::new_unexpected(&format!("failed to spawn event_coordinator: {e}"))
            })?;

        Ok((Self { _handle: h }, button_tx, ble_tx))
    }
}
