#!/bin/bash

WORKSPACE=${GITHUB_WORKSPACE:-/src}
BUILDROOT_DIR=${GITHUB_WORKSPACE:-/atomskygaze}/build/buildroot-2024.02

cat > $WORKSPACE/.cargo/config.toml << EOF
[build]
target = "mipsel-unknown-linux-gnu"

[target.mipsel-unknown-linux-gnu]
linker = "$BUILDROOT_DIR/output/host/bin/mipsel-ingenic-linux-gnu-gcc"
ar = "$BUILDROOT_DIR/output/host/bin/mipsel-ingenic-linux-gnu-ar"

[env]
OPENCV_LINK_LIBS = "opencv_core,opencv_gapi,opencv_imgcodecs,opencv_imgproc,opencv_videoio,opencv_ximgproc,opencv_video,opencv_calib3d,opencv_features2d,opencv_flann"
OPENCV_LINK_PATHS = "$BUILDROOT_DIR/output/host/mipsel-ingenic-linux-gnu/sysroot/usr/lib"
OPENCV_INCLUDE_PATHS = "$BUILDROOT_DIR/output/host/mipsel-ingenic-linux-gnu/sysroot/usr/include/opencv4"
EOF

cd $WORKSPACE
cp -pR "$WORKSPACE"/crates/isvp-sys/lib/* $BUILDROOT_DIR/output/target/usr/lib
cargo +nightly build -Z build-std --target mipsel-unknown-linux-gnu --release
cp -dpf $WORKSPACE/target/mipsel-unknown-linux-gnu/release/atom-skygaze $TARGET_DIR/usr/bin/atom-skygaze
