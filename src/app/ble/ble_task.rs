use esp_idf_hal::delay::FreeRtos;

use crate::{
    app::ble::{ble_command::BleCommand, ble_event::BleEvent, ble_handle::BleHandle, Ble},
    common::{Error, Result},
};
use termination_detector::TerminationDetector;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub struct BleTask {
    detector: TerminationDetector,
}

impl BleTask {
    /// BleTask を起動する
    /// - event_tx: BleEvent を BleController へ送るチャンネル
    /// - 戻り値: (BleTask, BleHandle) — BleHandle で BleCommand を送信できる
    pub fn start(event_tx: mpsc::Sender<BleEvent>) -> Result<(Self, BleHandle)> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<BleCommand>();
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown_requested.clone();

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
                            BleCommand::Shutdown => {
                                log::info!("Processing Shutdown");
                                let _ = ble.stop_pairing();
                                let _ = ble.on_disconnected();
                                shutdown_flag.store(true, Ordering::Relaxed);
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

        Ok((
            Self {
                detector: TerminationDetector::new(handle, shutdown_requested),
            },
            BleHandle { tx: cmd_tx },
        ))
    }

    /// Shutdown コマンドを経由しない予期しない終了かどうかを返す
    pub fn is_abnormally_terminated(&self) -> bool {
        self.detector.is_abnormally_terminated()
    }
}
