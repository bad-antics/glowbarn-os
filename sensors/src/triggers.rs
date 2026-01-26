//! Event Triggers and Alerting
//!
//! Configurable triggers for automated responses to paranormal events.

use crate::{EventType, ParanormalEvent, Result};
use std::time::{Duration, SystemTime};
use std::pin::Pin;
use std::future::Future;

/// Trigger condition
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    /// Trigger on specific event type
    EventType(EventType),
    /// Trigger when confidence exceeds threshold
    ConfidenceAbove(f64),
    /// Trigger when multiple events occur in time window
    EventBurst { count: usize, window: Duration },
    /// Trigger on specific sensor anomaly
    SensorAnomaly { sensor_pattern: String, threshold: f64 },
    /// Compound condition (AND)
    All(Vec<TriggerCondition>),
    /// Compound condition (OR)
    Any(Vec<TriggerCondition>),
}

impl TriggerCondition {
    /// Check if condition is satisfied
    pub fn check(&self, event: &ParanormalEvent, history: &[ParanormalEvent]) -> bool {
        match self {
            TriggerCondition::EventType(et) => event.event_type == *et,
            
            TriggerCondition::ConfidenceAbove(threshold) => event.confidence > *threshold,
            
            TriggerCondition::EventBurst { count, window } => {
                let cutoff = event.timestamp - *window;
                let recent_count = history.iter()
                    .filter(|e| e.timestamp > cutoff)
                    .count() + 1;  // Include current event
                recent_count >= *count
            }
            
            TriggerCondition::SensorAnomaly { sensor_pattern, threshold } => {
                event.sensor_data.iter().any(|s| {
                    s.sensor_name.to_lowercase().contains(&sensor_pattern.to_lowercase()) &&
                    s.deviation.map(|d| d.abs() > *threshold).unwrap_or(false)
                })
            }
            
            TriggerCondition::All(conditions) => {
                conditions.iter().all(|c| c.check(event, history))
            }
            
            TriggerCondition::Any(conditions) => {
                conditions.iter().any(|c| c.check(event, history))
            }
        }
    }
}

/// Trigger action
#[derive(Debug, Clone)]
pub enum TriggerAction {
    /// Log message
    Log { level: String, message: String },
    /// Play sound
    PlaySound { file: String },
    /// Send notification
    Notify { title: String, body: String },
    /// Execute command
    Execute { command: String, args: Vec<String> },
    /// Control GPIO (for lights, alarms, etc.)
    GpioControl { pin: u32, state: bool },
    /// Start recording
    StartRecording { name: String },
    /// Mark timestamp
    MarkTimestamp { label: String },
    /// Multiple actions
    Multiple(Vec<TriggerAction>),
}

impl TriggerAction {
    /// Execute the action
    pub fn execute<'a>(&'a self, event: &'a ParanormalEvent) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                TriggerAction::Log { level, message } => {
                    let formatted = message
                        .replace("{event_type}", &format!("{:?}", event.event_type))
                        .replace("{confidence}", &format!("{:.1}%", event.confidence * 100.0))
                        .replace("{id}", &event.id);
                    
                    match level.as_str() {
                        "error" => tracing::error!("{}", formatted),
                        "warn" => tracing::warn!("{}", formatted),
                        "info" => tracing::info!("{}", formatted),
                        "debug" => tracing::debug!("{}", formatted),
                        _ => tracing::info!("{}", formatted),
                    }
                }
                
                TriggerAction::PlaySound { file } => {
                    // In production, this would use audio playback
                    tracing::info!("Playing sound: {}", file);
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("aplay")
                            .arg(file)
                            .spawn();
                    }
                }
                
                TriggerAction::Notify { title, body } => {
                    let formatted_body = body
                        .replace("{event_type}", &format!("{:?}", event.event_type))
                        .replace("{confidence}", &format!("{:.1}%", event.confidence * 100.0));
                    
                    tracing::info!("Notification: {} - {}", title, formatted_body);
                    
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("notify-send")
                            .arg(title)
                            .arg(&formatted_body)
                            .spawn();
                    }
                }
                
                TriggerAction::Execute { command, args } => {
                    tracing::info!("Executing: {} {:?}", command, args);
                    
                    let _ = std::process::Command::new(command)
                        .args(args)
                        .spawn();
                }
                
                TriggerAction::GpioControl { pin, state } => {
                    tracing::info!("GPIO {}: {}", pin, if *state { "HIGH" } else { "LOW" });
                    
                    // In production, this would use glowbarn-hal GPIO
                    let path = format!("/sys/class/gpio/gpio{}/value", pin);
                    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&path) {
                        use std::io::Write;
                        let _ = file.write_all(if *state { b"1" } else { b"0" });
                    }
                }
                
                TriggerAction::StartRecording { name } => {
                    tracing::info!("Start recording: {}", name);
                    // Signal to recording system
                }
                
                TriggerAction::MarkTimestamp { label } => {
                    let timestamp = chrono::Utc::now();
                    tracing::info!("Timestamp marked: {} at {}", label, timestamp);
                }
                
                TriggerAction::Multiple(actions) => {
                    for action in actions {
                        action.execute(event).await?;
                    }
                }
            }
            
            Ok(())
        })
    }
}

