pub mod ble_command;
pub mod ble_event;
pub mod ble_handle;
mod ble_state;
pub mod ble_task;

use esp32_nimble::{
    utilities::mutex::Mutex, uuid128, BLEAdvertisementData, BLEAdvertising, BLEDevice, BLEServer,
    NimbleProperties,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::app::ble::ble_event::BleEvent;
use crate::app::ble::ble_state::BleState;
use crate::common::{Error, Result};
use crate::config::ble::BleConfig;

pub struct Ble {
    advertising: Arc<AtomicBool>,
    error: Arc<AtomicBool>,
    server: Option<&'static mut BLEServer>,
    advertiser: Option<&'static Mutex<BLEAdvertising>>,
    event_sink: Option<Arc<dyn Fn(BleEvent) + Send + Sync>>,
}

impl Ble {
    pub fn new() -> Self {
        Self {
            advertising: Arc::new(AtomicBool::new(false)),
            error: Arc::new(AtomicBool::new(false)),
            advertiser: None,
            server: None,
            event_sink: None,
        }
    }

    pub fn set_event_sink(&mut self, sink: Arc<dyn Fn(BleEvent) + Send + Sync>) {
        self.event_sink = Some(sink);
    }

    /// BLEスタック初期化（1回だけ呼ばれる想定）
    pub fn init(&mut self) -> Result<()> {
        if self.advertiser.is_some() {
            log::debug!("BLE already initialized");
            return Ok(());
        }

        log::info!("BLE initializing...");
        let device = BLEDevice::take();
        let server = device.get_server();
        let advertiser = device.get_advertising();

        // 切断時の自動アドバタイズ再開を無効化
        server.advertise_on_disconnect(false);

        // ===== GATT Service（最小）=====
        let service = server.create_service(uuid128!(BleConfig::SERVICE_UUID));
        let chr = service.lock().create_characteristic(
            uuid128!(BleConfig::CHARACTERISTIC_UUID),
            NimbleProperties::READ,
        );

        // キャラクタリスティックに値を設定
        chr.lock().set_value(b"hello");

        log::info!(
            "GATT service created: {}, characteristic: {}",
            BleConfig::SERVICE_UUID,
            BleConfig::CHARACTERISTIC_UUID
        );
        log::debug!("GATT service and characteristic created");

        if let Some(sink) = self.event_sink.as_ref().cloned() {
            let connect_sink = sink.clone();
            let disconnect_sink = sink.clone();
            let advertiser_on_connect = advertiser;
            let advertising_state = self.advertising.clone();

            server.on_connect(move |_, _| {
                log::info!("BLE device connected - auto-stopping advertising");
                match advertiser_on_connect.lock().stop() {
                    Ok(_) => {
                        advertising_state.store(false, Ordering::Release);
                    }
                    Err(e) => {
                        log::error!("Failed to stop advertising on connect: {e:?}");
                    }
                }
                (connect_sink)(BleEvent::Connected);
            });

            server.on_disconnect(move |_, _| {
                log::info!("BLE device disconnected");
                (disconnect_sink)(BleEvent::Disconnected);
            });
            log::debug!("Connection callbacks registered");
        } else {
            log::warn!("Event sink not set, callbacks not registered");
        }

        // ===== Advertise データ =====
        advertiser
            .lock()
            .set_data(
                BLEAdvertisementData::new()
                    .name(BleConfig::DEVICE_NAME)
                    .add_service_uuid(uuid128!(BleConfig::SERVICE_UUID)),
            )
            .map_err(|e| Error::new_esp(&format!("set adv data failed: {e:?}")))?;
        log::debug!("Advertisement data configured");

        self.server = Some(server);
        self.advertiser = Some(advertiser);
        log::info!("BLE initialization completed");

        Ok(())
    }

    /// ペアリング(アドバタイズ)を開始
    pub fn start_pairing(&mut self) -> Result<()> {
        log::debug!("start_pairing called");
        self.init()?;

        if self.is_advertising() {
            log::debug!("Advertising already active, skipping start");
            return Ok(());
        }

        if let Some(adv) = &self.advertiser {
            adv.lock()
                .start()
                .map_err(|e| Error::new_esp(&format!("adv start failed: {e:?}")))?;
            self.advertising.store(true, Ordering::Release);
            log::info!("Advertising started");
        }
        Ok(())
    }

    /// ペアリング(アドバタイズ)停止
    pub fn stop_pairing(&mut self) -> Result<()> {
        log::debug!("stop_pairing called");

        if !self.is_advertising() {
            log::debug!("Advertising already stopped, skipping stop");
            return Ok(());
        }

        if let Some(adv) = &self.advertiser {
            adv.lock()
                .stop()
                .map_err(|e| Error::new_esp(&format!("adv stop failed: {e:?}")))?;
            self.advertising.store(false, Ordering::Release);
            log::info!("Advertising stopped");
        } else {
            log::warn!("Advertiser not initialized; skipping stop");
            self.advertising.store(false, Ordering::Release);
        }
        Ok(())
    }

    pub fn on_disconnected(&mut self) -> Result<()> {
        // TODO: 切断後の処理
        Ok(())
    }

    /// 現在のBLE接続状態を取得
    pub fn is_connected(&self) -> bool {
        if let Some(server) = &self.server {
            server.connected_count() > 0
        } else {
            false
        }
    }

    /// 現在のアドバタイズ状態を取得
    pub fn is_advertising(&self) -> bool {
        self.advertising.load(Ordering::Acquire)
    }

    /// エラー状態を設定
    pub fn set_error(&self, value: bool) {
        self.error.store(value, Ordering::Release)
    }

    /// 現在のエラー状態を取得
    pub fn has_error(&self) -> bool {
        self.error.load(Ordering::Acquire)
    }

    /// 現在のBLE状態を取得
    pub fn state(&self) -> BleState {
        BleState {
            connected: self.is_connected(),
            advertising: self.is_advertising(),
            error: self.has_error(),
        }
    }
}
