use std::sync::mpsc;
use std::thread;

use esp_idf_hal::delay::FreeRtos;

use crate::app::events::ui_cmd::UiCommand;
use crate::app::led::{led_command::LedCommand, led_handle::LedHandle};
use crate::common::{Error, Result};
use termination_detector::TerminationDetector;

/// UI 表示を管理するコントローラ
/// - AppController からの UiCommand を受信して LedTask に LedCommand を発行する
pub struct UiController {
    detector: TerminationDetector,
}

impl UiController {
    pub fn start(ui_cmd_rx: mpsc::Receiver<UiCommand>, ledhandle: LedHandle) -> Result<Self> {
        let handle = thread::Builder::new()
            .name("ui_controller".into())
            .stack_size(4096)
            .spawn(move || {
                log::info!("UiController started");

                loop {
                    while let Ok(cmd) = ui_cmd_rx.try_recv() {
                        log::debug!("UiController: ui command {:?}", cmd);
                        let led_cmd = match cmd {
                            UiCommand::ShowPairing => LedCommand::Blink { interval_ms: 500 },
                            UiCommand::ShowConnected => LedCommand::On,
                            UiCommand::ShowIdle => LedCommand::Off,
                            UiCommand::ShowError => LedCommand::Blink { interval_ms: 100 },
                        };
                        let _ = ledhandle.tx.send(led_cmd);
                    }

                    FreeRtos::delay_ms(20);
                }
            })
            .map_err(|e| {
                Error::new_unexpected(&format!("failed to spawn ui_controller: {e}"))
            })?;

        Ok(Self {
            detector: TerminationDetector::new_no_shutdown(handle),
        })
    }

    /// スレッドが予期せず終了しているかを返す（シャットダウン手段がないため終了は常に異常）
    pub fn is_abnormally_terminated(&self) -> bool {
        self.detector.is_abnormally_terminated()
    }
}
