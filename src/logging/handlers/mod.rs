mod file;
mod console;

pub use file::FileHandler;
pub use console::ConsoleHandler;

use std::fmt;
use tracing::Subscriber;
use tracing_subscriber::Layer;

pub trait LogHandler: Send + Sync {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool;
    fn log(&self, event: &tracing::Event<'_>) -> fmt::Result;
    fn flush(&self);
}

pub struct FilteredLayer<S> {
    handler: Box<dyn LogHandler>,
    _subscriber: std::marker::PhantomData<S>,
}

impl<S> FilteredLayer<S> {
    pub fn new(handler: Box<dyn LogHandler>) -> Self {
        Self {
            handler,
            _subscriber: std::marker::PhantomData,
        }
    }
}

impl<S> Layer<S> for FilteredLayer<S>
where
    S: Subscriber,
{
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        self.handler.enabled(metadata)
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if let Err(e) = self.handler.log(event) {
            eprintln!("Error logging event: {}", e);
        }
    }

    fn on_close(&self, _id: tracing_subscriber::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.handler.flush();
    }
} 