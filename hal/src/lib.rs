//! GlowBarn Hardware Abstraction Layer
//! 
//! Provides unified access to sensors and hardware interfaces
//! for the GlowBarn Paranormal Detection Suite.
//!
//! # Modules
//! 
//! - [`i2c`] - I2C bus interface for sensors like HMC5883L, BME280, MLX90614
//! - [`spi`] - SPI interface for high-precision ADCs (ADS1256, MCP3008)
//! - [`gpio`] - GPIO for PIR sensors, laser grids, and PWM control
//! - [`usb`] - USB device enumeration and serial communication
//! - [`audio`] - ALSA audio capture for EVP detection
//! - [`camera`] - V4L2 video capture, thermal imaging, night vision
//! - [`sdr`] - RTL-SDR for EMF spectrum analysis
//!
//! # Example
//! 
//! ```rust,no_run
//! use glowbarn_hal::{HardwareManager, HalConfig};
//! 
//! #[tokio::main]
//! async fn main() {
//!     let config = HalConfig::default();
//!     let (mut manager, mut readings) = HardwareManager::new(config);
//!     
//!     manager.init().await.unwrap();
//!     manager.start_polling(std::time::Duration::from_millis(100)).await;
//!     
//!     while let Some(reading) = readings.recv().await {
//!         println!("{}: {} {}", reading.sensor_name, reading.value, reading.unit);
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;

pub mod i2c;
pub mod spi;
pub mod gpio;
pub mod usb;
pub mod audio;
pub mod camera;
pub mod sdr;

// Re-exports for convenience
pub use i2c::{I2CBus, I2CSensor, HMC5883L, BME280, MLX90614};
pub use spi::{SpiDevice, SpiConfig, SpiMode, ADS1256, MCP3008};
pub use gpio::{GpioPin, Direction, Level, PIRSensor, LaserGrid, PwmOutput};
pub use usb::{UsbSerial, UsbHid, UsbDeviceInfo};
pub use audio::{AudioCapture, AudioPlayback, AudioFormat, SpiritBox, InfrasoundDetector};
pub use camera::{Camera, ThermalCamera, NightVisionCamera, Frame, ThermalFrame, VideoFormat};
pub use sdr::{RtlSdr, SdrConfig, EmfAnalyzer, RadioScanner};

/// Hardware device trait
pub trait HardwareDevice: Send + Sync {
    /// Device name
    fn name(&self) -> &str;
    
    /// Device type
    fn device_type(&self) -> DeviceType;
    
    /// Initialize the device
    fn init(&mut self) -> Result<(), HalError>;
    
    /// Check if device is ready
    fn is_ready(&self) -> bool;
    
    /// Close the device
    fn close(&mut self) -> Result<(), HalError>;
}

/// Sensor trait for data acquisition
pub trait Sensor: HardwareDevice {
    /// Read raw data from sensor
    fn read_raw(&self) -> Result<Vec<u8>, HalError>;
    
    /// Read calibrated value
    fn read_value(&self) -> Result<f64, HalError>;
    
    /// Get sensor unit
    fn unit(&self) -> &str;
    
    /// Calibrate sensor
    fn calibrate(&mut self, offset: f64) -> Result<(), HalError>;
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    I2C,
    SPI,
    GPIO,
    USB,
    Audio,
    Camera,
    SDR,
    Serial,
}

/// HAL Error types
#[derive(Debug, thiserror::Error)]
pub enum HalError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Device busy: {0}")]
    DeviceBusy(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Communication error: {0}")]
    CommunicationError(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Calibration required")]
    CalibrationRequired,
}

/// Sensor reading with metadata
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: std::time::SystemTime,
    pub quality: f32,  // 0.0 - 1.0
}

/// Hardware manager
pub struct HardwareManager {
    devices: Arc<RwLock<HashMap<String, Box<dyn HardwareDevice>>>>,
    sensors: Arc<RwLock<HashMap<String, Box<dyn Sensor>>>>,
    reading_tx: mpsc::Sender<SensorReading>,
    config: HalConfig,
}

/// HAL Configuration
#[derive(Debug, Clone)]
pub struct HalConfig {
    pub scan_interval: Duration,
    pub hotplug_enabled: bool,
    pub watchdog_timeout: Duration,
    pub i2c_buses: Vec<String>,
    pub spi_devices: Vec<String>,
    pub gpio_chip: String,
}