/// Event trigger
#[derive(Debug, Clone)]
pub struct Trigger {
    pub name: String,
    pub enabled: bool,
    pub condition: TriggerCondition,
    pub action: TriggerAction,
    pub cooldown: Duration,
    last_triggered: Option<SystemTime>,
}

impl Trigger {
    /// Create new trigger
    pub fn new(name: &str, condition: TriggerCondition, action: TriggerAction) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            condition,
            action,
            cooldown: Duration::from_secs(5),
            last_triggered: None,
        }
    }
    
    /// Set cooldown period
    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }
    
    /// Check and execute trigger
    pub async fn check_and_execute(&mut self, event: &ParanormalEvent, history: &[ParanormalEvent]) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }
        
        // Check cooldown
        if let Some(last) = self.last_triggered {
            if let Ok(elapsed) = event.timestamp.duration_since(last) {
                if elapsed < self.cooldown {
                    return Ok(false);
                }
            }
        }
        
        // Check condition
        if !self.condition.check(event, history) {
            return Ok(false);
        }
        
        // Execute action
        tracing::info!("Trigger activated: {}", self.name);
        self.action.execute(event).await?;
        self.last_triggered = Some(event.timestamp);
        
        Ok(true)
    }
}

/// Trigger manager
pub struct TriggerManager {
    triggers: Vec<Trigger>,
    event_history: Vec<ParanormalEvent>,
    history_limit: usize,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            event_history: Vec::new(),
            history_limit: 1000,
        }
    }
    
    /// Add trigger
    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger);
    }
    
    /// Remove trigger by name
    pub fn remove_trigger(&mut self, name: &str) {
        self.triggers.retain(|t| t.name != name);
    }
    
    /// Enable/disable trigger
    pub fn set_trigger_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(trigger) = self.triggers.iter_mut().find(|t| t.name == name) {
            trigger.enabled = enabled;
        }
    }
    
    /// Process event through all triggers
    pub async fn process_event(&mut self, event: ParanormalEvent) -> Result<Vec<String>> {
        let mut triggered = Vec::new();
        
        for trigger in &mut self.triggers {
            if trigger.check_and_execute(&event, &self.event_history).await? {
                triggered.push(trigger.name.clone());
            }
        }
        
        // Add to history
        self.event_history.push(event);
        
        // Trim history
        while self.event_history.len() > self.history_limit {
            self.event_history.remove(0);
        }
        
        Ok(triggered)
    }
    
    /// List all triggers
    pub fn list_triggers(&self) -> Vec<&Trigger> {
        self.triggers.iter().collect()
    }
    
    /// Load default triggers
    pub fn load_defaults(&mut self) {
        // High confidence EMF alert
        self.add_trigger(Trigger::new(
            "high_emf_alert",
            TriggerCondition::All(vec![
                TriggerCondition::EventType(EventType::EmfAnomaly),
                TriggerCondition::ConfidenceAbove(0.8),
            ]),
            TriggerAction::Multiple(vec![
                TriggerAction::Log {
                    level: "warn".to_string(),
                    message: "High EMF anomaly detected! {confidence}".to_string(),
                },
                TriggerAction::PlaySound {
                    file: "/usr/share/glowbarn/sounds/alert.wav".to_string(),
                },
            ]),
        ));
        
        // Temperature anomaly alert
        self.add_trigger(Trigger::new(
            "cold_spot_alert",
            TriggerCondition::All(vec![
                TriggerCondition::EventType(EventType::TemperatureAnomaly),
                TriggerCondition::SensorAnomaly {
                    sensor_pattern: "temp".to_string(),
                    threshold: 3.0,
                },
            ]),
            TriggerAction::Notify {
                title: "Cold Spot Detected".to_string(),
                body: "Temperature anomaly: {confidence} confidence".to_string(),
            },
        ).with_cooldown(Duration::from_secs(30)));
        
        // Multi-sensor event alert
        self.add_trigger(Trigger::new(
            "multi_sensor_alert",
            TriggerCondition::All(vec![
                TriggerCondition::EventType(EventType::MultiSensorEvent),
                TriggerCondition::ConfidenceAbove(0.7),
            ]),
            TriggerAction::Multiple(vec![
                TriggerAction::Log {
                    level: "warn".to_string(),
                    message: "Multi-sensor event! ID: {id}".to_string(),
                },
                TriggerAction::MarkTimestamp {
                    label: "multi_sensor".to_string(),
                },
            ]),
        ));
        
        // Event burst detection
        self.add_trigger(Trigger::new(
            "activity_burst",
            TriggerCondition::EventBurst {
                count: 5,
                window: Duration::from_secs(60),
            },
            TriggerAction::Multiple(vec![
                TriggerAction::Notify {
                    title: "Activity Burst".to_string(),
                    body: "High paranormal activity detected!".to_string(),
                },
                TriggerAction::StartRecording {
                    name: "burst_recording".to_string(),
                },
            ]),
        ).with_cooldown(Duration::from_secs(120)));
        
        tracing::info!("Loaded {} default triggers", self.triggers.len());
    }
}

impl Default for TriggerManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.load_defaults();
        manager
    }
}
