ESP32 DevKit V1 (Rust + ESP-IDF)
=================================

概要
----
ESP32 DevKit V1 向けの BLE + LED 制御サンプルです。
ボタン入力でアドバタイズ開始、BLE接続状態に応じて LED を制御します。

特徴
----
- BLE アドバタイズ開始/停止
- 接続/切断イベントの通知
- BLE 状態に応じた LED 制御
- 状態は `Ble` 構造体が一元管理

構成図 (PlantUML)
-----------------
```plantuml
@startuml
title BLE/LED 現行構成（2026-01-31）

actor User as User

package "App Tasks" {
	[Button Task] as ButtonTask
	[BLE Task] as BleTask
	[LED Task] as LedTask
	[Event Coordinator] as EventCoord
}

package "BLE Module" {
	[Ble (mod.rs)] as Ble
	[BleState] as BleState
	[BleEvent] as BleEvent
	[BleCommand] as BleCommand
}

package "NimBLE" {
	[BLEServer] as BLEServer
	[BLEAdvertising] as BLEAdvertising
}

User --> ButtonTask : ButtonEvent
ButtonTask --> EventCoord : ButtonEvent

EventCoord --> BleTask : BleCommand\n(StartAdvertise/StopAdvertise)
BleTask --> Ble : start_pairing/stop_pairing
BleTask --> EventCoord : BleEvent\n(AdvertisingStarted/Stopped/Connected/Disconnected/Error)

Ble --> BLEServer : on_connect/on_disconnect
Ble --> BLEAdvertising : start/stop
Ble --> BleState : maintains current state

EventCoord --> LedTask : LedCommand\n(on BLE events)

BleState ..> BleEvent : (optional) StateResponse(BleState)

@enduml
```

ビルド
------
```bash
cargo build
```

実行
----
```bash
cargo run
```
