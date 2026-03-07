/// AppController → UiController へのコマンド
#[derive(Debug, Clone, Copy)]
pub enum UiCommand {
    /// ペアリング中（点滅）
    ShowPairing,
    /// 接続済み（点灯）
    ShowConnected,
    /// 待機中（消灯）
    ShowIdle,
    /// エラー（高速点滅）
    ShowError,
}
