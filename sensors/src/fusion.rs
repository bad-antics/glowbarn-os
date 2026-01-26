//! Sensor Fusion Engine
//!
//! Combines multiple sensor inputs using statistical methods
//! to improve detection accuracy and reduce false positives.

use crate::{EventType, ParanormalEvent, SensorSnapshot, Result};
use glowbarn_hal::SensorReading;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

/// Baseline statistics for a sensor
#[derive(Debug, Clone)]
pub struct SensorBaseline {
    pub name: String,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: usize,
    pub last_calibration: SystemTime,
}

impl SensorBaseline {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mean: 0.0,
            std_dev: 1.0,
            min: f64::MAX,
            max: f64::MIN,
            sample_count: 0,
            last_calibration: SystemTime::now(),
        }
    }
    
    /// Update baseline with new sample
    pub fn update(&mut self, value: f64) {
        self.sample_count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        
        // Welford's online algorithm for mean and variance
        let delta = value - self.mean;
        self.mean += delta / self.sample_count as f64;
        let delta2 = value - self.mean;
        
        if self.sample_count > 1 {
            let m2 = (self.std_dev * self.std_dev) * (self.sample_count - 1) as f64;
            let new_m2 = m2 + delta * delta2;
            self.std_dev = (new_m2 / (self.sample_count - 1) as f64).sqrt();
        }
    }
    
    /// Calculate z-score for a value
    pub fn z_score(&self, value: f64) -> f64 {
        if self.std_dev == 0.0 {
            return 0.0;
        }
        (value - self.mean) / self.std_dev
    }
    
    /// Check if value is anomalous (beyond n standard deviations)
    pub fn is_anomalous(&self, value: f64, threshold: f64) -> bool {
        self.z_score(value).abs() > threshold
    }
}

/// Configuration for fusion engine
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// Z-score threshold for anomaly detection
    pub anomaly_threshold: f64,
    /// Minimum samples before baseline is valid
    pub min_baseline_samples: usize,
    /// Time window for correlated events (ms)
    pub correlation_window_ms: u64,
    /// Minimum confidence for event reporting
    pub min_confidence: f64,
    /// Weight factors for different sensor types
    pub sensor_weights: HashMap<String, f64>,
}

impl Default for FusionConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("emf".to_string(), 1.5);
        weights.insert("temperature".to_string(), 1.2);
        weights.insert("audio".to_string(), 1.0);
        weights.insert("motion".to_string(), 0.8);
        weights.insert("infrared".to_string(), 1.3);
        
        Self {
            anomaly_threshold: 2.5,  // 2.5 standard deviations
            min_baseline_samples: 100,
            correlation_window_ms: 5000,  // 5 second window
            min_confidence: 0.4,
            sensor_weights: weights,
        }
    }
}

/// Sensor Fusion Engine
pub struct FusionEngine {
    config: FusionConfig,
    baselines: Arc<RwLock<HashMap<String, SensorBaseline>>>,
    recent_readings: Arc<RwLock<Vec<(SystemTime, SensorReading)>>>,
    event_tx: mpsc::Sender<ParanormalEvent>,
}

impl FusionEngine {
    /// Create new fusion engine
    pub fn new(config: FusionConfig) -> (Self, mpsc::Receiver<ParanormalEvent>) {
        let (tx, rx) = mpsc::channel(100);
        
        (Self {
            config,
            baselines: Arc::new(RwLock::new(HashMap::new())),
            recent_readings: Arc::new(RwLock::new(Vec::new())),
            event_tx: tx,
        }, rx)
    }
    
    /// Process incoming sensor reading
    pub async fn process_reading(&self, reading: SensorReading) -> Result<Option<ParanormalEvent>> {
        let now = SystemTime::now();
        
        // Store reading for correlation analysis
        {
            let mut recent = self.recent_readings.write().unwrap();
            recent.push((now, reading.clone()));
            
            // Prune old readings
            let cutoff = now - Duration::from_millis(self.config.correlation_window_ms * 2);
            recent.retain(|(t, _)| *t > cutoff);
        }
        
        // Update baseline
        let is_baseline_valid = {
            let mut baselines = self.baselines.write().unwrap();
            let baseline = baselines
                .entry(reading.sensor_name.clone())
                .or_insert_with(|| SensorBaseline::new(&reading.sensor_name));
            
            baseline.update(reading.value);
            baseline.sample_count >= self.config.min_baseline_samples
        };
        
        // Skip anomaly detection during baseline collection
        if !is_baseline_valid {
            tracing::debug!(
                "Collecting baseline for {}: {}/{}",
                reading.sensor_name,
                self.baselines.read().unwrap()[&reading.sensor_name].sample_count,
                self.config.min_baseline_samples
            );
            return Ok(None);
        }
        
        // Check for anomaly
        let (z_score, baseline) = {
            let baselines = self.baselines.read().unwrap();
            let baseline = &baselines[&reading.sensor_name];
            (baseline.z_score(reading.value), baseline.clone())
        };
        
        if z_score.abs() <= self.config.anomaly_threshold {
            return Ok(None);
        }
        
        // Anomaly detected - calculate confidence
        let base_confidence = self.calculate_confidence(z_score);
        
        // Check for correlated events
        let correlated = self.find_correlated_anomalies(&reading.sensor_name, now);
        let correlation_boost = correlated.len() as f64 * 0.1;
        
        let final_confidence = (base_confidence + correlation_boost).min(0.99);
        
        if final_confidence < self.config.min_confidence {
            return Ok(None);
        }
        
        // Determine event type
        let event_type = self.classify_event(&reading, &correlated);
        
        // Create event
        let mut event = ParanormalEvent::new(event_type, final_confidence)
            .with_sensor_data(SensorSnapshot {
                sensor_name: reading.sensor_name.clone(),
                sensor_type: self.get_sensor_type(&reading.sensor_name),
                value: reading.value,
                unit: reading.unit,
                baseline: Some(baseline.mean),
                deviation: Some(z_score),
            })
            .with_metadata("z_score", &format!("{:.2}", z_score))
            .with_metadata("correlated_sensors", &format!("{}", correlated.len()));
        
        // Add correlated sensor data
        for (_, corr_reading) in correlated {
            let corr_baselines = self.baselines.read().unwrap();
            if let Some(corr_baseline) = corr_baselines.get(&corr_reading.sensor_name) {
                event = event.with_sensor_data(SensorSnapshot {
                    sensor_name: corr_reading.sensor_name.clone(),
                    sensor_type: self.get_sensor_type(&corr_reading.sensor_name),
                    value: corr_reading.value,
                    unit: corr_reading.unit,
                    baseline: Some(corr_baseline.mean),
                    deviation: Some(corr_baseline.z_score(corr_reading.value)),
                });
            }
        }
        
        // Send event
        let _ = self.event_tx.send(event.clone()).await;
        
        Ok(Some(event))
    }
    
