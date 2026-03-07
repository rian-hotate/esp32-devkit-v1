pub mod led_command;
pub mod led_handle;
pub mod led_task;

use esp_idf_hal::gpio::{Output, PinDriver};

use crate::app::led::led_command::LedCommand;
use crate::common::{Error, Result};
use crate::config::pins::LedPinType;

pub struct Led {
    pin: PinDriver<'static, LedPinType, Output>,
}

impl Led {
    pub fn new(pin: PinDriver<'static, LedPinType, Output>) -> Self {
        Self { pin }
    }

    pub fn on(&mut self) -> Result<()> {
        self.pin
            .set_high()
            .map_err(|e| Error::new_invalid_state(&format!("failed to set LED HIGH: {e}")))?;
        Ok(())
    }

    pub fn off(&mut self) -> Result<()> {
        self.pin
            .set_low()
            .map_err(|e| Error::new_invalid_state(&format!("failed to set LED LOW: {e}")))?;
        Ok(())
    }
}
