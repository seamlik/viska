#!/usr/bin/bash

# Configure cross-compilation toolchains and build the crate for all supported Android targets.
# This script accepts additional parameters and redirects them to Cargo.

# Must be run under the root directory.

set -e

if [ -z "$ANDROID_HOME" ]
then
  echo "Must set \$ANDROID_HOME!"
  exit 1
fi

SDK_VERSION=29
NDK_VERSION=21.0.6113669

# For `.cargo/config` which does not support environment variables
ln --symbolic --force "${ANDROID_HOME}/ndk/${NDK_VERSION}" android/NDK_HOME

TARGETS=(
  aarch64-linux-android
  x86_64-linux-android
)
for target in "${TARGETS[@]}"
do
  export AR=${ANDROID_HOME}/ndk/${NDK_VERSION}/toolchains/llvm/prebuilt/linux-x86_64/bin/${target}-ar
  export CC=${ANDROID_HOME}/ndk/${NDK_VERSION}/toolchains/llvm/prebuilt/linux-x86_64/bin/${target}${SDK_VERSION}-clang
  cargo build --target $target --features "android" $@
  cargo build --target $target --features "android" --release $@
done