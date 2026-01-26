//! USB device interface for GlowBarn HAL

use crate::{HalError, HardwareDevice, DeviceType};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
    pub bus: u8,
    pub device: u8,
    pub path: PathBuf,
}

impl UsbDeviceInfo {
    /// Parse device info from sysfs
    fn from_sysfs(path: &PathBuf) -> Result<Self, HalError> {
        let read_attr = |attr: &str| -> String {
            let p = path.join(attr);
            if let Ok(mut f) = File::open(&p) {
                let mut s = String::new();
                let _ = f.read_to_string(&mut s);
                return s.trim().to_string();
            }
            String::new()
        };
        
        let vendor_str = read_attr("idVendor");
        let product_str = read_attr("idProduct");
        let bus_str = read_attr("busnum");
        let dev_str = read_attr("devnum");
        
        Ok(Self {
            vendor_id: u16::from_str_radix(&vendor_str, 16).unwrap_or(0),
            product_id: u16::from_str_radix(&product_str, 16).unwrap_or(0),
            manufacturer: read_attr("manufacturer"),
            product: read_attr("product"),
            serial: read_attr("serial"),
            bus: bus_str.parse().unwrap_or(0),
            device: dev_str.parse().unwrap_or(0),
            path: path.clone(),
        })
    }
}

/// Enumerate USB devices
pub fn enumerate_devices() -> Result<Vec<UsbDeviceInfo>, HalError> {
    let mut devices = Vec::new();
    let usb_path = PathBuf::from("/sys/bus/usb/devices");
    
    if let Ok(entries) = std::fs::read_dir(&usb_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip non-device entries (hubs, interfaces, etc.)
            if name.contains(":") || name.starts_with("usb") {
                continue;
            }
            
            // Check if it has vendor/product
            if path.join("idVendor").exists() {
                if let Ok(info) = UsbDeviceInfo::from_sysfs(&path) {
                    devices.push(info);
                }
            }
        }
    }
    
    Ok(devices)
}

/// Find device by vendor/product ID
pub fn find_device(vendor_id: u16, product_id: u16) -> Result<Option<UsbDeviceInfo>, HalError> {
    let devices = enumerate_devices()?;
    Ok(devices.into_iter().find(|d| d.vendor_id == vendor_id && d.product_id == product_id))
}

/// USB Serial device (CDC ACM, FTDI, etc.)
pub struct UsbSerial {
    name: String,
    port: String,
    file: Option<File>,
    baud: u32,
    ready: bool,
}

impl UsbSerial {
    /// Open USB serial port
    pub fn open(port: &str, baud: u32) -> Result<Self, HalError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(port)?;
        
        // Configure serial port
        #[cfg(target_os = "linux")]
        unsafe {
            let fd = std::os::unix::io::AsRawFd::as_raw_fd(&file);
            
            // Get current settings
            let mut termios: libc::termios = std::mem::zeroed();
            libc::tcgetattr(fd, &mut termios);
            
            // Raw mode
            libc::cfmakeraw(&mut termios);
            
            // Set baud rate
            let baud_const = match baud {
                9600 => libc::B9600,
                19200 => libc::B19200,
                38400 => libc::B38400,
                57600 => libc::B57600,
                115200 => libc::B115200,
                230400 => libc::B230400,
                460800 => libc::B460800,
                921600 => libc::B921600,
                _ => libc::B115200,
            };
            
            libc::cfsetispeed(&mut termios, baud_const);
            libc::cfsetospeed(&mut termios, baud_const);
            
            // 8N1
            termios.c_cflag &= !libc::CSIZE;
            termios.c_cflag |= libc::CS8;
            termios.c_cflag &= !libc::PARENB;
            termios.c_cflag &= !libc::CSTOPB;
            
            // Apply settings
            libc::tcsetattr(fd, libc::TCSANOW, &termios);
            libc::tcflush(fd, libc::TCIOFLUSH);
        }
        
