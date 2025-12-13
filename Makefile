KERNEL_ELF=kernel/target/x86_64-fanga-kernel/release/fanga-kernel
ISO=build/fangaos.iso

.PHONY: iso run kernel clean limine

kernel:
	cd kernel && cargo build \
  -Zbuild-std=core,compiler_builtins \
  -Zbuild-std-features=compiler-builtins-mem \
  -p fanga-kernel --release

limine:
	@if [ ! -f boot/limine/limine-bios.sys ]; then \
		echo "Initializing Limine submodule..."; \
		git submodule update --init --recursive boot/limine; \
	fi
	@if [ ! -f boot/limine/limine ]; then \
		echo "Building Limine utility..."; \
		$(MAKE) -C boot/limine; \
	fi

iso: kernel limine
	mkdir -p build boot/iso_root/boot boot/iso_root/EFI/BOOT
	cp $(KERNEL_ELF) boot/iso_root/boot/kernel.elf

	# Limine files (built in boot/limine)
	cp boot/limine/limine-bios.sys boot/iso_root/boot/limine-bios.sys
	cp boot/limine/limine-bios-cd.bin boot/iso_root/boot/limine-bios-cd.bin
	cp boot/limine/limine-uefi-cd.bin boot/iso_root/boot/limine-uefi-cd.bin
	cp boot/limine/BOOTX64.EFI boot/iso_root/EFI/BOOT/BOOTX64.EFI
	cp boot/limine/BOOTIA32.EFI boot/iso_root/EFI/BOOT/BOOTIA32.EFI || true

	xorriso -as mkisofs \
		-b boot/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		boot/iso_root -o $(ISO)

	./boot/limine/limine bios-install $(ISO) || ./boot/limine/limine-install $(ISO)

run: iso
	qemu-system-x86_64 \
		-m 512M \
		-cdrom $(ISO) \
		-serial stdio \
		-display none

clean:
	rm -rf build
	cd kernel && cargo clean

