#!/bin/bash
set -e

WORKSPACE=${GITHUB_WORKSPACE:-/src}

cd output/images
echo "atomcam" > hostname
touch authorized_keys
cp -dpf uImage.lzma factory_t31_ZMC6tiIDQN
mv rootfs.squashfs rootfs_hack.squashfs
cp -f factory_t31_ZMC6tiIDQN rootfs_hack.squashfs $WORKSPACE/output
cp -r $WORKSPACE/assets $WORKSPACE/output

cd $WORKSPACE/output
VERSION=$(cat $WORKSPACE/configs/skygaze.ver)
ZIP_NAME="$WORKSPACE/atom-skygaze.v$VERSION.zip"
[ -f $ZIP_NAME ] || rm "$ZIP_NAME"
zip -r "$ZIP_NAME" .
