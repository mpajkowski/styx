#!/bin/sh

set -e -x

cargo build --release

ISOROOT="build/iso_root"
ISO="build/image.iso"

# Create a directory which will be our ISO root.
mkdir -p "$ISOROOT"

# Copy the relevant files over.
cp -v \
    target/x86_64-unknown-none/release/kernel \
    assets/* \
    limine/* \
    "$ISOROOT"

rm build/image.iso || true

# Create the bootable ISO.
xorriso \
    -as mkisofs \
    -b limine-cd.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    --efi-boot limine-cd-efi.bin \
    -efi-boot-part \
    --efi-boot-image \
    --protective-msdos-label \
   "$ISOROOT" -o "$ISO"

# Install Limine stage 1 and 2 for legacy BIOS boot.
utils/limine-deploy "$ISO"

rm -rf "$ISOROOT" || true

qemu-system-x86_64 -enable-kvm -smp 2 -serial stdio -cdrom "$ISO" -d int,cpu_reset -m 512M
