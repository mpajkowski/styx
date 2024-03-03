#!/bin/sh

set -e -x

SYSTEM=$(uname -s)

cargo build --manifest-path bare_crates/Cargo.toml --release

ESP="build/esp"

# create virtual esp partition
mkdir -p "$ESP/efi/boot"

# copy kernel image and assets to esp partition
cp -v \
    bare_crates/target/x86_64-unknown-none/release/kernel \
    assets/* \
    "$ESP"

# deploy limine uefi image
cp "limine/LIMINEX64.EFI" "$ESP/efi/boot/BOOTX64.EFI"

# launch qemu
qemu-system-x86_64 \
    -cpu qemu64,apic,fsgsbase,rdtscp,xsave,fxsr \
    -enable-kvm \
    -smp 2 \
    -m 512M \
    -serial stdio \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:build/esp \
    -d int,cpu_reset
