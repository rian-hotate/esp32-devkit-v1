/// コーディネータ間で共有されるアプリレベルのイベント
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    /// ペアリング（アドバタイズ）開始
    PairingStarted,
    /// デバイス接続
    DeviceConnected,
    /// デバイス切断
    DeviceDisconnected,
    /// ペアリング（アドバタイズ）停止
    PairingStopped,
    /// BLE エラー発生
    BleError,
}
