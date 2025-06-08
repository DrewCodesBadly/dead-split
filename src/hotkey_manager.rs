use std::{collections::HashMap, str::FromStr, sync::{Arc, RwLock}};

use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use livesplit_core::hotkey::{Hook, Hotkey};

pub struct HotkeyManager {
    wayland_hook: Option<Hook>,
    x11_manager: Option<GlobalHotKeyManager>,
    string_map: HashMap<i32, String>,
    key_map: HashMap<u32, i32>, // Only used with X11
    last_pressed_index: Arc<RwLock<Option<i32>>>, // Only used with Wayland
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

    pub fn bind_key(&mut self, key_string: String, hotkey_idx: i32) -> Result<(), ()> {
        if let Some(hook) = &self.wayland_hook {
            match Hotkey::from_str(&key_string) {
                Ok(hotkey) => {
                    self.string_map.insert(hotkey_idx, key_string);
                    let arc = self.last_pressed_index.clone();
                    let _ = hook.register(hotkey, move || {
                    let mut bind = arc.try_write().unwrap();
                    *bind = Some(hotkey_idx);
                    });
                }
                Err(_) => {
                    return Err(());
                }
            }
        } else if let Some(manager) = &self.x11_manager {
            match HotKey::from_str(&key_string) {
                Ok(hotkey) => {
                    self.string_map.insert(hotkey_idx, key_string);
                    self.key_map.insert(hotkey.id, hotkey_idx);
                    let _ = manager.register(hotkey);
                }
                _ => return Err(()),
            }
        }

        Ok(())
    }

    pub fn remove_key(&mut self, hotkey_id: i32) -> Result<(), ()> {
        let key_string = match self.string_map.get(&hotkey_id) {
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
    pub fn poll_keypress(&mut self) -> Option<i32> {
        if self.wayland_hook.is_some() {
            let last_pressed_index: Option<i32>;
            // Make sure the lock is dropped before we try writing to it later.
            {
                last_pressed_index = *self.last_pressed_index.try_read().unwrap();
            }
            if let Some(idx) = last_pressed_index {
                // Clear the last pressed hotkey
                let mut bind = self.last_pressed_index.try_write().unwrap();
                *bind = None;
                return Some(idx)
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

    pub fn get_hotkey_string(&self, hotkey_id: i32) -> Option<String> {
        self.string_map.get(&hotkey_id).cloned()
    }
}
