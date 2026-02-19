#!/bin/bash

add_to_history() {
	echo "$PWD" >> "${HOME}/.cd_history"
}

cd() {
	builtin cd "$@" && add_to_history
}

goahead() {
	local dest
	dest="$(cdm goahead "$@")"
	if [ -n "$dest" ]; then
		cd "$dest" || return 1
	fi
}

cdr() {
	local dest
	dest="$(cdm cdr "$@")"
	if [ -n "$dest" ]; then
		cd "$dest" || return 1
	fi
}

cdf() {
	local dest
	dest="$(cdm cdf "$@")"
	if [ -n "$dest" ]; then
		cd "$dest" || return 1
	fi
}

cdp() {
	local dest
	dest="$(cdm cdp "$@")"
	if [ -n "$dest" ]; then
		cd "$dest" || return 1
	fi
}
