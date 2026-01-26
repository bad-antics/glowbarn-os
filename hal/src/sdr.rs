//! SDR (Software Defined Radio) interface for GlowBarn HAL
//! Supports RTL-SDR for radio spectrum analysis

use crate::{HalError, HardwareDevice, DeviceType};
use std::sync::{Arc, Mutex};

/// SDR device configuration
#[derive(Debug, Clone)]
pub struct SdrConfig {
    pub center_frequency: u64,  // Hz
    pub sample_rate: u32,       // Hz
    pub gain: i32,              // 0.1 dB units
    pub agc: bool,
}

impl Default for SdrConfig {
    fn default() -> Self {
        Self {
            center_frequency: 100_000_000,  // 100 MHz
            sample_rate: 2_000_000,         // 2 MSPS
            gain: 400,                      // 40.0 dB
            agc: false,
        }
    }
}

/// RTL-SDR device
pub struct RtlSdr {
    name: String,
    config: SdrConfig,
    device_index: u32,
    ready: bool,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl RtlSdr {
    /// Open RTL-SDR device
    pub fn open(device_index: u32) -> Result<Self, HalError> {
        Ok(Self {
            name: format!("RTL-SDR #{}", device_index),
            config: SdrConfig::default(),
            device_index,
            ready: false,
            buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Set center frequency
    pub fn set_frequency(&mut self, freq: u64) -> Result<(), HalError> {
        if freq < 24_000_000 || freq > 1_766_000_000 {
            return Err(HalError::InvalidConfig(
                "Frequency must be between 24 MHz and 1766 MHz".to_string()
            ));
        }
        self.config.center_frequency = freq;
        // In production: rtlsdr_set_center_freq()
        Ok(())
    }
    
    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<(), HalError> {
        if rate < 225_000 || rate > 3_200_000 {
            return Err(HalError::InvalidConfig(
                "Sample rate must be between 225 kHz and 3.2 MHz".to_string()
            ));
        }
        self.config.sample_rate = rate;
        Ok(())
    }
    
    /// Set gain (in 0.1 dB units)
    pub fn set_gain(&mut self, gain: i32) -> Result<(), HalError> {
        self.config.gain = gain;
        self.config.agc = false;
        Ok(())
    }
    
    /// Enable automatic gain control
    pub fn enable_agc(&mut self) -> Result<(), HalError> {
        self.config.agc = true;
        Ok(())
    }
    
    /// Read IQ samples
    pub fn read_samples(&self, count: usize) -> Result<Vec<Complex>, HalError> {
        if !self.ready {
            return Err(HalError::DeviceNotFound("SDR not initialized".to_string()));
        }
        
        // In production, this would read from RTL-SDR
        // RTL-SDR outputs interleaved I/Q bytes (unsigned 8-bit)
        let mut samples = Vec::with_capacity(count);
        
        // Simulate noise for testing
        for _ in 0..count {
            samples.push(Complex {
                i: (rand_byte() as f64 - 127.5) / 127.5,
                q: (rand_byte() as f64 - 127.5) / 127.5,
            });
        }
        
        Ok(samples)
    }
    
    /// Calculate power spectrum (simplified FFT)
    pub fn power_spectrum(&self, samples: &[Complex]) -> Vec<f64> {
        // In production, use rustfft for proper FFT
        samples.iter()
            .map(|c| (c.i * c.i + c.q * c.q).sqrt())
            .collect()
    }
    
    /// Scan frequency range for signals
    pub fn scan_range(&mut self, start: u64, end: u64, step: u64) -> Result<Vec<SignalPeak>, HalError> {
        let mut peaks = Vec::new();
        let mut freq = start;
        
        while freq <= end {
            self.set_frequency(freq)?;
            
            // Read and analyze
            let samples = self.read_samples(1024)?;
            let spectrum = self.power_spectrum(&samples);
            
            let max_power = spectrum.iter().cloned().fold(0.0, f64::max);
            let avg_power = spectrum.iter().sum::<f64>() / spectrum.len() as f64;
            
            // Detect peaks above noise floor
            if max_power > avg_power * 3.0 {
                peaks.push(SignalPeak {
                    frequency: freq,
                    power: max_power,
                    bandwidth: step,
                });
            }
            
            freq += step;
        }
        
        Ok(peaks)
    }
}

impl HardwareDevice for RtlSdr {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::SDR
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        // In production: rtlsdr_open()
        self.ready = true;
        tracing::info!("RTL-SDR #{} initialized", self.device_index);
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

/// Complex IQ sample
#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub i: f64,
    pub q: f64,
}

impl Complex {
    pub fn magnitude(&self) -> f64 {
        (self.i * self.i + self.q * self.q).sqrt()
    }
    
    pub fn phase(&self) -> f64 {
        self.q.atan2(self.i)
    }
}

/// Detected signal peak
#[derive(Debug, Clone)]
pub struct SignalPeak {
    pub frequency: u64,
    pub power: f64,
    pub bandwidth: u64,
}

/// EMF spectrum analyzer using SDR
pub struct EmfAnalyzer {
    sdr: RtlSdr,
    baseline: Option<Vec<f64>>,
}

impl EmfAnalyzer {
    /// Create EMF analyzer
    pub fn new(device_index: u32) -> Result<Self, HalError> {
        let sdr = RtlSdr::open(device_index)?;
        Ok(Self {
            sdr,
            baseline: None,
        })
    }
    
    /// Capture baseline (ambient EMF)
    pub fn capture_baseline(&mut self) -> Result<(), HalError> {
        let samples = self.sdr.read_samples(4096)?;
        self.baseline = Some(self.sdr.power_spectrum(&samples));
        tracing::info!("EMF baseline captured");
        Ok(())
    }
    
    /// Detect EMF anomalies compared to baseline
    pub fn detect_anomalies(&self, threshold: f64) -> Result<Vec<EmfAnomaly>, HalError> {
        let samples = self.sdr.read_samples(4096)?;
        let current = self.sdr.power_spectrum(&samples);
        
        let baseline = self.baseline.as_ref()
            .ok_or_else(|| HalError::InvalidConfig("No baseline captured".to_string()))?;
        
        let mut anomalies = Vec::new();
        
        for (i, (&curr, &base)) in current.iter().zip(baseline.iter()).enumerate() {
            let ratio = if base > 0.0 { curr / base } else { curr };
            
            if ratio > threshold {
                // Calculate approximate frequency offset
                let bin_hz = self.sdr.config.sample_rate as f64 / baseline.len() as f64;
                let freq_offset = (i as f64 - baseline.len() as f64 / 2.0) * bin_hz;
                
                anomalies.push(EmfAnomaly {
                    frequency_offset: freq_offset as i64,
                    power_ratio: ratio,
                    absolute_power: curr,
                });
            }
        }
        
        Ok(anomalies)
    }
    
    /// Monitor for sudden EMF bursts
    pub fn monitor_bursts(&self, duration_ms: u64) -> Result<Vec<EmfBurst>, HalError> {
        let mut bursts = Vec::new();
        let start = std::time::Instant::now();
        let mut prev_power = 0.0;
        
        while start.elapsed().as_millis() < duration_ms as u128 {
            let samples = self.sdr.read_samples(1024)?;
            let power: f64 = samples.iter().map(|c| c.magnitude()).sum::<f64>() / samples.len() as f64;
            
            // Detect sudden increase
            if power > prev_power * 2.0 && prev_power > 0.0 {
                bursts.push(EmfBurst {
                    timestamp: std::time::SystemTime::now(),
                    power_increase: power / prev_power,
                    absolute_power: power,
                });
            }
            
            prev_power = power;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        Ok(bursts)
    }
}

#[derive(Debug, Clone)]
pub struct EmfAnomaly {
    pub frequency_offset: i64,
    pub power_ratio: f64,
    pub absolute_power: f64,
}

#[derive(Debug, Clone)]
pub struct EmfBurst {
    pub timestamp: std::time::SystemTime,
    pub power_increase: f64,
    pub absolute_power: f64,
}

/// Radio scanner for EVP sessions
pub struct RadioScanner {
    sdr: RtlSdr,
    sweep_start: u64,
    sweep_end: u64,
    dwell_time_ms: u32,
}

impl RadioScanner {
    /// Create radio scanner for FM band
    pub fn new_fm(device_index: u32) -> Result<Self, HalError> {
        let sdr = RtlSdr::open(device_index)?;
        Ok(Self {
            sdr,
            sweep_start: 88_000_000,   // 88 MHz
            sweep_end: 108_000_000,    // 108 MHz
            dwell_time_ms: 50,
        })
    }
    
    /// Create radio scanner for AM band
    pub fn new_am(device_index: u32) -> Result<Self, HalError> {
        let sdr = RtlSdr::open(device_index)?;
        Ok(Self {
            sdr,
            sweep_start: 530_000,      // 530 kHz
            sweep_end: 1_700_000,      // 1700 kHz
            dwell_time_ms: 30,
        })
    }
    
    /// Set sweep range
    pub fn set_range(&mut self, start: u64, end: u64) {
        self.sweep_start = start;
        self.sweep_end = end;
    }
    
    /// Set dwell time per frequency
    pub fn set_dwell_time(&mut self, ms: u32) {
        self.dwell_time_ms = ms;
    }
    
    /// Perform single sweep
    pub fn sweep(&mut self) -> Result<Vec<RadioSample>, HalError> {
        let step = 200_000;  // 200 kHz steps
        let mut samples = Vec::new();
        
        let mut freq = self.sweep_start;
        while freq <= self.sweep_end {
            self.sdr.set_frequency(freq)?;
            std::thread::sleep(std::time::Duration::from_millis(self.dwell_time_ms as u64));
            
            let iq = self.sdr.read_samples(1024)?;
            let power = iq.iter().map(|c| c.magnitude()).sum::<f64>() / iq.len() as f64;
            
            samples.push(RadioSample {
                frequency: freq,
                power,
            });
            
            freq += step;
        }
        
        Ok(samples)
    }
    
    /// Continuous sweep with callback
    pub fn continuous_sweep<F>(&mut self, mut callback: F) -> Result<(), HalError>
    where
        F: FnMut(u64, f64) -> bool,  // frequency, power -> continue?
    {
        let step = 200_000;
        let mut freq = self.sweep_start;
        
        loop {
            self.sdr.set_frequency(freq)?;
            std::thread::sleep(std::time::Duration::from_millis(self.dwell_time_ms as u64));
            
            let iq = self.sdr.read_samples(1024)?;
            let power = iq.iter().map(|c| c.magnitude()).sum::<f64>() / iq.len() as f64;
            
            if !callback(freq, power) {
                break;
            }
            
            freq += step;
            if freq > self.sweep_end {
                freq = self.sweep_start;
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RadioSample {
    pub frequency: u64,
    pub power: f64,
}

/// Simple pseudo-random byte generator for testing
fn rand_byte() -> u8 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED >> 16) as u8
    }
}

/// Enumerate RTL-SDR devices
pub fn enumerate_devices() -> Vec<u32> {
    // In production: rtlsdr_get_device_count()
    // For now, assume up to 4 devices
    let mut devices = Vec::new();
    for i in 0..4 {
        // Check if device exists
        let path = format!("/dev/bus/usb/001/{:03}", i + 1);
        if std::path::Path::new(&path).exists() {
            devices.push(i);
        }
    }
    devices
}
