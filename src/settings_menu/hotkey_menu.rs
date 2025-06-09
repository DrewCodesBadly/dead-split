use strum::IntoEnumIterator;

use crate::{hotkey_manager::HotkeyAction, settings_menu::HotkeyReloadData, timer_components::UpdateData, DeadSplit};

use super::SettingsMenu;

impl SettingsMenu {
    pub fn show_hotkey_menu(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        for val in HotkeyAction::iter() {
            ui.horizontal(|ui| {
                ui.label(val.to_string() + &":");
                ui.label(update_data.hotkey_manager.get_hotkey_string(val)
                    .unwrap_or("No hotkey".to_string()));
                if Some(val) == self.action_awaiting_bind {
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
                        if let Some(data) = &mut self.hotkey_reload_data {
                            data.new_bind = Some((key_str, val));
                        } else {
                            self.hotkey_reload_data = Some(HotkeyReloadData {
                                clear: None,
                                new_bind: Some((key_str, val)),
                            });
                        } 
                        self.action_awaiting_bind = None;
                    }
                } else if ui.button("Rebind").clicked() {
                    self.action_awaiting_bind = Some(val);
                }
            });
        }
    }
}
