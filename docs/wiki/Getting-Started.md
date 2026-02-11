# Getting Started

## Build

```bash
git clone https://github.com/bad-antics/glowbarn-os
cd glowbarn-os
make defconfig
make
```

## Flash to SD Card

```bash
dd if=output/images/sdcard.img of=/dev/sdX bs=4M status=progress
sync
```

## Supported Hardware

| Board | Status | Notes |
|-------|--------|-------|
| Raspberry Pi 4 | ✅ Full | Primary target |
| Raspberry Pi 3 | ✅ Full | Tested |
| Raspberry Pi Zero 2W | ⚠️ Partial | Limited GPIO |
| BeagleBone Black | ✅ Full | Industrial GPIO |

## First Boot

1. Insert SD card and power on
2. Connect via HDMI or SSH (glowbarn.local)
3. Default credentials: `researcher` / `glowbarn`
4. Run `glowbarn-setup` for initial configuration
