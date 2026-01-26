//! SPI interface for GlowBarn HAL

use crate::{HalError, HardwareDevice, DeviceType};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

/// SPI mode configuration
#[derive(Debug, Clone, Copy)]
pub enum SpiMode {
    Mode0,  // CPOL=0, CPHA=0
    Mode1,  // CPOL=0, CPHA=1
    Mode2,  // CPOL=1, CPHA=0
    Mode3,  // CPOL=1, CPHA=1
}

/// SPI bus configuration
#[derive(Debug, Clone)]
pub struct SpiConfig {
    pub mode: SpiMode,
    pub speed_hz: u32,
    pub bits_per_word: u8,
    pub lsb_first: bool,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            mode: SpiMode::Mode0,
            speed_hz: 1_000_000,
            bits_per_word: 8,
            lsb_first: false,
        }
    }
}

/// SPI Device wrapper
pub struct SpiDevice {
    path: String,
    fd: Option<i32>,
    config: SpiConfig,
}

impl SpiDevice {
    /// Open SPI device
    pub fn open(path: &str, config: SpiConfig) -> Result<Self, HalError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        
        let fd = file.as_raw_fd();
        let mut device = Self {
            path: path.to_string(),
            fd: Some(fd),
            config,
        };
        
        device.configure()?;
        Ok(device)
    }
    
    /// Configure SPI device
    fn configure(&mut self) -> Result<(), HalError> {
        #[cfg(target_os = "linux")]
        unsafe {
            if let Some(fd) = self.fd {
                // Set mode (SPI_IOC_WR_MODE = 0x40016B01)
                let mode = match self.config.mode {
                    SpiMode::Mode0 => 0,
                    SpiMode::Mode1 => 1,
                    SpiMode::Mode2 => 2,
                    SpiMode::Mode3 => 3,
                };
                libc::ioctl(fd, 0x40016B01, &mode);
                
                // Set bits per word (SPI_IOC_WR_BITS_PER_WORD = 0x40016B03)
                libc::ioctl(fd, 0x40016B03, &self.config.bits_per_word);
                
                // Set max speed (SPI_IOC_WR_MAX_SPEED_HZ = 0x40046B04)
                libc::ioctl(fd, 0x40046B04, &self.config.speed_hz);
            }
        }
        Ok(())
    }
    
    /// Transfer data (full-duplex)
    pub fn transfer(&self, tx: &[u8], rx: &mut [u8]) -> Result<(), HalError> {
        if tx.len() != rx.len() {
            return Err(HalError::InvalidConfig("TX/RX buffer size mismatch".to_string()));
        }
        
        #[cfg(target_os = "linux")]
        unsafe {
            if let Some(fd) = self.fd {
                // spi_ioc_transfer structure
                #[repr(C)]
                struct SpiIocTransfer {
                    tx_buf: u64,
                    rx_buf: u64,
                    len: u32,
                    speed_hz: u32,
                    delay_usecs: u16,
                    bits_per_word: u8,
                    cs_change: u8,
                    tx_nbits: u8,
                    rx_nbits: u8,
                    word_delay_usecs: u8,
                    pad: u8,
                }
                
                let transfer = SpiIocTransfer {
                    tx_buf: tx.as_ptr() as u64,
                    rx_buf: rx.as_mut_ptr() as u64,
                    len: tx.len() as u32,
                    speed_hz: self.config.speed_hz,
                    delay_usecs: 0,
                    bits_per_word: self.config.bits_per_word,
                    cs_change: 0,
                    tx_nbits: 0,
                    rx_nbits: 0,
                    word_delay_usecs: 0,
                    pad: 0,
                };
                
                // SPI_IOC_MESSAGE(1) = 0x40206B00
                let ret = libc::ioctl(fd, 0x40206B00, &transfer);
                if ret < 0 {
                    return Err(HalError::CommunicationError("SPI transfer failed".to_string()));
                }
            }
        }
        Ok(())
    }
    
    /// Write only
    pub fn write(&self, data: &[u8]) -> Result<(), HalError> {
        let mut rx = vec![0u8; data.len()];
        self.transfer(data, &mut rx)
    }
    
    /// Read only
    pub fn read(&self, len: usize) -> Result<Vec<u8>, HalError> {
        let tx = vec![0u8; len];
        let mut rx = vec![0u8; len];
        self.transfer(&tx, &mut rx)?;
        Ok(rx)
    }
    
    /// Write then read (for register access)
    pub fn write_read(&self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>, HalError> {
        let total_len = tx.len() + rx_len;
        let mut full_tx = vec![0u8; total_len];
        full_tx[..tx.len()].copy_from_slice(tx);
        
        let mut full_rx = vec![0u8; total_len];
        self.transfer(&full_tx, &mut full_rx)?;
        
        Ok(full_rx[tx.len()..].to_vec())
    }
}

