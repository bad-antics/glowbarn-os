//! GlowBarn CLI Tool
//!
//! Command-line interface for managing GlowBarn sessions and data.

use anyhow::Result;
use clap::{Parser, Subcommand};
use glowbarn_sensors::recording::EventRecorder;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "glowbarn-cli")]
#[command(author = "GlowBarn Team")]
#[command(version = "0.1.0")]
#[command(about = "GlowBarn Paranormal Detection Suite CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Data directory
    #[arg(short, long, default_value = "/var/lib/glowbarn/data")]
    data_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// List recording sessions
    Sessions {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show events from a session
    Events {
        /// Session ID
        session_id: String,
        
        /// Filter by event type
        #[arg(short = 't', long)]
        event_type: Option<String>,
        
        /// Minimum confidence threshold
        #[arg(short, long)]
        min_confidence: Option<f64>,
        
        /// Output format (json, table)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Export session data
    Export {
        /// Session ID
        session_id: String,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Show sensor status
    Sensors,
    
    /// Generate sample configuration
    Config {
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// System information
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Sessions { verbose } => {
            list_sessions(&cli.data_dir, verbose)?;
        }
        
        Commands::Events { session_id, event_type, min_confidence, format } => {
            show_events(&cli.data_dir, &session_id, event_type, min_confidence, &format)?;
        }
        
        Commands::Export { session_id, output } => {
            export_session(&cli.data_dir, &session_id, &output)?;
        }
        
        Commands::Sensors => {
            show_sensors()?;
        }
        
        Commands::Config { output } => {
            generate_config(output)?;
        }
        
        Commands::Info => {
            show_info()?;
        }
    }
    
    Ok(())
}

fn list_sessions(data_dir: &PathBuf, verbose: bool) -> Result<()> {
    let recorder = EventRecorder::new(data_dir)?;
    let sessions = recorder.list_sessions()?;
    
    if sessions.is_empty() {
        println!("No recording sessions found.");
        return Ok(());
    }
    
    println!("╭────────────────────────────────────────────────────────────────────╮");
    println!("│                      Recording Sessions                             │");
    println!("├────────────────────┬──────────────────────┬────────────┬───────────┤");
    println!("│ Session ID         │ Name                 │ Events     │ Duration  │");
    println!("├────────────────────┼──────────────────────┼────────────┼───────────┤");
    
    for session in &sessions {
        let duration = session.duration();
        let duration_str = format!("{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60);
        
        println!("│ {:18} │ {:20} │ {:>10} │ {:>9} │",
            truncate(&session.id, 18),
            truncate(&session.name, 20),
            session.event_count,
            duration_str);
    }
    
    println!("╰────────────────────┴──────────────────────┴────────────┴───────────╯");
    
    if verbose {
        for session in &sessions {
            println!("\n{}", "─".repeat(60));
            println!("Session: {}", session.id);
            println!("  Name: {}", session.name);
            println!("  Location: {}", session.location);
            println!("  Start: {}", session.start_time);
            if let Some(end) = session.end_time {
                println!("  End: {}", end);
            }
            println!("  Events: {}", session.event_count);
            
            if !session.notes.is_empty() {
                println!("  Notes:");
                for note in &session.notes {
                    println!("    - {}", note);
                }
            }
        }
    }
    
    Ok(())
}

fn show_events(data_dir: &PathBuf, session_id: &str, event_type: Option<String>, 
               min_confidence: Option<f64>, format: &str) -> Result<()> {
    let recorder = EventRecorder::new(data_dir)?;
    let mut events = recorder.load_events(session_id)?;
    
    // Apply filters
    if let Some(ref et) = event_type {
        events.retain(|e| format!("{:?}", e.event_type).to_lowercase().contains(&et.to_lowercase()));
    }
    
    if let Some(min_conf) = min_confidence {
        events.retain(|e| e.confidence >= min_conf);
    }
    
    if events.is_empty() {
        println!("No events found matching criteria.");
        return Ok(());
    }
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&events)?;
            println!("{}", json);
        }
        _ => {
            println!("╭─────────────────────────────────────────────────────────────────────────╮");
            println!("│                           Event Log                                     │");
            println!("├────────────────────┬──────────────────────┬──────────────┬─────────────┤");
            println!("│ Time               │ Event Type           │ Confidence   │ Sensors     │");
            println!("├────────────────────┼──────────────────────┼──────────────┼─────────────┤");
            
            for event in &events {
                let time = chrono::DateTime::<chrono::Utc>::from(event.timestamp);
                let time_str = time.format("%H:%M:%S%.3f").to_string();
                
                println!("│ {:18} │ {:20} │ {:>10.1}% │ {:>11} │",
                    time_str,
                    format!("{:?}", event.event_type),
                    event.confidence * 100.0,
                    event.sensor_data.len());
            }
            
            println!("╰────────────────────┴──────────────────────┴──────────────┴─────────────╯");
            println!("\nTotal events: {}", events.len());
        }
    }
    
    Ok(())
}

