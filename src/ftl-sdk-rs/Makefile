.PHONY: all build test lint format check clean publish publish-dry-run verify-publish version-patch version-minor version-major

all: format lint test build

build:
	cargo build --release --target wasm32-wasip1

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

doc:
	cargo doc --open

# Publishing commands
verify-publish: check
	@echo "Verifying package is ready for publishing..."
	@cargo publish --dry-run
	@echo "✅ Package verification complete"

publish-dry-run: verify-publish
	@echo "Performing dry-run publish to crates.io..."
	@cargo publish --dry-run

publish: verify-publish
	@echo "Publishing to crates.io..."
	@echo "Current version: $$(grep '^version' Cargo.toml | cut -d'"' -f2)"
	@read -p "Are you sure you want to publish? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	cargo publish
	@echo "✅ Published successfully!"
	@echo "View at: https://crates.io/crates/ftl-sdk"

# Version bumping
version-patch:
	@echo "Bumping patch version..."
	@current=$$(grep '^version' Cargo.toml | cut -d'"' -f2); \
	new=$$(echo $$current | awk -F. '{print $$1"."$$2"."$$3+1}'); \
	sed -i.bak "s/version = \"$$current\"/version = \"$$new\"/" Cargo.toml && rm Cargo.toml.bak; \
	echo "Version bumped from $$current to $$new"

version-minor:
	@echo "Bumping minor version..."
	@current=$$(grep '^version' Cargo.toml | cut -d'"' -f2); \
	new=$$(echo $$current | awk -F. '{print $$1"."$$2+1".0"}'); \
	sed -i.bak "s/version = \"$$current\"/version = \"$$new\"/" Cargo.toml && rm Cargo.toml.bak; \
	echo "Version bumped from $$current to $$new"

version-major:
	@echo "Bumping major version..."
	@current=$$(grep '^version' Cargo.toml | cut -d'"' -f2); \
	new=$$(echo $$current | awk -F. '{print $$1+1".0.0"}'); \
	sed -i.bak "s/version = \"$$current\"/version = \"$$new\"/" Cargo.toml && rm Cargo.toml.bak; \
	echo "Version bumped from $$current to $$new"