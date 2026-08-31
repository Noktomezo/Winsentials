# Winsentials Justfile

# List available recipes
default:
    @just --list

# Run complete quality gate (fmt-check, check, clippy, test)
gate:
    cargo run --package xtask -- gate

# Check project compilation across all workspace crates
check:
    cargo run --package xtask -- check

# Run clippy linter with warnings treated as errors
clippy:
    cargo run --package xtask -- clippy

# Format all workspace code
fmt:
    cargo run --package xtask -- fmt

# Check workspace code formatting
fmt-check:
    cargo run --package xtask -- fmt-check

# Run all workspace unit tests
test:
    cargo run --package xtask -- test

# Apply all patches from patches/ directory to cargo git checkouts
patch:
    cargo run --package xtask -- patch

# Revert all patches in cargo git checkouts
unpatch:
    cargo run --package xtask -- unpatch

# Export current cargo git checkout changes to patches/0001-gpui-custom.patch
diff:
    cargo run --package xtask -- diff

# Build release binaries (Winsentials & Installer), compress with UPX and package
build:
    cargo run --package xtask --release -- build

# Run development server with watchexec auto-reload on file change
dev *ARGS: patch
    watchexec -r -e rs,hlsl,toml,json -- cargo run --package winsentials -- {{ARGS}}

# Run Winsentials application
run *ARGS:
    cargo run --package winsentials -- {{ARGS}}

# Run custom xtask command
xtask *ARGS:
    cargo run --package xtask -- {{ARGS}}
