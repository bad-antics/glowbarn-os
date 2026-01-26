//! Camera interface for GlowBarn HAL
//! Supports V4L2 for video capture and thermal imaging

use crate::{HalError, HardwareDevice, DeviceType};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

/// Video format configuration
#[derive(Debug, Clone)]
pub struct VideoFormat {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub fps: u32,
}

impl Default for VideoFormat {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            pixel_format: PixelFormat::YUYV,
            fps: 30,
        }
    }
}

/// Pixel format
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    YUYV,
    MJPEG,
    RGB24,
    BGR24,
    GREY,
    Y16,  // 16-bit grayscale (thermal)
}

impl PixelFormat {
    fn fourcc(&self) -> u32 {
        match self {
            PixelFormat::YUYV => 0x56595559,   // 'YUYV'
            PixelFormat::MJPEG => 0x47504A4D,  // 'MJPG'
            PixelFormat::RGB24 => 0x33424752,  // 'RGB3'
            PixelFormat::BGR24 => 0x33524742,  // 'BGR3'
            PixelFormat::GREY => 0x59455247,   // 'GREY'
            PixelFormat::Y16 => 0x20363159,    // 'Y16 '
        }
    }
}

/// V4L2 camera device
pub struct Camera {
    name: String,
    device: String,
    format: VideoFormat,
    file: Option<File>,
    ready: bool,
    buffers: Vec<Vec<u8>>,
}

impl Camera {
    /// Open camera device
    pub fn open(device: &str, format: VideoFormat) -> Result<Self, HalError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)?;
        
        Ok(Self {
            name: format!("Camera {}", device),
            device: device.to_string(),
            format,
            file: Some(file),
            ready: false,
            buffers: Vec::new(),
        })
    }
    
    /// Configure video format
    fn configure_format(&mut self) -> Result<(), HalError> {
        #[cfg(target_os = "linux")]
        if let Some(ref file) = self.file {
            let fd = file.as_raw_fd();
            
            // VIDIOC_S_FMT = 0xC0D05605
            #[repr(C)]
            struct V4l2Format {
                format_type: u32,
                pix: V4l2PixFormat,
                raw_data: [u8; 156],
            }
            
            #[repr(C)]
            #[derive(Default)]
            struct V4l2PixFormat {
                width: u32,
                height: u32,
                pixelformat: u32,
                field: u32,
                bytesperline: u32,
                sizeimage: u32,
                colorspace: u32,
                priv_: u32,
                flags: u32,
                quantization: u32,
                xfer_func: u32,
            }
            
            let mut fmt = V4l2Format {
                format_type: 1,  // V4L2_BUF_TYPE_VIDEO_CAPTURE
                pix: V4l2PixFormat {
                    width: self.format.width,
                    height: self.format.height,
                    pixelformat: self.format.pixel_format.fourcc(),
                    ..Default::default()
                },
                raw_data: [0; 156],
            };
            
            unsafe {
                let ret = libc::ioctl(fd, 0xC0D05605, &mut fmt);
                if ret < 0 {
                    return Err(HalError::CommunicationError("Failed to set video format".to_string()));
                }
            }
        }
        Ok(())
    }
    
    /// Request and map buffers
    fn setup_buffers(&mut self, count: u32) -> Result<(), HalError> {
        // Allocate internal buffers
        let buffer_size = (self.format.width * self.format.height * 2) as usize;
        self.buffers = (0..count).map(|_| vec![0u8; buffer_size]).collect();
        Ok(())
    }
    
    /// Start streaming
    pub fn start_streaming(&mut self) -> Result<(), HalError> {
        self.setup_buffers(4)?;
        
        #[cfg(target_os = "linux")]
        if let Some(ref file) = self.file {
            let fd = file.as_raw_fd();
            let buf_type: u32 = 1;  // V4L2_BUF_TYPE_VIDEO_CAPTURE
            
            unsafe {
                // VIDIOC_STREAMON = 0x40045612
                libc::ioctl(fd, 0x40045612, &buf_type);
            }
        }
        
        self.ready = true;
        Ok(())
    }
    
    /// Stop streaming
    pub fn stop_streaming(&mut self) -> Result<(), HalError> {
        #[cfg(target_os = "linux")]
        if let Some(ref file) = self.file {
            let fd = file.as_raw_fd();
            let buf_type: u32 = 1;
            
            unsafe {
                // VIDIOC_STREAMOFF = 0x40045613
                libc::ioctl(fd, 0x40045613, &buf_type);
            }
        }
        
        self.ready = false;
        Ok(())
    }
    
    /// Capture single frame
    pub fn capture_frame(&mut self) -> Result<Frame, HalError> {
        if !self.ready {
            return Err(HalError::DeviceNotFound("Camera not streaming".to_string()));
        }
        
        // In production, this would dequeue a buffer from V4L2
        let data = self.buffers.first()
            .cloned()
            .unwrap_or_else(|| vec![0u8; (self.format.width * self.format.height * 2) as usize]);
        
        Ok(Frame {
            width: self.format.width,
            height: self.format.height,
            format: self.format.pixel_format,
            data,
            timestamp: std::time::SystemTime::now(),
        })
    }
}

