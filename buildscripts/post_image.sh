#!/bin/bash
set -e

cd output/images
echo "atomcam" > hostname
touch authorized_keys
cp -dpf uImage.lzma factory_t31_ZMC6tiIDQN
mv rootfs.squashfs rootfs_hack.squashfs
cp -f factory_t31_ZMC6tiIDQN rootfs_hack.squashfs /src/output
