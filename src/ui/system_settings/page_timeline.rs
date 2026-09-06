use super::fields::{int_field, toggle_field};
use super::window::SystemSettingsWindow;
use crate::ecs::EcsWorld;
use std::sync::{Arc, Mutex};

impl SystemSettingsWindow {
    pub(super) fn page_timeline_defaults(
        &mut self,
        ui: &mut egui::Ui,
        world_holder: &Arc<Mutex<EcsWorld>>,
    ) {
        let mut default_snap = self.default_snap;
        let mut magnetic_snap_range = self.magnetic_snap_range;
        let changed = toggle_field(ui, "スナップを既定で有効化", &mut default_snap)
            | int_field(
                ui,
                "磁力スナップ範囲（px）",
                &mut magnetic_snap_range,
                0,
                100,
            );
        if changed {
            self.default_snap = default_snap;
            self.magnetic_snap_range = magnetic_snap_range;
            let (snap, range) = (self.default_snap, self.magnetic_snap_range);
            self.persist(world_holder, |s| {
                s.default_snap = snap;
                s.magnetic_snap_range = range;
            });
        }
    }
}