fn export_session(data_dir: &PathBuf, session_id: &str, output: &PathBuf) -> Result<()> {
    let recorder = EventRecorder::new(data_dir)?;
    recorder.export_session(session_id, output)?;
    println!("Session exported to: {:?}", output);
    Ok(())
}

fn show_sensors() -> Result<()> {
    use glowbarn_hal::{i2c, usb, camera};
    
    println!("╭──────────────────────────────────────────────────────────────╮");
    println!("│                     Sensor Status                            │");
    println!("╰──────────────────────────────────────────────────────────────╯\n");
    
    // I2C devices
    println!("I2C Devices:");
    for bus in ["/dev/i2c-0", "/dev/i2c-1", "/dev/i2c-2"] {
        if std::path::Path::new(bus).exists() {
            print!("  {}: ", bus);
            match i2c::scan_bus(bus) {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("No devices found");
                    } else {
                        println!("{}", devices.iter()
                            .map(|d| format!("0x{:02X}", d))
                            .collect::<Vec<_>>()
                            .join(", "));
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }
    
    // USB devices
    println!("\nUSB Devices:");
    match usb::enumerate_devices() {
        Ok(devices) => {
            for device in devices {
                println!("  {:04X}:{:04X} - {} {}",
                    device.vendor_id, device.product_id,
                    device.manufacturer, device.product);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    
    // Cameras
    println!("\nCameras:");
    match camera::enumerate_cameras() {
        Ok(cameras) => {
            for cam in cameras {
                println!("  {:?}", cam);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    
    Ok(())
}

fn generate_config(output: Option<PathBuf>) -> Result<()> {
    let example = r#"# GlowBarn Configuration File
# 
# Copy this file to /etc/glowbarn/config.toml or ~/.config/glowbarn/config.toml

# Location name for recordings
location = "My Investigation Site"

# Default session name (auto-generated if not set)
session_name = "investigation_001"

# Data directory for recordings
data_directory = "/var/lib/glowbarn/data"

# Auto-start recording on launch
auto_record = true

# I2C bus paths
i2c_buses = ["/dev/i2c-1"]

# SPI device paths  
spi_devices = ["/dev/spidev0.0"]

# GPIO chip path
gpio_chip = "/dev/gpiochip0"

# Sensor poll interval in milliseconds
poll_interval_ms = 100

# Anomaly detection threshold (standard deviations)
anomaly_threshold = 2.5

# Minimum samples for baseline calibration
baseline_samples = 100

# Correlation window in milliseconds
correlation_window_ms = 5000

# Minimum confidence for reporting events (0.0 - 1.0)
min_confidence = 0.4
"#;
    
    if let Some(path) = output {
        std::fs::write(&path, example)?;
        println!("Configuration written to: {:?}", path);
    } else {
        println!("{}", example);
    }
    
    Ok(())
}

fn show_info() -> Result<()> {
    use sysinfo::System;
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    println!("╭──────────────────────────────────────────────────────────────╮");
    println!("│             GlowBarn System Information                      │");
    println!("╰──────────────────────────────────────────────────────────────╯\n");
    
    println!("System:");
    println!("  Hostname: {}", System::host_name().unwrap_or_default());
    println!("  OS: {} {}", 
        System::name().unwrap_or_default(),
        System::os_version().unwrap_or_default());
    println!("  Kernel: {}", System::kernel_version().unwrap_or_default());
    
    println!("\nHardware:");
    println!("  CPU: {}", sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown"));
    println!("  Cores: {}", sys.cpus().len());
    println!("  Memory: {} MB total, {} MB used",
        sys.total_memory() / 1024 / 1024,
        sys.used_memory() / 1024 / 1024);
    
    println!("\nGlowBarn:");
    println!("  Version: 0.1.0");
    println!("  HAL Version: 0.1.0");
    println!("  Sensors Version: 0.1.0");
    
    // Check for hardware
    println!("\nHardware Availability:");
    println!("  I2C: {}", if std::path::Path::new("/dev/i2c-1").exists() { "✓" } else { "✗" });
    println!("  SPI: {}", if std::path::Path::new("/dev/spidev0.0").exists() { "✓" } else { "✗" });
    println!("  GPIO: {}", if std::path::Path::new("/dev/gpiochip0").exists() { "✓" } else { "✗" });
    println!("  Camera: {}", if std::path::Path::new("/dev/video0").exists() { "✓" } else { "✗" });
    
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max-3])
    }
}
