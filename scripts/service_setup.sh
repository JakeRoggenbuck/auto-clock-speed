#!/bin/sh

service="acs.service"
sed -i "s/your-user-here/$USER/" $service

cp $service /etc/systemd/system/

systemctl start acs
systemctl enable acs
systemctl status acs
