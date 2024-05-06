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
cd /src
cp -r /src/lib/* /atomskygaze/build/buildroot-2024.02/output/target/usr/lib/
cargo +nightly build -Z build-std --target mipsel-unknown-linux-gnu --release
cp -dpf /src/target/mipsel-unknown-linux-gnu/release/atom-skygaze $TARGET_DIR/usr/bin/atom-skygaze
