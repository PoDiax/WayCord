use std::time::{Duration, Instant};

use waycord::{
    ThemePalette, VoiceUser, apply_theme,
    discord_bridge::DiscordBridge,
    overlay_window::OverlayWindow,
    presets::OverlayConfig,
    render_voice_overlay,
    window_finder::X11Tracker,
};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BannerState {
    Connecting,
    ConnectedUntil(Instant),
    Dismissed,
}

struct OverlayApp {
    overlay: Option<OverlayWindow>,
    palette: ThemePalette,
    start_time: Instant,
    next_frame_time: Instant,
    click_through: bool,
    force_demo: bool,
    tracker: Option<X11Tracker>,
    bridge: DiscordBridge,
    mock_users: Vec<VoiceUser>,
    config: OverlayConfig,
    alt_held: bool,
    show_settings: bool,
    screen_width: f32,
    screen_height: f32,
    monitors: Vec<waycord::presets::MonitorBounds>,
    tray_state: waycord::tray::TrayState,
    _tray_handle: Option<ksni::blocking::Handle<waycord::tray::WayCordTray>>,
    is_visible: bool,
    banner_state: BannerState,
}

impl OverlayApp {
    fn new(click_through: bool, force_demo: bool) -> Self {
        let config = OverlayConfig::load();
        let palette = ThemePalette::from_kind(config.theme);
        let tracker = X11Tracker::new();
        let bridge = DiscordBridge::start();
        let tray_state = waycord::tray::TrayState::new();
        tray_state.only_speaking.store(config.only_speaking, std::sync::atomic::Ordering::Relaxed);
        let tray_handle = tray_state.start_tray();

        Self {
            overlay: None,
            palette,
            start_time: Instant::now(),
            next_frame_time: Instant::now(),
            click_through,
            force_demo,
            tracker,
            bridge,
            mock_users: vec![
                VoiceUser {
                    id: "demo_podiax".to_string(),
                    name: "PoDiax".to_string(),
                    avatar_initials: "PD".to_string(),
                    avatar_url: Some("https://cdn.discordapp.com/embed/avatars/1.png".to_string()),
                    is_speaking: true,
                    is_muted: false,
                    is_deafened: false,
                    volume_level: 0.8,
                },
                VoiceUser {
                    id: "demo_ghost".to_string(),
                    name: "Ghost".to_string(),
                    avatar_initials: "GH".to_string(),
                    avatar_url: Some("https://cdn.discordapp.com/embed/avatars/2.png".to_string()),
                    is_speaking: false,
                    is_muted: false,
                    is_deafened: false,
                    volume_level: 0.0,
                },
                VoiceUser {
                    id: "demo_viper".to_string(),
                    name: "Viper".to_string(),
                    avatar_initials: "VP".to_string(),
                    avatar_url: Some("https://cdn.discordapp.com/embed/avatars/3.png".to_string()),
                    is_speaking: false,
                    is_muted: true,
                    is_deafened: false,
                    volume_level: 0.0,
                },
                VoiceUser {
                    id: "demo_echo".to_string(),
                    name: "Echo".to_string(),
                    avatar_initials: "EC".to_string(),
                    avatar_url: Some("https://cdn.discordapp.com/embed/avatars/4.png".to_string()),
                    is_speaking: false,
                    is_muted: false,
                    is_deafened: true,
                    volume_level: 0.0,
                },
            ],
            config,
            alt_held: false,
            show_settings: false,
            screen_width: 2560.0,
            screen_height: 1440.0,
            monitors: Vec::new(),
            tray_state,
            _tray_handle: tray_handle,
            is_visible: true,
            banner_state: BannerState::Connecting,
        }
    }

    fn update_mock_simulation(&mut self, elapsed: f64) {
        let cycle = (elapsed % 6.0) as f32;
        if cycle < 3.0 {
            self.mock_users[0].is_speaking = true;
            self.mock_users[0].volume_level = ((elapsed * 8.0).sin().abs() as f32 * 0.7) + 0.3;
            self.mock_users[1].is_speaking = false;
        } else {
            self.mock_users[0].is_speaking = false;
            self.mock_users[1].is_speaking = true;
            self.mock_users[1].volume_level = ((elapsed * 10.0).cos().abs() as f32 * 0.8) + 0.2;
        }
    }
}