    /// Calculate confidence from z-score
    fn calculate_confidence(&self, z_score: f64) -> f64 {
        // Sigmoid-like mapping from z-score to confidence
        let abs_z = z_score.abs();
        let base = 1.0 - (-0.5 * (abs_z - self.config.anomaly_threshold)).exp();
        base.clamp(0.0, 0.95)
    }
    
    /// Find correlated anomalies in time window
    fn find_correlated_anomalies(&self, exclude_sensor: &str, time: SystemTime) -> Vec<(SystemTime, SensorReading)> {
        let window = Duration::from_millis(self.config.correlation_window_ms);
        let baselines = self.baselines.read().unwrap();
        let recent = self.recent_readings.read().unwrap();
        
        recent.iter()
            .filter(|(t, r)| {
                r.sensor_name != exclude_sensor &&
                time.duration_since(*t).unwrap_or(Duration::MAX) < window
            })
            .filter(|(_, r)| {
                if let Some(baseline) = baselines.get(&r.sensor_name) {
                    baseline.is_anomalous(r.value, self.config.anomaly_threshold * 0.8)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Classify event type based on sensor data
    fn classify_event(&self, primary: &SensorReading, correlated: &[(SystemTime, SensorReading)]) -> EventType {
        let sensor_type = self.get_sensor_type(&primary.sensor_name);
        
        // Check for multi-sensor event
        if correlated.len() >= 2 {
            return EventType::MultiSensorEvent;
        }
        
        // Single sensor classification
        match sensor_type.as_str() {
            "emf" | "magnetometer" => EventType::EmfAnomaly,
            "temperature" | "ir_temperature" | "thermal" => EventType::TemperatureAnomaly,
            "audio" | "microphone" => EventType::AudioAnomaly,
            "camera" | "ir_camera" => EventType::VisualAnomaly,
            "pir" | "motion" | "laser" => EventType::MotionDetected,
            "infrasound" => EventType::InfrasoundDetected,
            "sdr" | "rf" | "radio" => EventType::RfAnomaly,
            _ => EventType::EmfAnomaly,
        }
    }
    
    /// Get sensor type from name
    fn get_sensor_type(&self, name: &str) -> String {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("emf") || name_lower.contains("mag") || name_lower.contains("hmc") {
            "emf".to_string()
        } else if name_lower.contains("temp") || name_lower.contains("mlx") || name_lower.contains("bme") {
            "temperature".to_string()
        } else if name_lower.contains("audio") || name_lower.contains("mic") {
            "audio".to_string()
        } else if name_lower.contains("pir") || name_lower.contains("motion") {
            "motion".to_string()
        } else if name_lower.contains("camera") || name_lower.contains("video") {
            "camera".to_string()
        } else if name_lower.contains("sdr") || name_lower.contains("rtl") {
            "sdr".to_string()
        } else if name_lower.contains("infra") {
            "infrasound".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Get baseline for sensor
    pub fn get_baseline(&self, sensor_name: &str) -> Option<SensorBaseline> {
        self.baselines.read().unwrap().get(sensor_name).cloned()
    }
    
    /// Reset baseline for sensor
    pub fn reset_baseline(&self, sensor_name: &str) {
        let mut baselines = self.baselines.write().unwrap();
        if let Some(baseline) = baselines.get_mut(sensor_name) {
            *baseline = SensorBaseline::new(sensor_name);
        }
    }
    
    /// Reset all baselines
    pub fn reset_all_baselines(&self) {
        let mut baselines = self.baselines.write().unwrap();
        for (name, baseline) in baselines.iter_mut() {
            *baseline = SensorBaseline::new(name);
        }
    }
}
