//! Read-only planning over a caller-verified, complete cache registry snapshot.
//! A plan is not deletion authority; storage must revalidate it under a GC barrier.

use std::collections::{BTreeMap, BTreeSet};

pub const CACHE_LIFECYCLE_PROTOCOL: &str = "nuis-rc-cache-lifecycle-v1";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PackageLifetime {
    #[default]
    Heap,
    Stack,
}

/// Project-bound content cannot be promoted into the shared heap by a consumer.
pub fn resolve_package_lifetime(
    requested: Option<PackageLifetime>,
    requires_project_scope: bool,
) -> Result<PackageLifetime, &'static str> {
    match (requested, requires_project_scope) {
        (Some(PackageLifetime::Heap), true) => {
            Err("project-bound packages cannot use heap lifetime")
        }
        (_, true) => Ok(PackageLifetime::Stack),
        (requested, false) => Ok(requested.unwrap_or_default()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheOwner {
    Heap,
    Project([u8; 32]),
}

/// Logical ownership identity, not a physically deduplicated CAS blob key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheObjectKey {
    pub owner: CacheOwner,
    pub content_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheObject {
    pub key: CacheObjectKey,
    pub bytes: u64,
    pub dependencies: Vec<CacheObjectKey>,
    /// First continuously unreachable observation, not file mtime or last access.
    /// Unknown history keeps the object until a later verified observation.
    pub unreachable_since_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    ProjectLock,
    BuildLease,
    ExplicitPin,
    ToolchainPin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRoot {
    pub kind: RootKind,
    pub objects: Vec<CacheObjectKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheSnapshot {
    pub generation: u64,
    pub captured_at_seconds: u64,
    /// False for missing volumes, unreadable roots, or unresolved lease ownership.
    pub complete: bool,
    pub objects: Vec<CacheObject>,
    pub roots: Vec<CacheRoot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcPolicy {
    pub grace_seconds: u64,
    pub max_heap_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcPlan {
    pub protocol: &'static str,
    pub snapshot_generation: u64,
    pub candidates: Vec<CacheObjectKey>,
    /// These totals account logical objects, not physical CAS allocation or savings.
    pub heap_bytes: u64,
    pub reclaimable_bytes: u64,
    pub retained_heap_bytes: u64,
    /// A nonzero value must cause backpressure, not eviction of protected objects.
    pub over_budget_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipEdge {
    pub from: CacheObjectKey,
    pub to: CacheObjectKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GcPlanError {
    IncompleteSnapshot,
    DuplicateObject(CacheObjectKey),
    MissingObject(CacheObjectKey),
    InvalidOwnershipEdge(Box<OwnershipEdge>),
    FutureUnreachableTime(CacheObjectKey),
    ByteCountOverflow,
}

/// Computes the complete eligible set, without inspecting paths or mutating state.
pub fn plan_gc(snapshot: &CacheSnapshot, policy: GcPolicy) -> Result<GcPlan, GcPlanError> {
    if !snapshot.complete {
        return Err(GcPlanError::IncompleteSnapshot);
    }
    let mut inventory = BTreeMap::new();
    let mut heap_bytes = 0_u64;
    for object in &snapshot.objects {
        if inventory.insert(object.key, object).is_some() {
            return Err(GcPlanError::DuplicateObject(object.key));
        }
        if object.key.owner == CacheOwner::Heap {
            heap_bytes = heap_bytes
                .checked_add(object.bytes)
                .ok_or(GcPlanError::ByteCountOverflow)?;
        }
    }

    let mut protected = BTreeSet::new();
    let mut pending = Vec::new();
    for object in inventory.values() {
        for dependency in &object.dependencies {
            if !inventory.contains_key(dependency) {
                return Err(GcPlanError::MissingObject(*dependency));
            }
            let allowed = match (object.key.owner, dependency.owner) {
                (_, CacheOwner::Heap) => true,
                (CacheOwner::Project(owner), CacheOwner::Project(dependency_owner)) => {
                    owner == dependency_owner
                }
                (CacheOwner::Heap, CacheOwner::Project(_)) => false,
            };
            if !allowed {
                return Err(GcPlanError::InvalidOwnershipEdge(Box::new(OwnershipEdge {
                    from: object.key,
                    to: *dependency,
                })));
            }
        }
        let grace_elapsed = match object.unreachable_since_seconds {
            Some(since) => {
                snapshot
                    .captured_at_seconds
                    .checked_sub(since)
                    .ok_or(GcPlanError::FutureUnreachableTime(object.key))?
                    >= policy.grace_seconds
            }
            None => false,
        };
        if object.key.owner != CacheOwner::Heap || !grace_elapsed {
            protect(object.key, &mut protected, &mut pending);
        }
    }
    for root in &snapshot.roots {
        for key in &root.objects {
            if !inventory.contains_key(key) {
                return Err(GcPlanError::MissingObject(*key));
            }
            protect(*key, &mut protected, &mut pending);
        }
    }

    // Retention is transitive even for grace-period objects, and cycles do not leak.
    while let Some(key) = pending.pop() {
        for dependency in &inventory[&key].dependencies {
            protect(*dependency, &mut protected, &mut pending);
        }
    }

    let mut candidates = Vec::new();
    let mut reclaimable_bytes = 0_u64;
    for (key, object) in inventory {
        if !protected.contains(&key) {
            candidates.push(key);
            reclaimable_bytes = reclaimable_bytes
                .checked_add(object.bytes)
                .ok_or(GcPlanError::ByteCountOverflow)?;
        }
    }
    let retained_heap_bytes = heap_bytes - reclaimable_bytes;
    Ok(GcPlan {
        protocol: CACHE_LIFECYCLE_PROTOCOL,
        snapshot_generation: snapshot.generation,
        candidates,
        heap_bytes,
        reclaimable_bytes,
        retained_heap_bytes,
        over_budget_bytes: retained_heap_bytes.saturating_sub(policy.max_heap_bytes),
    })
}

fn protect(
    key: CacheObjectKey,
    protected: &mut BTreeSet<CacheObjectKey>,
    pending: &mut Vec<CacheObjectKey>,
) {
    if protected.insert(key) {
        pending.push(key);
    }
}

#[cfg(test)]
#[path = "cache_lifecycle_tests.rs"]
mod tests;
