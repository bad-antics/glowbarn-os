//! Event Recording and Logging
//!
//! Persistent storage for paranormal events and sensor data.

use crate::{ParanormalEvent, SensorSnapshot, Result, SensorError};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub name: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_count: usize,
    pub notes: Vec<String>,
}

impl RecordingSession {
    pub fn new(name: &str, location: &str) -> Self {
        let id = format!("session_{}", Utc::now().timestamp());
        
        Self {
            id,
            name: name.to_string(),
            location: location.to_string(),
            start_time: Utc::now(),
            end_time: None,
            event_count: 0,
            notes: Vec::new(),
        }
    }
    
    pub fn add_note(&mut self, note: &str) {
        self.notes.push(format!("[{}] {}", Utc::now().format("%H:%M:%S"), note));
    }
    
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }
    
    pub fn duration(&self) -> chrono::Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end - self.start_time
    }
}

/// Event recorder
pub struct EventRecorder {
    base_path: PathBuf,
    session: Option<RecordingSession>,
    event_writer: Option<BufWriter<File>>,
    sensor_writer: Option<BufWriter<File>>,
    max_file_size: usize,
}

impl EventRecorder {
    /// Create new recorder
    pub fn new(base_path: &Path) -> Result<Self> {
        create_dir_all(base_path)
            .map_err(|e| SensorError::Recording(format!("Failed to create directory: {}", e)))?;
        
        Ok(Self {
            base_path: base_path.to_path_buf(),
            session: None,
            event_writer: None,
            sensor_writer: None,
            max_file_size: 100 * 1024 * 1024,  // 100 MB
        })
    }
    
    /// Start new recording session
    pub fn start_session(&mut self, name: &str, location: &str) -> Result<()> {
        let session = RecordingSession::new(name, location);
        let session_path = self.base_path.join(&session.id);
        
        create_dir_all(&session_path)
            .map_err(|e| SensorError::Recording(format!("Failed to create session dir: {}", e)))?;
        
        // Create event log file
        let event_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(session_path.join("events.jsonl"))
            .map_err(|e| SensorError::Recording(format!("Failed to create event file: {}", e)))?;
        
        // Create sensor log file
        let sensor_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(session_path.join("sensors.jsonl"))
            .map_err(|e| SensorError::Recording(format!("Failed to create sensor file: {}", e)))?;
        
        // Write session metadata
        let metadata_path = session_path.join("session.json");
        let metadata_json = serde_json::to_string_pretty(&session)
            .map_err(|e| SensorError::Recording(format!("Failed to serialize session: {}", e)))?;
        
        std::fs::write(&metadata_path, metadata_json)
            .map_err(|e| SensorError::Recording(format!("Failed to write metadata: {}", e)))?;
        
        self.event_writer = Some(BufWriter::new(event_file));
        self.sensor_writer = Some(BufWriter::new(sensor_file));
        self.session = Some(session);
        
        tracing::info!("Recording session started: {}", name);
        
        Ok(())
    }
    
    /// End current session
    pub fn end_session(&mut self) -> Result<Option<RecordingSession>> {
        if let Some(mut session) = self.session.take() {
            session.end();
            
            // Update metadata
            let session_path = self.base_path.join(&session.id);
            let metadata_path = session_path.join("session.json");
            
            let metadata_json = serde_json::to_string_pretty(&session)
                .map_err(|e| SensorError::Recording(format!("Failed to serialize session: {}", e)))?;
            
            std::fs::write(&metadata_path, metadata_json)
                .map_err(|e| SensorError::Recording(format!("Failed to write metadata: {}", e)))?;
            
            // Flush and close writers
            if let Some(ref mut writer) = self.event_writer {
                writer.flush().ok();
            }
            if let Some(ref mut writer) = self.sensor_writer {
                writer.flush().ok();
            }
            
            self.event_writer = None;
            self.sensor_writer = None;
            
            tracing::info!("Recording session ended: {} ({} events)", 
                session.name, session.event_count);
            
            return Ok(Some(session));
        }
        
        Ok(None)
    }
    
