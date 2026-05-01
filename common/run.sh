#!/bin/bash

platypus_host="10.1.1.4"
port=9000
if [[ "$1" == "--nginx" ]]; then
    port=9000
    shift
elif [[ "$1" == "--bigip" ]]; then
    port=9001
    shift
fi

metadata=$(cargo metadata --format-version 1)

if [[ -n "$1" ]]; then
    # Search for binary target with name matching $1
    binary_name=$(echo "$metadata" | jq -r --arg name "$1" '[.packages[] | select(.manifest_path | endswith("Cargo.toml")) | .targets[] | select(.kind[] == "bin" and .name == $name)] | if length == 0 then error("Binary target not found: \($name)") else .[0].name end')
else
    # Require exactly one binary target
    binary_name=$(echo "$metadata" | jq -r '[.packages[] | select(.manifest_path | endswith("Cargo.toml")) | .targets[] | select(.kind[] == "bin")] | if length > 1 then error("More than one binary target found. Provide a binary name as an argument") else .[0].name end')
fi

file_path="target/wasm32-wasip2/debug/${binary_name}.wasm"

set -ex
curl "http://${platypus_host}:${port}/services?name=${binary_name}" --data-binary "@${file_path}"
