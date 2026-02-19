#!/bin/bash
set -e

bin_dir="${HOME}/.local/bin"
repo_dir="$(cd "$(dirname "$0")" && pwd)"

info() { printf "\033[0;34m%s\033[0m\n" "$*"; }
err()  { printf "\033[0;31m%s\033[0m\n" "$*" >&2; exit 1; }

main() {
	if [ ! -f "Cargo.toml" ] || ! grep -q 'name = "cdm"' Cargo.toml; then
		err "Run this from the cdm repo root."
	fi

	if ! command -v cargo &>/dev/null; then
		err "cargo not found. Install Rust from https://rustup.rs or use a precompiled release."
	fi

	info "Building cdm..."
	cargo build --release --quiet
	mkdir -p "$bin_dir"
	cp target/release/cdm "$bin_dir/cdm"
	info "Installed binary to ${bin_dir}/cdm"

	echo ""
	info "Add the following to your shell profile (.bashrc, .zshrc, etc.):"
	echo ""
	if ! echo "$PATH" | tr ':' '\n' | grep -qxF "$bin_dir"; then
		echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
	fi
	echo "  source \"${repo_dir}/shell/cdm.sh\""
	echo ""
	info "Then restart your shell or source your profile."
	info "Commands available: goahead, cdr, cdf, cdp"
}

main
