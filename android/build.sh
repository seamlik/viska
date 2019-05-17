#!/usr/bin/bash

# Configure cross-compilation toolchains and build the crate for all supported Android targets.
# This script accepts additional parameters and redirects them to Cargo.

set -e

if [ -z "$NDK_HOME" ]
then
  echo "Must set \$NDK_HOME!"
  exit 1
fi

ln --symbolic --force "${NDK_HOME}" android/NDK_HOME

ANDROID_MINSDK=28
TARGETS=(
  aarch64-linux-android
  i686-linux-android
  x86_64-linux-android
)
for target in "${TARGETS[@]}"
do
  export AR=${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/${target}-ar
  export CC=${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/${target}${ANDROID_MINSDK}-clang
  cargo build --target $target $@
done