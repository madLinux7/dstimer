VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
BIN     := dstimer
TARGET  := target/release/$(BIN)

.PHONY: build release clean

## Default: build release binary + compress with UPX
release: build upx

## Build optimized release binary
build:
	cargo build --release

## Compress binary with UPX (skip on macOS)
upx: build
ifeq ($(shell uname),Darwin)
	@echo "Skipping UPX on macOS (breaks code signing)"
else
	@command -v upx >/dev/null 2>&1 || { echo "Error: upx not found. Install it first."; exit 1; }
	upx --best --lzma $(TARGET)
endif

## Build without UPX
build-only: build

## Show binary size before/after
size: build
	@echo "Before UPX:"
	@ls -lh $(TARGET) | awk '{print $$5, $$9}'
	@$(MAKE) upx
	@echo "After UPX:"
	@ls -lh $(TARGET) | awk '{print $$5, $$9}'

## Clean build artifacts
clean:
	cargo clean

## Print version from Cargo.toml
version:
	@echo $(VERSION)
