use crate::common::{Error, Result};
use esp_idf_hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;

// build.rs で生成されるピン設定
include!(concat!(env!("OUT_DIR"), "/pins_gen.rs"));

pub struct Pins {
    pub led: PinDriver<'static, LedPinType, Output>,
    pub button: PinDriver<'static, ButtonPinType, Input>,
}

impl Pins {
    pub fn take() -> Result<Self> {
        let peripherals = Peripherals::take()
            .map_err(|e| Error::new_esp(&format!("failed to take peripherals: {e}")))?;

        // build.rs で生成した関数で、必要なピンだけを取り出す
        let (led_raw, button_raw) = split_pins(peripherals);

        let led = PinDriver::output(led_raw)
            .map_err(|e| Error::new_esp(&format!("failed to output: {e}")))?;

        let mut button = PinDriver::input(button_raw)
            .map_err(|e| Error::new_esp(&format!("failed to init button pin: {e}")))?;
        button
            .set_pull(Pull::Up)
            .map_err(|e| Error::new_esp(&format!("failed to set button pullup: {e}")))?;

        Ok(Self { led, button })
    }
}
