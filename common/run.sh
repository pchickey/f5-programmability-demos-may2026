#!/bin/bash
# If you are using these scripts external to UDF, set the environment
# variables PLATYPUS_NGINX and PLATYPUS_BIGIP to the URLs you copy out of the
# UDF Access Methods.
# Otherwise, these default to the addresses which work locally on the UDF
# ubuntu host and in the vscode terminal.
platypus_nginx="${PLATYPUS_NGINX:-10.1.1.4:9000}"
platypus_bigip="${PLATYPUS_BIGIP:-10.1.1.4:9001}"

# Default to running on nginx, but accept --nginx or --bigip to set which to
# run on explicitly
platypus="${platypus_nginx}"
if [[ "$1" == "--nginx" ]]; then
    platypus="${platypus_nginx}"
    shift
elif [[ "$1" == "--bigip" ]]; then
    platypus="${platypus_bigip}"
    shift
fi

# Get cargo metadata for the current project
metadata=$(cargo metadata --format-version 1)

# If there is an argument passed to this script, upload that target
if [[ -n "$1" ]]; then
    # Search for binary target with name matching $1
    binary_name=$(echo "$metadata" | jq -r --arg name "$1" '[.packages[] | select(.manifest_path | endswith("Cargo.toml") and (contains("/.cargo/") | not)) | .targets[] | select(.kind[] == "bin" and .name == $name)] | if length == 0 then error("Binary target not found: \($name)") else .[0].name end')

else
    # Require exactly one binary target
    binary_name=$(echo "$metadata" | jq -r '[.packages[] | select(.manifest_path | endswith("Cargo.toml") and (contains("/.cargo/") | not)) | .targets[] | select(.kind[] == "bin")] | if length > 1 then error("More than one binary target found") else .[0].name end')

fi

# Upload the debug build for that target. Probably would be smart to accept a
# --release flag in the arguments and allow that instead, but this is just a
# demo and I'd have to do proper argument parsing...
file_path="target/wasm32-wasip2/debug/${binary_name}.wasm"

# Use curl to POST the wasm to the platypus control plane. Name it after the
# binary target.
set -ex
curl "http://${platypus}/services?name=${binary_name}" --data-binary "@${file_path}"
