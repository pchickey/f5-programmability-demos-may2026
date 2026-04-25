#!/bin/bash
set -ex
curl "http://10.1.1.4:9000/services?name=hello-world" --data-binary "@$1"
