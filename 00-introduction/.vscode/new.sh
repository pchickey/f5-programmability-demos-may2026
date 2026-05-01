#!/bin/bash
set -ex

nextnum=$(jq -r '[.folders[].name | split(":")[0] | tonumber | select(. <= 90)] | ((max // 0) + 1 | tostring | if length == 1 then "0" + . else . end)' ../.code-workspace)
newdir="${nextnum}-new-project"
newrel="../${newdir}"
hellorel="../01-hello-world"
cp -R "${hellorel}" "${newrel}"
rm -rf "${newrel}"/target || true
rm -rf "${newrel}"/Cargo.lock || true
jq -Rrs 'gsub("hello-world"; "my-project")' "${hellorel}/Cargo.toml" > "${newrel}/Cargo.toml"
jq ".folders += [{\"path\": \"${newdir}\", \"name\": \"${nextnum}: My Project\"}]" ../.code-workspace > ../.code-workspace.tmp && mv ../.code-workspace.tmp ../.code-workspace