impl HardwareDevice for Camera {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Camera
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.configure_format()?;
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.ready
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.stop_streaming()?;
        self.file = None;
        Ok(())
    }
}

/// Video frame
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

impl Frame {
    /// Convert to grayscale
    pub fn to_grayscale(&self) -> Vec<u8> {
        match self.format {
            PixelFormat::GREY | PixelFormat::Y16 => self.data.clone(),
            PixelFormat::YUYV => {
                // Extract Y channel
                self.data.iter()
                    .step_by(2)
                    .cloned()
                    .collect()
            }
            _ => {
                // Placeholder for other formats
                vec![0; (self.width * self.height) as usize]
            }
        }
    }
    
    /// Calculate average brightness
    pub fn average_brightness(&self) -> f64 {
        let gray = self.to_grayscale();
        if gray.is_empty() {
            return 0.0;
        }
        gray.iter().map(|&v| v as f64).sum::<f64>() / gray.len() as f64
    }
    
    /// Detect motion between frames
    pub fn motion_difference(&self, other: &Frame) -> f64 {
        let gray1 = self.to_grayscale();
        let gray2 = other.to_grayscale();
        
        if gray1.len() != gray2.len() || gray1.is_empty() {
            return 0.0;
        }
        
        let diff: u64 = gray1.iter()
            .zip(gray2.iter())
            .map(|(&a, &b)| (a as i32 - b as i32).unsigned_abs() as u64)
            .sum();
        
        diff as f64 / gray1.len() as f64
    }
}

/// Thermal camera (FLIR, Seek, etc.)
pub struct ThermalCamera {
    camera: Camera,
    min_temp: f64,
    max_temp: f64,
}

impl ThermalCamera {
    /// Open thermal camera
    pub fn open(device: &str) -> Result<Self, HalError> {
        let format = VideoFormat {
            width: 160,
            height: 120,
            pixel_format: PixelFormat::Y16,
            fps: 9,
        };
        
        let camera = Camera::open(device, format)?;
        
        Ok(Self {
            camera,
            min_temp: -40.0,
            max_temp: 330.0,
        })
    }
    
    /// Set temperature range
    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min_temp = min;
        self.max_temp = max;
    }
    
    /// Capture thermal frame
    pub fn capture(&mut self) -> Result<ThermalFrame, HalError> {
        let frame = self.camera.capture_frame()?;
        
        // Convert Y16 to temperature values
        let temps: Vec<f64> = frame.data.chunks(2)
            .map(|chunk| {
                let raw = u16::from_le_bytes([chunk[0], chunk.get(1).copied().unwrap_or(0)]);
                self.raw_to_temperature(raw)
            })
            .collect();
        
        Ok(ThermalFrame {
            width: frame.width,
            height: frame.height,
            temperatures: temps,
            timestamp: frame.timestamp,
        })
    }
    
    /// Convert raw value to temperature
    fn raw_to_temperature(&self, raw: u16) -> f64 {
        // Linear mapping (actual conversion depends on camera model)
        let normalized = raw as f64 / 65535.0;
        self.min_temp + normalized * (self.max_temp - self.min_temp)
    }
}

impl HardwareDevice for ThermalCamera {
    fn name(&self) -> &str {
        self.camera.name()
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Camera
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.camera.init()
    }
    
    fn is_ready(&self) -> bool {
        self.camera.is_ready()
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.camera.close()
    }
}

