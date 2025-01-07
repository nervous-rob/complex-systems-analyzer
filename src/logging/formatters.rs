use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut dyn fmt::Write,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format timestamp
        let now = chrono::Utc::now();
        write!(writer, "{} ", now.format("%Y-%m-%d %H:%M:%S%.3f"))?;

        // Format level
        let level = *event.metadata().level();
        write!(writer, "{:>5} ", level)?;

        // Format target
        write!(writer, "{}: ", event.metadata().target())?;

        // Format fields
        ctx.field_format().format_fields(writer, event)?;

        writeln!(writer)
    }
}

pub struct JsonFormatter;

impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut dyn fmt::Write,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut json = serde_json::Map::new();

        // Add timestamp
        let now = chrono::Utc::now();
        json.insert(
            "timestamp".to_string(),
            serde_json::Value::String(now.to_rfc3339()),
        );

        // Add level
        json.insert(
            "level".to_string(),
            serde_json::Value::String(event.metadata().level().to_string()),
        );

        // Add target
        json.insert(
            "target".to_string(),
            serde_json::Value::String(event.metadata().target().to_string()),
        );

        // Add fields
        let mut fields = serde_json::Map::new();
        ctx.field_format().format_fields(writer, event)?;
        json.insert("fields".to_string(), serde_json::Value::Object(fields));

        writeln!(writer, "{}", serde_json::to_string(&json).unwrap())
    }
}

pub struct CompactFormatter;

impl<S, N> FormatEvent<S, N> for CompactFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut dyn fmt::Write,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format timestamp (compact)
        let now = chrono::Utc::now();
        write!(writer, "{} ", now.format("%H:%M:%S"))?;

        // Format level (first letter only)
        let level = event.metadata().level().as_str().chars().next().unwrap();
        write!(writer, "{} ", level)?;

        // Format fields
        ctx.field_format().format_fields(writer, event)?;

        writeln!(writer)
    }
} 