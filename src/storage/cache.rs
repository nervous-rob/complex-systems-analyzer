use std::time::{Duration, Instant};
use dashmap::DashMap;
use uuid::Uuid;

use crate::core::{System, Component, Relationship};
use crate::error::Result;

const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

pub struct Cache {
    systems: DashMap<Uuid, CacheEntry<System>>,
    components: DashMap<Uuid, CacheEntry<Component>>,
    relationships: DashMap<Uuid, CacheEntry<Relationship>>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl: Option<Duration>) -> Self {
        Self {
            systems: DashMap::new(),
            components: DashMap::new(),
            relationships: DashMap::new(),
            ttl: ttl.unwrap_or(DEFAULT_CACHE_TTL),
        }
    }

    pub fn get_system(&self, id: &Uuid) -> Option<System> {
        self.components.retain(|_, v| !v.is_expired());
        self.systems
            .get(id)
            .and_then(|entry| {
                if entry.is_expired() {
                    self.systems.remove(id);
                    None
                } else {
                    Some(entry.value.clone())
                }
            })
    }

    pub fn store_system(&self, system: System) {
        self.systems.insert(
            system.id,
            CacheEntry::new(system, self.ttl),
        );
    }

    pub fn get_component(&self, id: &Uuid) -> Option<Component> {
        self.components.retain(|_, v| !v.is_expired());
        self.components
            .get(id)
            .and_then(|entry| {
                if entry.is_expired() {
                    self.components.remove(id);
                    None
                } else {
                    Some(entry.value.clone())
                }
            })
    }

    pub fn store_component(&self, component: Component) {
        self.components.insert(
            component.id,
            CacheEntry::new(component, self.ttl),
        );
    }

    pub fn get_relationship(&self, id: &Uuid) -> Option<Relationship> {
        self.relationships.retain(|_, v| !v.is_expired());
        self.relationships
            .get(id)
            .and_then(|entry| {
                if entry.is_expired() {
                    self.relationships.remove(id);
                    None
                } else {
                    Some(entry.value.clone())
                }
            })
    }

    pub fn store_relationship(&self, relationship: Relationship) {
        self.relationships.insert(
            relationship.id,
            CacheEntry::new(relationship, self.ttl),
        );
    }

    pub fn invalidate_system(&self, id: &Uuid) {
        self.systems.remove(id);
        // Also invalidate related components and relationships
        // Note: Since components don't have a direct reference to their system,
        // we can't invalidate them here. This would need to be handled at a higher level.
        // self.components.retain(|_, entry| !entry.is_expired());
        // Relationships would need system context to invalidate
    }

    pub fn invalidate_component(&self, id: &Uuid) {
        self.components.remove(id);
        // Also invalidate related relationships
        self.relationships.retain(|_, entry| {
            entry.value.source_id != *id && 
            entry.value.target_id != *id && 
            !entry.is_expired()
        });
    }

    pub fn invalidate_relationship(&self, id: &Uuid) {
        self.relationships.remove(id);
    }

    pub fn clear(&self) {
        self.systems.clear();
        self.components.clear();
        self.relationships.clear();
    }

    pub fn cleanup_expired(&self) {
        self.systems.retain(|_, v| !v.is_expired());
        self.components.retain(|_, v| !v.is_expired());
        self.relationships.retain(|_, v| !v.is_expired());
    }

    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            systems_count: self.systems.len(),
            components_count: self.components.len(),
            relationships_count: self.relationships.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub systems_count: usize,
    pub components_count: usize,
    pub relationships_count: usize,
} 