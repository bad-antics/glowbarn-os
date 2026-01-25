################################################################################
#
# glowbarn - Paranormal Research Operating System Core Application
#
################################################################################

GLOWBARN_VERSION = 1.0.0
GLOWBARN_SITE = $(BR2_EXTERNAL_GLOWBARN_PATH)/overlay/opt/glowbarn
GLOWBARN_SITE_METHOD = local
GLOWBARN_LICENSE = GPL-3.0
GLOWBARN_LICENSE_FILES = LICENSE

# Python-based application
GLOWBARN_DEPENDENCIES = python3 python-flask python-pyyaml python-flask-cors

# No build needed - Python scripts
define GLOWBARN_BUILD_CMDS
	@echo "GlowBarn: Python package - no build required"
endef

define GLOWBARN_INSTALL_TARGET_CMDS
	# Install main application
	$(INSTALL) -D -m 755 $(@D)/glowbarn.py \
		$(TARGET_DIR)/opt/glowbarn/glowbarn.py
	$(INSTALL) -D -m 755 $(@D)/glowbarn-cli.py \
		$(TARGET_DIR)/opt/glowbarn/glowbarn-cli.py
	
	# Create symlinks for easy access
	ln -sf /opt/glowbarn/glowbarn.py $(TARGET_DIR)/usr/bin/glowbarn
	ln -sf /opt/glowbarn/glowbarn-cli.py $(TARGET_DIR)/usr/bin/glowbarn-cli
	
	# Install systemd service
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.service \
		$(TARGET_DIR)/usr/lib/systemd/system/glowbarn.service
	
	# Install desktop file
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.desktop \
		$(TARGET_DIR)/usr/share/applications/glowbarn.desktop
	
	# Create required directories
	mkdir -p $(TARGET_DIR)/opt/glowbarn/{data,logs,recordings,captures,lib}
	mkdir -p $(TARGET_DIR)/etc/glowbarn
	mkdir -p $(TARGET_DIR)/var/lib/glowbarn
endef

define GLOWBARN_INSTALL_INIT_SYSTEMD
	$(INSTALL) -D -m 644 $(GLOWBARN_PKGDIR)/glowbarn.service \
		$(TARGET_DIR)/usr/lib/systemd/system/glowbarn.service
endef

$(eval $(generic-package))