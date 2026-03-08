# アーキテクチャを自動検出してホストターゲットを決定する
ARCH := $(shell uname -m)
ifeq ($(ARCH), arm64)
    HOST_TARGET := aarch64-apple-darwin
else
    HOST_TARGET := x86_64-apple-darwin
endif

.PHONY: test

## ホストで実行可能なユニットテストを実行する
test:
	cargo test -p termination-detector --target $(HOST_TARGET)
