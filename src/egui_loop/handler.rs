use super::main_window::EguiMainWindow;
use super::preview::{AppWakeEvent, PreviewSlot};
use super::window_kind::WindowKind;
use crate::gpu_shared::SharedGpu;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

impl ApplicationHandler<AppWakeEvent> for EguiMainWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            let (w, h) = crate::splash::WINDOW_SIZE;
            let native = super::native_window::NativeWindow::create_sized(
                event_loop,
                &self.gpu,
                WindowKind::Splash,
                w,
                h,
            );
            self.windows.insert(native.window.id(), native);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(native) = self.windows.get_mut(&id) else {
            return;
        };
        if native.visible && native.state.on_window_event(&native.window, &event).repaint {
            native.window.request_redraw();
        }
        let kind = native.kind;
        match event {
            WindowEvent::CloseRequested => match kind {
                WindowKind::Splash | WindowKind::Launcher | WindowKind::Preview => {
                    event_loop.exit()
                }
                WindowKind::Timeline | WindowKind::Properties => {
                    native.visible = false;
                    native.window.set_visible(false);
                }
                _ if kind.is_lazy_dialog() => {
                    EguiMainWindow::set_dialog_open(&self.slot, kind, false)
                }
                _ => {}
            },
            WindowEvent::Resized(size) => {
                native.config.width = size.width.max(1);
                native.config.height = size.height.max(1);
                native.surface.configure(&self.gpu.device, &native.config);
            }
            WindowEvent::RedrawRequested => self.redraw(id),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.init_done {
            match self.init_rx.try_recv() {
                Ok(()) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.init_done = true;
                    let splash_ids: Vec<WindowId> = self
                        .windows
                        .iter()
                        .filter(|(_, native)| native.kind == WindowKind::Splash)
                        .map(|(id, _)| *id)
                        .collect();
                    for id in splash_ids {
                        self.windows.remove(&id);
                    }
                    self.add_window(event_loop, WindowKind::Launcher);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        self.ensure_project_windows(event_loop);
        if let Some(p) = self.slot.borrow().as_ref() {
            p.dialogs
                .borrow_mut()
                .sync_preview_requests(&p.state, &p.panel);
        }
        self.sync_dialog_windows(event_loop);
        let visible_ids: Vec<WindowId> = self
            .windows
            .iter()
            .filter(|(_, native)| native.window.is_visible().unwrap_or(true))
            .map(|(id, _)| *id)
            .collect();
        let any_visible = !visible_ids.is_empty();
        for id in visible_ids {
            self.redraw(id);
        }
        if any_visible {
            let _ = self.wake_proxy.send_event(AppWakeEvent::ContinueLoop);
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: AppWakeEvent) {}
}

pub fn run(
    gpu: Rc<SharedGpu>,
    slot: PreviewSlot,
    init_rx: std::sync::mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::<AppWakeEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let wake_proxy = event_loop.create_proxy();
    let mut app = EguiMainWindow::new(gpu, slot, init_rx, wake_proxy);
    event_loop.run_app(&mut app)?;
    Ok(())
}

pub(super) fn install_locale_fonts(ctx: &egui::Context) {
    egui_system_fonts::set_auto(ctx, egui_system_fonts::FontStyle::Sans);
}
