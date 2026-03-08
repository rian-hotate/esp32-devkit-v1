/// AppController → UiController へのコマンド
#[derive(Debug, Clone, Copy)]
pub enum UiCommand {
    /// ペアリング中（点滅）
    Pairing,
    /// 接続済み（点灯）
    Connected,
    /// 待機中（消灯）
    Idle,
    /// エラー（高速点滅）
    Error,
}
