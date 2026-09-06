use super::native_window::NativeWindow;
use super::preview::PreviewSlot;
use super::window_kind::WindowKind;
use crate::gpu_shared::SharedGpu;
use crate::ui::launcher::LauncherPanel;
use std::collections::HashMap;
use std::rc::Rc;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::WindowId;

pub(super) struct EguiMainWindow {
    pub(super) gpu: Rc<SharedGpu>,
    pub(super) slot: PreviewSlot,
    pub(super) launcher: LauncherPanel,
    pub(super) windows: HashMap<WindowId, NativeWindow>,
    pub(super) project_windows_created: bool,
    pub(super) init_rx: std::sync::mpsc::Receiver<()>,
    pub(super) init_done: bool,
    pub(super) wake_proxy: EventLoopProxy<super::preview::AppWakeEvent>,
}

impl EguiMainWindow {
    pub(super) fn new(
        gpu: Rc<SharedGpu>,
        slot: PreviewSlot,
        init_rx: std::sync::mpsc::Receiver<()>,
        wake_proxy: EventLoopProxy<super::preview::AppWakeEvent>,
    ) -> Self {
        if let Some(loaded) = crate::ui::system_settings::load_from_disk() {
            crate::theme::restore(&loaded.theme_id);
        }
        Self {
            gpu,
            slot,
            launcher: LauncherPanel::new(),
            windows: HashMap::new(),
            project_windows_created: false,
            init_rx,
            init_done: false,
            wake_proxy,
        }
    }

