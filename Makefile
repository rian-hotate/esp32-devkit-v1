# アーキテクチャを自動検出してホストターゲットを決定する
ARCH := $(shell uname -m)
ifeq ($(ARCH), arm64)
    HOST_TARGET := aarch64-apple-darwin
else
    HOST_TARGET := x86_64-apple-darwin
endif

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
