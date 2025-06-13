use egui::Button;
use strum::IntoEnumIterator;

use crate::{hotkey_manager::HotkeyAction, settings_menu::UpdateRequest, timer_components::UpdateData};

use super::SettingsMenu;

impl SettingsMenu {
    pub fn show_hotkey_menu(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        // Toggle to switch between wayland/x11 hotkeys (using livesplit-core or tauri)
        let mut x11_compat = update_data.hotkey_manager.is_x11();
        if ui.checkbox(&mut x11_compat, "Use X11 Compatible Hotkeys").changed() {
            self.update_requests.push(UpdateRequest::ReloadHotkeyManager(x11_compat));
        }

        for val in HotkeyAction::iter() {
            ui.horizontal(|ui| {
                ui.label(val.to_string() + &":");
                ui.label(update_data.hotkey_manager.get_hotkey_string(val)
                    .unwrap_or("No hotkey".to_string()));
                if Some(val) == self.action_awaiting_bind {
                    // Show a disabled button
                    ui.add_enabled(false, Button::new("Awaiting input..."));
                    if let Some(key_str) = ui.input(|i| {
                        for event in &i.events {
                            match event {
                                egui::Event::Key { key, physical_key: _, pressed, repeat: _, modifiers } => {
                                    // Sort of wishy-washy key string setup here.
                                    // I'm basically just praying they key strings of all the
                                    // libraries are... close enough
                                    if *pressed {
                                        let mut out_str = String::new();
                                        if modifiers.ctrl {
                                            out_str += "Ctrl+";
                                        }
                                        if modifiers.alt {
                                            out_str += "Alt+";
                                        }
                                        if modifiers.shift {
                                            out_str += "Shift+";
                                        }
                                        if modifiers.mac_cmd {
                                            out_str += "Cmd+";
                                        }
                                        // modifiers.command does not seem useful here.
                                        out_str += key.name();
                                        return Some(out_str);
                                    }
                                },
                                _ => {},
                            }
                        }

                        return None;
                    }) {
                        self.update_requests.push(UpdateRequest::NewHotkeyBind(key_str, val));
                        self.action_awaiting_bind = None;
                    }
                } else { 
                    if ui.button("Rebind").clicked() {
                        self.action_awaiting_bind = Some(val);
                    }
                    if ui.button("Clear Hotkey").clicked() {
                        self.update_requests.push(UpdateRequest::ClearHotkey(val));
                    }
                }
            });
        }
    }
}
