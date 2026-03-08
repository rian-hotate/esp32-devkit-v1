# rustc からホストターゲットを取得する（macOS / Linux 問わず動作）
HOST_TARGET := $(shell rustc -vV | grep '^host:' | cut -d' ' -f2)

.PHONY: test lint lint-esp

## lint チェックしてからユニットテストを実行する
test: lint
	cargo test -p termination-detector --target $(HOST_TARGET)

## termination-detector クレートの lint チェックを実行する（ホスト向け）
lint:
	cargo clippy -p termination-detector --target $(HOST_TARGET) -- -D warnings

## メインクレートの lint チェックを実行する（ESP32 ツールチェーン要）
lint-esp:
	cargo clippy --target xtensa-esp32-espidf -- -D warnings
