use std::error::Error as StdError;
use std::fmt;
use std::sync::PoisonError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    Computation(String),
    Validation(String),
    IO(String),
    System(String),
    Configuration(String),
    ComponentNotFound(uuid::Uuid),
    RelationshipNotFound(uuid::Uuid),
    DuplicateComponent(uuid::Uuid),
    DuplicateRelationship(uuid::Uuid),
    OrphanedRelationship(uuid::Uuid, uuid::Uuid),
    CircularDependency(uuid::Uuid, uuid::Uuid),
    Runtime(String),
    Storage(String),
    LockPoisoned(String),
}

impl Error {
    pub fn computation<T: ToString>(msg: T) -> Self {
        Error::Computation(msg.to_string())
    }

    pub fn validation<T: ToString>(msg: T) -> Self {
        Error::Validation(msg.to_string())
    }

    pub fn io<T: ToString>(msg: T) -> Self {
        Error::IO(msg.to_string())
    }

    pub fn system<T: ToString>(msg: T) -> Self {
        Error::System(msg.to_string())
    }

    pub fn configuration<T: ToString>(msg: T) -> Self {
        Error::Configuration(msg.to_string())
    }

    pub fn runtime<T: ToString>(msg: T) -> Self {
        Error::Runtime(msg.to_string())
    }

    pub fn component_not_found(id: uuid::Uuid) -> Self {
        Error::ComponentNotFound(id)
    }

    pub fn relationship_not_found(id: uuid::Uuid) -> Self {
        Error::RelationshipNotFound(id)
    }

    pub fn duplicate_component(id: uuid::Uuid) -> Self {
        Error::DuplicateComponent(id)
    }

    pub fn duplicate_relationship(id: uuid::Uuid) -> Self {
        Error::DuplicateRelationship(id)
    }

    pub fn orphaned_relationship(relationship_id: uuid::Uuid, component_id: uuid::Uuid) -> Self {
        Error::OrphanedRelationship(relationship_id, component_id)
    }

    pub fn circular_dependency(source: uuid::Uuid, target: uuid::Uuid) -> Self {
        Error::CircularDependency(source, target)
    }

    pub fn lock_poisoned<T: ToString>(msg: T) -> Self {
        Error::LockPoisoned(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Computation(msg) => write!(f, "Computation error: {}", msg),
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::IO(msg) => write!(f, "IO error: {}", msg),
            Error::System(msg) => write!(f, "System error: {}", msg),
            Error::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            Error::ComponentNotFound(id) => write!(f, "Component not found: {}", id),
            Error::RelationshipNotFound(id) => write!(f, "Relationship not found: {}", id),
            Error::DuplicateComponent(id) => write!(f, "Duplicate component: {}", id),
            Error::DuplicateRelationship(id) => write!(f, "Duplicate relationship: {}", id),
            Error::OrphanedRelationship(rel_id, comp_id) => write!(f, "Orphaned relationship {}: missing component {}", rel_id, comp_id),
            Error::CircularDependency(source, target) => write!(f, "Circular dependency detected between components {} and {}", source, target),
            Error::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            Error::Storage(msg) => write!(f, "Storage error: {}", msg),
            Error::LockPoisoned(msg) => write!(f, "Lock poisoned: {}", msg),
        }
    }
}

impl StdError for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl<W> From<csv::IntoInnerError<W>> for Error {
    fn from(err: csv::IntoInnerError<W>) -> Self {
        Error::IO(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::LockPoisoned(err.to_string())
    }
}

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        Error::System(err.to_string())
    }
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        Error::System(err.to_string())
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(err: wgpu::SurfaceError) -> Self {
        Error::System(err.to_string())
    }
}

impl From<winit::error::EventLoopError> for Error {
    fn from(err: winit::error::EventLoopError) -> Self {
        Error::System(err.to_string())
    }
}

impl From<winit::error::OsError> for Error {
    fn from(err: winit::error::OsError) -> Self {
        Error::System(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>; 