    /// Record paranormal event
    pub fn record_event(&mut self, event: &ParanormalEvent) -> Result<()> {
        if let Some(ref mut writer) = self.event_writer {
            let json = serde_json::to_string(event)
                .map_err(|e| SensorError::Recording(format!("Serialization error: {}", e)))?;
            
            writeln!(writer, "{}", json)
                .map_err(|e| SensorError::Recording(format!("Write error: {}", e)))?;
            
            writer.flush()
                .map_err(|e| SensorError::Recording(format!("Flush error: {}", e)))?;
            
            if let Some(ref mut session) = self.session {
                session.event_count += 1;
            }
        }
        
        Ok(())
    }
    
    /// Record sensor snapshot
    pub fn record_sensor(&mut self, snapshot: &SensorSnapshot) -> Result<()> {
        if let Some(ref mut writer) = self.sensor_writer {
            let record = SensorRecord {
                timestamp: SystemTime::now(),
                sensor_name: snapshot.sensor_name.clone(),
                value: snapshot.value,
                unit: snapshot.unit.clone(),
            };
            
            let json = serde_json::to_string(&record)
                .map_err(|e| SensorError::Recording(format!("Serialization error: {}", e)))?;
            
            writeln!(writer, "{}", json)
                .map_err(|e| SensorError::Recording(format!("Write error: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Flush writers
    pub fn flush(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.event_writer {
            writer.flush()
                .map_err(|e| SensorError::Recording(format!("Flush error: {}", e)))?;
        }
        if let Some(ref mut writer) = self.sensor_writer {
            writer.flush()
                .map_err(|e| SensorError::Recording(format!("Flush error: {}", e)))?;
        }
        Ok(())
    }
    
    /// Add note to current session
    pub fn add_note(&mut self, note: &str) {
        if let Some(ref mut session) = self.session {
            session.add_note(note);
        }
    }
    
    /// List all sessions
    pub fn list_sessions(&self) -> Result<Vec<RecordingSession>> {
        let mut sessions = Vec::new();
        
        for entry in std::fs::read_dir(&self.base_path)
            .map_err(|e| SensorError::Recording(format!("Read dir error: {}", e)))? 
        {
            let entry = entry.map_err(|e| SensorError::Recording(format!("Entry error: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata_path = path.join("session.json");
                if metadata_path.exists() {
                    let content = std::fs::read_to_string(&metadata_path)
                        .map_err(|e| SensorError::Recording(format!("Read error: {}", e)))?;
                    
                    if let Ok(session) = serde_json::from_str::<RecordingSession>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }
        
        // Sort by start time (newest first)
        sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        
        Ok(sessions)
    }
    
    /// Load events from session
    pub fn load_events(&self, session_id: &str) -> Result<Vec<ParanormalEvent>> {
        let path = self.base_path.join(session_id).join("events.jsonl");
        
        let file = File::open(&path)
            .map_err(|e| SensorError::Recording(format!("Open error: {}", e)))?;
        
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| SensorError::Recording(format!("Read error: {}", e)))?;
            
            if let Ok(event) = serde_json::from_str::<ParanormalEvent>(&line) {
                events.push(event);
            }
        }
        
        Ok(events)
    }
    
    /// Export session to portable format
    pub fn export_session(&self, session_id: &str, output_path: &Path) -> Result<()> {
        let session_path = self.base_path.join(session_id);
        
        // Load session metadata
        let metadata_path = session_path.join("session.json");
        let session: RecordingSession = serde_json::from_str(
            &std::fs::read_to_string(&metadata_path)
                .map_err(|e| SensorError::Recording(format!("Read error: {}", e)))?
        ).map_err(|e| SensorError::Recording(format!("Parse error: {}", e)))?;
        
        // Load events
        let events = self.load_events(session_id)?;
        
        // Create export structure
        let export = SessionExport {
            session,
            events,
            exported_at: Utc::now(),
            version: "1.0".to_string(),
        };
        
        // Write to output file
        let json = serde_json::to_string_pretty(&export)
            .map_err(|e| SensorError::Recording(format!("Serialize error: {}", e)))?;
        
        std::fs::write(output_path, json)
            .map_err(|e| SensorError::Recording(format!("Write error: {}", e)))?;
        
        tracing::info!("Exported session {} to {:?}", session_id, output_path);
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SensorRecord {
    timestamp: SystemTime,
    sensor_name: String,
    value: f64,
    unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionExport {
    session: RecordingSession,
    events: Vec<ParanormalEvent>,
    exported_at: DateTime<Utc>,
    version: String,
}
