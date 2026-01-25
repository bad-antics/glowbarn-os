# 🌟 GlowBarn OS - Paranormal Research Operating System

> A specialized, lightweight operating system for paranormal investigation, environmental monitoring, and multi-modal anomaly detection research.

## 🎯 Project Vision

GlowBarn OS is a custom Linux-based operating system designed to run the GlowBarn Paranormal Detection Suite as a first-class citizen. Built for deployment on:

- **Live USB** - Boot on any x86_64 machine without installation
- **Raspberry Pi** - ARM-based field deployment
- **Lightweight Machines** - Intel NUC, mini PCs, old laptops
- **Dedicated Research Stations** - Purpose-built detection hardware

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    GlowBarn OS Stack                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              GlowBarn Application Layer                  │   │
│  │  • Visual Console (egui native)                         │   │
│  │  • Sensor Fusion Engine                                  │   │
│  │  • Real-time Analysis                                    │   │
│  │  • Security & Encryption                                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              GlowBarn Framework Layer                    │   │
│  │  • Hardware Abstraction Layer (HAL)                     │   │
│  │  • Sensor Drivers (I2C, SPI, GPIO, USB)                 │   │
│  │  • GPU Compute Interface (wgpu/Vulkan)                  │   │
│  │  • Network Stack (MQTT, WebSocket, BLE)                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              GlowBarn Firmware Layer                     │   │
│  │  • Custom initramfs                                      │   │
│  │  • Minimal systemd services                              │   │
│  │  • Real-time kernel patches (PREEMPT_RT)                │   │
│  │  • Secure boot chain                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Base System                                 │   │
│  │  • Linux Kernel 6.x (custom config)                     │   │
│  │  • Musl libc / glibc                                    │   │
│  │  • BusyBox / CoreUtils                                   │   │
│  │  • SquashFS root (read-only, integrity verified)        │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 📋 Roadmap

### Phase 1: GlowBarn Application (CURRENT)
- [x] Core engine architecture
- [x] 50+ sensor type implementations
- [x] Analysis algorithms (entropy, anomaly detection)
- [x] Multi-sensor fusion (Bayesian, Dempster-Shafer)
- [x] Security module (AES-256-GCM, Argon2id)
- [x] Visual console UI
- [x] Streaming (MQTT, WebSocket)
- [ ] Compilation & testing
- [ ] Code signing
- [ ] GitHub release

### Phase 2: Hardware Abstraction Layer
- [ ] I2C sensor interface
- [ ] SPI sensor interface  
- [ ] GPIO control
- [ ] USB device enumeration
- [ ] Audio capture (ALSA/PipeWire)
- [ ] Camera/thermal imaging
- [ ] Software-defined radio (RTL-SDR)

### Phase 3: Firmware Foundation
- [ ] Custom Linux kernel config
- [ ] PREEMPT_RT patches for real-time
- [ ] Minimal initramfs
- [ ] SquashFS root filesystem
- [ ] Overlay filesystem for persistence
- [ ] Secure boot integration

### Phase 4: Live USB Image
- [ ] ISO generation pipeline
- [ ] UEFI + Legacy BIOS boot
- [ ] Persistence partition support
- [ ] Auto-detection of sensors
- [ ] Network configuration UI
- [ ] First-boot wizard

### Phase 5: Raspberry Pi Support
- [ ] ARM64 cross-compilation
- [ ] Pi-specific kernel config
- [ ] GPIO sensor integration
- [ ] Pi Camera support
- [ ] Pi Sense HAT integration
- [ ] SD card image generation

### Phase 6: Distribution
- [ ] Package repositories (APT, RPM, Pacman)
- [ ] OTA update mechanism
- [ ] Telemetry (opt-in)
- [ ] Community sensor plugins
- [ ] Research data sharing network

## 🛠️ Technology Stack

| Component | Technology |
|-----------|------------|
| **Application** | Rust, egui, wgpu |
| **Build System** | Buildroot / Yocto |
| **Kernel** | Linux 6.x + PREEMPT_RT |
| **Init System** | systemd (minimal) or s6 |
| **Filesystem** | SquashFS + OverlayFS |
| **Bootloader** | systemd-boot / GRUB2 |
| **Containers** | Podman (optional) |

## 🔧 Development Setup

```bash
# Clone the repository
git clone https://github.com/bad-antics/glowbarn-os.git
cd glowbarn-os

# Install build dependencies (Debian/Ubuntu)
sudo apt install build-essential git wget cpio unzip rsync bc \
    libncurses5-dev libssl-dev flex bison

# Initialize Buildroot
make setup

# Configure for target
make menuconfig

# Build the image
make build

# Create bootable USB
sudo dd if=output/glowbarn-os.img of=/dev/sdX bs=4M status=progress
```

## 📁 Project Structure

```
glowbarn-os/
├── buildroot/              # Buildroot external tree
│   ├── board/glowbarn/     # Board-specific files
│   ├── configs/            # Defconfigs for targets
│   ├── package/            # Custom packages
│   └── overlay/            # Root filesystem overlay
├── kernel/                 # Kernel patches and configs
├── firmware/               # Firmware blobs (if needed)
├── tools/                  # Build and deployment tools
├── docs/                   # Documentation
└── tests/                  # Integration tests
```

## 🎯 Target Platforms

| Platform | Architecture | Status |
|----------|-------------|--------|
| Generic x86_64 | x86_64 | Planned |
| Raspberry Pi 4/5 | ARM64 | Planned |
| Raspberry Pi Zero 2W | ARM64 | Planned |
| Intel NUC | x86_64 | Planned |
| NVIDIA Jetson Nano | ARM64 | Future |

## 🔐 Security Features

- **Secure Boot** - Signed bootloader and kernel
- **dm-verity** - Verified root filesystem
- **Full Disk Encryption** - LUKS2 for data partitions
- **Measured Boot** - TPM integration
- **Minimal Attack Surface** - No unnecessary services
- **Automatic Updates** - Signed OTA updates

## 📜 License

GNU General Public License v3.0 (GPLv3)

## 🔗 Related Projects

- [glowbarn-rs](https://github.com/bad-antics/glowbarn-rs) - Main application
- [glowbarn](https://github.com/bad-antics/glowbarn) - Original Python prototype

---

**Status:** 🟡 Planning Phase - Focus on completing glowbarn-rs first

**Last Updated:** 2026-01-24

---

## Quick Start

### Prerequisites
- Linux build system (Ubuntu 22.04+ recommended)
- 20GB+ free disk space
- 4GB+ RAM

### Build Steps

```bash
# Install dependencies
make deps

# Setup Buildroot
make setup

# Build for Raspberry Pi 4
make build-rpi4

# Or build for x86_64 PC
make build-x86
```

### Write to SD Card

```bash
sudo dd if=buildroot-2024.02.9/output/images/glowbarn-os-rpi4.img of=/dev/sdX bs=4M status=progress
```

### Default Login
- **Username:** root
- **Password:** paranormal

### Access Web Dashboard
Open browser to: `http://<device-ip>:8765`

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

GNU General Public License v3.0 - see [LICENSE](LICENSE) for details.
