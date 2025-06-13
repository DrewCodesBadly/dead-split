use std::{collections::HashMap, fmt::Display, str::FromStr, sync::{Arc, RwLock}};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use livesplit_core::hotkey::{Hook, Hotkey};
use strum::EnumIter;

#[derive(PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum HotkeyAction {
    StartSplit,
    Pause,
    Unpause,
    TogglePause,
    Reset,
    OpenSettings,
    ToggleTimingMethod,
}

impl Display for HotkeyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotkeyAction::StartSplit => f.write_str("Start/Split"),
            HotkeyAction::Pause => f.write_str("Pause"),
            HotkeyAction::Unpause => f.write_str("Unpause"),
            HotkeyAction::TogglePause => f.write_str("Toggle Paused"),
            HotkeyAction::Reset => f.write_str("Reset"),
            HotkeyAction::OpenSettings => f.write_str("Open Settings Window"),
            HotkeyAction::ToggleTimingMethod => f.write_str(
                "Toggle Timing Method (Switches IGT/RTA)",
            ),
        }
    }
}

pub struct HotkeyManager {
    wayland_hook: Option<Hook>,
    x11_manager: Option<GlobalHotKeyManager>,
    string_map: HashMap<HotkeyAction, String>,
    key_map: HashMap<u32, HotkeyAction>, // Only used with X11
    last_pressed_index: Arc<RwLock<Option<HotkeyAction>>>, // Only used with Wayland
}

impl HotkeyManager {
    pub fn new_wayland(hook: Hook) -> Self {
        Self {
            wayland_hook: Some(hook),
            x11_manager: None,
            string_map: HashMap::new(),
            key_map: HashMap::new(),
            last_pressed_index: Arc::new(None.into()),
        }
    }

    pub fn new_x11(manager: GlobalHotKeyManager) -> Self {
        Self {
            wayland_hook: None,
            x11_manager: Some(manager),
            string_map: HashMap::new(),
            key_map: HashMap::new(),
            last_pressed_index: Arc::new(None.into()),
        } 
    }

    pub fn new_with_type(&self, x11: bool) -> Self {
        if x11 {
            Self {
                wayland_hook: None,
                x11_manager: Some(GlobalHotKeyManager::new()
                    .expect("Failed to create global hotkey manager.")),
                string_map: self.string_map.clone(),
                key_map: self.key_map.clone(),
                last_pressed_index: Arc::new(None.into()),
            }
        } else {
            Self {
                wayland_hook: Some(Hook::new()
                    .expect("Failed to create global hotkey hook.")),
                x11_manager: None,
                string_map: self.string_map.clone(),
                key_map: self.key_map.clone(),
                last_pressed_index: Arc::new(None.into()),
            }
        }
    }

    pub fn setup_old_keys(&mut self) {
        for (k, v) in self.string_map.clone() {
            let _ = self.bind_key(v.to_owned(), k.clone());
        }
    }

    pub fn bind_key(&mut self, key_string: String, action: HotkeyAction) -> Result<(), ()> {
        if let Some(hook) = &self.wayland_hook {
            match Hotkey::from_str(&key_string) {
                Ok(hotkey) => {
                    self.string_map.insert(action, key_string);
                    let arc = self.last_pressed_index.clone();
                    let _ = hook.register(hotkey, move || {
                    let mut bind = arc.try_write().unwrap();
                    *bind = Some(action);
                    });
                }
                Err(_) => {
                    return Err(());
                }
            }
        } else if let Some(manager) = &self.x11_manager {
            match HotKey::from_str(&key_string) {
                Ok(hotkey) => {
                    self.string_map.insert(action, key_string);
                    self.key_map.insert(hotkey.id, action);
                    let _ = manager.register(hotkey);
                }
                _ => return Err(()),
            }
        }

        Ok(())
    }

    pub fn remove_key(&mut self, action: HotkeyAction) -> Result<(), ()> {
        let key_string = match self.string_map.remove(&action) {
            Some(s) => s,
            None => return Err(()),
        };
        if let Some(hook) = &self.wayland_hook {
            if let Ok(key) = Hotkey::from_str(&key_string) {
                hook.unregister(key).map_err(|_| ())?
            }
        } else if let Some(manager) = &self.x11_manager {
            if let Ok(key) = HotKey::from_str(&key_string) {
                manager.unregister(key).map_err(|_| ())?
            }
        }
        Err(())
    }

    // Called every frame by the timer. Checks for a key press.
    // If a hotkey is pressed, it's associated ID is returned.
    pub fn poll_keypress(&mut self) -> Option<HotkeyAction> {
        if self.wayland_hook.is_some() {
            let last_pressed_index: Option<HotkeyAction>;
            // Make sure the lock is dropped before we try writing to it later.
            {
                last_pressed_index = *self.last_pressed_index.try_read().unwrap();
            }
            if let Some(action) = last_pressed_index {
                // Clear the last pressed hotkey
                let mut bind = self.last_pressed_index.try_write().unwrap();
                *bind = None;
                return Some(action)
            }
        }
        else {
            if let Ok(e) = GlobalHotKeyEvent::receiver().try_recv() {
                if e.state() == HotKeyState::Pressed {
                    if let Some(idx) = self.key_map.get(&e.id()) {
                        return Some(*idx)
                    }
                }
            }
        }

        None
    }

    pub fn get_hotkey_string(&self, action: HotkeyAction) -> Option<String> {
        self.string_map.get(&action).cloned()
    }

    pub fn is_x11(&self) -> bool {
        return self.x11_manager.is_some();
    }
}
