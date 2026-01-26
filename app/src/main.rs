//! GlowBarn Paranormal Detection Suite
//!
//! Main application entry point for the GlowBarn system.

use anyhow::Result;
use glowbarn_hal::{HardwareManager, HalConfig};
use glowbarn_sensors::{
    fusion::{FusionEngine, FusionConfig},
    recording::EventRecorder,
    triggers::TriggerManager,
    EventHandler, LoggingEventHandler,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

mod config;

use config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();
    
    tracing::info!("╔══════════════════════════════════════════╗");
    tracing::info!("║   GlowBarn Paranormal Detection Suite    ║");
    tracing::info!("║            Version 0.1.0                 ║");
    tracing::info!("╚══════════════════════════════════════════╝");
    
    // Load configuration
    let config = AppConfig::load()?;
    tracing::info!("Configuration loaded from {:?}", config.config_path);
    
    // Initialize hardware abstraction layer
    tracing::info!("Initializing Hardware Abstraction Layer...");
    let hal_config = HalConfig {
        i2c_buses: config.i2c_buses.clone(),
        spi_devices: config.spi_devices.clone(),
        gpio_chip: config.gpio_chip.clone(),
        ..Default::default()
    };
    
    let (mut hardware_manager, sensor_rx) = HardwareManager::new(hal_config);
    hardware_manager.init().await?;
    tracing::info!("HAL initialized successfully");
    
    // Initialize sensor fusion engine
    tracing::info!("Initializing Sensor Fusion Engine...");
    let fusion_config = FusionConfig {
        anomaly_threshold: config.anomaly_threshold,
        min_baseline_samples: config.baseline_samples,
        correlation_window_ms: config.correlation_window_ms,
        min_confidence: config.min_confidence,
        ..Default::default()
    };
    
    let (fusion_engine, event_rx) = FusionEngine::new(fusion_config);
    let fusion_engine = Arc::new(RwLock::new(fusion_engine));
    tracing::info!("Fusion engine initialized");
    
    // Initialize event recorder
    tracing::info!("Initializing Event Recorder...");
    let data_dir = PathBuf::from(&config.data_directory);
    let mut recorder = EventRecorder::new(&data_dir)?;
    
    if config.auto_record {
        recorder.start_session(&config.session_name, &config.location)?;
    }
    let recorder = Arc::new(RwLock::new(recorder));
    tracing::info!("Event recorder ready");
    
    // Initialize trigger manager
    tracing::info!("Initializing Trigger Manager...");
    let trigger_manager = Arc::new(RwLock::new(TriggerManager::default()));
    tracing::info!("Trigger manager ready with {} triggers", 
        trigger_manager.read().await.list_triggers().len());
    
    // Start sensor polling
    tracing::info!("Starting sensor polling (interval: {:?})...", 
        Duration::from_millis(config.poll_interval_ms));
    hardware_manager.start_polling(Duration::from_millis(config.poll_interval_ms)).await;
    
    // Spawn sensor reading processor
    let fusion_clone = fusion_engine.clone();
    let sensor_task = tokio::spawn(async move {
        let mut rx = sensor_rx;
        while let Some(reading) = rx.recv().await {
            let engine = fusion_clone.read().await;
            if let Err(e) = engine.process_reading(reading).await {
                tracing::error!("Error processing reading: {}", e);
            }
        }
    });
    
    // Spawn event processor
    let recorder_clone = recorder.clone();
    let trigger_clone = trigger_manager.clone();
    let event_task = tokio::spawn(async move {
        let mut rx = event_rx;
        while let Some(event) = rx.recv().await {
            // Log event
            let handler = LoggingEventHandler;
            handler.on_event(&event);
            
            // Record event
            if let Err(e) = recorder_clone.write().await.record_event(&event) {
                tracing::error!("Error recording event: {}", e);
            }
            
            // Process triggers
            if let Err(e) = trigger_clone.write().await.process_event(event).await {
                tracing::error!("Error processing triggers: {}", e);
            }
        }
    });
    
    // Print system status
    print_system_status(&config).await;
    
    tracing::info!("GlowBarn is now monitoring for paranormal activity...");
    tracing::info!("Press Ctrl+C to stop");
    
    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
        _ = sensor_task => {
            tracing::warn!("Sensor task ended unexpectedly");
        }
        _ = event_task => {
            tracing::warn!("Event task ended unexpectedly");
        }
    }
    
    // Cleanup
    tracing::info!("Shutting down...");
    
    // End recording session
    if let Some(session) = recorder.write().await.end_session()? {
        tracing::info!("Recording session ended: {} events captured", session.event_count);
    }
    
    tracing::info!("GlowBarn shutdown complete");
    
    Ok(())
}

fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,glowbarn=debug,glowbarn_hal=debug,glowbarn_sensors=debug"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false))
        .init();
}

async fn print_system_status(config: &AppConfig) {
    use sysinfo::System;
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    tracing::info!("╭─────────────── System Status ───────────────╮");
    tracing::info!("│ Hostname: {:>32} │", System::host_name().unwrap_or_default());
    tracing::info!("│ OS: {:>38} │", System::name().unwrap_or_default());
    tracing::info!("│ Kernel: {:>34} │", System::kernel_version().unwrap_or_default());
    tracing::info!("│ CPU: {:>37} │", sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown"));
    tracing::info!("│ Memory: {:>26} MB / {} MB │", 
        sys.used_memory() / 1024 / 1024,
        sys.total_memory() / 1024 / 1024);
    tracing::info!("├──────────────── Configuration ────────────────┤");
    tracing::info!("│ Location: {:>32} │", config.location);
    tracing::info!("│ Session: {:>33} │", config.session_name);
    tracing::info!("│ Poll Interval: {:>23} ms │", config.poll_interval_ms);
    tracing::info!("│ Anomaly Threshold: {:>22} σ │", config.anomaly_threshold);
    tracing::info!("│ Min Confidence: {:>23}% │", (config.min_confidence * 100.0) as i32);
    tracing::info!("╰──────────────────────────────────────────────╯");
}