        Ok(Self {
            name: format!("USB Serial {}", port),
            port: port.to_string(),
            file: Some(file),
            baud,
            ready: true,
        })
    }
    
    /// Write data
    pub fn write(&mut self, data: &[u8]) -> Result<usize, HalError> {
        if let Some(ref mut file) = self.file {
            let n = file.write(data)?;
            file.flush()?;
            Ok(n)
        } else {
            Err(HalError::DeviceNotFound("Port not open".to_string()))
        }
    }
    
    /// Read data
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, HalError> {
        if let Some(ref mut file) = self.file {
            Ok(file.read(buf)?)
        } else {
            Err(HalError::DeviceNotFound("Port not open".to_string()))
        }
    }
    
    /// Read line (until newline)
    pub fn read_line(&mut self) -> Result<String, HalError> {
        let mut result = String::new();
        let mut buf = [0u8; 1];
        
        loop {
            let n = self.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let c = buf[0] as char;
            if c == '\n' {
                break;
            }
            result.push(c);
        }
        
        Ok(result.trim().to_string())
    }
    
    /// Write string with newline
    pub fn writeln(&mut self, s: &str) -> Result<(), HalError> {
        self.write(s.as_bytes())?;
        self.write(b"\n")?;
        Ok(())
    }
    
    /// Send command and read response
    pub fn command(&mut self, cmd: &str) -> Result<String, HalError> {
        self.writeln(cmd)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.read_line()
    }
}

impl HardwareDevice for UsbSerial {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::USB
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.ready = true;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.ready && self.file.is_some()
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.file = None;
        self.ready = false;
        Ok(())
    }
}

/// USB HID device (for custom sensors)
pub struct UsbHid {
    name: String,
    vendor_id: u16,
    product_id: u16,
    file: Option<File>,
    ready: bool,
}

impl UsbHid {
    /// Open HID device
    pub fn open(vendor_id: u16, product_id: u16) -> Result<Self, HalError> {
        // Find the hidraw device
        let hidraw_path = Self::find_hidraw(vendor_id, product_id)?;
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&hidraw_path)?;
        
        Ok(Self {
            name: format!("HID {:04X}:{:04X}", vendor_id, product_id),
            vendor_id,
            product_id,
            file: Some(file),
            ready: true,
        })
    }
    
    fn find_hidraw(vendor_id: u16, product_id: u16) -> Result<PathBuf, HalError> {
        let hidraw_base = PathBuf::from("/sys/class/hidraw");
        
        if let Ok(entries) = std::fs::read_dir(&hidraw_base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let device_path = path.join("device");
                
                // Navigate to USB device
                if let Ok(link) = std::fs::read_link(&device_path) {
                    let usb_path = device_path.join(link).canonicalize().ok();
                    
                    if let Some(usb) = usb_path {
                        // Go up to find vendor/product
                        if let Some(parent) = usb.parent().and_then(|p| p.parent()) {
                            let vid_path = parent.join("idVendor");
                            let pid_path = parent.join("idProduct");
                            
                            if vid_path.exists() && pid_path.exists() {
                                let vid = std::fs::read_to_string(&vid_path)
                                    .ok()
                                    .and_then(|s| u16::from_str_radix(s.trim(), 16).ok())
                                    .unwrap_or(0);
                                let pid = std::fs::read_to_string(&pid_path)
                                    .ok()
                                    .and_then(|s| u16::from_str_radix(s.trim(), 16).ok())
                                    .unwrap_or(0);
                                
                                if vid == vendor_id && pid == product_id {
                                    let dev_name = entry.file_name();
                                    return Ok(PathBuf::from("/dev").join(dev_name));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(HalError::DeviceNotFound(format!(
            "HID device {:04X}:{:04X} not found", vendor_id, product_id
        )))
    }
    
    /// Send feature report
    pub fn send_feature_report(&mut self, report: &[u8]) -> Result<(), HalError> {
        if let Some(ref mut file) = self.file {
            file.write_all(report)?;
            Ok(())
        } else {
            Err(HalError::DeviceNotFound("Device not open".to_string()))
        }
    }
    
    /// Read input report
    pub fn read_report(&mut self, buf: &mut [u8]) -> Result<usize, HalError> {
        if let Some(ref mut file) = self.file {
            Ok(file.read(buf)?)
        } else {
            Err(HalError::DeviceNotFound("Device not open".to_string()))
        }
    }
}

impl HardwareDevice for UsbHid {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::USB
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.ready = true;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.ready && self.file.is_some()
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.file = None;
        self.ready = false;
        Ok(())
    }
}

/// Known paranormal equipment USB IDs
pub mod known_devices {
    /// Ghost hunting devices
    pub const MEL_METER: (u16, u16) = (0x16D0, 0x0CE1);  // Example
    pub const K2_METER: (u16, u16) = (0x16D0, 0x0CE2);   // Example
    pub const SPIRIT_BOX: (u16, u16) = (0x16D0, 0x0CE3); // Example
    
    /// RTL-SDR dongles
    pub const RTL2832U: (u16, u16) = (0x0BDA, 0x2832);
    pub const RTL2838: (u16, u16) = (0x0BDA, 0x2838);
    
    /// Audio devices
    pub const GENERIC_AUDIO: (u16, u16) = (0x0D8C, 0x0014);
}
