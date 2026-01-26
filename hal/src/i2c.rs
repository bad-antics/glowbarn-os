//! I2C interface for GlowBarn HAL

use crate::{HalError, HardwareDevice, Sensor, DeviceType};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

/// I2C Bus wrapper
pub struct I2CBus {
    path: String,
    fd: Option<i32>,
}

impl I2CBus {
    /// Open I2C bus
    pub fn open(path: &str) -> Result<Self, HalError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        
        Ok(Self {
            path: path.to_string(),
            fd: Some(file.as_raw_fd()),
        })
    }
    
    /// Set slave address
    pub fn set_slave(&self, addr: u8) -> Result<(), HalError> {
        // ioctl I2C_SLAVE = 0x0703
        #[cfg(target_os = "linux")]
        unsafe {
            if let Some(fd) = self.fd {
                let ret = libc::ioctl(fd, 0x0703, addr as libc::c_ulong);
                if ret < 0 {
                    return Err(HalError::CommunicationError(
                        format!("Failed to set I2C slave address 0x{:02X}", addr)
                    ));
                }
            }
        }
        Ok(())
    }
    
    /// Read bytes from I2C device
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, HalError> {
        #[cfg(target_os = "linux")]
        unsafe {
            if let Some(fd) = self.fd {
                let ret = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                if ret < 0 {
                    return Err(HalError::CommunicationError("I2C read failed".to_string()));
                }
                return Ok(ret as usize);
            }
        }
        Err(HalError::DeviceNotFound("I2C bus not open".to_string()))
    }
    
    /// Write bytes to I2C device
    pub fn write(&self, buf: &[u8]) -> Result<usize, HalError> {
        #[cfg(target_os = "linux")]
        unsafe {
            if let Some(fd) = self.fd {
                let ret = libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len());
                if ret < 0 {
                    return Err(HalError::CommunicationError("I2C write failed".to_string()));
                }
                return Ok(ret as usize);
            }
        }
        Err(HalError::DeviceNotFound("I2C bus not open".to_string()))
    }
    
    /// Read register
    pub fn read_register(&self, addr: u8, reg: u8) -> Result<u8, HalError> {
        self.set_slave(addr)?;
        self.write(&[reg])?;
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(buf[0])
    }
    
    /// Write register
    pub fn write_register(&self, addr: u8, reg: u8, value: u8) -> Result<(), HalError> {
        self.set_slave(addr)?;
        self.write(&[reg, value])?;
        Ok(())
    }
    
    /// Read multiple bytes from register
    pub fn read_registers(&self, addr: u8, reg: u8, buf: &mut [u8]) -> Result<usize, HalError> {
        self.set_slave(addr)?;
        self.write(&[reg])?;
        self.read(buf)
    }
}

/// Scan I2C bus for devices
pub fn scan_bus(path: &str) -> Result<Vec<u8>, HalError> {
    let bus = I2CBus::open(path)?;
    let mut found = Vec::new();
    
    // Scan addresses 0x03 to 0x77
    for addr in 0x03..=0x77 {
        if bus.set_slave(addr).is_ok() {
            let mut buf = [0u8; 1];
            if bus.read(&mut buf).is_ok() {
                found.push(addr);
                tracing::info!("Found I2C device at 0x{:02X}", addr);
            }
        }
    }
    
    Ok(found)
}

/// Generic I2C sensor
pub struct I2CSensor {
    name: String,
    bus: I2CBus,
    address: u8,
    unit: String,
    calibration_offset: f64,
    ready: bool,
}

impl I2CSensor {
    /// Create new I2C sensor
    pub fn new(name: &str, bus_path: &str, address: u8, unit: &str) -> Result<Self, HalError> {
        let bus = I2CBus::open(bus_path)?;
        
        Ok(Self {
            name: name.to_string(),
            bus,
            address,
            unit: unit.to_string(),
            calibration_offset: 0.0,
            ready: false,
        })
    }
}

impl HardwareDevice for I2CSensor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::I2C
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        // Verify device responds
        self.bus.set_slave(self.address)?;
        self.ready = true;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.ready
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.ready = false;
        Ok(())
    }
}

