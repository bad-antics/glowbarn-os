#!/bin/bash
# GlowBarn OS Post-Build Script
# Runs after rootfs is built, before image creation

set -e

EXTERNAL_PATH="$1"
TARGET_DIR="$2"

echo "═══════════════════════════════════════════════════════════════"
echo "  GlowBarn OS Post-Build Script"
echo "═══════════════════════════════════════════════════════════════"

# Create GlowBarn directories
mkdir -p "$TARGET_DIR/opt/glowbarn"
mkdir -p "$TARGET_DIR/opt/glowbarn/data"
mkdir -p "$TARGET_DIR/opt/glowbarn/logs"
mkdir -p "$TARGET_DIR/opt/glowbarn/recordings"
mkdir -p "$TARGET_DIR/opt/glowbarn/captures"
mkdir -p "$TARGET_DIR/var/lib/glowbarn"
mkdir -p "$TARGET_DIR/etc/glowbarn"

# Set permissions
chmod 755 "$TARGET_DIR/opt/glowbarn"
chmod 777 "$TARGET_DIR/opt/glowbarn/data"
chmod 777 "$TARGET_DIR/opt/glowbarn/logs"
chmod 777 "$TARGET_DIR/opt/glowbarn/recordings"
chmod 777 "$TARGET_DIR/opt/glowbarn/captures"

# Create glowbarn user if not exists
if ! grep -q "^glowbarn:" "$TARGET_DIR/etc/passwd"; then
    echo "glowbarn:x:1000:1000:GlowBarn User:/home/glowbarn:/bin/bash" >> "$TARGET_DIR/etc/passwd"
    echo "glowbarn:x:1000:" >> "$TARGET_DIR/etc/group"
    echo "glowbarn:*:19000:0:99999:7:::" >> "$TARGET_DIR/etc/shadow"
    mkdir -p "$TARGET_DIR/home/glowbarn"
    chown -R 1000:1000 "$TARGET_DIR/home/glowbarn" 2>/dev/null || true
fi

# Add glowbarn user to required groups
for group in gpio i2c spi audio video dialout plugdev; do
    if grep -q "^${group}:" "$TARGET_DIR/etc/group"; then
        sed -i "s/^${group}:\([^:]*\):\([^:]*\):$/&glowbarn/" "$TARGET_DIR/etc/group"
        sed -i "s/^${group}:\([^:]*\):\([^:]*\):\(.*\)$/&,glowbarn/" "$TARGET_DIR/etc/group"
        # Clean up double commas
        sed -i "s/,,/,/g" "$TARGET_DIR/etc/group"
        sed -i "s/:,/:/g" "$TARGET_DIR/etc/group"
    fi
done

# Enable systemd services
if [ -d "$TARGET_DIR/etc/systemd/system" ]; then
    mkdir -p "$TARGET_DIR/etc/systemd/system/multi-user.target.wants"
    
    # Enable GlowBarn service
    if [ -f "$TARGET_DIR/usr/lib/systemd/system/glowbarn.service" ]; then
        ln -sf /usr/lib/systemd/system/glowbarn.service \
            "$TARGET_DIR/etc/systemd/system/multi-user.target.wants/glowbarn.service"
    fi
    
    # Enable GlowBarn sensors service
    if [ -f "$TARGET_DIR/usr/lib/systemd/system/glowbarn-sensors.service" ]; then
        ln -sf /usr/lib/systemd/system/glowbarn-sensors.service \
            "$TARGET_DIR/etc/systemd/system/multi-user.target.wants/glowbarn-sensors.service"
    fi
fi

# Set up auto-login to GlowBarn interface (optional)
if [ -d "$TARGET_DIR/etc/systemd/system/getty@tty1.service.d" ]; then
    cat > "$TARGET_DIR/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin glowbarn --noclear %I $TERM
EOF
fi

# Create default config if not exists
if [ ! -f "$TARGET_DIR/etc/glowbarn/config.yaml" ]; then
    cat > "$TARGET_DIR/etc/glowbarn/config.yaml" << 'EOF'
# GlowBarn OS Configuration
version: "1.0"

system:
  hostname: glowbarn
  timezone: America/New_York
  
web:
  enabled: true
  port: 8765
  host: "0.0.0.0"
  ssl: false
  
sensors:
  emf:
    enabled: true
    pin: 17
    sample_rate: 100
  temperature:
    enabled: true
    type: ds18b20
    pin: 4
  humidity:
    enabled: true
    type: dht22
    pin: 22
  motion:
    enabled: true
    pin: 27
  vibration:
    enabled: true
    pin: 23
  pressure:
    enabled: true
    type: bmp280
    i2c_address: 0x76
    
audio:
  evp_detection:
    enabled: true
    sample_rate: 44100
    channels: 2
    device: "default"
    threshold: -40
    
camera:
  enabled: true
  device: /dev/video0
  resolution: "1280x720"
  framerate: 30
  night_vision: true
  motion_detection: true
  
logging:
  level: INFO
  file: /opt/glowbarn/logs/glowbarn.log
  max_size: 10M
  backup_count: 5
  
data:
  storage_path: /opt/glowbarn/data
  auto_export: false
  export_format: csv
  
alerts:
  enabled: true
  email: false
  sound: true
  visual: true
  thresholds:
    emf_high: 5.0
    temp_change: 5.0
    motion_duration: 5
EOF
fi

# Set up boot splash/banner
cat > "$TARGET_DIR/etc/issue" << 'EOF'

╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║     ██████╗ ██╗      ██████╗ ██╗    ██╗██████╗  █████╗ ██████╗   ║
║    ██╔════╝ ██║     ██╔═══██╗██║    ██║██╔══██╗██╔══██╗██╔══██╗  ║
║    ██║  ███╗██║     ██║   ██║██║ █╗ ██║██████╔╝███████║██████╔╝  ║
║    ██║   ██║██║     ██║   ██║██║███╗██║██╔══██╗██╔══██║██╔══██╗  ║
║    ╚██████╔╝███████╗╚██████╔╝╚███╔███╔╝██████╔╝██║  ██║██║  ██║  ║
║     ╚═════╝ ╚══════╝ ╚═════╝  ╚══╝╚══╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝  ║
║                                                                   ║
║                  Paranormal Research Operating System             ║
║                                                                   ║
║        Web Interface: http://\4:8765                              ║
║        SSH Access:    ssh glowbarn@\4                             ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝

EOF

# Set up MOTD
cat > "$TARGET_DIR/etc/motd" << 'EOF'

  Welcome to GlowBarn OS!
  
  Quick Start:
    glowbarn-cli status    - Check system status
    glowbarn-cli sensors   - View sensor readings
    glowbarn-cli record    - Start recording session
    
  Web Interface: http://localhost:8765
  
  Documentation: https://github.com/bad-antics/glowbarn-os

EOF

echo "═══════════════════════════════════════════════════════════════"
echo "  Post-build complete!"
echo "═══════════════════════════════════════════════════════════════"
