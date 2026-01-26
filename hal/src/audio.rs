//! Audio interface for GlowBarn HAL
//! Supports ALSA for audio capture and playback

use crate::{HalError, HardwareDevice, DeviceType};
use std::sync::{Arc, Mutex};

/// Audio format configuration
#[derive(Debug, Clone)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        }
    }
}

/// Audio capture device
pub struct AudioCapture {
    name: String,
    device: String,
    format: AudioFormat,
    buffer: Arc<Mutex<Vec<i16>>>,
    recording: bool,
}

impl AudioCapture {
    /// Create new audio capture device
    pub fn new(device: &str, format: AudioFormat) -> Result<Self, HalError> {
        Ok(Self {
            name: format!("Audio Capture {}", device),
            device: device.to_string(),
            format,
            buffer: Arc::new(Mutex::new(Vec::new())),
            recording: false,
        })
    }
    
    /// Start recording
    pub fn start(&mut self) -> Result<(), HalError> {
        self.recording = true;
        tracing::info!("Audio capture started on {}", self.device);
        Ok(())
    }
    
    /// Stop recording
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.recording = false;
        Ok(())
    }
    
    /// Read samples (returns number of samples read)
    pub fn read_samples(&self, samples: &mut [i16]) -> Result<usize, HalError> {
        // In production, this would read from ALSA
        // For now, simulate reading silence
        for sample in samples.iter_mut() {
            *sample = 0;
        }
        Ok(samples.len())
    }
    
    /// Get RMS level (for visualization)
    pub fn get_rms_level(&self, samples: &[i16]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = samples.iter()
            .map(|&s| (s as f64).powi(2))
            .sum();
        
        (sum / samples.len() as f64).sqrt()
    }
    
    /// Calculate frequency spectrum (simple FFT placeholder)
    pub fn calculate_spectrum(&self, samples: &[i16]) -> Vec<f64> {
        // Placeholder - in production use rustfft
        let mut spectrum = vec![0.0; samples.len() / 2];
        
        // Simple magnitude calculation (not real FFT)
        for (i, chunk) in samples.chunks(2).enumerate() {
            if chunk.len() == 2 {
                let mag = ((chunk[0] as f64).powi(2) + (chunk[1] as f64).powi(2)).sqrt();
                if i < spectrum.len() {
                    spectrum[i] = mag;
                }
            }
        }
        
        spectrum
    }
    
    /// Detect EVP-like anomalies (frequency patterns not matching ambient)
    pub fn detect_anomalies(&self, samples: &[i16], threshold: f64) -> Vec<AudioAnomaly> {
        let mut anomalies = Vec::new();
        let rms = self.get_rms_level(samples);
        
        // Simple spike detection
        for (i, window) in samples.windows(1024).enumerate() {
            let window_rms = self.get_rms_level(window);
            let ratio = if rms > 0.0 { window_rms / rms } else { 0.0 };
            
            if ratio > threshold {
                anomalies.push(AudioAnomaly {
                    timestamp_samples: i * 1024,
                    duration_samples: 1024,
                    intensity: ratio,
                    anomaly_type: AnomalyType::Spike,
                });
            }
        }
        
        anomalies
    }
}

impl HardwareDevice for AudioCapture {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        // Initialize ALSA device
        tracing::info!("Initializing audio device: {}", self.device);
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.stop()
    }
}

/// Audio anomaly detection result
#[derive(Debug, Clone)]
pub struct AudioAnomaly {
    pub timestamp_samples: usize,
    pub duration_samples: usize,
    pub intensity: f64,
    pub anomaly_type: AnomalyType,
}

#[derive(Debug, Clone)]
pub enum AnomalyType {
    Spike,
    Pattern,
    Voice,
    Ultrasonic,
    Infrasonic,
}

/// Audio playback device
pub struct AudioPlayback {
    name: String,
    device: String,
    format: AudioFormat,
    playing: bool,
}

impl AudioPlayback {
    /// Create new playback device
    pub fn new(device: &str, format: AudioFormat) -> Result<Self, HalError> {
        Ok(Self {
            name: format!("Audio Playback {}", device),
            device: device.to_string(),
            format,
            playing: false,
        })
    }
    
    /// Play samples
    pub fn play_samples(&mut self, samples: &[i16]) -> Result<(), HalError> {
        if samples.is_empty() {
            return Ok(());
        }
        
        self.playing = true;
        // In production, write to ALSA
        self.playing = false;
        Ok(())
    }
    
