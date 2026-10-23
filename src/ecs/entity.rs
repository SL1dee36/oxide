use slotmap::{new_key_type, Key, KeyData};

new_key_type! {
    /// Generational entity identifier guaranteed unique across allocations in an ECS [`super::world::World`].
    pub struct EntityId;
}

/// Backwards-compatibility alias for [`EntityId`].
pub type Entity = EntityId;

impl EntityId {
    /// Sentinel invalid / null entity key.
    pub const INVALID: Self = Self(KeyData::from_ffi(0));

    /// Converts this entity key to a 64-bit unsigned integer suitable for C FFI.
    #[inline]
    pub fn to_ffi(self) -> u64 {
        self.data().as_ffi()
    }

    /// Reconstructs an entity key from a 64-bit unsigned integer obtained via [`Self::to_ffi`].
    #[inline]
    pub fn from_ffi(id: u64) -> Self {
        Self::from(KeyData::from_ffi(id))
    }
}
