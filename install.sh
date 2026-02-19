#!/bin/bash
set -e

cdm_dir="${HOME}/.cdm"
bin_dir="${HOME}/.local/bin"
shell_rc=""

info() { printf "\033[0;34m%s\033[0m\n" "$*"; }
err()  { printf "\033[0;31m%s\033[0m\n" "$*" >&2; exit 1; }

detect_shell_rc() {
	case "$(basename "$SHELL")" in
		zsh)  shell_rc="${HOME}/.zshrc" ;;
		bash)
			if [ -f "${HOME}/.bash_profile" ]; then
				shell_rc="${HOME}/.bash_profile"
			else
				shell_rc="${HOME}/.bashrc"
			fi
			;;
		*)    shell_rc="${HOME}/.profile" ;;
	esac
}

build_from_source() {
	if ! command -v cargo &>/dev/null; then
		err "cargo not found. Install Rust from https://rustup.rs or use a precompiled release."
	fi
	info "Building cdm..."
	cargo build --release --quiet
	mkdir -p "$bin_dir"
	cp target/release/cdm "$bin_dir/cdm"
	info "Installed binary to ${bin_dir}/cdm"
}

install_shell() {
	mkdir -p "$cdm_dir"
	cp shell/cdm.sh "$cdm_dir/cdm.sh"
	info "Installed shell functions to ${cdm_dir}/cdm.sh"
}

configure_profile() {
	detect_shell_rc

	local source_line="source \"${cdm_dir}/cdm.sh\""
	local path_line="export PATH=\"${bin_dir}:\$PATH\""

	local changes=false

	if ! grep -qF "$cdm_dir/cdm.sh" "$shell_rc" 2>/dev/null; then
		{
			echo ""
			echo "# cdm — cd with memory"
			# only add PATH line if ~/.local/bin isn't already on PATH
			if ! echo "$PATH" | tr ':' '\n' | grep -qxF "$bin_dir"; then
				echo "$path_line"
			fi
			echo "$source_line"
		} >> "$shell_rc"
		changes=true
	fi

	if $changes; then
		info "Added cdm to ${shell_rc}"
	else
		info "cdm already configured in ${shell_rc}"
	fi
}

main() {
	if [ ! -f "Cargo.toml" ] || ! grep -q 'name = "cdm"' Cargo.toml; then
		err "Run this from the cdm repo root."
	fi

	build_from_source
	install_shell
	configure_profile

	echo ""
	info "Done! Restart your shell or run:"
	info "  source ${shell_rc}"
	echo ""
	info "Commands available: goahead, cdr, cdf, cdp"
}

main
