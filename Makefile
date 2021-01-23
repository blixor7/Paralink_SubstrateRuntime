.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --all

.PHONY: run
run:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo run -- --dev --tmp

.PHONY: build
build:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo build --release

.PHONY: watch
watch:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo watch -c -x build

.PHONY: doc
doc:
	SKIP_WASM_BUILD=1 cargo doc --open