    /// Generate tone
    pub fn generate_tone(&self, frequency: f64, duration_ms: u32) -> Vec<i16> {
        let num_samples = (self.format.sample_rate as f64 * duration_ms as f64 / 1000.0) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        
        for i in 0..num_samples {
            let t = i as f64 / self.format.sample_rate as f64;
            let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
            samples.push((sample * 32767.0) as i16);
        }
        
        samples
    }
    
    /// Play tone
    pub fn play_tone(&mut self, frequency: f64, duration_ms: u32) -> Result<(), HalError> {
        let samples = self.generate_tone(frequency, duration_ms);
        self.play_samples(&samples)
    }
}

impl HardwareDevice for AudioPlayback {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.playing = false;
        Ok(())
    }
}

/// Spirit Box emulation (frequency sweeping radio scanner)
pub struct SpiritBox {
    capture: AudioCapture,
    sweep_rate: f64,  // MHz per second
    current_freq: f64,
    running: bool,
}

impl SpiritBox {
    pub fn new(device: &str, sweep_rate: f64) -> Result<Self, HalError> {
        let format = AudioFormat {
            sample_rate: 48000,
            channels: 1,
            bits_per_sample: 16,
        };
        
        let capture = AudioCapture::new(device, format)?;
        
        Ok(Self {
            capture,
            sweep_rate,
            current_freq: 88.0,  // FM range start
            running: false,
        })
    }
    
    /// Start sweep
    pub fn start(&mut self) -> Result<(), HalError> {
        self.running = true;
        self.capture.start()?;
        Ok(())
    }
    
    /// Stop sweep
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.running = false;
        self.capture.stop()?;
        Ok(())
    }
    
    /// Get current frequency
    pub fn current_frequency(&self) -> f64 {
        self.current_freq
    }
    
    /// Step frequency
    pub fn step(&mut self) {
        self.current_freq += self.sweep_rate / 100.0;
        if self.current_freq > 108.0 {
            self.current_freq = 88.0;
        }
    }
}

/// Infrasound detector (0-20 Hz)
pub struct InfrasoundDetector {
    capture: AudioCapture,
    threshold_db: f64,
}

impl InfrasoundDetector {
    pub fn new(device: &str, threshold_db: f64) -> Result<Self, HalError> {
        let format = AudioFormat {
            sample_rate: 96000,  // High sample rate for low freq
            channels: 1,
            bits_per_sample: 24,
        };
        
        let capture = AudioCapture::new(device, format)?;
        
        Ok(Self {
            capture,
            threshold_db,
        })
    }
    
    /// Check for infrasound presence
    pub fn detect(&self, samples: &[i16]) -> Option<InfrasoundEvent> {
        // Apply low-pass filter and detect presence
        let filtered = self.low_pass_filter(samples, 20.0);
        let rms = self.capture.get_rms_level(&filtered);
        let db = 20.0 * (rms / 32767.0).log10();
        
        if db > self.threshold_db {
            Some(InfrasoundEvent {
                level_db: db,
                estimated_frequency: self.estimate_frequency(&filtered),
            })
        } else {
            None
        }
    }
    
    fn low_pass_filter(&self, samples: &[i16], cutoff: f64) -> Vec<i16> {
        // Simple RC low-pass filter
        let rc = 1.0 / (2.0 * std::f64::consts::PI * cutoff);
        let dt = 1.0 / self.capture.format.sample_rate as f64;
        let alpha = dt / (rc + dt);
        
        let mut filtered = Vec::with_capacity(samples.len());
        let mut prev = 0.0;
        
        for &sample in samples {
            let curr = alpha * sample as f64 + (1.0 - alpha) * prev;
            filtered.push(curr as i16);
            prev = curr;
        }
        
        filtered
    }
    
    fn estimate_frequency(&self, samples: &[i16]) -> f64 {
        // Zero-crossing frequency estimation
        let mut crossings = 0;
        for window in samples.windows(2) {
            if (window[0] >= 0) != (window[1] >= 0) {
                crossings += 1;
            }
        }
        
        let duration = samples.len() as f64 / self.capture.format.sample_rate as f64;
        crossings as f64 / (2.0 * duration)
    }
}

#[derive(Debug, Clone)]
pub struct InfrasoundEvent {
    pub level_db: f64,
    pub estimated_frequency: f64,
}
