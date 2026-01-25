################################################################################
#
# glowbarn-sensors
#
################################################################################

GLOWBARN_SENSORS_VERSION = 1.0.0
GLOWBARN_SENSORS_SITE = $(BR2_EXTERNAL_GLOWBARN_EXTERNAL_PATH)/package/glowbarn-sensors/src
GLOWBARN_SENSORS_SITE_METHOD = local
GLOWBARN_SENSORS_LICENSE = GPL-3.0

GLOWBARN_SENSORS_DEPENDENCIES = glowbarn eudev

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
	$(INSTALL) -D -m 644 $(GLOWBARN_SENSORS_PKGDIR)/99-glowbarn-sensors.rules \
		$(TARGET_DIR)/etc/udev/rules.d/99-glowbarn-sensors.rules
endef

$(eval $(generic-package))
