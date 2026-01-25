# GlowBarn OS Build System
# A lightweight paranormal research operating system

BUILDROOT_VERSION := 2024.02.9
BUILDROOT_URL := https://buildroot.org/downloads/buildroot-$(BUILDROOT_VERSION).tar.gz
BUILDROOT_DIR := buildroot-$(BUILDROOT_VERSION)

OUTPUT_DIR := output
DOWNLOAD_DIR := $(OUTPUT_DIR)/downloads

# Targets
TARGETS := x86_64 rpi4 rpi5 rpizero2w

.PHONY: all setup clean distclean help menuconfig build

all: help

help:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║        GlowBarn OS - Paranormal Research Operating System    ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Setup targets:"
	@echo "  setup           - Download and setup Buildroot"
	@echo "  deps            - Install build dependencies"
	@echo ""
	@echo "Configuration targets:"
	@echo "  menuconfig      - Configure Buildroot (interactive)"
	@echo "  defconfig-x86   - Load x86_64 default config"
	@echo "  defconfig-rpi4  - Load Raspberry Pi 4 config"
	@echo "  defconfig-rpi5  - Load Raspberry Pi 5 config"
	@echo ""
	@echo "Build targets:"
	@echo "  build           - Build the OS image"
	@echo "  build-x86       - Build x86_64 live USB image"
	@echo "  build-rpi4      - Build Raspberry Pi 4 SD image"
	@echo "  build-rpi5      - Build Raspberry Pi 5 SD image"
	@echo ""
	@echo "Utility targets:"
	@echo "  clean           - Clean build artifacts"
	@echo "  distclean       - Remove everything including downloads"
	@echo "  write-usb       - Write image to USB (requires USB_DEV)"
	@echo ""

deps:
	@echo "Installing build dependencies..."
	sudo apt-get update
	sudo apt-get install -y \
		build-essential git wget cpio unzip rsync bc \
		libncurses5-dev libssl-dev flex bison \
		python3 python3-pip file \
		qemu-system-x86 qemu-system-arm

setup: $(BUILDROOT_DIR)
	@echo "Buildroot setup complete!"
	@echo "Run 'make menuconfig' to configure"

$(DOWNLOAD_DIR):
	mkdir -p $(DOWNLOAD_DIR)

$(DOWNLOAD_DIR)/buildroot-$(BUILDROOT_VERSION).tar.gz: $(DOWNLOAD_DIR)
	@echo "Downloading Buildroot $(BUILDROOT_VERSION)..."
	wget -O $@ $(BUILDROOT_URL)

$(BUILDROOT_DIR): $(DOWNLOAD_DIR)/buildroot-$(BUILDROOT_VERSION).tar.gz
	@echo "Extracting Buildroot..."
	tar xzf $<
	@echo "Applying GlowBarn external tree..."
	ln -sf $(CURDIR)/buildroot $(BUILDROOT_DIR)/glowbarn-external

menuconfig: $(BUILDROOT_DIR)
	cd $(BUILDROOT_DIR) && make menuconfig BR2_EXTERNAL=glowbarn-external

defconfig-x86: $(BUILDROOT_DIR)
	cd $(BUILDROOT_DIR) && make glowbarn_x86_64_defconfig BR2_EXTERNAL=glowbarn-external

defconfig-rpi4: $(BUILDROOT_DIR)
	cd $(BUILDROOT_DIR) && make glowbarn_rpi4_defconfig BR2_EXTERNAL=glowbarn-external

defconfig-rpi5: $(BUILDROOT_DIR)
	cd $(BUILDROOT_DIR) && make glowbarn_rpi5_defconfig BR2_EXTERNAL=glowbarn-external

build: $(BUILDROOT_DIR)
	cd $(BUILDROOT_DIR) && make BR2_EXTERNAL=glowbarn-external

build-x86: defconfig-x86 build
	@echo "x86_64 image built: $(BUILDROOT_DIR)/output/images/glowbarn-os-x86_64.img"

build-rpi4: defconfig-rpi4 build
	@echo "Raspberry Pi 4 image built: $(BUILDROOT_DIR)/output/images/glowbarn-os-rpi4.img"

build-rpi5: defconfig-rpi5 build
	@echo "Raspberry Pi 5 image built: $(BUILDROOT_DIR)/output/images/glowbarn-os-rpi5.img"

write-usb:
ifndef USB_DEV
	$(error USB_DEV is not set. Use: make write-usb USB_DEV=/dev/sdX)
endif
	@echo "WARNING: This will erase all data on $(USB_DEV)"
	@read -p "Are you sure? [y/N] " confirm && [ "$$confirm" = "y" ]
	sudo dd if=$(BUILDROOT_DIR)/output/images/glowbarn-os-x86_64.img of=$(USB_DEV) bs=4M status=progress conv=fsync

qemu-test:
	qemu-system-x86_64 \
		-M q35 \
		-m 2G \
		-smp 2 \
		-enable-kvm \
		-drive file=$(BUILDROOT_DIR)/output/images/disk.img,format=raw \
		-device virtio-net-pci,netdev=net0 \
		-netdev user,id=net0,hostfwd=tcp::8765-:8765 \
		-device virtio-gpu-pci \
		-display gtk

clean:
	cd $(BUILDROOT_DIR) && make clean

distclean:
	rm -rf $(BUILDROOT_DIR) $(OUTPUT_DIR)
