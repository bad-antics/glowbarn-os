################################################################################
#
# glowbarn-sensors - Sensor drivers and support for GlowBarn OS
#
################################################################################

GLOWBARN_SENSORS_VERSION = 1.0.0
GLOWBARN_SENSORS_SITE = $(BR2_EXTERNAL_GLOWBARN_PATH)/package/glowbarn-sensors/src
GLOWBARN_SENSORS_SITE_METHOD = local
GLOWBARN_SENSORS_LICENSE = GPL-3.0

GLOWBARN_SENSORS_DEPENDENCIES = glowbarn python3

ifeq ($(BR2_PACKAGE_GLOWBARN_SENSORS_I2C),y)
GLOWBARN_SENSORS_DEPENDENCIES += i2c-tools
endif

ifeq ($(BR2_PACKAGE_GLOWBARN_SENSORS_GPIO),y)
GLOWBARN_SENSORS_DEPENDENCIES += libgpiod
endif

ifeq ($(BR2_PACKAGE_GLOWBARN_SENSORS_SDR),y)
GLOWBARN_SENSORS_DEPENDENCIES += rtl-sdr libusb
endif

define GLOWBARN_SENSORS_INSTALL_TARGET_CMDS
# Install udev rules
$(INSTALL) -D -m 644 $(GLOWBARN_SENSORS_PKGDIR)/99-glowbarn-sensors.rules \
$(TARGET_DIR)/etc/udev/rules.d/99-glowbarn-sensors.rules

# Install sensor drivers
mkdir -p $(TARGET_DIR)/opt/glowbarn/lib/sensors
if [ -d $(@D)/sensors ]; then \
cp -r $(@D)/sensors/* $(TARGET_DIR)/opt/glowbarn/lib/sensors/; \
fi

# Install systemd service
$(INSTALL) -D -m 644 $(GLOWBARN_SENSORS_PKGDIR)/glowbarn-sensors.service \
$(TARGET_DIR)/usr/lib/systemd/system/glowbarn-sensors.service
endef

define GLOWBARN_SENSORS_INSTALL_INIT_SYSTEMD
$(INSTALL) -D -m 644 $(GLOWBARN_SENSORS_PKGDIR)/glowbarn-sensors.service \
$(TARGET_DIR)/usr/lib/systemd/system/glowbarn-sensors.service
endef

$(eval $(generic-package))
