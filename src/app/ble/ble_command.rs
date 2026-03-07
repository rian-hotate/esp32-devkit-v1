#[derive(Clone, Debug)]
pub enum BleCommand {
    StartAdvertise {
        timeout_ms: u32,
    },
    #[allow(dead_code)]
    StopAdvertise,
    /// 現在のBLE接続状態を取得
    GetState,
    #[allow(dead_code)]
    Shutdown,
}
