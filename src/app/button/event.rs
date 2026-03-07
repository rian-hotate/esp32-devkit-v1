/// ボタンから発行されるイベント
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    /// 短押し（未使用の場合は将来用）
    #[allow(dead_code)]
    ShortPress,
    /// 長押し（3秒以上）
    LongPress,
}