impl Default for HalConfig {
    fn default() -> Self {
        Self {
            scan_interval: Duration::from_secs(10),
            hotplug_enabled: true,
            watchdog_timeout: Duration::from_secs(30),
            i2c_buses: vec!["/dev/i2c-1".to_string()],
            spi_devices: vec!["/dev/spidev0.0".to_string()],
            gpio_chip: "/dev/gpiochip0".to_string(),
        }
    }
}

impl HardwareManager {
    /// Create new hardware manager
    pub fn new(config: HalConfig) -> (Self, mpsc::Receiver<SensorReading>) {
        let (tx, rx) = mpsc::channel(1000);
        
        (Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            sensors: Arc::new(RwLock::new(HashMap::new())),
            reading_tx: tx,
            config,
        }, rx)
    }
    
    /// Initialize all hardware
    pub async fn init(&mut self) -> Result<(), HalError> {
        // Scan I2C buses
        let buses = self.config.i2c_buses.clone();
        for bus in buses {
            if let Err(e) = self.scan_i2c_bus(&bus).await {
                tracing::warn!("Failed to scan I2C bus {}: {}", bus, e);
            }
        }
        
        // Initialize GPIO
        if let Err(e) = self.init_gpio().await {
            tracing::warn!("Failed to initialize GPIO: {}", e);
        }
        
        // Scan USB devices
        if let Err(e) = self.scan_usb_devices().await {
            tracing::warn!("Failed to scan USB devices: {}", e);
        }
        
        // Initialize audio
        if let Err(e) = self.init_audio().await {
            tracing::warn!("Failed to initialize audio: {}", e);
        }
        
        Ok(())
    }
    
    /// Scan I2C bus for devices
    async fn scan_i2c_bus(&mut self, bus: &str) -> Result<Vec<u8>, HalError> {
        tracing::info!("Scanning I2C bus: {}", bus);
        i2c::scan_bus(bus)
    }
    
    /// Initialize GPIO
    async fn init_gpio(&mut self) -> Result<(), HalError> {
        tracing::info!("Initializing GPIO: {}", self.config.gpio_chip);
        Ok(())  // GPIO pins are initialized on demand
    }
    
    /// Scan USB devices
    async fn scan_usb_devices(&mut self) -> Result<(), HalError> {
        tracing::info!("Scanning USB devices");
        let devices = usb::enumerate_devices()?;
        for device in &devices {
            tracing::info!("Found USB device: {:04X}:{:04X} - {} {}",
                device.vendor_id, device.product_id,
                device.manufacturer, device.product);
        }
        Ok(())
    }
    
    /// Initialize audio subsystem
    async fn init_audio(&mut self) -> Result<(), HalError> {
        tracing::info!("Initializing audio subsystem");
        Ok(())  // Audio devices are initialized on demand
    }
    
    /// Register a sensor
    pub fn register_sensor(&mut self, name: &str, sensor: Box<dyn Sensor>) {
        let mut sensors = self.sensors.write().unwrap();
        sensors.insert(name.to_string(), sensor);
    }
    
    /// Read from all sensors
    pub async fn read_all_sensors(&self) -> Vec<SensorReading> {
        let sensors = self.sensors.read().unwrap();
        let mut readings = Vec::new();
        
        for (name, sensor) in sensors.iter() {
            match sensor.read_value() {
                Ok(value) => {
                    let reading = SensorReading {
                        sensor_name: name.clone(),
                        value,
                        unit: sensor.unit().to_string(),
                        timestamp: std::time::SystemTime::now(),
                        quality: 1.0,
                    };
                    readings.push(reading);
                }
                Err(e) => {
                    tracing::warn!("Failed to read sensor {}: {}", name, e);
                }
            }
        }
        
        readings
    }
    
    /// Start continuous sensor polling
    pub async fn start_polling(&self, interval: Duration) {
        let sensors = self.sensors.clone();
        let tx = self.reading_tx.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Clone readings out of the lock to avoid holding it across await
                let readings: Vec<(String, f64, String)> = {
                    let sensors = sensors.read().unwrap();
                    sensors.iter()
                        .filter_map(|(name, sensor)| {
                            sensor.read_value().ok().map(|value| {
                                (name.clone(), value, sensor.unit().to_string())
                            })
                        })
                        .collect()
                };
                
                for (sensor_name, value, unit) in readings {
                    let reading = SensorReading {
                        sensor_name,
                        value,
                        unit,
                        timestamp: std::time::SystemTime::now(),
                        quality: 1.0,
                    };
                    
                    if tx.send(reading).await.is_err() {
                        tracing::error!("Failed to send sensor reading");
                        return;
                    }
                }
            }
        });
    }
}
