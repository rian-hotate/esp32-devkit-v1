// app/tasks/ble_handle.rs
use std::sync::mpsc;

use crate::app::ble::ble_command::BleCommand;

/// 外部公開：BLEコマンドを送るためのハンドル（Queue送信のみ）
#[derive(Clone)]
pub struct BleHandle {
    pub(crate) tx: mpsc::Sender<BleCommand>,
}
