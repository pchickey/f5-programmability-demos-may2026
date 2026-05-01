#!/bin/bash
build=("build" "--target" "wasm32-wasip2")
if command -v "cargo-auditable" &> /dev/null; then
    set -ex
    cargo auditable "${build[@]}" "$@"
else
    set -ex
    cargo "${build[@]}" "$@"
fi
