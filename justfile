# Tuner: Rust/WASM pitch core + SvelteKit app.
# Quickstart: `just dev` (installs what it needs, then serves the app).

set shell := ["bash", "-euo", "pipefail", "-c"]

web := "web"

# List recipes
default:
    @just --list

# Install the wasm target and web dependencies (safe to rerun)
setup:
    rustup target add wasm32-unknown-unknown
    cd {{web}} && pnpm install --frozen-lockfile

# Build the wasm and start the dev server (pass e.g. `--host` to test on a phone)
dev *args: setup
    cd {{web}} && pnpm dev {{args}}

# Build the wasm and the static site into web/build
build: setup
    cd {{web}} && pnpm build

# Serve the production build locally
preview: build
    cd {{web}} && pnpm preview

# Rust and web unit tests
test: setup
    cargo test
    cd {{web}} && pnpm test

# Browser tests in the installed Chrome with a fake microphone
e2e: setup
    cd {{web}} && pnpm test:e2e

# Formatting, clippy, svelte-check, eslint and prettier
lint: setup
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cd {{web}} && pnpm check && pnpm lint

# Everything CI should run: lint, unit tests, build and browser tests
check: lint test build e2e

# Build the production Docker image and serve it on http://localhost:8080
docker:
    docker build -t tuner .
    docker run --rm -p 8080:80 tuner

# Format Rust and web sources
fmt:
    cargo fmt
    cd {{web}} && pnpm format

# Remove build output
clean:
    cargo clean
    rm -rf {{web}}/build {{web}}/.svelte-kit {{web}}/static/wasm {{web}}/test-results
