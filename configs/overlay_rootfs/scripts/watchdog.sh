#!/bin/sh

echo $(date +"%Y/%m/%d %H:%M:%S") ": Reboot & Start watchdog" >> /media/mmc/atomhack.log
ip route | awk '/default/ { print $3 }' > /tmp/router_address