/// Thermal frame with temperature data
#[derive(Debug, Clone)]
pub struct ThermalFrame {
    pub width: u32,
    pub height: u32,
    pub temperatures: Vec<f64>,
    pub timestamp: std::time::SystemTime,
}

impl ThermalFrame {
    /// Get min/max/avg temperature
    pub fn stats(&self) -> ThermalStats {
        if self.temperatures.is_empty() {
            return ThermalStats::default();
        }
        
        let min = self.temperatures.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.temperatures.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = self.temperatures.iter().sum();
        let avg = sum / self.temperatures.len() as f64;
        
        ThermalStats { min, max, avg }
    }
    
    /// Detect cold spots (potential paranormal indicators)
    pub fn detect_cold_spots(&self, threshold: f64) -> Vec<ColdSpot> {
        let stats = self.stats();
        let mut spots = Vec::new();
        
        for (i, &temp) in self.temperatures.iter().enumerate() {
            if temp < stats.avg - threshold {
                let x = (i as u32) % self.width;
                let y = (i as u32) / self.width;
                
                spots.push(ColdSpot {
                    x,
                    y,
                    temperature: temp,
                    deviation: stats.avg - temp,
                });
            }
        }
        
        spots
    }
    
    /// Calculate temperature at specific point
    pub fn temperature_at(&self, x: u32, y: u32) -> Option<f64> {
        let idx = (y * self.width + x) as usize;
        self.temperatures.get(idx).copied()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThermalStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Debug, Clone)]
pub struct ColdSpot {
    pub x: u32,
    pub y: u32,
    pub temperature: f64,
    pub deviation: f64,
}

/// Night vision camera (IR sensitive)
pub struct NightVisionCamera {
    camera: Camera,
    ir_led_enabled: bool,
}

impl NightVisionCamera {
    pub fn open(device: &str) -> Result<Self, HalError> {
        let format = VideoFormat {
            width: 1920,
            height: 1080,
            pixel_format: PixelFormat::YUYV,
            fps: 30,
        };
        
        let camera = Camera::open(device, format)?;
        
        Ok(Self {
            camera,
            ir_led_enabled: false,
        })
    }
    
    /// Enable IR illumination
    pub fn enable_ir(&mut self) -> Result<(), HalError> {
        // In production, this would control IR LED GPIO
        self.ir_led_enabled = true;
        Ok(())
    }
    
    /// Disable IR illumination
    pub fn disable_ir(&mut self) -> Result<(), HalError> {
        self.ir_led_enabled = false;
        Ok(())
    }
    
    /// Capture frame
    pub fn capture(&mut self) -> Result<Frame, HalError> {
        self.camera.capture_frame()
    }
    
    /// Detect light anomalies (orbs, etc.)
    pub fn detect_anomalies(&mut self, sensitivity: f64) -> Result<Vec<LightAnomaly>, HalError> {
        let frame = self.capture()?;
        let gray = frame.to_grayscale();
        
        let avg = gray.iter().map(|&v| v as f64).sum::<f64>() / gray.len() as f64;
        let threshold = avg + (255.0 - avg) * sensitivity;
        
        let mut anomalies = Vec::new();
        for (i, &pixel) in gray.iter().enumerate() {
            if pixel as f64 > threshold {
                let x = (i as u32) % frame.width;
                let y = (i as u32) / frame.width;
                
                anomalies.push(LightAnomaly {
                    x,
                    y,
                    intensity: pixel as f64 / 255.0,
                });
            }
        }
        
        Ok(anomalies)
    }
}

impl HardwareDevice for NightVisionCamera {
    fn name(&self) -> &str {
        self.camera.name()
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Camera
    }
    
    fn init(&mut self) -> Result<(), HalError> {
        self.camera.init()
    }
    
    fn is_ready(&self) -> bool {
        self.camera.is_ready()
    }
    
    fn close(&mut self) -> Result<(), HalError> {
        self.camera.close()
    }
}

#[derive(Debug, Clone)]
pub struct LightAnomaly {
    pub x: u32,
    pub y: u32,
    pub intensity: f64,
}

/// Enumerate available cameras
pub fn enumerate_cameras() -> Result<Vec<PathBuf>, HalError> {
    let mut cameras = Vec::new();
    
    for i in 0..10 {
        let path = PathBuf::from(format!("/dev/video{}", i));
        if path.exists() {
            cameras.push(path);
        }
    }
    
    Ok(cameras)
}
