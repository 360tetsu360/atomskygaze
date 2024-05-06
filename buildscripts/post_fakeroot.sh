#!/bin/bash
set -e

echo "Executing pre filesystem image creation script"

# The environment variables BR2_CONFIG, HOST_DIR, STAGING_DIR,
# TARGET_DIR, BUILD_DIR, BINARIES_DIR and BASE_DIR are defined

WORKSPACE=${GITHUB_WORKSPACE:-/src}
$WORKSPACE/buildscripts/local_build.sh

find $TARGET_DIR -name .DS_Store -delete
cp $WORKSPACE/configs/skygaze.ver $TARGET_DIR/etc

DEFAULT_IMAGE_DIR=$BASE_DIR/images
BASE_DIR=$BASE_DIR
IMAGES=$BASE_DIR/images
HOST_DIR=$HOST_DIR
TARGET_DIR=$TARGET_DIR
