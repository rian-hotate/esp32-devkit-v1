pub mod event;
pub mod task;

use crate::common::{Error, Result};

use esp_idf_hal::gpio::{Gpio14, Input, PinDriver, Pull};

/// ボタン（Gpio14 / Active-Low：押すとLOW）
pub struct Button {
    pin: PinDriver<'static, Gpio14, Input>,
}

impl Button {
    pub fn new(mut pin: PinDriver<'static, Gpio14, Input>) -> Result<Self> {
        pin.set_pull(Pull::Up)
            .map_err(|e| Error::new_invalid_state(&format!("failed to set pull-up: {e}")))?;

        Ok(Self { pin })
    }

    pub fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }
}
