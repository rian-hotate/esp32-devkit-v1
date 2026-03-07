pub mod event_coordinator;
pub mod task_manager;

use std::sync::{mpsc, Arc, Mutex, PoisonError};

use crate::app::ble::{ble_event::BleEvent, ble_handle::BleHandle};
use crate::app::button::event::ButtonEvent;
use crate::app::led::led_handle::LedHandle;

pub use task_manager::TaskManager;

/// タスク間で共有される状態を持つ構造体（各タスクのハンドルとイベント送信チャネルを保持）
pub struct Tasks {
    led_handle: Mutex<Option<LedHandle>>,
    ble_handle: Mutex<Option<BleHandle>>,
    button_event_tx: Mutex<Option<mpsc::Sender<ButtonEvent>>>,
    ble_event_tx: Mutex<Option<mpsc::Sender<BleEvent>>>,
}

impl Tasks {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            led_handle: Mutex::new(None),
            ble_handle: Mutex::new(None),
            button_event_tx: Mutex::new(None),
            ble_event_tx: Mutex::new(None),
        })
    }

    // Helper to handle mutex poison gracefully
    fn lock_or_log<'a, T>(mutex: &'a Mutex<T>, msg: &str) -> Option<std::sync::MutexGuard<'a, T>> {
        match mutex.lock() {
            Ok(guard) => Some(guard),
            Err(PoisonError { .. }) => {
                log::error!("mutex poisoned ({}); dropping operation", msg);
                None
            }
        }
    }

    pub fn set_led_handle(&self, handle: LedHandle) {
        if let Some(mut guard) = Self::lock_or_log(&self.led_handle, "led_handle") {
            *guard = Some(handle);
        }
    }

    pub fn set_ble_handle(&self, handle: BleHandle) {
        if let Some(mut guard) = Self::lock_or_log(&self.ble_handle, "ble_handle") {
            *guard = Some(handle);
        }
    }

    pub fn send_led_command(&self, cmd: crate::app::led::led_command::LedCommand) {
        if let Some(guard) = Self::lock_or_log(&self.led_handle, "led_handle") {
            match guard.as_ref() {
                Some(handle) => {
                    if let Err(e) = handle.tx.send(cmd) {
                        log::error!("failed to send led command: {e}");
                    }
                }
                None => log::warn!("led handle not set; dropping led command"),
            }
        }
    }

    #[allow(dead_code)]
    pub fn send_ble_command(&self, cmd: crate::app::ble::ble_command::BleCommand) {
        if let Some(guard) = Self::lock_or_log(&self.ble_handle, "ble_handle") {
            match guard.as_ref() {
                Some(handle) => {
                    if let Err(e) = handle.tx.send(cmd) {
                        log::error!("failed to send ble command: {e}");
                    }
                }
                None => log::warn!("ble handle not set; dropping ble command"),
            }
        }
    }

    pub fn set_button_event_tx(&self, tx: mpsc::Sender<ButtonEvent>) {
        if let Some(mut guard) = Self::lock_or_log(&self.button_event_tx, "button_event_tx") {
            *guard = Some(tx);
        }
    }

    pub fn send_button_event(&self, event: ButtonEvent) {
        if let Some(guard) = Self::lock_or_log(&self.button_event_tx, "button_event_tx") {
            if let Some(tx) = guard.as_ref() {
                if let Err(e) = tx.send(event) {
                    log::error!("failed to send button event: {e}");
                }
            } else {
                log::warn!("button event channel not set; dropping event");
            }
        }
    }

    pub fn set_ble_event_tx(&self, tx: mpsc::Sender<BleEvent>) {
        if let Some(mut guard) = Self::lock_or_log(&self.ble_event_tx, "ble_event_tx") {
            *guard = Some(tx);
        }
    }

    pub fn send_ble_event(&self, event: BleEvent) {
        if let Some(guard) = Self::lock_or_log(&self.ble_event_tx, "ble_event_tx") {
            if let Some(tx) = guard.as_ref() {
                if let Err(e) = tx.send(event) {
                    log::error!("failed to send ble event: {e}");
                }
            } else {
                log::warn!("ble event channel not set; dropping event");
            }
        }
    }
}
