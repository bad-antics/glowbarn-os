//! GPIO interface for GlowBarn HAL

use crate::{HalError, HardwareDevice, DeviceType};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// GPIO direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

/// GPIO edge trigger mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Edge {
    None,
    Rising,
    Falling,
    Both,
}

/// GPIO pull mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

/// GPIO pin state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Low = 0,
    High = 1,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        if val { Level::High } else { Level::Low }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> Self {
        level == Level::High
    }
}

/// Sysfs GPIO controller
pub struct SysfsGpio {
    pin: u32,
    exported: bool,
}

impl SysfsGpio {
    const GPIO_PATH: &'static str = "/sys/class/gpio";
    
    /// Export a GPIO pin
    pub fn export(pin: u32) -> Result<Self, HalError> {
        let export_path = format!("{}/export", Self::GPIO_PATH);
        
        // Check if already exported
        let pin_path = format!("{}/gpio{}", Self::GPIO_PATH, pin);
        if Path::new(&pin_path).exists() {
            return Ok(Self { pin, exported: true });
        }
        
        let mut file = OpenOptions::new()
            .write(true)
            .open(&export_path)?;
        
        file.write_all(pin.to_string().as_bytes())?;
        
        // Wait for sysfs to create the directory
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(Self { pin, exported: true })
    }
    
    /// Unexport GPIO pin
    pub fn unexport(&mut self) -> Result<(), HalError> {
        if !self.exported {
            return Ok(());
        }
        
        let unexport_path = format!("{}/unexport", Self::GPIO_PATH);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&unexport_path)?;
        
        file.write_all(self.pin.to_string().as_bytes())?;
        self.exported = false;
        Ok(())
    }
    
    /// Set direction
    pub fn set_direction(&self, direction: Direction) -> Result<(), HalError> {
        let path = format!("{}/gpio{}/direction", Self::GPIO_PATH, self.pin);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)?;
        
        let dir_str = match direction {
            Direction::Input => "in",
            Direction::Output => "out",
        };
        
        file.write_all(dir_str.as_bytes())?;
        Ok(())
    }
    
    /// Get current direction
    pub fn get_direction(&self) -> Result<Direction, HalError> {
        let path = format!("{}/gpio{}/direction", Self::GPIO_PATH, self.pin);
        let mut file = File::open(&path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        
        match buf.trim() {
            "in" => Ok(Direction::Input),
            "out" => Ok(Direction::Output),
            _ => Err(HalError::InvalidConfig("Unknown direction".to_string())),
        }
    }
    
    /// Set output value
    pub fn set_value(&self, level: Level) -> Result<(), HalError> {
        let path = format!("{}/gpio{}/value", Self::GPIO_PATH, self.pin);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)?;
        
        file.write_all((level as u8).to_string().as_bytes())?;
        Ok(())
    }
    
    /// Get input value
    pub fn get_value(&self) -> Result<Level, HalError> {
        let path = format!("{}/gpio{}/value", Self::GPIO_PATH, self.pin);
        let mut file = File::open(&path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        
        match buf.trim() {
            "0" => Ok(Level::Low),
            "1" => Ok(Level::High),
            _ => Err(HalError::InvalidConfig("Invalid GPIO value".to_string())),
        }
    }
    
    /// Set edge trigger mode
    pub fn set_edge(&self, edge: Edge) -> Result<(), HalError> {
        let path = format!("{}/gpio{}/edge", Self::GPIO_PATH, self.pin);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)?;
        
        let edge_str = match edge {
            Edge::None => "none",
            Edge::Rising => "rising",
            Edge::Falling => "falling",
            Edge::Both => "both",
        };
        
        file.write_all(edge_str.as_bytes())?;
        Ok(())
    }
    
    /// Toggle output
    pub fn toggle(&self) -> Result<Level, HalError> {
        let current = self.get_value()?;
        let new = if current == Level::High { Level::Low } else { Level::High };
        self.set_value(new)?;
        Ok(new)
    }
}

impl Drop for SysfsGpio {
    fn drop(&mut self) {
        let _ = self.unexport();
    }
}

/// GPIO Pin wrapper with higher-level interface
pub struct GpioPin {
    gpio: SysfsGpio,
    name: String,
    direction: Direction,
}

impl GpioPin {
    /// Create new GPIO pin
    pub fn new(name: &str, pin: u32, direction: Direction) -> Result<Self, HalError> {
        let gpio = SysfsGpio::export(pin)?;
        gpio.set_direction(direction)?;
        
        Ok(Self {
            gpio,
            name: name.to_string(),
            direction,
        })
    }
    
    /// Read pin value
    pub fn read(&self) -> Result<bool, HalError> {
        Ok(self.gpio.get_value()? == Level::High)
    }
    
    /// Write pin value
    pub fn write(&self, value: bool) -> Result<(), HalError> {
        self.gpio.set_value(value.into())
    }
    
    /// Pulse output (high then low)
    pub fn pulse(&self, duration: std::time::Duration) -> Result<(), HalError> {
        self.write(true)?;
        std::thread::sleep(duration);
        self.write(false)?;
        Ok(())
    }
}

impl HardwareDevice for GpioPin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::GPIO
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.gpio.set_direction(self.direction)?;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.gpio.unexport()
    }
}

