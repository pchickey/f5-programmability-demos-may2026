#!/bin/bash
set -ex

PWD=/home/ubuntu/code-server-config/workspace/.udf

if ! systemctl list-units --full --all | grep -Fq "udf-git-pull.service"; then
    cp $PWD/udf-git-pull.service /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable udf-git-pull.service
    systemctl start udf-git-pull.service
fi

cp $PWD/udf-setup.service /etc/systemd/system/
cp $PWD/platypus-nginx.service /etc/systemd/system/
cp $PWD/platypus-tmm.service /etc/systemd/system/
cp $PWD/nginx-for-tmm.service /etc/systemd/system/
systemctl daemon-reload

systemctl enable udf-setup
systemctl start udf-setup
systemctl enable platypus-nginx
systemctl start platypus-nginx
systemctl enable platypus-tmm
systemctl start platypus-tmm
systemctl enable nginx-for-tmm
systemctl start nginx-for-tmm
