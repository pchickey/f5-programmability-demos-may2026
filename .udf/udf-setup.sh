#!/bin/bash
PWD=/home/ubuntu/code-server-config/workspace/.udf

# Get the latest version
cd $PWD
git fetch
git pull

# Check if systemctl service is installed
if ! systemctl list-units --full --all | grep -Fq "udf-setup.service"; then
    cp $PWD/udf-setup.service /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable udf-setup
    systemctl start udf-setup
fi