impl Sensor for I2CSensor {
    fn read_raw(&self) -> Result<Vec<u8>, HalError> {
        let mut buf = vec![0u8; 6];
        self.bus.read_registers(self.address, 0x00, &mut buf)?;
        Ok(buf)
    }
    
    fn read_value(&self) -> Result<f64, HalError> {
        let raw = self.read_raw()?;
        // Convert raw bytes to value (sensor-specific)
        let value = ((raw[0] as i16) << 8 | raw[1] as i16) as f64 / 100.0;
        Ok(value + self.calibration_offset)
    }
    
    fn unit(&self) -> &str {
        &self.unit
    }
    
    fn calibrate(&mut self, offset: f64) -> Result<(), HalError> {
        self.calibration_offset = offset;
        Ok(())
    }
}

// Common I2C sensor implementations

/// HMC5883L Magnetometer (EMF sensor)
pub struct HMC5883L {
    base: I2CSensor,
}

impl HMC5883L {
    pub fn new(bus_path: &str) -> Result<Self, HalError> {
        let base = I2CSensor::new("HMC5883L", bus_path, 0x1E, "mG")?;
        Ok(Self { base })
    }
    
    pub fn read_xyz(&self) -> Result<(f64, f64, f64), HalError> {
        let mut buf = [0u8; 6];
        self.base.bus.read_registers(self.base.address, 0x03, &mut buf)?;
        
        let x = ((buf[0] as i16) << 8 | buf[1] as i16) as f64 * 0.92;
        let y = ((buf[2] as i16) << 8 | buf[3] as i16) as f64 * 0.92;
        let z = ((buf[4] as i16) << 8 | buf[5] as i16) as f64 * 0.92;
        
        Ok((x, y, z))
    }
    
    pub fn read_magnitude(&self) -> Result<f64, HalError> {
        let (x, y, z) = self.read_xyz()?;
        Ok((x * x + y * y + z * z).sqrt())
    }
}

/// BME280 Temperature/Humidity/Pressure sensor
pub struct BME280 {
    base: I2CSensor,
}

impl BME280 {
    pub fn new(bus_path: &str) -> Result<Self, HalError> {
        let base = I2CSensor::new("BME280", bus_path, 0x77, "C")?;
        Ok(Self { base })
    }
    
    pub fn read_all(&self) -> Result<(f64, f64, f64), HalError> {
        // Read temperature, humidity, pressure
        let mut buf = [0u8; 8];
        self.base.bus.read_registers(self.base.address, 0xF7, &mut buf)?;
        
        // Simplified conversion (real implementation needs calibration data)
        let pressure = ((buf[0] as u32) << 12 | (buf[1] as u32) << 4 | (buf[2] as u32) >> 4) as f64 / 256.0;
        let temperature = ((buf[3] as u32) << 12 | (buf[4] as u32) << 4 | (buf[5] as u32) >> 4) as f64 / 5120.0 - 40.0;
        let humidity = ((buf[6] as u16) << 8 | buf[7] as u16) as f64 / 1024.0;
        
        Ok((temperature, humidity, pressure))
    }
}

/// MLX90614 IR Temperature sensor
pub struct MLX90614 {
    base: I2CSensor,
}

impl MLX90614 {
    pub fn new(bus_path: &str) -> Result<Self, HalError> {
        let base = I2CSensor::new("MLX90614", bus_path, 0x5A, "C")?;
        Ok(Self { base })
    }
    
    pub fn read_ambient(&self) -> Result<f64, HalError> {
        let mut buf = [0u8; 3];
        self.base.bus.read_registers(self.base.address, 0x06, &mut buf)?;
        let raw = (buf[0] as u16) | ((buf[1] as u16) << 8);
        Ok(raw as f64 * 0.02 - 273.15)
    }
    
    pub fn read_object(&self) -> Result<f64, HalError> {
        let mut buf = [0u8; 3];
        self.base.bus.read_registers(self.base.address, 0x07, &mut buf)?;
        let raw = (buf[0] as u16) | ((buf[1] as u16) << 8);
        Ok(raw as f64 * 0.02 - 273.15)
    }
}
