//! GlowBarn Sensor Fusion Library
//!
//! Combines multiple sensor inputs to detect paranormal activity
//! with statistical confidence scoring.

pub mod fusion;
pub mod anomaly;
pub mod recording;
pub mod triggers;

use glowbarn_hal::{SensorReading, HalError};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

/// Paranormal event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Electromagnetic field anomaly
    EmfAnomaly,
    /// Temperature anomaly (cold/hot spot)
    TemperatureAnomaly,
    /// Audio anomaly (EVP candidate)
    AudioAnomaly,
    /// Visual anomaly (light/shadow)
    VisualAnomaly,
    /// Motion detected
    MotionDetected,
    /// Infrasound detected
    InfrasoundDetected,
    /// Combined multi-sensor event
    MultiSensorEvent,
    /// Radio frequency anomaly
    RfAnomaly,
}

/// Confidence level for detected events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Confidence {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => Confidence::VeryHigh,
            s if s >= 0.7 => Confidence::High,
            s if s >= 0.5 => Confidence::Medium,
            _ => Confidence::Low,
        }
    }
    
    pub fn to_score(&self) -> f64 {
        match self {
            Confidence::VeryHigh => 0.95,
            Confidence::High => 0.80,
            Confidence::Medium => 0.60,
            Confidence::Low => 0.30,
        }
    }
}

/// Detected paranormal event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParanormalEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Detection timestamp
    pub timestamp: SystemTime,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Confidence level
    pub confidence_level: Confidence,
    /// Raw sensor readings that triggered this event
    pub sensor_data: Vec<SensorSnapshot>,
    /// Location (if available)
    pub location: Option<Location>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ParanormalEvent {
    /// Create new event
    pub fn new(event_type: EventType, confidence: f64) -> Self {
        let id = format!("evt_{}", chrono::Utc::now().timestamp_millis());
        
        Self {
            id,
            event_type,
            timestamp: SystemTime::now(),
            confidence,
            confidence_level: Confidence::from_score(confidence),
            sensor_data: Vec::new(),
            location: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add sensor snapshot
    pub fn with_sensor_data(mut self, data: SensorSnapshot) -> Self {
        self.sensor_data.push(data);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set location
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }
}

/// Snapshot of sensor reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSnapshot {
    pub sensor_name: String,
    pub sensor_type: String,
    pub value: f64,
    pub unit: String,
    pub baseline: Option<f64>,
    pub deviation: Option<f64>,
}

impl From<SensorReading> for SensorSnapshot {
    fn from(reading: SensorReading) -> Self {
        Self {
            sensor_name: reading.sensor_name,
            sensor_type: "unknown".to_string(),
            value: reading.value,
            unit: reading.unit,
            baseline: None,
            deviation: None,
        }
    }
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub zone: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub floor: Option<i32>,
}

/// Sensor status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStatus {
    pub name: String,
    pub connected: bool,
    pub last_reading: Option<SystemTime>,
    pub error_count: u32,
    pub quality: f64,
}

/// System-wide event handler
pub trait EventHandler: Send + Sync {
    /// Called when a paranormal event is detected
    fn on_event(&self, event: &ParanormalEvent);
    
    /// Called when a sensor goes offline
    fn on_sensor_offline(&self, sensor_name: &str);
    
    /// Called when a sensor comes online
    fn on_sensor_online(&self, sensor_name: &str);
}

/// Simple logging event handler
pub struct LoggingEventHandler;

impl EventHandler for LoggingEventHandler {
    fn on_event(&self, event: &ParanormalEvent) {
        tracing::info!(
            event_type = ?event.event_type,
            confidence = event.confidence,
            "Paranormal event detected: {:?} (confidence: {:.1}%)",
            event.event_type,
            event.confidence * 100.0
        );
    }
    
    fn on_sensor_offline(&self, sensor_name: &str) {
        tracing::warn!("Sensor offline: {}", sensor_name);
    }
    
    fn on_sensor_online(&self, sensor_name: &str) {
        tracing::info!("Sensor online: {}", sensor_name);
    }
}

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    #[error("HAL error: {0}")]
    Hal(#[from] HalError),
    
    #[error("Calibration required for sensor: {0}")]
    CalibrationRequired(String),
    
    #[error("Sensor not found: {0}")]
    SensorNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Recording error: {0}")]
    Recording(String),
}

pub type Result<T> = std::result::Result<T, SensorError>;
