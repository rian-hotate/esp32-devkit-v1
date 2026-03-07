# プロジェクト概要

ESP32 DevKit V1 向けの Rust ファームウェア（ESP-IDF）。
ボタン長押しで BLE アドバタイズを開始し、接続状態に応じて LED を制御する。

- **言語**: Rust（esp-idf-hal / esp32-nimble）
- **目的**: 組み込み Rust の学習・設計パターンの実験

すべての説明・コメントは日本語で記述する。

---

# アーキテクチャ設計方針

## 3層構造

```
ハードウェアTask層    → イベント発行・コマンド実行のみ（ロジックを持たない）
コントローラ層        → ドメインロジック・イベント変換・意思決定
イベント/コマンド層  → 層をまたぐ型定義
```

## 各層の責務

### ハードウェアTask（`src/app/ble/`, `src/app/button/`, `src/app/led/`）
- GPIO や BLE ラジオなどハードウェアを直接操作する
- イベントを上位へ送信し、コマンドを受信して実行するだけ
- **ビジネスロジックを持たない**

### コントローラ（`src/app/controllers/`）
- `AppController`: ボタン入力を受けて BLE/UI に指示を出す意思決定の中枢
- `BleController`: BLE ライフサイクル管理。低レベル `BleEvent` → `AppEvent` に変換
- `UiController`: `AppEvent` を LED コマンドに変換して表示を管理

### イベント/コマンド（`src/app/events/`）
- `AppEvent`: コントローラ間の高レベルイベント
- `BleCtrlCommand`: AppController → BleController へのコマンド
- `UiCommand`: AppController → UiController へのコマンド

## チャンネル設計
- `TaskManager` がすべての mpsc チャンネルを生成・配線する（配線ハーネス）
- 各コンポーネントは自分が受け取るチャンネルの receiver のみを持つ

---

# 新機能追加時のガイドライン

## 新しいハードウェアを追加する場合
1. `src/app/<device>/` にタスクと `XxxEvent`/`XxxCommand` を追加
2. タスクはイベント送信・コマンド受信のみ実装する
3. `TaskManager` でチャンネルを生成・配線する

## 新しい制御ロジックを追加する場合
1. 既存コントローラに追加するか、責務が独立していれば新しいコントローラを作る
2. コントローラ間通信は `AppEvent` を経由させ、直接参照しない
3. `TaskManager` で新しいコントローラを配線する

## 禁止事項
- ハードウェアTask にビジネスロジック（状態判断・制御フロー）を書かない
- コントローラがハードウェアを直接操作しない（必ずTask経由）
- コントローラ同士が直接参照しない（チャンネル経由のみ）
