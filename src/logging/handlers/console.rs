use std::fmt;
use std::io::{self, Write};
use std::sync::Mutex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::Level;

use super::LogHandler;
use crate::logging::LogFormat;

pub struct ConsoleHandler {
    stdout: Mutex<StandardStream>,
    stderr: Mutex<StandardStream>,
    level: Level,
    format: LogFormat,
}

impl ConsoleHandler {
    pub fn new(level: Level, format: LogFormat) -> Self {
        Self {
            stdout: Mutex::new(StandardStream::stdout(ColorChoice::Auto)),
            stderr: Mutex::new(StandardStream::stderr(ColorChoice::Auto)),
            level,
            format,
        }
    }

    fn get_level_color(level: &Level) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match *level {
            Level::ERROR => spec.set_fg(Some(Color::Red)).set_bold(true),
            Level::WARN => spec.set_fg(Some(Color::Yellow)).set_bold(true),
            Level::INFO => spec.set_fg(Some(Color::Green)),
            Level::DEBUG => spec.set_fg(Some(Color::Blue)),
            Level::TRACE => spec.set_fg(Some(Color::Magenta)),
        };
        spec
    }
}

impl LogHandler for ConsoleHandler {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.level() <= &self.level
    }

    fn log(&self, event: &tracing::Event<'_>) -> fmt::Result {
        let level = event.metadata().level();
        let mut writer = if *level <= Level::WARN {
            self.stderr.lock().unwrap()
        } else {
            self.stdout.lock().unwrap()
        };

        match self.format {
            LogFormat::Plain => {
                // Format timestamp
                let now = chrono::Utc::now();
                write!(writer, "{} ", now.format("%Y-%m-%d %H:%M:%S%.3f"))?;

                // Format level with color
                writer.set_color(&Self::get_level_color(level))?;
                write!(writer, "{:>5}", level)?;
                writer.reset()?;
                write!(writer, " ")?;

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
                    serde_json::Value::String(level.to_string()),
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

                // Format level (first letter only) with color
                writer.set_color(&Self::get_level_color(level))?;
                let level_char = level.as_str().chars().next().unwrap();
                write!(writer, "{}", level_char)?;
                writer.reset()?;
                write!(writer, " ")?;

                // Format fields
                write!(writer, "{:?}", event)?;
                writeln!(writer)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn flush(&self) {
        let _ = self.stdout.lock().unwrap().flush();
        let _ = self.stderr.lock().unwrap().flush();
    }
} 