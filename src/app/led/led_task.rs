use esp_idf_hal::delay::FreeRtos;

use crate::app::led::led_handle::LedHandle;
use crate::app::led::{Led, LedCommand};
use crate::common::{Error, Result};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// LEDタスク本体（スレッド寿命を保持）
pub struct LedTask {
    #[allow(dead_code)]
    handle: JoinHandle<()>,
}

impl LedTask {
    pub fn start(mut led: Led) -> Result<(Self, LedHandle)> {
        let (tx, rx) = mpsc::channel::<LedCommand>();

        let handle = thread::Builder::new()
            .name("led_task".into())
            .stack_size(4096)
            .spawn(move || {
                // 点滅制御用の状態
                let mut blink_interval: Option<u32> = None;
                let mut phase_on = false;
                let mut elapsed_ms: u32 = 0;

                loop {
                    // コマンド処理（キューが空になるまで）
                    while let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            LedCommand::On => {
                                blink_interval = None;
                                phase_on = true;
                                let _ = led.on();
                            }
                            LedCommand::Off => {
                                blink_interval = None;
                                phase_on = false;
                                let _ = led.off();
                            }
                            LedCommand::Blink { interval_ms } => {
                                // Constrain interval to [20ms, 65535ms] to prevent overflow
                                // and ensure reasonable blink rates
                                blink_interval = Some(interval_ms.max(20).min(65535));
                                phase_on = false;
                                elapsed_ms = 0;
                            }
                            LedCommand::Shutdown => return,
                        }
                    }

                    // 点滅処理
                    // Note: Timing assumes task loop execution is negligible vs 20ms delay
                    if let Some(interval) = blink_interval {
                        elapsed_ms = elapsed_ms.saturating_add(20);
                        if elapsed_ms >= interval {
                            if phase_on {
                                let _ = led.off();
                            } else {
                                let _ = led.on();
                            }
                            phase_on = !phase_on;
                            elapsed_ms = 0;
                        }
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| Error::new_unexpected(&format!("failed to spawn led_task: {e}")))?;

        Ok((Self { handle }, LedHandle { tx }))
    }
}
