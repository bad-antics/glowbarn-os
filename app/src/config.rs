// Application Configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Location name for recordings
    #[serde(default = "default_location")]
    pub location: String,
    
    /// Default session name
    #[serde(default = "default_session")]
    pub session_name: String,
    
    /// Data directory for recordings
    #[serde(default = "default_data_dir")]
    pub data_directory: String,
    
    /// Auto-start recording on launch
    #[serde(default)]
    pub auto_record: bool,
    
    /// I2C bus paths
    #[serde(default = "default_i2c")]
    pub i2c_buses: Vec<String>,
    
    /// SPI device paths
    #[serde(default = "default_spi")]
    pub spi_devices: Vec<String>,
    
    /// GPIO chip path
    #[serde(default = "default_gpio")]
    pub gpio_chip: String,
    
    /// Sensor poll interval in milliseconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    
    /// Anomaly detection threshold (standard deviations)
    #[serde(default = "default_anomaly_threshold")]
    pub anomaly_threshold: f64,
    
    /// Minimum samples for baseline
    #[serde(default = "default_baseline_samples")]
    pub baseline_samples: usize,
    
    /// Correlation window in milliseconds
    #[serde(default = "default_correlation_window")]
    pub correlation_window_ms: u64,
    
    /// Minimum confidence for reporting events
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
    
    /// Path to config file (for reference)
    #[serde(skip)]
    pub config_path: PathBuf,
}

fn default_location() -> String { "Unknown Location".to_string() }
fn default_session() -> String { format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")) }
fn default_data_dir() -> String { "/var/lib/glowbarn/data".to_string() }
fn default_i2c() -> Vec<String> { vec!["/dev/i2c-1".to_string()] }
fn default_spi() -> Vec<String> { vec!["/dev/spidev0.0".to_string()] }
fn default_gpio() -> String { "/dev/gpiochip0".to_string() }
fn default_poll_interval() -> u64 { 100 }
fn default_anomaly_threshold() -> f64 { 2.5 }
fn default_baseline_samples() -> usize { 100 }
fn default_correlation_window() -> u64 { 5000 }
fn default_min_confidence() -> f64 { 0.4 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            location: default_location(),
            session_name: default_session(),
            data_directory: default_data_dir(),
            auto_record: false,
            i2c_buses: default_i2c(),
            spi_devices: default_spi(),
            gpio_chip: default_gpio(),
            poll_interval_ms: default_poll_interval(),
            anomaly_threshold: default_anomaly_threshold(),
            baseline_samples: default_baseline_samples(),
            correlation_window_ms: default_correlation_window(),
            min_confidence: default_min_confidence(),
            config_path: PathBuf::new(),
        }
    }
}

impl AppConfig {
    /// Load configuration from standard paths
    pub fn load() -> Result<Self> {
        let config_paths = [
            PathBuf::from("/etc/glowbarn/config.toml"),
            dirs::config_dir()
                .map(|p| p.join("glowbarn/config.toml"))
                .unwrap_or_default(),
            PathBuf::from("./config.toml"),
        ];
        
        for path in &config_paths {
            if path.exists() {
                return Self::load_from(path);
            }
        }
        
        // Return default config
        tracing::warn!("No configuration file found, using defaults");
        Ok(Self::default())
    }
    
    /// Load configuration from specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        config.config_path = path.clone();
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Generate example configuration
    pub fn example() -> String {
        let config = Self {
            location: "My Investigation Site".to_string(),
            session_name: "investigation_001".to_string(),
            data_directory: "/var/lib/glowbarn/data".to_string(),
            auto_record: true,
            ..Default::default()
        };
        
        toml::to_string_pretty(&config).unwrap_or_default()
    }
}

/// Helper for getting config directories
mod dirs {
    use std::path::PathBuf;
    
    pub fn config_dir() -> Option<PathBuf> {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    }
}
