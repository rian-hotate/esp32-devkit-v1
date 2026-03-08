/// AppController → BleController へのコマンド
#[derive(Debug, Clone)]
pub enum BleCtrlCommand {
    /// ペアリング（アドバタイズ）開始
    StartPairing { timeout_ms: u32 },
}
