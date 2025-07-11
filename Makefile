.PHONY: all check-all test-all build-all clean-all help

# Default target
all: help

# Help command
help:
	@echo "FTL MCP Repository Management Commands"
	@echo "====================================="
	@echo ""
	@echo "Quality Checks:"
	@echo "  make check-all     - Run all checks for both SDKs"
	@echo "  make test-all      - Run all tests for both SDKs"
	@echo "  make build-all     - Build all components"
	@echo "  make clean-all     - Clean all build artifacts"
	@echo ""
	@echo "SDK-specific commands:"
	@echo "  make check-rs      - Check Rust SDK only"
	@echo "  make check-ts      - Check TypeScript SDK only"
	@echo "  make test-rs       - Test Rust SDK only"
	@echo "  make test-ts       - Test TypeScript SDK only"
	@echo ""
	@echo "Publishing (use from within SDK directories):"
	@echo "  cd src/ftl-sdk-rs && make publish"
	@echo "  cd src/ftl-sdk-ts && npm run publish:npm"

# Check everything
check-all: check-rs check-ts
	@echo "✅ All checks passed!"

check-rs:
	@echo "Checking Rust SDK..."
	@cd src/ftl-sdk-rs && make check

check-ts:
	@echo "Checking TypeScript SDK..."
	@cd src/ftl-sdk-ts && npm run check

# Test everything
test-all: test-rs test-ts
	@echo "✅ All tests passed!"

test-rs:
	@echo "Testing Rust SDK..."
	@cd src/ftl-sdk-rs && make test

test-ts:
	@echo "Testing TypeScript SDK..."
	@cd src/ftl-sdk-ts && npm test

# Build everything
build-all: build-rs build-ts build-examples
	@echo "✅ All builds completed!"

build-rs:
	@echo "Building Rust SDK..."
	@cd src/ftl-sdk-rs && make build

build-ts:
	@echo "Building TypeScript SDK..."
	@cd src/ftl-sdk-ts && npm run build

build-examples:
	@echo "Building example components..."
	@cd examples/gateway-demo && spin build

# Clean everything
clean-all: clean-rs clean-ts
	@echo "✅ All artifacts cleaned!"

clean-rs:
	@echo "Cleaning Rust SDK..."
	@cd src/ftl-sdk-rs && make clean

clean-ts:
	@echo "Cleaning TypeScript SDK..."
	@cd src/ftl-sdk-ts && rm -rf dist node_modules

# Install dependencies
install-deps:
	@echo "Installing TypeScript SDK dependencies..."
	@cd src/ftl-sdk-ts && npm install