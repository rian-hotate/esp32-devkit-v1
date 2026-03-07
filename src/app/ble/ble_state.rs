/// BLE接続/アドバタイズ状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BleState {
    pub(crate) connected: bool,
    pub(crate) advertising: bool,
    pub(crate) error: bool,
}
