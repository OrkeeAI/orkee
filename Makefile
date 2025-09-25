# Orkee Release Automation Makefile

.PHONY: help test release release-dry release-major release-minor release-patch

# Default version from package.json
VERSION := $(shell cat npm/package.json | grep '"version"' | cut -d'"' -f4)

help: ## Show this help message
	@echo "Orkee Release Automation"
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

test: ## Run all tests
	cargo test --all
	pnpm test

build: ## Build release binaries
	cargo build --release --bin orkee

release: ## Create a release with current version
	@echo "Creating release for version $(VERSION)..."
	@./scripts/release.sh --version $(VERSION)

release-dry: ## Dry run release (no actual changes)
	@echo "Dry run for version $(VERSION)..."
	@./scripts/release.sh --version $(VERSION) --dry-run

release-patch: ## Release a patch version (0.0.x)
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1"."$$2"."$$3+1}') && \
	echo "New version: $$NEW_VERSION" && \
	cd npm && npm version $$NEW_VERSION --no-git-tag-version && \
	cd .. && \
	git add npm/package.json && \
	git commit -m "chore: bump version to $$NEW_VERSION" && \
	./scripts/release.sh --version $$NEW_VERSION

release-minor: ## Release a minor version (0.x.0)
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1"."$$2+1".0"}') && \
	echo "New version: $$NEW_VERSION" && \
	cd npm && npm version $$NEW_VERSION --no-git-tag-version && \
	cd .. && \
	git add npm/package.json && \
	git commit -m "chore: bump version to $$NEW_VERSION" && \
	./scripts/release.sh --version $$NEW_VERSION

release-major: ## Release a major version (x.0.0)  
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1+1".0.0"}') && \
	echo "New version: $$NEW_VERSION" && \
	cd npm && npm version $$NEW_VERSION --no-git-tag-version && \
	cd .. && \
	git add npm/package.json && \
	git commit -m "chore: bump version to $$NEW_VERSION" && \
	./scripts/release.sh --version $$NEW_VERSION

install-tools: ## Install required tools for releases
	@echo "Installing release tools..."
	@command -v gh >/dev/null 2>&1 || (echo "Installing GitHub CLI..." && brew install gh)
	@command -v cross >/dev/null 2>&1 || (echo "Installing cross..." && cargo install cross)
	@echo "Adding Rust targets..."
	@rustup target add aarch64-apple-darwin
	@rustup target add x86_64-apple-darwin
	@rustup target add x86_64-unknown-linux-gnu
	@rustup target add aarch64-unknown-linux-gnu
	@rustup target add x86_64-pc-windows-msvc
	@echo "✅ All tools installed"

clean: ## Clean build artifacts
	cargo clean
	rm -rf release-artifacts/
	rm -rf target/