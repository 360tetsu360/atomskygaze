#!/bin/bash

# Create the cpio root filesystem that is embedded in the kernel
# This is a minimal root filesystem to bootstrap the bigger rootfs
/src/buildscripts/make_initramfs.sh /atomskygaze/build/buildroot-2024.02/output
