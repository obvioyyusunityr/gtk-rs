use gtk::glib;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

pub type Shared = Rc<RefCell<State>>;
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)] // missing fields (older files) fall back to defaults
pub struct State {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub sidebar_visible: bool,
    pub page: String,
    pub dark: Option<bool>, // None = follow the system
    pub devtools: bool,
    pub zoom: f64,
    pub accounts: Vec<String>,
    pub account: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            width: 800,
            height: 500,
            maximized: false,
            sidebar_visible: true,
            page: "home".into(),
            dark: None,
            devtools: true,
            zoom: 1.0,
            accounts: Vec::new(),
            account: None,
        }
    }
}

fn path() -> PathBuf {
    glib::user_config_dir()
        .join("gtk-sidebar")
        .join("state.json")
}

pub fn load() -> Shared {
    let state = fs::read_to_string(path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Rc::new(RefCell::new(state))
}

pub fn save(state: &State) {
    let p = path();
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(p, json);
    }
}
