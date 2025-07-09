# WASMCP Project Makefile
# Coordinates building and testing the MCP framework on Spin

.PHONY: all build clean test help
.DEFAULT_GOAL := help

# Build targets
WASMCP_TARGET = wasmcp/target/wasm32-wasip1/release/libwasmcp.rlib
ROUTER_TARGET = mcp-router/target/wasm32-wasip1/release/mcp_router.wasm
WEATHER_TARGET = weather_new/target/wasm32-wasip1/release/weather_new.wasm
TEST_RUNNER_TARGET = target/release/mcp-test-runner

help: ## Show this help message
	@echo "WASMCP Project Build System"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

clean: ## Clean all build artifacts
	@echo "Cleaning build artifacts..."
	rm -rf wasmcp/target
	rm -rf mcp-router/target
	rm -rf weather_new/target
	rm -rf target
	rm -rf .spin/
	@echo "Clean complete"

build-wasmcp: ## Build the wasmcp SDK crate
	@echo "Building wasmcp SDK..."
	cd wasmcp && cargo build --target wasm32-wasip1 --release

build-router: build-wasmcp ## Build the MCP router component
	@echo "Building MCP router..."
	cd mcp-router && cargo build --target wasm32-wasip1 --release

build-weather: build-wasmcp ## Build the weather plugin component
	@echo "Building weather plugin..."
	cd weather_new && cargo build --target wasm32-wasip1 --release

build-activity: build-wasmcp ## Build the activity plugin component
	@echo "Building activity plugin..."
	cd quote_plugin && cargo build --target wasm32-wasip1 --release

build-test-runner: ## Build the test runner
	@echo "Building test runner..."
	cargo build --release --bin mcp-test-runner

build: build-wasmcp build-router build-weather build-activity build-test-runner ## Build all components
	@echo "All components built successfully"

test-runner: build kill-spin ## Run the test runner
	@echo "Running MCP test runner..."
	./target/release/mcp-test-runner || ($(MAKE) kill-spin && exit 1)

test-cloud: build-test-runner ## Test cloud deployment (Usage: make test-cloud URL=https://your-deployment.com)
	@echo "Testing cloud deployment..."
	./target/release/mcp-test-runner $(URL)

dev: ## Development mode - build and test
	@echo "Development mode: building and testing..."
	$(MAKE) test-runner

logs: ## Show recent logs from Spin components
	@echo "Recent router logs:"
	@if [ -f .spin/logs/mcp-router_stdout.txt ]; then echo "=== STDOUT ==="; tail -20 .spin/logs/mcp-router_stdout.txt; fi
	@if [ -f .spin/logs/mcp-router_stderr.txt ]; then echo "=== STDERR ==="; tail -20 .spin/logs/mcp-router_stderr.txt; fi

kill-spin: ## Kill any running Spin processes
	@echo "Killing Spin processes..."
	@pkill -f "spin up" 2>/dev/null || true
	@pkill -f "spin trigger" 2>/dev/null || true
	@lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
	@echo "Spin processes killed"

reset: kill-spin clean ## Full reset - kill processes, clean, and rebuild
	@echo "Full reset complete. Run 'make build' to rebuild."

# Utility targets
check-wasm-target: ## Check if wasm32-wasip1 target is installed
	@rustup target list --installed | grep -q wasm32-wasip1 || (echo "Installing wasm32-wasip1 target..." && rustup target add wasm32-wasip1)

setup: check-wasm-target ## Setup development environment
	@echo "Development environment setup complete"