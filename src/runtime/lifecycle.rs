use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(SystemError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemError {
    pub code: i32,
    pub message: String,
}

#[async_trait]
pub trait LifecycleHook: Send + Sync {
    async fn on_startup(&self) -> Result<()>;
    async fn on_shutdown(&self) -> Result<()>;
    fn get_dependencies(&self) -> Vec<String>;
}

pub struct LifecycleManager {
    state: Arc<RwLock<SystemState>>,
    hooks: Arc<RwLock<Vec<Box<dyn LifecycleHook>>>>,
}

impl LifecycleManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: Arc::new(RwLock::new(SystemState::Stopped)),
            hooks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start_system(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != SystemState::Stopped {
            return Err(Error::Runtime("System is already running".into()));
        }

        *state = SystemState::Starting;
        drop(state);

        // Execute startup hooks in dependency order
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = hook.on_startup().await {
                self.state.write().await.set(SystemState::Error(SystemError {
                    code: 1,
                    message: format!("Startup hook failed: {}", e),
                }));
                return Err(e);
            }
        }

        self.state.write().await.set(SystemState::Running);
        Ok(())
    }

    pub async fn stop_system(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != SystemState::Running {
            return Err(Error::Runtime("System is not running".into()));
        }

        *state = SystemState::Stopping;
        drop(state);

        // Execute shutdown hooks in reverse dependency order
        let hooks = self.hooks.read().await;
        for hook in hooks.iter().rev() {
            if let Err(e) = hook.on_shutdown().await {
                self.state.write().await.set(SystemState::Error(SystemError {
                    code: 2,
                    message: format!("Shutdown hook failed: {}", e),
                }));
                return Err(e);
            }
        }

        self.state.write().await.set(SystemState::Stopped);
        Ok(())
    }

    pub async fn register_hook(&self, hook: Box<dyn LifecycleHook>) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        // Sort hooks by dependencies (would need more complex implementation)
    }

    pub async fn get_system_state(&self) -> SystemState {
        (*self.state.read().await).clone()
    }
}

impl SystemState {
    pub fn set(&mut self, new_state: SystemState) {
        *self = new_state;
    }

    pub fn is_running(&self) -> bool {
        *self == SystemState::Running
    }

    pub fn is_error(&self) -> bool {
        matches!(self, SystemState::Error(_))
    }
} 