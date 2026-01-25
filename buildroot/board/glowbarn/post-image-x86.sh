#!/bin/bash
# GlowBarn OS Post-Image Script for x86_64
# Creates bootable disk image with GRUB2

set -e

BOARD_DIR="$(dirname $0)"
GENIMAGE_CFG="${BOARD_DIR}/genimage-x86.cfg"
GENIMAGE_TMP="${BUILD_DIR}/genimage.tmp"

# Create genimage config if not exists
if [ ! -f "${GENIMAGE_CFG}" ]; then
    cat > "${GENIMAGE_CFG}" << 'EOF'
image efi-part.vfat {
    vfat {
        file EFI {
            image = "efi-part/EFI"
        }
    }
    size = 256M
}

image glowbarn-os-x86_64.img {
    hdimage {
        partition-table-type = "gpt"
    }

    partition esp {
        partition-type-uuid = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b"
        bootable = "true"
        image = "efi-part.vfat"
        offset = 1M
    }

    partition rootfs {
        partition-type-uuid = "0fc63daf-8483-4772-8e79-3d69d8477de4"
        image = "rootfs.ext4"
    }
}
EOF
fi

# Create EFI structure
mkdir -p "${BINARIES_DIR}/efi-part/EFI/BOOT"

# Copy GRUB EFI binary
if [ -f "${BINARIES_DIR}/grub-efi-bootx64.efi" ]; then
    cp "${BINARIES_DIR}/grub-efi-bootx64.efi" "${BINARIES_DIR}/efi-part/EFI/BOOT/BOOTX64.EFI"
fi

# Create GRUB config
cat > "${BINARIES_DIR}/efi-part/EFI/BOOT/grub.cfg" << 'EOF'
set default=0
set timeout=5

menuentry "GlowBarn OS" {
    linux /boot/bzImage root=/dev/sda2 rootfstype=ext4 rw console=tty0 quiet
}

menuentry "GlowBarn OS (Debug Mode)" {
    linux /boot/bzImage root=/dev/sda2 rootfstype=ext4 rw console=tty0 debug loglevel=7
}

menuentry "GlowBarn OS (Recovery)" {
    linux /boot/bzImage root=/dev/sda2 rootfstype=ext4 rw console=tty0 init=/bin/bash
}
EOF

rm -rf "${GENIMAGE_TMP}"

genimage \
    --rootpath "${TARGET_DIR}" \
    --tmppath "${GENIMAGE_TMP}" \
    --inputpath "${BINARIES_DIR}" \
    --outputpath "${BINARIES_DIR}" \
    --config "${GENIMAGE_CFG}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  GlowBarn OS x86_64 Image Created!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  Image location: ${BINARIES_DIR}/glowbarn-os-x86_64.img"
echo ""
echo "  To write to USB drive:"
echo "    sudo dd if=${BINARIES_DIR}/glowbarn-os-x86_64.img of=/dev/sdX bs=4M status=progress"
echo ""
echo "  To test in QEMU:"
echo "    qemu-system-x86_64 -enable-kvm -m 2G -drive file=${BINARIES_DIR}/glowbarn-os-x86_64.img,format=raw"
echo ""
echo "═══════════════════════════════════════════════════════════════"
