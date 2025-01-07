use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::Level;

use super::LogHandler;
use crate::logging::{LogFormat, LogRotation};

pub struct FileHandler {
    writer: Mutex<BufWriter<File>>,
    level: Level,
    format: LogFormat,
    rotation: LogRotation,
    current_path: PathBuf,
}

impl FileHandler {
    pub fn new(path: PathBuf, level: Level, format: LogFormat, rotation: LogRotation) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            level,
            format,
            rotation,
            current_path: path,
        })
    }

    fn rotate_if_needed(&self) -> io::Result<()> {
        let metadata = std::fs::metadata(&self.current_path)?;
        
        if metadata.len() as usize > self.rotation.max_size {
            // Rotate files
            for i in (1..self.rotation.max_files).rev() {
                let src = self.current_path.with_extension(format!("log.{}", i));
                let dst = self.current_path.with_extension(format!("log.{}", i + 1));
                if src.exists() {
                    std::fs::rename(src, dst)?;
                }
            }

            // Rename current file
            let backup = self.current_path.with_extension("log.1");
            std::fs::rename(&self.current_path, backup)?;

            // Create new file
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.current_path)?;

            *self.writer.lock().unwrap() = BufWriter::new(file);
        }

        Ok(())
    }
}

impl LogHandler for FileHandler {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.level() <= &self.level
    }

    fn log(&self, event: &tracing::Event<'_>) -> fmt::Result {
        // Check rotation
        if let Err(e) = self.rotate_if_needed() {
            eprintln!("Error rotating log file: {}", e);
        }

        let mut writer = self.writer.lock().unwrap();

        match self.format {
            LogFormat::Plain => {
                // Format timestamp
                let now = chrono::Utc::now();
                write!(writer, "{} ", now.format("%Y-%m-%d %H:%M:%S%.3f"))?;

                // Format level
                write!(writer, "{:>5} ", event.metadata().level())?;

                // Format target
                write!(writer, "{}: ", event.metadata().target())?;

                // Format fields
                write!(writer, "{:?}", event)?;
                writeln!(writer)?;
            }
            LogFormat::Json => {
                let mut json = serde_json::Map::new();
                json.insert(
                    "timestamp".to_string(),
                    serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
                );
                json.insert(
                    "level".to_string(),
                    serde_json::Value::String(event.metadata().level().to_string()),
                );
                json.insert(
                    "target".to_string(),
                    serde_json::Value::String(event.metadata().target().to_string()),
                );
                json.insert(
                    "fields".to_string(),
                    serde_json::Value::String(format!("{:?}", event)),
                );

                writeln!(writer, "{}", serde_json::to_string(&json).unwrap())?;
            }
            LogFormat::Compact => {
                // Format timestamp (compact)
                let now = chrono::Utc::now();
                write!(writer, "{} ", now.format("%H:%M:%S"))?;

                // Format level (first letter only)
                let level = event.metadata().level().as_str().chars().next().unwrap();
                write!(writer, "{} ", level)?;

                // Format fields
                write!(writer, "{:?}", event)?;
                writeln!(writer)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn flush(&self) {
        if let Err(e) = self.writer.lock().unwrap().flush() {
            eprintln!("Error flushing log file: {}", e);
        }
    }
} 