use std::sync::Arc;

use crate::app::{
    ble::ble_task::BleTask,
    button::{task::ButtonTask, Button},
    led::{led_task::LedTask, Led},
    tasks::{event_coordinator, Tasks},
};
use crate::common::Result;
use crate::config::pins::Pins;

/// タスク起動の入口
pub struct TaskManager {
    pub tasks: Arc<Tasks>,

    led_task: Option<LedTask>,
    button_task: Option<ButtonTask>,
    ble_task: Option<BleTask>,
    event_coordinator: Option<event_coordinator::EventCoordinator>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Tasks::new(),
            led_task: None,
            button_task: None,
            ble_task: None,
            event_coordinator: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let pins = Pins::take()?;
        let led = Led::new(pins.led);
        let button = Button::new(pins.button)?;

        self.start_event_coordinator()?;
        self.start_led_task(led)?;
        self.start_button_task(button)?;
        self.start_ble_task()?;

        Ok(())
    }

    fn start_led_task(&mut self, led: Led) -> Result<()> {
        let (led_task, led_handle) = LedTask::start(led)?;
        self.led_task = Some(led_task);
        self.tasks.set_led_handle(led_handle);

        Ok(())
    }

    fn start_button_task(&mut self, button: Button) -> Result<()> {
        let t = ButtonTask::start(self.tasks.clone(), button)?;
        self.button_task = Some(t);
        Ok(())
    }

    fn start_ble_task(&mut self) -> Result<()> {
        let (t, handle) = BleTask::start(self.tasks.clone())?;
        self.ble_task = Some(t);
        self.tasks.set_ble_handle(handle);
        Ok(())
    }

    fn start_event_coordinator(&mut self) -> Result<()> {
        let (coordinator, button_event_tx, ble_event_tx) =
            event_coordinator::EventCoordinator::start(self.tasks.clone())?;
        self.tasks.set_button_event_tx(button_event_tx);
        self.tasks.set_ble_event_tx(ble_event_tx);
        self.event_coordinator = Some(coordinator);
        Ok(())
    }
}
