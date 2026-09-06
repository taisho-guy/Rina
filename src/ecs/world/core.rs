use crate::ecs::EcsWorld;
use crate::ecs::history::{self, HistoryCommand, HistoryStack};
use crate::ecs::resources;
use crate::ecs::resources::{
    LayerStates, PresetStore, ProjectResource, SceneResource, SystemSettingsResource,
    TimelineResource,
};
use crate::ecs::transform::Camera;
use shipyard::{UniqueView, UniqueViewMut, World};

impl EcsWorld {
    pub fn new() -> Self {
        let world = World::new();
        world.add_unique(TimelineResource::new());
        world.add_unique(ProjectResource::new());
        world.add_unique(LayerStates::new(resources::DEFAULT_LAYER_COUNT));
        world.add_unique(SceneResource::new());
        world.add_unique(SystemSettingsResource::new());
        world.add_unique(Camera::default());
        world.add_unique(PresetStore::new());
        world.add_unique(HistoryStack::new());
        Self {
            world,
            selected_ids: std::collections::HashSet::new(),
            revision: 0,
        }
    }

    pub(crate) fn touch(&mut self) {
        self.revision += 1;
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn set_selected_ids(&mut self, ids: std::collections::HashSet<usize>) {
        self.selected_ids = ids;
    }

    pub fn is_selected(&self, id: usize) -> bool {
        self.selected_ids.contains(&id)
    }

    #[allow(dead_code)]
    pub fn undo(&mut self) -> bool {
        self.undo_internal()
    }

    #[allow(dead_code)]
    pub fn redo(&mut self) -> bool {
        self.redo_internal()
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        self.world
            .borrow::<UniqueView<HistoryStack>>()
            .map(|s| s.can_undo())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        self.world
            .borrow::<UniqueView<HistoryStack>>()
            .map(|s| s.can_redo())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn push_history_command(&mut self, command: Box<dyn HistoryCommand>) {
        if let Ok(mut stack) = self.world.borrow::<UniqueViewMut<HistoryStack>>() {
            stack.push(command);
        }
        self.touch();
    }

    #[allow(dead_code)]
    pub fn history_done_len(&self) -> usize {
        self.world
            .borrow::<UniqueView<HistoryStack>>()
            .map(|s| s.done_len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn history_undone_len(&self) -> usize {
        self.world
            .borrow::<UniqueView<HistoryStack>>()
            .map(|s| s.undone_len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn poll_effect_writebacks(&mut self) -> usize {
        let count = history::poll_all_effect_writebacks(&mut self.world);
        if count > 0 {
            self.touch();
        }
        count
    }

    fn undo_internal(&mut self) -> bool {
        let cmd = match self.world.borrow::<UniqueViewMut<HistoryStack>>() {
            Ok(mut stack) => stack.pop_done(),
            Err(_) => return false,
        };

        if let Some(mut command) = cmd {
            command.revert(&mut self.world);
            if let Ok(mut stack) = self.world.borrow::<UniqueViewMut<HistoryStack>>() {
                stack.push_undone(command);
            }
            self.touch();
            true
        } else {
            false
        }
    }

    fn redo_internal(&mut self) -> bool {
        let cmd = match self.world.borrow::<UniqueViewMut<HistoryStack>>() {
            Ok(mut stack) => stack.pop_undone(),
            Err(_) => return false,
        };

        if let Some(mut command) = cmd {
            command.apply(&mut self.world);
            if let Ok(mut stack) = self.world.borrow::<UniqueViewMut<HistoryStack>>() {
                stack.push_done(command);
            }
            self.touch();
            true
        } else {
            false
        }
    }
}
