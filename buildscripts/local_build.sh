#!/bin/sh

#remove init.d files
rm -f $TARGET_DIR/etc/init.d/S40network
rm -f $TARGET_DIR/etc/init.d/S41dhcpcd
rm -f $TARGET_DIR/etc/init.d/S50sshd
rm -f $TARGET_DIR/etc/init.d/S80dnsmasq

#add mount-point
mkdir -p $TARGET_DIR/media/mmc
mkdir -p $TARGET_DIR/boot
mkdir -p $TARGET_DIR/atom
mkdir -p $TARGET_DIR/configs

#build atom-skygaze
${GITHUB_WORKSPACE:-/src}/buildscripts/build_rust.sh