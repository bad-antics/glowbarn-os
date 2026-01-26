//! EMF Scanner Example
//! 
//! Demonstrates continuous EMF monitoring using RTL-SDR

use glowbarn_hal::sdr::{RtlSdr, EmfAnalyzer, RadioScanner};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("╔══════════════════════════════════════╗");
    println!("║    GlowBarn EMF Spectrum Scanner     ║");
    println!("╚══════════════════════════════════════╝\n");
    
    // Initialize SDR
    let mut sdr = RtlSdr::open(0)?;
    sdr.init()?;
    
    println!("SDR initialized: {}", sdr.name());
    
    // Scan HF/VHF range for anomalies
    println!("\n[1] Scanning for RF signals...\n");
    sdr.set_sample_rate(2_000_000)?;
    
    let peaks = sdr.scan_range(
        88_000_000,   // 88 MHz (FM start)
        108_000_000,  // 108 MHz (FM end)
        500_000       // 500 kHz steps
    )?;
    
    println!("Found {} signal peaks:", peaks.len());
    for peak in &peaks {
        println!("  {:.1} MHz - Power: {:.2}", 
            peak.frequency as f64 / 1_000_000.0,
            peak.power);
    }
    
    // EMF anomaly detection
    println!("\n[2] EMF Anomaly Detection Mode...\n");
    
    let mut analyzer = EmfAnalyzer::new(0)?;
    analyzer.sdr.init()?;
    analyzer.sdr.set_frequency(100_000_000)?;  // 100 MHz center
    
    println!("Capturing baseline EMF signature...");
    analyzer.capture_baseline()?;
    println!("Baseline captured. Monitoring for anomalies...\n");
    
    let duration = Duration::from_secs(30);
    let start = std::time::Instant::now();
    let mut anomaly_count = 0;
    
    println!("Press Ctrl+C to stop\n");
    println!("Time    | Anomalies | Status");
    println!("--------|-----------|--------");
    
    while start.elapsed() < duration {
        let elapsed = start.elapsed().as_secs();
        
        match analyzer.detect_anomalies(3.0) {
            Ok(anomalies) if !anomalies.is_empty() => {
                anomaly_count += anomalies.len();
                for anomaly in &anomalies {
                    println!("{:>6}s | {:>9} | ⚠️  EMF SPIKE: {:.1}x baseline @ {:+.0} Hz",
                        elapsed, anomaly_count,
                        anomaly.power_ratio,
                        anomaly.frequency_offset);
                }
            }
            Ok(_) => {
                if elapsed % 5 == 0 {
                    println!("{:>6}s | {:>9} | Normal", elapsed, anomaly_count);
                }
            }
            Err(e) => {
                println!("{:>6}s | {:>9} | Error: {}", elapsed, anomaly_count, e);
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // Burst detection
    println!("\n[3] EMF Burst Detection...\n");
    
    let bursts = analyzer.monitor_bursts(5000)?;
    
    if bursts.is_empty() {
        println!("No EMF bursts detected in 5 second window");
    } else {
        println!("Detected {} EMF bursts:", bursts.len());
        for burst in &bursts {
            println!("  Power increase: {:.1}x, Absolute: {:.2}",
                burst.power_increase, burst.absolute_power);
        }
    }
    
    // Spirit Box mode (radio sweep)
    println!("\n[4] Spirit Box Mode (FM Sweep)...\n");
    
    let mut scanner = RadioScanner::new_fm(0)?;
    scanner.sdr.init()?;
    scanner.set_dwell_time(50);  // 50ms per frequency
    
    println!("Starting FM band sweep (88-108 MHz)...");
    println!("Listening for voice patterns in white noise...\n");
    
    let samples = scanner.sweep()?;
    
    // Find frequencies with unusual activity
    let avg_power: f64 = samples.iter().map(|s| s.power).sum::<f64>() / samples.len() as f64;
    
    println!("Frequencies with elevated activity:");
    for sample in samples.iter().filter(|s| s.power > avg_power * 2.0) {
        println!("  {:.1} MHz - Power: {:.2}", 
            sample.frequency as f64 / 1_000_000.0,
            sample.power);
    }
    
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Scan Complete                ║");
    println!("║  Total anomalies detected: {:>5}     ║", anomaly_count);
    println!("╚══════════════════════════════════════╝");
    
    Ok(())
}
