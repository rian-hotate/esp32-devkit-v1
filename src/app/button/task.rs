use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use esp_idf_hal::delay::FreeRtos;

use super::{event::ButtonEvent, Button};
use crate::common::{Error, Result};

pub struct ButtonTask {
    _handle: JoinHandle<()>,
}

impl ButtonTask {
    /// ButtonTask を起動する
    /// - event_tx: ButtonEvent を AppController へ送るチャンネル
    pub fn start(button: Button, event_tx: mpsc::Sender<ButtonEvent>) -> Result<Self> {
        let h = thread::Builder::new()
            .name("button_task".into())
            .stack_size(4096)
            .spawn(move || {
                const POLL_MS: u32 = 20;
                const LONG_PRESS_MS: u32 = 3000;

                let mut pressed_ms: u32 = 0;
                let mut fired: bool = false;

                loop {
                    let pressed = button.is_pressed();

                    if pressed {
                        if pressed_ms < LONG_PRESS_MS {
                            pressed_ms = pressed_ms.saturating_add(POLL_MS);
                        }

                        // 3秒到達で1回だけ発火
                        if !fired && pressed_ms >= LONG_PRESS_MS {
                            let _ = event_tx.send(ButtonEvent::LongPress);
                            fired = true;
                        }
                    } else {
                        pressed_ms = 0;
                        fired = false;
                    }

                    FreeRtos::delay_ms(POLL_MS);
                }
            })
            .map_err(|e| Error::new_unexpected(&format!("failed to spawn button_task: {e}")))?;

        Ok(Self { _handle: h })
    }

    /// スレッドが終了しているか確認する（異常終了検知用）
    pub fn is_finished(&self) -> bool {
        self._handle.is_finished()
    }
}
