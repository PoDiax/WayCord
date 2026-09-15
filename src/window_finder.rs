use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

#[derive(Clone, Debug, PartialEq)]
pub struct TargetWindow {
    pub id: Window,
    pub title: String,
    pub class_name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub is_borderless: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowSummary {
    pub id: Window,
    pub title: String,
    pub class_name: String,
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub is_borderless: bool,
}

pub struct X11Tracker {
    pub conn: RustConnection,
    pub root: Window,
    pub net_wm_name: Atom,
    pub net_wm_state: Atom,
    pub net_wm_state_fullscreen: Atom,
    pub net_wm_state_hidden: Atom,
    pub net_wm_state_focused: Atom,
    pub net_active_window: Atom,
    pub net_client_list: Atom,
    pub net_wm_bypass_compositor: Atom,
    pub steam_game: Atom,
}

impl X11Tracker {
    pub fn new() -> Option<Self> {
        let (conn, screen_num) = x11rb::connect(None).ok()?;
        let root = conn.setup().roots[screen_num].root;

        let net_wm_name = Self::intern(&conn, b"_NET_WM_NAME")?;
        let net_wm_state = Self::intern(&conn, b"_NET_WM_STATE")?;
        let net_wm_state_fullscreen = Self::intern(&conn, b"_NET_WM_STATE_FULLSCREEN")?;
        let net_wm_state_hidden = Self::intern(&conn, b"_NET_WM_STATE_HIDDEN")?;
        let net_wm_state_focused = Self::intern(&conn, b"_NET_WM_STATE_FOCUSED")?;
        let net_active_window = Self::intern(&conn, b"_NET_ACTIVE_WINDOW")?;
        let net_client_list = Self::intern(&conn, b"_NET_CLIENT_LIST")?;
        let net_wm_bypass_compositor = Self::intern(&conn, b"_NET_WM_BYPASS_COMPOSITOR")?;
        let steam_game = Self::intern(&conn, b"STEAM_GAME")?;

        Some(Self {
            conn,
            root,
            net_wm_name,
            net_wm_state,
            net_wm_state_fullscreen,
            net_wm_state_hidden,
            net_wm_state_focused,
            net_active_window,
            net_client_list,
            net_wm_bypass_compositor,
            steam_game,
        })
    }

    fn intern(conn: &RustConnection, name: &[u8]) -> Option<Atom> {
        conn.intern_atom(false, name)
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    }

    pub fn get_active_window(&self) -> Option<Window> {
        if let Ok(reply) = self.conn.get_property(
            false,
            self.root,
            self.net_active_window,
            AtomEnum::WINDOW,
            0,
            1,
        ) {
            if let Ok(prop) = reply.reply() {
                if let Some(win) = prop.value32().and_then(|mut it| it.next()) {
                    if win != 0 && win != self.root {
                        return Some(win);
                    }
                }
            }
        }
        None
    }

    pub fn get_client_list(&self) -> Vec<Window> {
        if let Ok(reply) = self.conn.get_property(
            false,
            self.root,
            self.net_client_list,
            AtomEnum::WINDOW,
            0,
            1024,
        ) {
            if let Ok(prop) = reply.reply() {
                let wins: Vec<Window> = prop.value32().map(|it| it.collect()).unwrap_or_default();
                if !wins.is_empty() {
                    return wins;
                }
            }
        }
        if let Ok(cookie) = self.conn.query_tree(self.root) {
            if let Ok(tree) = cookie.reply() {
                return tree.children;
            }
        }
        Vec::new()
    }

