use std::{num::NonZeroU32, sync::Arc};
use egui::Color32;
use egui_glow::glow::{self, HasContext as _};
use glutin::prelude::*;
use winit::platform::x11::{WindowAttributesExtX11, WindowType};

pub struct OverlayWindow {
    pub window: winit::window::Window,
    pub gl_context: glutin::context::PossiblyCurrentContext,
    pub _gl_display: glutin::display::Display,
    pub gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub glow: Arc<glow::Context>,
    pub egui_glow: egui_glow::EguiGlow,
}

impl OverlayWindow {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        width: u32,
        height: u32,
        click_through: bool,
    ) -> Self {
        Self::new_with_bounds(event_loop, 0, 0, width, height, click_through)
    }

    pub fn new_with_bounds(
        event_loop: &winit::event_loop::ActiveEventLoop,
        pos_x: i32,
        pos_y: i32,
        width: u32,
        height: u32,
        click_through: bool,
    ) -> Self {
        use glutin::context::NotCurrentGlContext as _;
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::prelude::GlSurface as _;
        use winit::raw_window_handle::HasWindowHandle as _;

        let icon = image::load_from_memory(crate::WAYCORD_LOGO_BYTES)
            .ok()
            .and_then(|img| {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                winit::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
            });

        let mut winit_window_builder = winit::window::WindowAttributes::default()
            .with_decorations(false)
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .with_position(winit::dpi::PhysicalPosition::new(pos_x, pos_y))
            .with_resizable(false)
            .with_transparent(true)
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_override_redirect(true)
            .with_x11_window_type(vec![WindowType::Tooltip])
            .with_title("WayCord");

        if let Some(ic) = icon {
            winit_window_builder = winit_window_builder.with_window_icon(Some(ic));
        }

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(Some(true))
            .with_transparency(true);

        let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(winit_window_builder.clone()))
            .build(
                event_loop,
                config_template_builder,
                |mut config_iterator| {
                    config_iterator.next().expect(
                        "failed to find a matching configuration for creating glutin config",
                    )
                },
            )
            .expect("failed to create gl_config");

        let gl_display = gl_config.display();

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context even with fallback attributes")
                })
        };

        let window = window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(event_loop, winit_window_builder, &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (cur_width, cur_height): (u32, u32) = window.inner_size().into();
        let cur_width = NonZeroU32::new(cur_width).unwrap_or(NonZeroU32::MIN);
        let cur_height = NonZeroU32::new(cur_height).unwrap_or(NonZeroU32::MIN);

        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    cur_width,
                    cur_height,
                );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(&gl_context, glutin::surface::SwapInterval::DontWait)
            .unwrap();

        if click_through {
            let _ = window.set_cursor_hittest(false);
        }
        window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));

        let glow = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");
                gl_display.get_proc_address(&s)
            })
        };

        let glow = Arc::new(glow);
        let egui_glow = egui_glow::EguiGlow::new(event_loop, glow.clone(), None, None, true);

        Self {
            window,
            gl_context,
            _gl_display: gl_display,
            gl_surface,
            glow,
            egui_glow,
        }
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        let width = NonZeroU32::new(physical_size.width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(physical_size.height).unwrap_or(NonZeroU32::MIN);
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn render(&mut self, run_ui: impl FnMut(&mut egui::Ui)) {
        if let Err(err) = self.gl_context.make_current(&self.gl_surface) {
            eprintln!("Failed to make GL context current: {err}");
            return;
        }

        self.egui_glow.run(&self.window, run_ui);

        let [r, g, b, a] = Color32::TRANSPARENT.to_normalized_gamma_f32();
        unsafe {
            self.glow.clear_color(r, g, b, a);
            self.glow.clear(glow::COLOR_BUFFER_BIT);
        }

        self.egui_glow.paint(&self.window);

        if let Err(err) = self.gl_surface.swap_buffers(&self.gl_context) {
            eprintln!("Failed to swap buffers: {err}");
        }
    }

    pub fn x11_window_id(&self) -> Option<x11rb::protocol::xproto::Window> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let Ok(handle) = self.window.window_handle() {
            if let RawWindowHandle::Xlib(xlib) = handle.as_raw() {
                return Some(xlib.window as x11rb::protocol::xproto::Window);
            } else if let RawWindowHandle::Xcb(xcb) = handle.as_raw() {
                return Some(xcb.window.get());
            }
        }
        None
    }

    pub fn set_cursor_hittest(&self, hittest: bool) {
        let _ = self.window.set_cursor_hittest(hittest);
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        self.egui_glow.destroy();
    }
}
