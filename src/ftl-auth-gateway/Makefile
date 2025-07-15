.PHONY: build test clean lint check release

# Default target
build:
	cargo build --target wasm32-wasip1 --release

# Build the component for publishing
build-component:
	cargo component build --target wasm32-wasip1 --release

test-cargo:
	cargo test

test-spin:
	spin test

# Run tests
test: test-cargo test-spin

# Clean build artifacts
clean:
	cargo clean

# Run linter
lint:
	cargo clippy -- -D warnings

# Format code
format:
	cargo fmt

# Check formatting
format-check:
	cargo fmt -- --check

# Run all checks (format, lint, test)
check: format-check lint test

# Build optimized release
release: clean
	cargo build --target wasm32-wasip1 --release
	@echo "Release build complete: target/wasm32-wasip1/release/ftl_auth_gateway.wasm"

publish: build-component
	@VERSION=$$(cargo read-manifest | jq -r .version) && \
	wkg oci push ghcr.io/fastertools/ftl-auth-gateway:$$VERSION target/wasm32-wasip1/release/ftl_auth_gateway.wasm
	wkg oci push ghcr.io/fastertools/ftl-auth-gateway:latest target/wasm32-wasip1/release/ftl_auth_gateway.wasm

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the auth gateway for WASM"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run clippy linter"
	@echo "  format       - Format code"
	@echo "  format-check - Check code formatting"
	@echo "  check        - Run all checks (format, lint, test)"
	@echo "  release      - Build optimized release"
	@echo "  publish      - Publish to ghcr.io"
	@echo "  help         - Show this help message"