impl ApplicationHandler for OverlayApp {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.next_frame_time += Duration::from_millis(16);
            let now = Instant::now();
            if self.next_frame_time < now {
                self.next_frame_time = now + Duration::from_millis(16);
            }
        }

        if cause == StartCause::Poll || matches!(cause, StartCause::WaitCancelled { .. }) || matches!(cause, StartCause::ResumeTimeReached { .. }) {
            if self.tray_state.exit_requested.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Quit requested via System Tray. Exiting...");
                event_loop.exit();
                return;
            }

            if self.tray_state.toggle_settings_requested.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.show_settings = !self.show_settings;
                println!("Settings toggled ");
            }

            if self.tray_state.toggle_visibility_requested.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.is_visible = !self.is_visible;
                self.tray_state.is_visible.store(self.is_visible, std::sync::atomic::Ordering::Relaxed);
                println!("Overlay visibility toggled via System Tray (visible={})", self.is_visible);
            }

            if self.tray_state.reset_position_requested.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.config.x = 32.0;
                self.config.y = 48.0;
                self.config.save();
                println!("Overlay position reset via System Tray to (32, 48)");
            }

            if self.tray_state.reset_size_requested.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.config.reset_size();
                println!("Overlay size reset via System Tray to defaults");
            }

            if self.tray_state.toggle_only_speaking_requested.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.config.only_speaking = !self.config.only_speaking;
                self.config.save();
                self.tray_state.only_speaking.store(self.config.only_speaking, std::sync::atomic::Ordering::Relaxed);
                println!("Show Only Speaking toggled via System Tray ({})", self.config.only_speaking);
            }
            self.tray_state.only_speaking.store(self.config.only_speaking, std::sync::atomic::Ordering::Relaxed);

            let alt_is_pressed = self
                .tracker
                .as_ref()
                .map(|t| t.is_alt_pressed())
                .unwrap_or(false);
            if alt_is_pressed != self.alt_held {
                self.alt_held = alt_is_pressed;
                if self.click_through {
                    if let Some(overlay) = &self.overlay {
                        overlay.set_cursor_hittest(self.alt_held);
                    }
                    if self.alt_held {
                        println!("Overlay interactive mode (ALT held) - Drag/Resize/Settings enabled");
                    } else {
                        println!("Overlay click-through restored (Position & Size saved)");
                        self.config.save();
                        self.show_settings = false;
                    }
                }
            }

            let target_palette = ThemePalette::from_kind(self.config.theme);
            if self.palette.name != target_palette.name {
                self.palette = target_palette;
                if let Some(overlay) = &self.overlay {
                    apply_theme(&overlay.egui_glow.egui_ctx, &self.palette, 8.0);
                }
            }

            let elapsed = self.start_time.elapsed().as_secs_f64();
            if self.force_demo {
                self.update_mock_simulation(elapsed);
            }

            let discord_lock = self.bridge.state.lock().ok();
            let connected = discord_lock.as_ref().map(|d| d.connected).unwrap_or(false);
            let has_real_channel = discord_lock.as_ref().and_then(|d| d.channel_name.clone());
            let real_users = discord_lock
                .as_ref()
                .map(|d| d.users.clone())
                .unwrap_or_default();
            let current_user = discord_lock.as_ref().and_then(|d| d.current_user.clone());

            let now = Instant::now();
            match self.banner_state {
                BannerState::Connecting => {
                    if connected {
                        self.banner_state = BannerState::ConnectedUntil(now + Duration::from_secs(4));
                    }
                }
                BannerState::ConnectedUntil(until) => {
                    if now >= until {
                        self.banner_state = BannerState::Dismissed;
                    }
                }
                BannerState::Dismissed => {}
            }

            let in_call = has_real_channel.is_some() || !real_users.is_empty();

            let (users_to_draw, channel_title, should_render) = {
                if self.force_demo {
                    (
                        self.mock_users.clone(),
                        "Demo Simulation".to_string(),
                        true,
                    )
                } else if in_call {
                    (
                        real_users,
                        has_real_channel.unwrap_or_else(|| "Voice Call".to_string()),
                        true,
                    )
                } else {
                    let status_text = match &current_user {
                        Some(name) if !name.is_empty() => format!("Connected to Discord as {}", name),
                        _ => "Connected to Discord".to_string(),
                    };

                    match self.banner_state {
                        BannerState::Connecting => {
                            let dot_count = (elapsed * 2.5) as usize % 4;
                            let dots = match dot_count {
                                0 => "",
                                1 => ".",
                                2 => "..",
                                _ => "...",
                            };
                            (
                                Vec::new(),
                                format!("Connecting to Discord{}", dots),
                                true,
                            )
                        }
                        BannerState::ConnectedUntil(_) => {
                            (
                                Vec::new(),
                                status_text,
                                true,
                            )
                        }
                        BannerState::Dismissed => {
                            (
                                Vec::new(),
                                status_text,
                                self.alt_held,
                            )
                        }
                    }
                }
            };

            self.tray_state.discord_connected.store(connected, std::sync::atomic::Ordering::Relaxed);
            if let Ok(mut ch) = self.tray_state.channel_name.write() {
                *ch = channel_title.clone();
            }
            if let Ok(mut u) = self.tray_state.current_user.write() {
                *u = current_user;
            }

            if let Some(overlay) = &mut self.overlay {
                if self.is_visible && should_render {
                    let palette = &self.palette;
                    let config = &mut self.config;
                    let show_settings = &mut self.show_settings;
                    let force_demo = &mut self.force_demo;
                    let alt_held = self.alt_held;
                    let screen_width = self.screen_width;
                    let screen_height = self.screen_height;
                    let avatar_cache = &self.bridge.avatar_cache;
                    let monitors = &self.monitors;

                    overlay.render(|ui| {
                        render_voice_overlay(
                            ui.ctx(),
                            palette,
                            &users_to_draw,
                            &channel_title,
                            elapsed,
                            config,
                            alt_held,
                            show_settings,
                            screen_width,
                            screen_height,
                            connected,
                            force_demo,
                            avatar_cache,
                            monitors,
                        );
                    });
                } else {
                    overlay.render(|_| {});
                }
            }

            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut monitors = Vec::new();
        let mut min_x = 0i32;
        let mut min_y = 0i32;
        let mut max_x = 0i32;
        let mut max_y = 0i32;

        for monitor in event_loop.available_monitors() {
            let pos = monitor.position();
            let size = monitor.size();
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x + size.width as i32);
            max_y = max_y.max(pos.y + size.height as i32);
            monitors.push(waycord::presets::MonitorBounds::new(
                pos.x as f32,
                pos.y as f32,
                size.width as f32,
                size.height as f32,
            ));
        }

        if monitors.is_empty() {
            let (width, height) = if let Some(monitor) = event_loop.primary_monitor() {
                let size = monitor.size();
                (size.width, size.height)
            } else {
                (2560, 1440)
            };
            monitors.push(waycord::presets::MonitorBounds::new(
                0.0,
                0.0,
                width as f32,
                height as f32,
            ));
            max_x = width as i32;
            max_y = height as i32;
        }

        let total_width = (max_x - min_x).max(640) as u32;
        let total_height = (max_y - min_y).max(480) as u32;
        self.screen_width = total_width as f32;
        self.screen_height = total_height as f32;
        self.monitors = monitors;

        let overlay = OverlayWindow::new_with_bounds(
            event_loop,
            min_x,
            min_y,
            total_width,
            total_height,
            self.click_through,
        );
        apply_theme(&overlay.egui_glow.egui_ctx, &self.palette, 8.0);

        println!("====================================================");
        println!("WayCord - Discord Overlay Running!");
        println!("Overlay resolution: {}x{} at ({}, {})", total_width, total_height, min_x, min_y);
        println!("Detected monitors: {}", self.monitors.len());
        for (i, m) in self.monitors.iter().enumerate() {
            println!(
                "  [Monitor {}] {}x{} at ({}, {}) [midpoint: {}]",
                i + 1,
                m.width,
                m.height,
                m.x,
                m.y,
                m.midpoint_x()
            );
        }
        println!("Click-through default: {}", self.click_through);
        println!("Interactive mode: HOLD [ALT] at any time in-game");
        println!("Saved Position: ({}, {}), Size: {}", self.config.x, self.config.y, self.config.avatar_size);
        println!("Display Style: {:?}", self.config.style);
        println!("Listening for Discord: ws://127.0.0.1:9876");
        println!("Press Ctrl+C in this terminal to close the overlay.");
        println!("====================================================");

        self.overlay = Some(overlay);

        self.next_frame_time = Instant::now() + Duration::from_millis(16);
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(overlay) = &mut self.overlay {
            let _ = overlay.egui_glow.on_window_event(&overlay.window, &event);
            match &event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    self.screen_width = size.width as f32;
                    self.screen_height = size.height as f32;
                    overlay.resize(*size);
                }
                _ => {}
            }
        }
    }
}

fn main() {
    use winit::platform::x11::EventLoopBuilderExtX11;

    let args: Vec<String> = std::env::args().collect();
    let click_through = !args.iter().any(|arg| arg == "--interactive");
    let force_demo = args.iter().any(|arg| arg == "--demo");

    let event_loop = EventLoop::builder()
        .with_x11()
        .build()
        .expect("failed to build winit event loop");

    let mut app = OverlayApp::new(click_through, force_demo);
    event_loop.run_app(&mut app).expect("event loop error");
}
