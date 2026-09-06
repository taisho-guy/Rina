use shipyard::{Component, EntityId};

#[derive(Clone, Copy, Debug, Component)]
pub struct ParentEntity(pub EntityId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackMatteMode {
    Alpha,
    AlphaInvert,
    Luma,
    LumaInvert,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct TrackMatteSource {
    pub source: EntityId,
    pub mode: TrackMatteMode,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct AdjustmentLayer(pub bool);

impl Default for AdjustmentLayer {
    fn default() -> Self {
        Self(false)
    }
}