/// PIR Motion sensor
pub struct PIRSensor {
    gpio: GpioPin,
    last_state: bool,
    motion_count: u64,
}

impl PIRSensor {
    pub fn new(name: &str, pin: u32) -> Result<Self, HalError> {
        let gpio = GpioPin::new(name, pin, Direction::Input)?;
        
        Ok(Self {
            gpio,
            last_state: false,
            motion_count: 0,
        })
    }
    
    /// Check for motion (returns true on rising edge)
    pub fn check_motion(&mut self) -> Result<bool, HalError> {
        let current = self.gpio.read()?;
        let motion = current && !self.last_state;
        self.last_state = current;
        
        if motion {
            self.motion_count += 1;
            tracing::info!("Motion detected! Total count: {}", self.motion_count);
        }
        
        Ok(motion)
    }
    
    /// Get total motion events
    pub fn motion_count(&self) -> u64 {
        self.motion_count
    }
    
    /// Reset counter
    pub fn reset_count(&mut self) {
        self.motion_count = 0;
    }
}

/// Laser grid sensor (for detecting movement through light beams)
pub struct LaserGrid {
    transmitters: Vec<GpioPin>,
    receivers: Vec<GpioPin>,
}

impl LaserGrid {
    pub fn new(tx_pins: &[u32], rx_pins: &[u32]) -> Result<Self, HalError> {
        if tx_pins.len() != rx_pins.len() {
            return Err(HalError::InvalidConfig("TX/RX pin count mismatch".to_string()));
        }
        
        let mut transmitters = Vec::new();
        let mut receivers = Vec::new();
        
        for (i, &pin) in tx_pins.iter().enumerate() {
            transmitters.push(GpioPin::new(&format!("laser_tx_{}", i), pin, Direction::Output)?);
        }
        
        for (i, &pin) in rx_pins.iter().enumerate() {
            receivers.push(GpioPin::new(&format!("laser_rx_{}", i), pin, Direction::Input)?);
        }
        
        Ok(Self { transmitters, receivers })
    }
    
    /// Enable all lasers
    pub fn enable(&self) -> Result<(), HalError> {
        for tx in &self.transmitters {
            tx.write(true)?;
        }
        Ok(())
    }
    
    /// Disable all lasers
    pub fn disable(&self) -> Result<(), HalError> {
        for tx in &self.transmitters {
            tx.write(false)?;
        }
        Ok(())
    }
    
    /// Check if any beams are broken
    pub fn check_beams(&self) -> Result<Vec<bool>, HalError> {
        let mut results = Vec::new();
        for rx in &self.receivers {
            // Low = beam broken
            results.push(!rx.read()?);
        }
        Ok(results)
    }
    
    /// Check if any beam is broken
    pub fn any_broken(&self) -> Result<bool, HalError> {
        for rx in &self.receivers {
            if !rx.read()? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// PWM output for servos and dimmers
pub struct PwmOutput {
    pin: u32,
    period_ns: u32,
    duty_ns: u32,
}

impl PwmOutput {
    const PWM_PATH: &'static str = "/sys/class/pwm/pwmchip0";
    
    /// Create new PWM output
    pub fn new(pin: u32, frequency: u32) -> Result<Self, HalError> {
        let period_ns = 1_000_000_000 / frequency;
        
        // Export PWM
        let export_path = format!("{}/export", Self::PWM_PATH);
        if let Ok(mut file) = OpenOptions::new().write(true).open(&export_path) {
            let _ = file.write_all(pin.to_string().as_bytes());
        }
        
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let mut pwm = Self {
            pin,
            period_ns,
            duty_ns: 0,
        };
        
        pwm.set_period(period_ns)?;
        
        Ok(pwm)
    }
    
    fn write_attribute(&self, attr: &str, value: &str) -> Result<(), HalError> {
        let path = format!("{}/pwm{}/{}", Self::PWM_PATH, self.pin, attr);
        let mut file = OpenOptions::new().write(true).open(&path)?;
        file.write_all(value.as_bytes())?;
        Ok(())
    }
    
    /// Set period in nanoseconds
    pub fn set_period(&mut self, period_ns: u32) -> Result<(), HalError> {
        self.write_attribute("period", &period_ns.to_string())?;
        self.period_ns = period_ns;
        Ok(())
    }
    
    /// Set duty cycle in nanoseconds
    pub fn set_duty_ns(&mut self, duty_ns: u32) -> Result<(), HalError> {
        self.write_attribute("duty_cycle", &duty_ns.to_string())?;
        self.duty_ns = duty_ns;
        Ok(())
    }
    
    /// Set duty cycle as percentage (0.0 - 1.0)
    pub fn set_duty(&mut self, duty: f64) -> Result<(), HalError> {
        let duty_ns = (self.period_ns as f64 * duty.clamp(0.0, 1.0)) as u32;
        self.set_duty_ns(duty_ns)
    }
    
    /// Enable PWM output
    pub fn enable(&self) -> Result<(), HalError> {
        self.write_attribute("enable", "1")
    }
    
    /// Disable PWM output
    pub fn disable(&self) -> Result<(), HalError> {
        self.write_attribute("enable", "0")
    }
}
