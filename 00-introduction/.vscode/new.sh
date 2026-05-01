#!/bin/bash

# Calculate the next integer in the sequence...
nextnum=$(jq -r '[.folders[].name | split(":")[0] | tonumber | select(. <= 90)] | ((max // 0) + 1 | tostring | if length == 1 then "0" + . else . end)' ../.code-workspace)

# New directory in the root:
newdir="${nextnum}-new-project"
# relative path to it
newrel="../${newdir}"
# base things off hello-world as a template
hellorel="../01-hello-world"
cp -R "${hellorel}" "${newrel}"
# but delete these files
rm -rf "${newrel}"/target || true
rm -rf "${newrel}"/Cargo.lock || true
rm -rf "${newrel}"/README.md || true

# Then rewrite the project name in the Cargo.toml
jq -Rrs 'gsub("hello-world"; "my-project")' "${hellorel}/Cargo.toml" > "${newrel}/Cargo.toml"

# Then rewrite main.rs with a todo main
echo 'use wstd::http::{Body, Error, Request, Response};' > "${newrel}/src/main.rs"
echo '' >> "${newrel}/src/main.rs"
echo '#[wstd::http_server]' >> "${newrel}/src/main.rs"
echo 'async fn main(req: Request<Body>) -> Result<Response<Body>, Error> {' >> "${newrel}/src/main.rs"
echo '    todo!();' >> "${newrel}/src/main.rs"
echo '}' >> "${newrel}/src/main.rs"

# Finally, add the new folder to .code-workspace
jq ".folders += [{\"path\": \"${newdir}\", \"name\": \"${nextnum}: My Project\"}]" ../.code-workspace > ../.code-workspace.tmp && mv ../.code-workspace.tmp ../.code-workspace
