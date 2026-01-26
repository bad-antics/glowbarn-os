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
- [x] I2C sensor interface (HMC5883L, BME280, MLX90614)
- [x] SPI sensor interface (ADS1256, MCP3008)
- [x] GPIO control (PIR, laser grid, PWM)
- [x] USB device enumeration (serial, HID)
- [x] Audio capture (EVP, infrasound, spirit box)
- [x] Camera/thermal imaging (V4L2, FLIR, night vision)
- [x] Software-defined radio (RTL-SDR, EMF analyzer)

### Phase 3: Sensor Fusion Library
- [x] Statistical baseline tracking
- [x] Z-score anomaly detection
- [x] Multi-sensor event correlation
- [x] Sliding window analysis
- [x] EMA trend detection
- [x] CUSUM change point detection
- [x] Isolation Forest multivariate detection
- [x] Pattern matching

### Phase 4: Application Framework
- [x] Main application daemon
- [x] CLI management tool
- [x] Event recording & playback
- [x] Trigger system
- [x] Configuration management
- [x] Session export

### Phase 5: Firmware Foundation
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
├── Cargo.toml              # Rust workspace manifest
├── hal/                    # Hardware Abstraction Layer (Rust)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs          # Core traits & HardwareManager
│   │   ├── i2c.rs          # I2C sensors
│   │   ├── spi.rs          # SPI devices
│   │   ├── gpio.rs         # GPIO control
│   │   ├── usb.rs          # USB enumeration
│   │   ├── audio.rs        # Audio capture
│   │   ├── camera.rs       # Camera/thermal
│   │   └── sdr.rs          # Software-defined radio
│   └── examples/
├── sensors/                # Sensor Fusion Library (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Event types
│       ├── fusion.rs       # Multi-sensor fusion
│       ├── anomaly.rs      # Anomaly detection
│       ├── recording.rs    # Session recording
│       └── triggers.rs     # Trigger automation
├── app/                    # Application (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Daemon
│       ├── cli.rs          # CLI tool
│       └── config.rs       # Configuration
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

**Status:** � Active Development - Rust HAL, Sensors, and Application crates complete

**Last Updated:** 2026-01-26

---

## Rust Workspace

The core GlowBarn system is implemented as a Rust workspace with three crates:

```
├── hal/                    # Hardware Abstraction Layer
│   ├── src/
│   │   ├── lib.rs          # Core HAL traits & HardwareManager
│   │   ├── i2c.rs          # I2C: HMC5883L, BME280, MLX90614
│   │   ├── spi.rs          # SPI: ADS1256, MCP3008
│   │   ├── gpio.rs         # GPIO: PIR, laser grid, PWM
│   │   ├── usb.rs          # USB: serial, HID devices
│   │   ├── audio.rs        # Audio: EVP, infrasound, spirit box
│   │   ├── camera.rs       # Camera: V4L2, thermal, night vision
│   │   └── sdr.rs          # SDR: RTL-SDR, EMF analyzer
│   └── examples/
│       ├── sensor_demo.rs
│       └── emf_scanner.rs
├── sensors/                # Sensor Fusion & Analysis
│   └── src/
│       ├── lib.rs          # Event types, ParanormalEvent
│       ├── fusion.rs       # FusionEngine, multi-sensor correlation
│       ├── anomaly.rs      # Z-score, EMA, CUSUM, IsolationForest
│       ├── recording.rs    # EventRecorder, session management
│       └── triggers.rs     # TriggerManager, automated responses
└── app/                    # Main Application
    └── src/
        ├── main.rs         # Daemon entry point
        ├── cli.rs          # CLI management tool
        └── config.rs       # AppConfig
```

### Building

```bash
# Build release binaries
cargo build --release

# Binaries are in target/release/
# - glowbarn       (daemon)
# - glowbarn-cli   (CLI tool)
```

### CLI Usage

```bash
# Show system information
glowbarn-cli info

# List recording sessions
glowbarn-cli sessions

# Show events from a session
glowbarn-cli events <session-id>

# Export session to JSON
glowbarn-cli export <session-id> --format json

# Generate sample config
glowbarn-cli config > /etc/glowbarn/config.toml
```

### Configuration

```toml
# /etc/glowbarn/config.toml or ~/.config/glowbarn/config.toml

location = "Investigation Site Alpha"
session_name = "session_001"
data_directory = "/var/lib/glowbarn/data"
poll_interval_ms = 100
anomaly_threshold = 3.0
min_confidence = 0.7
auto_record = true
```

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
