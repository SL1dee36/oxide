use crate::collision::collider::Aabb3D;
use crate::ecs::entity::EntityId;

/// Candidate pair of colliding entities identified during broadphase culling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BroadphasePair {
    /// First entity identifier (guaranteed `a <= b`).
    pub a: EntityId,
    /// Second entity identifier.
    pub b: EntityId,
}

impl BroadphasePair {
    /// Constructs a canonical collision pair where entity ordering is deterministic.
    #[inline]
    pub fn new(a: EntityId, b: EntityId) -> Self {
        if a <= b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

/// Bounding record for an entity in broadphase space.
#[derive(Debug, Clone, Copy)]
pub struct BroadphaseProxy {
    /// Entity identifier.
    pub entity: EntityId,
    /// World-space conservative axis-aligned bounding box.
    pub aabb: Aabb3D,
    /// True if entity is a non-dynamic or static obstacle.
    pub is_static: bool,
}

/// High-performance Sweep-and-Prune broadphase stage for O(N log N) collision culling.
#[derive(Default, Debug, Clone)]
pub struct BroadphaseGrid {
    proxies: Vec<BroadphaseProxy>,
    pairs: Vec<BroadphasePair>,
}

impl BroadphaseGrid {
    /// Creates an empty broadphase stage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears internal proxy and pair buffers, reusing allocated capacity across frames.
    pub fn clear(&mut self) {
        self.proxies.clear();
        self.pairs.clear();
    }

    /// Reserves buffer capacity for `capacity` proxies.
    pub fn reserve(&mut self, capacity: usize) {
        self.proxies.reserve(capacity);
    }

    /// Adds an entity bounding proxy to the broadphase.
    pub fn push(&mut self, entity: EntityId, aabb: Aabb3D, is_static: bool) {
        self.proxies.push(BroadphaseProxy {
            entity,
            aabb,
            is_static,
        });
    }

    /// Returns the number of proxies currently registered.
    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Returns `true` if no proxies are registered.
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    /// Executes Sort-and-Sweep along the X axis with Y and Z AABB pruning,
    /// populating and returning all candidate intersecting pairs.
    pub fn compute_pairs(&mut self) -> &[BroadphasePair] {
        self.pairs.clear();
        let len = self.proxies.len();
        if len < 2 {
            return &self.pairs;
        }

        // Sort proxies along principal X axis by minimum extent
        self.proxies.sort_unstable_by(|a, b| {
            a.aabb
                .min
                .x
                .partial_cmp(&b.aabb.min.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for i in 0..len {
            let a = &self.proxies[i];

            for j in (i + 1)..len {
                let b = &self.proxies[j];

                // If b.min.x > a.max.x, no subsequent proxy along sorted X can overlap with a
                if b.aabb.min.x > a.aabb.max.x {
                    break;
                }

                // Prune pairs where both bodies are static immovable obstacles
                if a.is_static && b.is_static {
                    continue;
                }

                // Check Y and Z axis intervals
                if a.aabb.min.y <= b.aabb.max.y
                    && a.aabb.max.y >= b.aabb.min.y
                    && a.aabb.min.z <= b.aabb.max.z
                    && a.aabb.max.z >= b.aabb.min.z
                {
                    self.pairs.push(BroadphasePair::new(a.entity, b.entity));
                }
            }
        }

        &self.pairs
    }
}
