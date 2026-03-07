use std::sync::Arc;
use std::thread::{self, JoinHandle};

use esp_idf_hal::delay::FreeRtos;

use super::{event::ButtonEvent, Button};
use crate::app::tasks::Tasks;
use crate::common::{Error, Result};

pub struct ButtonTask {
    _handle: JoinHandle<()>,
}

impl ButtonTask {
    pub fn start(tasks: Arc<Tasks>, button: Button) -> Result<Self> {
        let h = thread::Builder::new()
            .name("button_task".into())
            .stack_size(4096)
            .spawn(move || {
                // 設定値
                const POLL_MS: u32 = 20;
                const LONG_PRESS_MS: u32 = 3000;

                // 状態
                let mut pressed_ms: u32 = 0;
                let mut fired: bool = false;

                loop {
                    let pressed = button.is_pressed();

                    if pressed {
                        // 押下継続
                        if pressed_ms < LONG_PRESS_MS {
                            pressed_ms = pressed_ms.saturating_add(POLL_MS);
                        }

                        // 3秒到達で1回だけ発火
                        if !fired && pressed_ms >= LONG_PRESS_MS {
                            // ボタンイベントを発行
                            tasks.send_button_event(ButtonEvent::LongPress);
                            fired = true;
                        }
                    } else {
                        // 離したらリセット
                        pressed_ms = 0;
                        fired = false;
                    }

                    FreeRtos::delay_ms(POLL_MS);
                }
            })
            .map_err(|e| Error::new_unexpected(&format!("failed to spawn button_task: {e}")))?;

        Ok(Self { _handle: h })
    }
}
