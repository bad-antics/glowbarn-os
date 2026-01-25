# GlowBarn OS Architecture

## Overview

GlowBarn OS is a minimal Linux distribution built with Buildroot, optimized for LED lighting controllers and IoT devices.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Dashboard  │  │   CLI Tool  │  │   REST API Server   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    System Services                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   systemd   │  │ networking  │  │   LED controller    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Linux Kernel 6.6 LTS                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │    GPIO     │  │     SPI     │  │        I2C          │  │
│  │   Driver    │  │   Driver    │  │      Driver         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Hardware                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  RPi Zero   │  │  ESP32 via  │  │    LED Strips       │  │
│  │   Zero 2W   │  │    UART     │  │   WS2812/SK6812     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
glowbarn-os/
├── buildroot/
│   ├── configs/
│   │   └── glowbarn_defconfig    # Main configuration
│   ├── packages/
│   │   ├── glowbarn-core/        # Core LED control
│   │   ├── glowbarn-api/         # REST API server
│   │   └── glowbarn-web/         # Web dashboard
│   └── overlay/
│       ├── etc/                  # System configs
│       └── usr/                  # Application files
├── Makefile                      # Build automation
└── README.md                     # Documentation
```

## Build System

### Buildroot Integration

GlowBarn OS uses Buildroot for reproducible builds:

1. **defconfig**: Minimal kernel + userspace
2. **External packages**: Custom GlowBarn applications
3. **Root filesystem overlay**: Pre-configured system

### Build Targets

```bash
make all          # Full system build
make kernel       # Kernel only
make packages     # External packages
make rootfs       # Root filesystem
make image        # SD card image
```

## Hardware Abstraction

### GPIO Control

```
GPIO Pin Mapping (Raspberry Pi):
┌────────┬────────┬─────────────────┐
│ GPIO   │ Pin    │ Function        │
├────────┼────────┼─────────────────┤
│ GPIO18 │ 12     │ PWM0 (LED Data) │
│ GPIO12 │ 32     │ PWM1 (Backup)   │
│ GPIO21 │ 40     │ LED Clock (APA) │
│ GPIO17 │ 11     │ Button Input    │
│ GPIO27 │ 13     │ Status LED      │
└────────┴────────┴─────────────────┘
```

### Communication Protocols

- **SPI**: High-speed LED data (APA102/SK9822)
- **PWM**: WS2812/SK6812 timing
- **I2C**: Sensor integration
- **UART**: ESP32 co-processor

## Memory Layout

```
┌────────────────────┐ 0x00000000
│     Boot          │ 256MB (FAT32)
├────────────────────┤
│     Root FS       │ 512MB (ext4)
├────────────────────┤
│     Data          │ Remaining (ext4)
└────────────────────┘
```

## Network Stack

- **Ethernet**: Static or DHCP
- **WiFi**: wpa_supplicant
- **mDNS**: Avahi for discovery
- **API**: REST over HTTP/HTTPS

## Security

- Read-only root filesystem
- Encrypted data partition option
- Firewall with nftables
- SSH key-only authentication

## Power Management

- CPU frequency scaling
- LED brightness-based power limiting
- Graceful shutdown on low voltage

## Future Enhancements

- [ ] ESP32 firmware OTA
- [ ] Container support (Podman)
- [ ] Matter/Thread IoT protocols
- [ ] GPU-accelerated effects

---

*GlowBarn OS v1.0 - Built with Buildroot 2024.02 LTS*
