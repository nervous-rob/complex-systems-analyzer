use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    SystemUpdated,
    ComponentChanged,
    RelationshipModified,
    AnalysisCompleted,
    ValidationFailed,
    UserInteraction,
    StateChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub payload: EventPayload,
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    System { id: Uuid, action: SystemAction },
    Component { id: Uuid, action: ComponentAction },
    Relationship { id: Uuid, action: RelationshipAction },
    Analysis { id: Uuid, status: AnalysisStatus },
    Validation { errors: Vec<String> },
    User { action: UserAction },
    State { old: String, new: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemAction {
    Created,
    Updated,
    Deleted,
    Exported,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentAction {
    Created,
    Updated,
    Deleted,
    StateChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipAction {
    Created,
    Updated,
    Deleted,
    WeightChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Started,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    Login,
    Logout,
    ViewChanged,
    SettingsUpdated,
    ExportRequested,
    ImportRequested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub module: String,
    pub component: String,
    pub user_id: Option<Uuid>,
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    fn supports_event(&self, event_type: &EventType) -> bool;
}

pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventHandler>>>>>,
    event_queue: Arc<RwLock<Vec<Event>>>,
    tx: mpsc::Sender<Event>,
    rx: Option<mpsc::Receiver<Event>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000); // Buffer size of 1000 events
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(RwLock::new(Vec::new())),
            tx,
            rx: Some(rx),
        }
    }

    pub async fn subscribe(&self, event_type: EventType, handler: Arc<dyn EventHandler>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub async fn unsubscribe(&self, event_type: EventType, handler_id: Uuid) {
        let mut subscribers = self.subscribers.write().await;
        if let Some(handlers) = subscribers.get_mut(&event_type) {
            // In a real implementation, we'd need a way to identify handlers
            // This is just a placeholder
            handlers.retain(|_| true);
        }
    }

    pub async fn publish(&self, event: Event) -> Result<()> {
        // Store event in queue
        self.event_queue.write().await.push(event.clone());

        // Send event to channel
        self.tx.send(event).await.map_err(|e| {
            crate::error::Error::Runtime(format!("Failed to publish event: {}", e))
        })?;

        Ok(())
    }

    pub async fn start_processing(&mut self) -> Result<()> {
        let rx = self.rx.take().ok_or_else(|| {
            crate::error::Error::Runtime("Event processor already started".to_string())
        })?;

        let subscribers = Arc::clone(&self.subscribers);
        let event_queue = Arc::clone(&self.event_queue);

        // Spawn event processing task
        tokio::spawn(async move {
            Self::process_events(rx, subscribers, event_queue).await;
        });

        Ok(())
    }

    async fn process_events(
        mut rx: mpsc::Receiver<Event>,
        subscribers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventHandler>>>>>,
        event_queue: Arc<RwLock<Vec<Event>>>,
    ) {
        while let Some(event) = rx.recv().await {
            let handlers = {
                let subs = subscribers.read().await;
                subs.get(&event.event_type)
                    .cloned()
                    .unwrap_or_default()
            };

            for handler in handlers {
                if handler.supports_event(&event.event_type) {
                    if let Err(e) = handler.handle_event(&event).await {
                        // Log error but continue processing
                        eprintln!("Error handling event: {}", e);
                    }
                }
            }

            // Cleanup old events (keep last 1000)
            let mut queue = event_queue.write().await;
            let queue_len = queue.len();
            if queue_len > 1000 {
                let drain_end = queue_len - 1000;
                queue.drain(0..drain_end);
            }
        }
    }

    pub async fn get_recent_events(&self, limit: usize) -> Vec<Event> {
        let queue = self.event_queue.read().await;
        queue.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
} 