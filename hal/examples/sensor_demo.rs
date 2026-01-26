//! GlowBarn HAL Sensor Demo
//! 
//! Demonstrates usage of various sensors in the GlowBarn HAL.

use glowbarn_hal::{
    HardwareManager, HalConfig, HardwareDevice,
    i2c::{HMC5883L, BME280, MLX90614},
    gpio::PIRSensor,
    audio::InfrasoundDetector,
    camera::ThermalCamera,
    sdr::EmfAnalyzer,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("glowbarn_hal=info")
        .init();
    
    println!("=== GlowBarn HAL Sensor Demo ===\n");
    
    // Create hardware manager
    let config = HalConfig::default();
    let (mut manager, mut readings) = HardwareManager::new(config);
    
    // Initialize hardware
    println!("Initializing hardware...");
    manager.init().await?;
    
    // Demo: EMF Magnetometer (HMC5883L)
    println!("\n--- EMF Magnetometer (HMC5883L) ---");
    if let Ok(mut emf) = HMC5883L::new("/dev/i2c-1") {
        emf.init()?;
        if let Ok((x, y, z)) = emf.read_xyz() {
            println!("  Magnetic Field: X={:.2} mG, Y={:.2} mG, Z={:.2} mG", x, y, z);
            println!("  Magnitude: {:.2} mG", emf.read_magnitude()?);
        }
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: Environmental Sensor (BME280)
    println!("\n--- Environmental Sensor (BME280) ---");
    if let Ok(bme) = BME280::new("/dev/i2c-1") {
        if let Ok((temp, humidity, pressure)) = bme.read_all() {
            println!("  Temperature: {:.1}°C", temp);
            println!("  Humidity: {:.1}%", humidity);
            println!("  Pressure: {:.1} hPa", pressure);
        }
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: IR Temperature (MLX90614)
    println!("\n--- IR Temperature Sensor (MLX90614) ---");
    if let Ok(ir) = MLX90614::new("/dev/i2c-1") {
        if let Ok(ambient) = ir.read_ambient() {
            println!("  Ambient: {:.1}°C", ambient);
        }
        if let Ok(object) = ir.read_object() {
            println!("  Object: {:.1}°C", object);
        }
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: PIR Motion Sensor
    println!("\n--- PIR Motion Sensor ---");
    if let Ok(mut pir) = PIRSensor::new("PIR_Main", 17) {
        println!("  Monitoring for motion (5 seconds)...");
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if pir.check_motion()? {
                println!("  ! Motion detected!");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        println!("  Total motion events: {}", pir.motion_count());
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: Thermal Camera
    println!("\n--- Thermal Camera ---");
    if let Ok(mut thermal) = ThermalCamera::open("/dev/video0") {
        thermal.init()?;
        if let Ok(frame) = thermal.capture() {
            let stats = frame.stats();
            println!("  Min Temp: {:.1}°C", stats.min);
            println!("  Max Temp: {:.1}°C", stats.max);
            println!("  Avg Temp: {:.1}°C", stats.avg);
            
            let cold_spots = frame.detect_cold_spots(5.0);
            if !cold_spots.is_empty() {
                println!("  Cold spots detected: {}", cold_spots.len());
            }
        }
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: SDR EMF Analyzer
    println!("\n--- SDR EMF Spectrum Analyzer ---");
    if let Ok(mut analyzer) = EmfAnalyzer::new(0) {
        analyzer.sdr.init()?;
        println!("  Capturing EMF baseline...");
        analyzer.capture_baseline()?;
        
        println!("  Monitoring for EMF anomalies (5 seconds)...");
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Ok(anomalies) = analyzer.detect_anomalies(3.0) {
                for anomaly in anomalies {
                    println!("  ! EMF anomaly: +{:.0} Hz offset, {:.1}x power",
                        anomaly.frequency_offset, anomaly.power_ratio);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    } else {
        println!("  [Not connected]");
    }
    
    // Demo: Infrasound Detector
    println!("\n--- Infrasound Detector ---");
    if let Ok(infra) = InfrasoundDetector::new("plughw:0,0", -40.0) {
        println!("  Monitoring for infrasound (0-20 Hz)...");
        // In production, this would read actual audio samples
        println!("  [Monitoring active]");
    } else {
        println!("  [Not connected]");
    }
    
    println!("\n=== Demo Complete ===");
    Ok(())
}
