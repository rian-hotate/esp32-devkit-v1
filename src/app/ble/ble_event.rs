use crate::app::ble::ble_state::BleState;

/// BLEタスクから発行される状態変化イベント
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BleEvent {
    /// アドバタイズ開始
    AdvertisingStarted,
    /// アドバタイズ停止
    AdvertisingStopped,
    /// デバイス接続
    Connected,
    /// デバイス切断
    Disconnected,
    /// エラー発生
    Error,
    /// 接続/アドバタイズ状態応答
    StateResponse(BleState),
}
