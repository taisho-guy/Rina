use super::window_kind::WindowKind;
use crate::gpu_shared::SharedGpu;
use egui_wgpu::Renderer as EguiRenderer;
use egui_wgpu::wgpu;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub(super) const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub(super) struct NativeWindow {
    pub(super) kind: WindowKind,
    pub(super) window: Arc<Window>,
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) ctx: egui::Context,
    pub(super) state: egui_winit::State,
    pub(super) renderer: EguiRenderer,
    pub(super) visible: bool,
}

impl NativeWindow {
    pub(super) fn create(event_loop: &ActiveEventLoop, gpu: &SharedGpu, kind: WindowKind) -> Self {
        let (width, height) = kind.size();
        Self::create_sized(event_loop, gpu, kind, width, height)
    }

    pub(super) fn create_sized(
        event_loop: &ActiveEventLoop,
        gpu: &SharedGpu,
        kind: WindowKind,
        width: u32,
        height: u32,
    ) -> Self {
        let mut attrs = Window::default_attributes()
            .with_title(kind.title())
            .with_inner_size(winit::dpi::LogicalSize::new(width as f64, height as f64));
        if kind == WindowKind::Splash {
            attrs = attrs
                .with_decorations(false)
                .with_resizable(false)
                .with_transparent(true);
        }
        if let Some((min_w, min_h)) = kind.min_size() {
            attrs =
                attrs.with_min_inner_size(winit::dpi::LogicalSize::new(min_w as f64, min_h as f64));
        }

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("eguiウィンドウ生成失敗"),
        );

        let surface = gpu
            .instance
            .create_surface(window.clone())
            .expect("wgpu Surface生成失敗");
        let caps = surface.get_capabilities(&gpu.adapter);
        let alpha_mode = if kind == WindowKind::Splash {
            if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
            {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
            {
                wgpu::CompositeAlphaMode::PostMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            }
        } else {
            wgpu::CompositeAlphaMode::Auto
        };
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: SURFACE_FORMAT,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);

        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        egui_extras::install_image_loaders(&ctx);
        crate::theme::install(&ctx);
        super::handler::install_locale_fonts(&ctx);
        {
            let redraw_ctx = ctx.clone();
            neoutl_media_runtime::cache::global()
                .set_redraw_handle(std::sync::Arc::new(move || redraw_ctx.request_repaint()));
        }
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = EguiRenderer::new(&gpu.device, SURFACE_FORMAT, Default::default());
        Self {
            kind,
            window,
            surface,
            config,
            ctx,
            state,
            renderer,
            visible: true,
        }
    }

    pub(super) fn redraw(
        &mut self,
        gpu: &SharedGpu,
        draw: impl FnOnce(&mut egui::Ui, &mut EguiRenderer),
    ) {
        if !self.visible {
            return;
        }
        crate::theme::install(&self.ctx);
        let raw_input = self.state.take_egui_input(&self.window);
        let mut draw = Some(draw);
        let output = self.ctx.run_ui(raw_input, |ui| {
            if let Some(draw) = draw.take() {
                draw(ui, &mut self.renderer);
            }
        });
        self.state
            .handle_platform_output(&self.window, output.platform_output);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => {
                self.surface.configure(&gpu.device, &self.config);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(&gpu.device, &gpu.queue, *id, delta);
            }
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("egui-window-encoder"),
            });
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: output.pixels_per_point,
        };
        self.renderer
            .update_buffers(&gpu.device, &gpu.queue, &mut encoder, &primitives, &screen);
        let clear_color = if self.kind == WindowKind::Splash {
            wgpu::Color::TRANSPARENT
        } else {
            let bg = self.ctx.style_of(self.ctx.theme()).visuals.panel_fill;
            wgpu::Color {
                r: bg.r() as f64 / 255.0,
                g: bg.g() as f64 / 255.0,
                b: bg.b() as f64 / 255.0,
                a: 1.0,
            }
        };
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui-window-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.renderer.render(&mut pass, &primitives, &screen);
        }
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        gpu.queue.submit(Some(encoder.finish()));
        gpu.queue.present(frame);
        if self.ctx.has_requested_repaint() {
            self.window.request_redraw();
        }
    }
}
