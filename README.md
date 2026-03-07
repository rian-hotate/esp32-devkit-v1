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

### 図1: コンポーネント構成

```plantuml
@startuml
title コンポーネント構成

left to right direction

package "ハードウェア" {
  [ボタン GPIO] as ButtonHW
  [LED GPIO] as LEDHW
  [BLE ラジオ] as BLERadioHW
}

package "ハードウェアタスク\n(イベント発行・コマンド実行のみ)" {
  [ButtonTask] as ButtonTask
  [BleTask] as BleTask
  [LedTask] as LedTask
}

package "コントローラ\n(ドメインロジック)" {
  [AppController] as AppCtrl
  [BleController] as BleCtrl
  [UiController] as UiCtrl
}

package "esp32_nimble" {
  [BLEServer] as BLEServer
  [BLEAdvertising] as BLEAdv
}

ButtonHW --> ButtonTask
ButtonTask --> AppCtrl : ButtonEvent
AppCtrl --> BleCtrl : BleCtrlCommand
BleCtrl --> BleTask : BleCommand
BleTask --> BLEServer
BleTask --> BLEAdv
BLEAdv --> BLERadioHW
BLEServer --> BleTask : callback
BleTask --> BleCtrl : BleEvent
BleCtrl --> AppCtrl : AppEvent
AppCtrl --> UiCtrl : UiCommand
UiCtrl --> LedTask : LedCommand
LedTask --> LEDHW

@enduml
```

### 図2: イベントフロー

```plantuml
@startuml
title イベントフロー

actor "ユーザー" as User
actor "BLE デバイス" as BLEDevice
participant "ButtonTask" as BT
participant "AppController" as AC
participant "BleController" as BC
participant "BleTask" as BLE
participant "UiController" as UC
participant "LedTask" as LED

== ボタン長押し → アドバタイズ開始 ==
User -> BT : 3秒長押し
BT -> AC : ButtonEvent::LongPress
AC -> BC : BleCtrlCommand::StartPairing\n(timeout: 60s)
BC -> BLE : BleCommand::StartAdvertise
BLE -> BC : BleEvent::AdvertisingStarted
BC -> AC : AppEvent::PairingStarted
AC -> UC : UiCommand::ShowPairing
UC -> LED : LedCommand::Blink(500ms)

== BLE 接続 ==
BLEDevice -> BLE : 接続
BLE -> BC : BleEvent::Connected
BC -> AC : AppEvent::DeviceConnected
AC -> UC : UiCommand::ShowConnected
UC -> LED : LedCommand::On

== BLE 切断 ==
BLEDevice -> BLE : 切断
BLE -> BC : BleEvent::Disconnected
BC -> AC : AppEvent::DeviceDisconnected
AC -> UC : UiCommand::ShowIdle
UC -> LED : LedCommand::Off

== エラー ==
BLE -> BC : BleEvent::Error
BC -> AC : AppEvent::BleError
AC -> UC : UiCommand::ShowError
UC -> LED : LedCommand::Blink(100ms)

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