/// ADS1256 24-bit ADC for high-precision sensor readings
pub struct ADS1256 {
    spi: SpiDevice,
    name: String,
    ready: bool,
}

impl ADS1256 {
    pub fn new(spi_path: &str) -> Result<Self, HalError> {
        let config = SpiConfig {
            mode: SpiMode::Mode1,
            speed_hz: 1_920_000,
            bits_per_word: 8,
            lsb_first: false,
        };
        
        let spi = SpiDevice::open(spi_path, config)?;
        
        Ok(Self {
            spi,
            name: "ADS1256".to_string(),
            ready: false,
        })
    }
    
    /// Read single channel
    pub fn read_channel(&self, channel: u8) -> Result<i32, HalError> {
        // Set MUX register
        let mux = (channel << 4) | 0x08;  // Single-ended, AINCOM
        self.spi.write(&[0x50 | 0x01, 0x00, mux])?;  // WREG MUX
        
        // Sync and wakeup
        self.spi.write(&[0xFC])?;  // SYNC
        self.spi.write(&[0x00])?;  // WAKEUP
        
        // Read data
        self.spi.write(&[0x01])?;  // RDATA
        let data = self.spi.read(3)?;
        
        let raw = ((data[0] as i32) << 16) | ((data[1] as i32) << 8) | (data[2] as i32);
        
        // Sign extend 24-bit to 32-bit
        if raw & 0x800000 != 0 {
            Ok(raw | 0xFF000000u32 as i32)
        } else {
            Ok(raw)
        }
    }
    
    /// Convert raw to voltage (assuming 5V reference)
    pub fn raw_to_voltage(raw: i32) -> f64 {
        (raw as f64 / 8388607.0) * 5.0
    }
    
    /// Read all channels
    pub fn read_all_channels(&self) -> Result<Vec<f64>, HalError> {
        let mut results = Vec::new();
        for ch in 0..8 {
            let raw = self.read_channel(ch)?;
            results.push(Self::raw_to_voltage(raw));
        }
        Ok(results)
    }
}

impl HardwareDevice for ADS1256 {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::SPI
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        // Reset
        self.spi.write(&[0xFE])?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Configure for high precision
        self.spi.write(&[0x50 | 0x00, 0x00, 0x01])?;  // STATUS: Auto-calibrate
        self.spi.write(&[0x50 | 0x02, 0x00, 0x00])?;  // ADCON: Clock off, PGA=1
        self.spi.write(&[0x50 | 0x03, 0x00, 0x63])?;  // DRATE: 50 SPS
        
        // Self calibrate
        self.spi.write(&[0xF0])?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        
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

/// MCP3008 10-bit ADC (for simpler analog readings)
pub struct MCP3008 {
    spi: SpiDevice,
    name: String,
    ready: bool,
}

impl MCP3008 {
    pub fn new(spi_path: &str) -> Result<Self, HalError> {
        let config = SpiConfig {
            mode: SpiMode::Mode0,
            speed_hz: 1_000_000,
            bits_per_word: 8,
            lsb_first: false,
        };
        
        let spi = SpiDevice::open(spi_path, config)?;
        
        Ok(Self {
            spi,
            name: "MCP3008".to_string(),
            ready: false,
        })
    }
    
    /// Read single channel (0-7)
    pub fn read_channel(&self, channel: u8) -> Result<u16, HalError> {
        if channel > 7 {
            return Err(HalError::InvalidConfig("Channel must be 0-7".to_string()));
        }
        
        let tx = [1, (8 + channel) << 4, 0];
        let rx = self.spi.write_read(&tx, 3)?;
        
        let value = ((rx[0] as u16 & 0x03) << 8) | rx[1] as u16;
        Ok(value)
    }
    
    /// Read all channels
    pub fn read_all(&self) -> Result<[u16; 8], HalError> {
        let mut values = [0u16; 8];
        for i in 0..8 {
            values[i] = self.read_channel(i as u8)?;
        }
        Ok(values)
    }
}

impl HardwareDevice for MCP3008 {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::SPI
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        // MCP3008 needs no special init
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
