use std::sync::mpsc::Sender;

use crate::app::led::led_command::LedCommand;

/// 外部公開：LED状態を送るためのハンドル（Queue送信のみ）
#[derive(Clone)]
pub struct LedHandle {
    pub tx: Sender<LedCommand>,
}
