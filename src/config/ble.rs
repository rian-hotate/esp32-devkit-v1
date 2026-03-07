// build.rs で生成される BLE 設定
include!(concat!(env!("OUT_DIR"), "/ble_gen.rs"));

/// BLE設定の参照用ヘルパー（pins.rsと同様の配置）
pub struct BleConfig;

impl BleConfig {
    pub const SERVICE_UUID: &'static str = BLE_SERVICE_UUID;
    pub const CHARACTERISTIC_UUID: &'static str = BLE_CHARACTERISTIC_UUID;
    pub const DEVICE_NAME: &'static str = BLE_DEVICE_NAME;
}
