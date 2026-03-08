use std::sync::mpsc;

use crate::app::{
    ble::{ble_event::BleEvent, ble_task::BleTask},
    button::{event::ButtonEvent, task::ButtonTask, Button},
    controllers::{
        app_controller::AppController,
        ble_controller::BleController,
        ui_controller::UiController,
    },
    events::{app_event::AppEvent, ble_ctrl_cmd::BleCtrlCommand, ui_cmd::UiCommand},
    led::{led_task::LedTask, Led},
};
use crate::common::Result;
use crate::config::pins::Pins;

/// 全タスク・コントローラのハンドルを保持し、スレッドが生き続けるよう管理する
pub struct TaskManager {
    _led_task: LedTask,
    _ble_task: BleTask,
    _button_task: ButtonTask,
    _ui_ctrl: UiController,
    _ble_ctrl: BleController,
    _app_ctrl: AppController,
}

impl TaskManager {
    /// チャンネルを生成・配線し、全タスク・コントローラを起動する
    pub fn start() -> Result<Self> {
        let pins = Pins::take()?;
        let led = Led::new(pins.led);
        let button = Button::new(pins.button)?;

        // --- コーディネータ間チャンネル ---
        // BleController → AppController
        let (app_event_tx, app_event_rx) = mpsc::channel::<AppEvent>();
        // AppController → BleController
        let (ble_ctrl_tx, ble_ctrl_rx) = mpsc::channel::<BleCtrlCommand>();
        // AppController → UiController
        let (ui_cmd_tx, ui_cmd_rx) = mpsc::channel::<UiCommand>();

        // --- ハードウェアタスク用チャンネル ---
        // ButtonTask → AppController
        let (button_event_tx, button_event_rx) = mpsc::channel::<ButtonEvent>();
        // BleTask → BleController
        let (ble_event_tx, ble_event_rx) = mpsc::channel::<BleEvent>();

        // --- ハードウェアタスク起動 ---
        let (led_task, led_handle) = LedTask::start(led)?;
        let (ble_task, ble_handle) = BleTask::start(ble_event_tx)?;
        let button_task = ButtonTask::start(button, button_event_tx)?;

        // --- コントローラ起動 ---
        let ui_ctrl = UiController::start(ui_cmd_rx, led_handle)?;
        let ble_ctrl = BleController::start(ble_ctrl_rx, ble_event_rx, ble_handle, app_event_tx)?;
        let app_ctrl =
            AppController::start(button_event_rx, app_event_rx, ble_ctrl_tx, ui_cmd_tx)?;

        Ok(Self {
            _led_task: led_task,
            _ble_task: ble_task,
            _button_task: button_task,
            _ui_ctrl: ui_ctrl,
            _ble_ctrl: ble_ctrl,
            _app_ctrl: app_ctrl,
        })
    }

    /// 異常終了したタスク名を返す。全タスク正常動作中は None を返す。
    /// Shutdown コマンドによる正常終了は異常終了と見なさない。
    pub fn terminated_task_name(&self) -> Option<&'static str> {
        if self._button_task.is_abnormally_terminated() {
            return Some("button_task");
        }
        if self._led_task.is_abnormally_terminated() {
            return Some("led_task");
        }
        if self._ble_task.is_abnormally_terminated() {
            return Some("ble_task");
        }
        if self._ui_ctrl.is_abnormally_terminated() {
            return Some("ui_controller");
        }
        if self._ble_ctrl.is_abnormally_terminated() {
            return Some("ble_controller");
        }
        if self._app_ctrl.is_abnormally_terminated() {
            return Some("app_controller");
        }
        None
    }
}
