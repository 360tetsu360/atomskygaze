#!/bin/bash
set -e

WORKSPACE=${GITHUB_WORKSPACE:-/src}

OUT_DIR=$WORKSPACE/output
[ -f $OUT_DIR ] || mkdir $OUT_DIR

cd output/images
echo "atomaskygaze" > hostname
touch authorized_keys
cp -dpf uImage.lzma factory_t31_ZMC6tiIDQN
mv rootfs.squashfs rootfs_hack.squashfs
cp -f factory_t31_ZMC6tiIDQN rootfs_hack.squashfs $OUT_DIR
cp -r $WORKSPACE/assets $OUT_DIR

cd $OUT_DIR
VERSION=$(cat $WORKSPACE/configs/skygaze.ver)
ZIP_NAME="$WORKSPACE/atom-skygaze.v$VERSION.zip"
[ -f $ZIP_NAME ] && rm "$ZIP_NAME"
zip -r "$ZIP_NAME" .
