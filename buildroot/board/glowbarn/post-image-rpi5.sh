#!/bin/bash
# GlowBarn OS Post-Image Script for Raspberry Pi 5
# Creates SD card image using genimage

set -e

BOARD_DIR="$(dirname $0)"
GENIMAGE_CFG="${BOARD_DIR}/genimage-rpi5.cfg"
GENIMAGE_TMP="${BUILD_DIR}/genimage.tmp"

# Create genimage config if not exists
if [ ! -f "${GENIMAGE_CFG}" ]; then
    cat > "${GENIMAGE_CFG}" << 'EOF'
image boot.vfat {
    vfat {
        files = {
            "bcm2712-rpi-5-b.dtb",
            "rpi-firmware/cmdline.txt",
            "rpi-firmware/config.txt",
            "rpi-firmware/fixup_cd.dat",
            "rpi-firmware/start_cd.elf",
            "Image",
            "overlays"
        }
    }
    size = 256M
}

image glowbarn-os-rpi5.img {
    hdimage {
        partition-table-type = "msdos"
    }

    partition boot {
        partition-type = 0xC
        bootable = "true"
        image = "boot.vfat"
    }

    partition rootfs {
        partition-type = 0x83
        image = "rootfs.ext4"
    }
}
EOF
fi

rm -rf "${GENIMAGE_TMP}"

genimage \
    --rootpath "${TARGET_DIR}" \
    --tmppath "${GENIMAGE_TMP}" \
    --inputpath "${BINARIES_DIR}" \
    --outputpath "${BINARIES_DIR}" \
    --config "${GENIMAGE_CFG}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  GlowBarn OS Raspberry Pi 5 Image Created!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  Image location: ${BINARIES_DIR}/glowbarn-os-rpi5.img"
echo ""
echo "  To write to SD card:"
echo "    sudo dd if=${BINARIES_DIR}/glowbarn-os-rpi5.img of=/dev/sdX bs=4M status=progress"
echo ""
echo "═══════════════════════════════════════════════════════════════"
