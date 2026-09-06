use crate::ecs::EcsWorld;
use crate::ecs::components::{
    AdjustmentLayer, BlendMode, MaskStack, ParentEntity, TimeRemap, TrackMatteSource,
};
use shipyard::{Get, View};

impl EcsWorld {
    pub fn set_parent(&mut self, object_id: usize, parent_id: Option<usize>) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        match parent_id.and_then(|pid| self.find_entity(pid)) {
            Some(parent_entity) => {
                self.world
                    .add_component(entity, ParentEntity(parent_entity));
            }
            None => {
                self.world.remove::<(ParentEntity,)>(entity);
            }
        }
        self.recompute_global_matrices();
        self.touch();
    }

    pub fn parent_of(&self, object_id: usize) -> Option<usize> {
        let entity = self.find_entity(object_id)?;
        let parent_entity = self
            .world
            .run(|parents: View<ParentEntity>| parents.get(entity).ok().map(|p| p.0))?;
        self.object_id_of(parent_entity)
    }

    pub fn set_track_matte(&mut self, object_id: usize, matte: Option<TrackMatteSource>) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        match matte {
            Some(m) => self.world.add_component(entity, m),
            None => {
                let _ = self.world.remove::<(TrackMatteSource,)>(entity);
            }
        }
        self.touch();
    }

    pub fn set_track_matte_by_id(
        &mut self,
        object_id: usize,
        source_object_id: Option<usize>,
        mode: crate::ecs::components::TrackMatteMode,
    ) {
        let matte = source_object_id
            .and_then(|sid| self.find_entity(sid))
            .map(|source| TrackMatteSource { source, mode });
        self.set_track_matte(object_id, matte);
    }

    pub fn track_matte_of(&self, object_id: usize) -> Option<TrackMatteSource> {
        let entity = self.find_entity(object_id)?;
        self.world
            .run(|mattes: View<TrackMatteSource>| mattes.get(entity).ok().copied())
    }

    pub fn track_matte_source_id_of(&self, object_id: usize) -> Option<usize> {
        let matte = self.track_matte_of(object_id)?;
        self.object_id_of(matte.source)
    }

    pub fn set_blend_mode(&mut self, object_id: usize, mode: BlendMode) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        self.world.add_component(entity, mode);
        self.touch();
    }

    pub fn blend_mode_of(&self, object_id: usize) -> BlendMode {
        let Some(entity) = self.find_entity(object_id) else {
            return BlendMode::default();
        };
        self.world
            .run(|modes: View<BlendMode>| modes.get(entity).copied().unwrap_or_default())
    }

    pub fn set_mask_stack(&mut self, object_id: usize, stack: MaskStack) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        self.world.add_component(entity, stack);
        self.touch();
    }

    pub fn mask_stack_of(&self, object_id: usize) -> MaskStack {
        let Some(entity) = self.find_entity(object_id) else {
            return MaskStack::default();
        };
        self.world
            .run(|stacks: View<MaskStack>| stacks.get(entity).cloned().unwrap_or_default())
    }

    pub fn set_time_remap(&mut self, object_id: usize, remap: TimeRemap) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        self.world.add_component(entity, remap);
        self.touch();
    }

    pub fn time_remap_of(&self, object_id: usize) -> TimeRemap {
        let Some(entity) = self.find_entity(object_id) else {
            return TimeRemap::default();
        };
        self.world
            .run(|remaps: View<TimeRemap>| remaps.get(entity).cloned().unwrap_or_default())
    }

    pub fn set_adjustment_layer(&mut self, object_id: usize, enabled: bool) {
        let Some(entity) = self.find_entity(object_id) else {
            return;
        };
        self.world.add_component(entity, AdjustmentLayer(enabled));
        self.touch();
    }

    pub fn is_adjustment_layer(&self, object_id: usize) -> bool {
        let Some(entity) = self.find_entity(object_id) else {
            return false;
        };
        self.world
            .run(|layers: View<AdjustmentLayer>| layers.get(entity).map(|l| l.0).unwrap_or(false))
    }
}
