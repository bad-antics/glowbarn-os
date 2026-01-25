################################################################################
#
# glowbarn
#
################################################################################

GLOWBARN_VERSION = 2.0.0
GLOWBARN_SITE = $(call github,bad-antics,glowbarn-rs,v$(GLOWBARN_VERSION))
GLOWBARN_LICENSE = GPL-3.0
GLOWBARN_LICENSE_FILES = LICENSE

GLOWBARN_DEPENDENCIES = host-rustc eudev

GLOWBARN_CARGO_ENV = \
	CARGO_HOME=$(HOST_DIR)/share/cargo

GLOWBARN_CARGO_OPTS = \
	--release \
	--target=$(RUSTC_TARGET_NAME)

# Build features based on config
GLOWBARN_FEATURES =

ifeq ($(BR2_PACKAGE_GLOWBARN_GUI),y)
GLOWBARN_FEATURES += gui
GLOWBARN_DEPENDENCIES += libdrm
ifeq ($(BR2_PACKAGE_HAS_LIBGL),y)
GLOWBARN_DEPENDENCIES += mesa3d
endif
endif

ifeq ($(BR2_PACKAGE_GLOWBARN_GPU),y)
GLOWBARN_FEATURES += gpu
GLOWBARN_DEPENDENCIES += vulkan-headers vulkan-loader
endif

ifeq ($(BR2_PACKAGE_GLOWBARN_AUDIO),y)
GLOWBARN_FEATURES += audio
GLOWBARN_DEPENDENCIES += alsa-lib pipewire
endif

ifeq ($(BR2_PACKAGE_GLOWBARN_SERIAL),y)
GLOWBARN_FEATURES += serial
endif

ifneq ($(GLOWBARN_FEATURES),)
GLOWBARN_CARGO_OPTS += --features "$(GLOWBARN_FEATURES)"
endif

define GLOWBARN_BUILD_CMDS
	cd $(@D) && \
	$(GLOWBARN_CARGO_ENV) \
	cargo build $(GLOWBARN_CARGO_OPTS)
endef

define GLOWBARN_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 755 $(@D)/target/$(RUSTC_TARGET_NAME)/release/glowbarn \
		$(TARGET_DIR)/usr/bin/glowbarn
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.service \
		$(TARGET_DIR)/usr/lib/systemd/system/glowbarn.service
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.desktop \
		$(TARGET_DIR)/usr/share/applications/glowbarn.desktop
endef

define GLOWBARN_INSTALL_INIT_SYSTEMD
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.service \
		$(TARGET_DIR)/usr/lib/systemd/system/glowbarn.service
endef

$(eval $(generic-package))
