#!/bin/bash
set -e

cd output/images
echo "atomcam" > hostname
touch authorized_keys
cp -dpf uImage.lzma factory_t31_ZMC6tiIDQN
mv rootfs.squashfs rootfs_hack.squashfs
cp -f factory_t31_ZMC6tiIDQN rootfs_hack.squashfs /src/output
cp -r /src/assets /src/output

cd /src/output
VERSION=$(cat /src/configs/skygaze.ver)
ZIP_NAME="/src/atom-skygaze.v$VERSION.zip"
rm "$ZIP_NAME"
zip -r "$ZIP_NAME" .
