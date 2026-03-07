use esp_idf_hal::delay::FreeRtos;

use crate::{
    app::ble::{ble_command::BleCommand, ble_event::BleEvent, ble_handle::BleHandle, Ble},
    common::{Error, Result},
};
use std::{
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub struct BleTask {
    #[allow(dead_code)]
    handle: JoinHandle<()>,
}

impl BleTask {
    /// BleTask を起動する
    /// - event_tx: BleEvent を BleController へ送るチャンネル
    /// - 戻り値: (BleTask, BleHandle) — BleHandle で BleCommand を送信できる
    pub fn start(event_tx: mpsc::Sender<BleEvent>) -> Result<(Self, BleHandle)> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<BleCommand>();

        let handle = thread::Builder::new()
            .name("ble_task".into())
            .stack_size(8192)
            .spawn(move || {
                log::info!("BLE task started");
                let mut ble = Ble::new();

                // NimBLE コールバック用に event_tx をクローン
                let event_tx_sink = event_tx.clone();
                ble.set_event_sink(Arc::new(move |event| {
                    log::debug!("BLE event emitted: {:?}", event);
                    let _ = event_tx_sink.send(event);
                }));

                let mut pairing_deadline: Option<Instant> = None;

                loop {
                    // コマンド処理
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        log::debug!("BLE command received: {:?}", cmd);
                        match cmd {
                            BleCommand::StartAdvertise { timeout_ms } => {
                                log::info!("Processing StartAdvertise (timeout: {}ms)", timeout_ms);
                                match ble.start_pairing() {
                                    Ok(()) => {
                                        ble.set_error(false);
                                        let _ = event_tx.send(BleEvent::AdvertisingStarted);
                                        pairing_deadline = Some(
                                            Instant::now()
                                                + Duration::from_millis(timeout_ms as u64),
                                        );
                                        log::info!("Advertising started, waiting for connections");
                                    }
                                    Err(e) => {
                                        ble.set_error(true);
                                        let _ = event_tx.send(BleEvent::Error);
                                        log::error!("Failed to start pairing: {e}");
                                    }
                                }
                            }
                            BleCommand::StopAdvertise => {
                                log::info!("Processing StopAdvertise");
                                match ble.stop_pairing() {
                                    Ok(()) => {
                                        ble.set_error(false);
                                        let _ = event_tx.send(BleEvent::AdvertisingStopped);
                                    }
                                    Err(e) => {
                                        ble.set_error(true);
                                        let _ = event_tx.send(BleEvent::Error);
                                        log::error!("Failed to stop pairing: {e}");
                                    }
                                }
                                pairing_deadline = None;
                            }
                            BleCommand::GetState => {
                                let state = ble.state();
                                log::debug!(
                                    "Processing GetState: connected={}, advertising={}",
                                    state.connected,
                                    state.advertising
                                );
                                let _ = event_tx.send(BleEvent::StateResponse(state));
                            }
                            BleCommand::Shutdown => {
                                log::info!("Processing Shutdown");
                                let _ = ble.stop_pairing();
                                let _ = ble.on_disconnected();
                                log::info!("BLE task shutting down");
                                return;
                            }
                        }
                    }

                    // タイムアウト処理
                    if let Some(deadline) = pairing_deadline {
                        if Instant::now() >= deadline {
                            log::warn!("Pairing timeout reached");
                            if ble.is_connected() {
                                log::info!(
                                    "Pairing timeout reached but device is connected; skipping stop_pairing"
                                );
                                pairing_deadline = None;
                            } else {
                                match ble.stop_pairing() {
                                    Ok(()) => {
                                        let _ = event_tx.send(BleEvent::AdvertisingStopped);
                                    }
                                    Err(e) => {
                                        ble.set_error(true);
                                        let _ = event_tx.send(BleEvent::Error);
                                        log::error!("Failed to stop pairing on timeout: {e}");
                                    }
                                }
                                pairing_deadline = None;
                            }
                        }
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| Error::new_unexpected(&format!("failed to spawn ble_task: {e}")))?;

        Ok((Self { handle }, BleHandle { tx: cmd_tx }))
    }

    /// スレッドが終了しているか確認する（異常終了検知用）
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}
