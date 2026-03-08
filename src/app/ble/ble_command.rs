#[derive(Clone, Debug)]
pub enum BleCommand {
    StartAdvertise {
        timeout_ms: u32,
    },
    #[allow(dead_code)]
    StopAdvertise,
    #[allow(dead_code)]
    Shutdown,
}