    pub fn get_window_title_and_class(&self, win: Window) -> (String, String) {
        let mut title = String::new();
        let mut class_name = String::new();

        if let Ok(reply) =
            self.conn
                .get_property(false, win, self.net_wm_name, AtomEnum::ANY, 0, 1024)
        {
            if let Ok(prop) = reply.reply() {
                if !prop.value.is_empty() {
                    title = String::from_utf8_lossy(&prop.value).trim().to_string();
                }
            }
        }

        if title.is_empty() {
            if let Ok(reply) =
                self.conn
                    .get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
            {
                if let Ok(prop) = reply.reply() {
                    if !prop.value.is_empty() {
                        title = String::from_utf8_lossy(&prop.value).trim().to_string();
                    }
                }
            }
        }

        if let Ok(reply) =
            self.conn
                .get_property(false, win, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
        {
            if let Ok(prop) = reply.reply() {
                if !prop.value.is_empty() {
                    let parts: Vec<String> = prop
                        .value
                        .split(|&b| b == 0)
                        .filter(|p| !p.is_empty())
                        .map(|p| String::from_utf8_lossy(p).to_string())
                        .collect();
                    class_name = parts.join(" ");
                }
            }
        }

        (title, class_name)
    }

    pub fn get_window_states(&self, win: Window) -> Vec<Atom> {
        if let Ok(reply) =
            self.conn
                .get_property(false, win, self.net_wm_state, AtomEnum::ATOM, 0, 64)
        {
            if let Ok(prop) = reply.reply() {
                return prop.value32().map(|it| it.collect()).unwrap_or_default();
            }
        }
        Vec::new()
    }

    pub fn is_steam_game(&self, win: Window, class_name: &str) -> bool {
        if class_name.to_lowercase().contains("steam_app_") {
            return true;
        }
        if let Ok(reply) =
            self.conn
                .get_property(false, win, self.steam_game, AtomEnum::CARDINAL, 0, 1)
        {
            if let Ok(prop) = reply.reply() {
                if !prop.value.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_bypass_compositor(&self, win: Window) -> bool {
        if let Ok(reply) = self.conn.get_property(
            false,
            win,
            self.net_wm_bypass_compositor,
            AtomEnum::CARDINAL,
            0,
            1,
        ) {
            if let Ok(prop) = reply.reply() {
                if let Some(val) = prop.value32().and_then(|mut it| it.next()) {
                    return val == 1;
                }
            }
        }
        false
    }

    pub fn find_target(&self, query: &str) -> Option<TargetWindow> {
        let query_lower = query.to_lowercase();
        let clients = self.get_client_list();

        for win in clients.into_iter().rev() {
            let (title, class_name) = self.get_window_title_and_class(win);
            if title.to_lowercase().contains(&query_lower)
                || class_name.to_lowercase().contains(&query_lower)
            {
                let geom = self.conn.get_geometry(win).ok()?.reply().ok()?;
                let coords = self
                    .conn
                    .translate_coordinates(win, self.root, 0, 0)
                    .ok()?
                    .reply()
                    .ok()?;
                let states = self.get_window_states(win);
                let is_fullscreen = states.contains(&self.net_wm_state_fullscreen);

                return Some(TargetWindow {
                    id: win,
                    title: if !title.is_empty() { title } else { class_name.clone() },
                    class_name,
                    x: coords.dst_x as i32,
                    y: coords.dst_y as i32,
                    width: geom.width as u32,
                    height: geom.height as u32,
                    is_fullscreen,
                    is_borderless: false,
                });
            }
        }
        None
    }

    pub fn list_open_windows(&self, exclude_win: Option<Window>) -> Vec<WindowSummary> {
        let mut list = Vec::new();
        let clients = self.get_client_list();
        let root_geom = self.conn.get_geometry(self.root).ok().and_then(|c| c.reply().ok());

        for win in clients {
            if Some(win) == exclude_win || win == self.root {
                continue;
            }

            let states = self.get_window_states(win);
            if states.contains(&self.net_wm_state_hidden) {
                continue;
            }

            let (title, class_name) = self.get_window_title_and_class(win);
            if title.is_empty() && class_name.is_empty() {
                continue;
            }

            let lower_class = class_name.to_lowercase();
            if lower_class.contains("plasmashell")
                || lower_class.contains("krunner")
                || lower_class.contains("waybar")
                || lower_class.contains("polybar")
                || lower_class.contains("xwaylandvideobridge")
                || title.to_lowercase().contains("discord voice overlay")
            {
                continue;
            }

            if let Ok(geom_reply) = self.conn.get_geometry(win) {
                if let Ok(geom) = geom_reply.reply() {
                    if geom.width < 200 || geom.height < 150 {
                        continue;
                    }

                    let is_fullscreen = states.contains(&self.net_wm_state_fullscreen);
                    let is_borderless = root_geom
                        .as_ref()
                        .map(|rg| {
                            geom.width >= (rg.width - 4) as u16
                                && geom.height >= (rg.height - 4) as u16
                        })
                        .unwrap_or(false);

                    list.push(WindowSummary {
                        id: win,
                        title: if !title.is_empty() { title } else { class_name.clone() },
                        class_name,
                        width: geom.width as u32,
                        height: geom.height as u32,
                        is_fullscreen,
                        is_borderless,
                    });
                }
            }
        }

        list
    }

    pub fn raise_above(&self, overlay_win: Window, target_win: Window) {
        let aux = ConfigureWindowAux::new()
            .sibling(target_win)
            .stack_mode(StackMode::ABOVE);
        let _ = self.conn.configure_window(overlay_win, &aux);
        let _ = self.conn.flush();
    }

    pub fn raise_top(&self, overlay_win: Window) {
        let aux = ConfigureWindowAux::new().stack_mode(StackMode::ABOVE);
        let _ = self.conn.configure_window(overlay_win, &aux);
        let _ = self.conn.flush();
    }

    pub fn is_alt_pressed(&self) -> bool {
        if let Ok(cookie) = self.conn.query_keymap() {
            if let Ok(reply) = cookie.reply() {
                let alt_l = (reply.keys[64 / 8] & (1 << (64 % 8))) != 0;
                let alt_r = (reply.keys[108 / 8] & (1 << (108 % 8))) != 0;
                return alt_l || alt_r;
            }
        }
        false
    }
}
