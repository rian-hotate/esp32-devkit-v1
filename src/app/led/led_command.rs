#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LedCommand {
    Blink {
        interval_ms: u32,
    },
    On,
    Off,
    #[allow(dead_code)]
    Shutdown,
}
