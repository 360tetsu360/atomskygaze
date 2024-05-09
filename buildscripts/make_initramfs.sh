#!/bin/bash

set -o errexit          # Exit on most errors (see the manual)
set -o errtrace         # Make sure any error trap is inherited
set -o nounset          # Disallow expansion of unset variables
set -o pipefail         # Use last non-zero exit code in a pipeline

echo "=== build initramfs ==="

WORKSPACE=${GITHUB_WORKSPACE:-/src}
BASE_DIR=${GITHUB_WORKSPACE:-/atomskygaze}/build/buildroot-2024.02
ROOTFS_DIR=$BASE_DIR/output/initramfs_root
OUT_DIR=$BASE_DIR/output

[ -f $BASE_DIR/staging/bin-init/fsck.fat ] || make dosfstools-init
[ -f $BASE_DIR/staging/bin-init/fsck.exfat ] || make exfatprogs-init
[ -f $BASE_DIR/staging/bin-init/busybox ] || make busybox-init
[ -f $BASE_DIR/host/usr/bin/mkimage ] || make host-uboot-tools

rm -rf $ROOTFS_DIR
mkdir -p $ROOTFS_DIR

cd $ROOTFS_DIR
mkdir -p {bin,dev,etc,lib,mnt,proc,root,sbin,sys,tmp}

cp -r $WORKSPACE/initramfs_skeleton/* $ROOTFS_DIR/
cp $OUT_DIR/build/dosfstools-init-3.0.28/fsck.fat $ROOTFS_DIR/bin/
cp $OUT_DIR/build/exfatprogs-init-1.2.2/fsck/fsck.exfat $ROOTFS_DIR/bin/
cp $OUT_DIR/build/busybox-init-1.24.1/busybox $ROOTFS_DIR/bin/

# Save a few bytes by removing the readme
rm -f $ROOTFS_DIR/README.md

mknod $ROOTFS_DIR/dev/console c 5 1
mknod $ROOTFS_DIR/dev/null c 1 3
mknod $ROOTFS_DIR/dev/tty0 c 4 0
mknod $ROOTFS_DIR/dev/tty1 c 4 1
mknod $ROOTFS_DIR/dev/tty2 c 4 2
mknod $ROOTFS_DIR/dev/tty3 c 4 3
mknod $ROOTFS_DIR/dev/tty4 c 4 4

find . | cpio -H newc -o > ../images/initramfs.cpio
