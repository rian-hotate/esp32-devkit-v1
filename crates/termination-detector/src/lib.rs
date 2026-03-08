use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::JoinHandle;

/// スレッドの「正常終了」と「異常終了」を区別する検出器
///
/// - [`TerminationDetector::new`][]: Shutdown コマンドを持つタスク向け。
///   フラグをセットせずにスレッドが終了した場合のみ異常と判定する。
/// - [`TerminationDetector::new_no_shutdown`][]: シャットダウン手段のないタスク向け。
///   スレッドが終了した時点で常に異常と判定する。
pub struct TerminationDetector {
    handle: JoinHandle<()>,
    shutdown_requested: Arc<AtomicBool>,
}

impl TerminationDetector {
    /// Shutdown コマンドを持つタスク向けのコンストラクタ。
    /// `shutdown_requested` に `true` がセットされた後の終了は正常終了とみなす。
    pub fn new(handle: JoinHandle<()>, shutdown_requested: Arc<AtomicBool>) -> Self {
        Self {
            handle,
            shutdown_requested,
        }
    }

    /// シャットダウン手段を持たないタスク向けのコンストラクタ。
    /// スレッドが終了した場合は常に異常終了とみなす。
    pub fn new_no_shutdown(handle: JoinHandle<()>) -> Self {
        Self {
            handle,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Shutdown コマンドを経由しない予期しない終了かどうかを返す。
    ///
    /// - スレッドがまだ動いている → `false`
    /// - `shutdown_requested` フラグが立った後に終了 → `false`（正常終了）
    /// - フラグなしで終了 → `true`（異常終了）
    pub fn is_abnormally_terminated(&self) -> bool {
        self.handle.is_finished() && !self.shutdown_requested.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// スレッドがまだ実行中の場合は異常終了と判定しない
    #[test]
    fn 実行中スレッドは異常終了ではない() {
        let flag = Arc::new(AtomicBool::new(false));
        let handle = thread::spawn(|| thread::sleep(Duration::from_secs(10)));
        let detector = TerminationDetector::new(handle, flag);

        assert!(!detector.is_abnormally_terminated());
    }

    /// Shutdown フラグをセットせずにスレッドが終了した場合は異常終了とみなす
    #[test]
    fn フラグなしで終了したスレッドは異常終了() {
        let flag = Arc::new(AtomicBool::new(false));
        let handle = thread::spawn(|| { /* 即座に終了（Shutdown コマンドなし） */ });
        handle.thread().unpark();
        // スレッドが終了するのを待つ
        thread::sleep(Duration::from_millis(50));

        let detector = TerminationDetector::new(handle, flag);
        assert!(detector.is_abnormally_terminated());
    }

    /// Shutdown フラグをセットしてからスレッドが終了した場合は正常終了とみなす
    #[test]
    fn フラグをセットして終了したスレッドは正常終了() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let handle = thread::spawn(move || {
            // Shutdown コマンド受信相当の処理
            flag_clone.store(true, Ordering::Relaxed);
        });
        thread::sleep(Duration::from_millis(50));

        let detector = TerminationDetector::new(handle, flag);
        assert!(!detector.is_abnormally_terminated());
    }

    /// new_no_shutdown: スレッドが終了した場合は常に異常終了とみなす
    #[test]
    fn シャットダウン手段なしタスクの終了は異常終了() {
        let handle = thread::spawn(|| { /* 即座に終了 */ });
        thread::sleep(Duration::from_millis(50));

        let detector = TerminationDetector::new_no_shutdown(handle);
        assert!(detector.is_abnormally_terminated());
    }

    /// new_no_shutdown: スレッドがまだ動いている場合は異常終了と判定しない
    #[test]
    fn シャットダウン手段なしタスクの実行中は異常終了ではない() {
        let handle = thread::spawn(|| thread::sleep(Duration::from_secs(10)));
        let detector = TerminationDetector::new_no_shutdown(handle);

        assert!(!detector.is_abnormally_terminated());
    }
}
