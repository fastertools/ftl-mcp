.PHONY: all build test lint format check clean

all: format lint test build

build:
	cargo component build --release --target wasm32-wasip1

test:
	cargo test

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

check:
	cargo fmt -- --check
	cargo clippy -- -D warnings
	cargo test

clean:
	cargo clean

# Development helpers
watch:
	cargo watch -x check -x test

publish:
	@VERSION=$$(cargo read-manifest | jq -r .version) && \
	wkg oci push ghcr.io/fastertools/ftl-mcp-gateway:$$VERSION target/wasm32-wasip1/release/ftl_mcp_gateway.wasm
	wkg oci push ghcr.io/fastertools/ftl-mcp-gateway:latest target/wasm32-wasip1/release/ftl_mcp_gateway.wasm
