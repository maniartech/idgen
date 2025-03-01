#!/bin/bash

# Define target platforms and architectures
WINDOWS_ARCHS=(i686 x86_64)
LINUX_ARCHS=(i686 x86_64 armv7-unknown-linux-gnueabihf aarch64-unknown-linux-gnu)

# Loop through Windows architectures and compile binaries
for arch in ${WINDOWS_ARCHS[@]}; do
    echo "Compiling for Windows ${arch}"
    rustup target add ${arch}-pc-windows-msvc
    cargo build --release --target=${arch}-pc-windows-msvc
    mv target/${arch}-pc-windows-msvc/release/idgen-${arch}-win.exe
done

# Loop through Linux architectures and compile binaries
for arch in ${LINUX_ARCHS[@]}; do
    echo "Compiling for Linux ${arch}"
    rustup target add ${arch}
    cargo build --release --target=${arch}
    mv target/${arch}/release/idgen-${arch}-linux
done

echo "Compilation complete!"
