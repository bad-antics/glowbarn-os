# GlowBarn OS Installation

## Build from Source
```bash
git clone https://github.com/bad-antics/glowbarn-os
cd glowbarn-os
make defconfig && make
```

## Flash
```bash
dd if=output/images/sdcard.img of=/dev/sdX bs=4M status=progress
```

## Requirements
- Linux build host
- 10GB+ disk space
- Internet connection (downloads toolchain)
