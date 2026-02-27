set windows-shell := ["powershell", "-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

# List all available tasks
_default:
    @just --list

# Run in dev mode with hot reload
dev:
	bun tauri icon assets/app-logo.svg
	bun run tauri dev

# Final release build
build:
    bun run tauri build
    upx --best --lzma src-tauri/target/release/Winsentials.exe

# Lint only backend
lint-back:
    cargo check --manifest-path=./src-tauri/Cargo.toml
    cargo clippy --manifest-path=./src-tauri/Cargo.toml

# Lint only frontend
lint-front:
    bun run typecheck
    bun run lint

# Lint both backend and frontend
lint: lint-back lint-front
    opengrep scan

# Format only backend
format-back:
    cargo fmt --manifest-path=./src-tauri/Cargo.toml
    cargo clippy --manifest-path=./src-tauri/Cargo.toml --fix --lib -p winsentials --allow-dirty

# Format only frontend
format-front:
    bun run format

# Format both backend and frontend
format: format-back format-front