    pub(super) fn add_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowKind) {
        let native = NativeWindow::create(event_loop, &self.gpu, kind);
        self.windows.insert(native.window.id(), native);
    }

    pub(super) fn ensure_project_windows(&mut self, event_loop: &ActiveEventLoop) {
        if self.slot.borrow().is_none() || self.project_windows_created {
            return;
        }
        let launcher_ids: Vec<WindowId> = self
            .windows
            .iter()
            .filter(|(_, native)| native.kind == WindowKind::Launcher)
            .map(|(id, _)| *id)
            .collect();
        for id in launcher_ids {
            self.windows.remove(&id);
        }
        self.add_window(event_loop, WindowKind::Preview);
        self.add_window(event_loop, WindowKind::Timeline);
        self.add_window(event_loop, WindowKind::Properties);
        self.project_windows_created = true;
    }

    pub(super) fn dialog_open_state(&self, kind: WindowKind) -> Option<bool> {
        let slot = self.slot.borrow();
        let p = slot.as_ref()?;
        Some(match kind {
            WindowKind::SystemSettings => p.dialogs.borrow().system_settings.open,
            WindowKind::ProjectSettings => p.dialogs.borrow().project_settings.open,
            WindowKind::SceneSettings => p.dialogs.borrow().scene_settings.open,
            WindowKind::Keybindings => p.dialogs.borrow().keybindings.open,
            WindowKind::Export => p.dialogs.borrow().export_dialog.open,
            WindowKind::EffectAdd => p.properties.borrow().effect_add.open,
            WindowKind::EasingEditor => crate::ui::properties::easing_editor::is_open(),
            _ => return None,
        })
    }

    pub(super) fn sync_dialog_windows(&mut self, event_loop: &ActiveEventLoop) {
        for kind in [
            WindowKind::SystemSettings,
            WindowKind::ProjectSettings,
            WindowKind::SceneSettings,
            WindowKind::Keybindings,
            WindowKind::Export,
            WindowKind::EffectAdd,
            WindowKind::EasingEditor,
        ] {
            let Some(desired_open) = self.dialog_open_state(kind) else {
                continue;
            };
            let existing_id = self
                .windows
                .iter()
                .find(|(_, native)| native.kind == kind)
                .map(|(id, _)| *id);
            match (desired_open, existing_id) {
                (true, None) => self.add_window(event_loop, kind),
                (false, Some(id)) => {
                    self.windows.remove(&id);
                }
                _ => {}
            }
        }
    }

    pub(super) fn set_dialog_open(slot: &PreviewSlot, kind: WindowKind, open: bool) {
        let slot_ref = slot.borrow();
        let Some(p) = slot_ref.as_ref() else {
            return;
        };
        match kind {
            WindowKind::SystemSettings => p.dialogs.borrow_mut().system_settings.open = open,
            WindowKind::ProjectSettings => p.dialogs.borrow_mut().project_settings.open = open,
            WindowKind::SceneSettings => p.dialogs.borrow_mut().scene_settings.open = open,
            WindowKind::Keybindings => p.dialogs.borrow_mut().keybindings.set_open(open),
            WindowKind::Export => p.dialogs.borrow_mut().export_dialog.open = open,
            WindowKind::EffectAdd => p.properties.borrow_mut().effect_add.open = open,
            WindowKind::EasingEditor => {
                if !open {
                    crate::ui::properties::easing_editor::close();
                }
            }
            _ => {}
        }
    }

    pub(super) fn redraw(&mut self, id: WindowId) {
        let Some(mut native) = self.windows.remove(&id) else {
            return;
        };
        match native.kind {
            WindowKind::Splash => {
                native.redraw(&self.gpu, |ui, _| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.add(egui::Image::new(crate::splash::SOURCE.clone()));
                            });
                        });
                });
            }
            WindowKind::Launcher => {
                let launcher = &mut self.launcher;
                let gpu = self.gpu.clone();
                let slot = self.slot.clone();
                native.redraw(&self.gpu, |ui, _| {
                    if slot.borrow().is_none() {
                        if let Some(meta) = launcher.show(ui) {
                            crate::ui::start_project(meta, gpu, slot);
                        }
                    }
                });
            }
            WindowKind::Preview => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native
                        .window
                        .set_title(&crate::app_state::active_project_window_title(&p.state));
                    native.redraw(&self.gpu, |ui, renderer| {
                        p.panel
                            .borrow_mut()
                            .show(ui, renderer, &p.state, &p.dialogs);
                    });
                }
            }
            WindowKind::Timeline => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.timeline
                            .borrow_mut()
                            .show(&ctx, ui, &p.state, &p.panel, &(), &p.dialogs);
                    });
                }
            }
            WindowKind::Properties => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.properties.borrow_mut().show(&ctx, ui, &p.state, &p.panel);
                    });
                }
            }
            WindowKind::SystemSettings => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let mut dialogs = p.dialogs.borrow_mut();
                        let ctx = ui.ctx().clone();
                        dialogs.system_settings.show(
                            &ctx,
                            ui,
                            &crate::app_state::settings_world(&p.state),
                        );
                    });
                }
            }
            WindowKind::ProjectSettings => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.dialogs
                            .borrow_mut()
                            .project_settings
                            .show(&ctx, ui, &p.state);
                    });
                }
            }
            WindowKind::SceneSettings => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.dialogs
                            .borrow_mut()
                            .scene_settings
                            .show(&ctx, ui, &p.state);
                    });
                }
            }
            WindowKind::Keybindings => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.dialogs.borrow_mut().keybindings.show(&ctx, ui)
                    });
                }
            }
            WindowKind::Export => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        p.dialogs
                            .borrow_mut()
                            .export_dialog
                            .show(&ctx, ui, &p.state)
                    });
                }
            }
            WindowKind::EffectAdd => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        p.properties.borrow_mut().show_effect_add(ui, &p.state);
                    });
                }
            }
            WindowKind::EasingEditor => {
                if let Some(p) = self.slot.borrow().as_ref() {
                    native.redraw(&self.gpu, |ui, _| {
                        let ctx = ui.ctx().clone();
                        let holder = crate::app_state::active_world(&p.state);
                        let mut world = holder.lock().unwrap();
                        if !crate::ui::properties::easing_editor::show(&ctx, ui, &mut world) {
                            crate::ui::properties::easing_editor::close();
                        }
                    });
                }
            }
        }
        self.windows.insert(id, native);
    }
}
