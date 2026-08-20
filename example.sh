#!/bin/zsh

set -u

bin="./target/release/ls-gi"
wsm="./target/wasm32-wasip1/release-wasi/ls-gi.wasm"

run_native() {
	"${bin}" \
		--ignore-hidden \
		--ignore-dirents-using-gitignore \
		.
}

run_wasi() {
	wazero \
		run \
		-mount "${PWD}:/guest.d:ro" \
		-timeout 1s \
		"${wsm}" \
		--ignore-hidden \
		--ignore-dirents-using-gitignore \
		/guest.d
}

find . -type f | wc -l
run_native | wc -l
run_wasi | wc -l

run_native

#run_wasi
