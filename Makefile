ESP=build/esp
SMP=2
MEM=512M
KERNEL_BIN=kernel/target/x86_64-unknown-none/release/kernel

default: test build-kernel-x86_64

test:
	cd deps && cargo test

build-kernel-x86_64:
	cd kernel && cargo build --release --target x86_64-unknown-none

create-esp-dirs:
	mkdir -p $(ESP)/efi/boot

deploy-limine: create-esp-dirs
	cp limine/LIMINEX64.EFI $(ESP)/efi/boot/BOOTX64.EFI

deploy-assets: create-esp-dirs
	cp assets/* $(ESP)

deploy-kernel: create-esp-dirs build-kernel-x86_64
	cp $(KERNEL_BIN) $(ESP)

deploy: deploy-kernel deploy-assets deploy-limine

qemu: deploy
	qemu-system-x86_64 \
		-cpu qemu64,apic,fsgsbase,rdtscp,xsave,fxsr \
		-enable-kvm \
		-smp $(SMP) \
		-m $(MEM) \
		-serial stdio \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_CODE.fd \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_VARS.fd \
		-drive format=raw,file=fat:rw:build/esp \
		-d int,cpu_reset
