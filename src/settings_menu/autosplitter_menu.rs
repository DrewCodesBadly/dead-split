use egui::Ui;
use livesplit_auto_splitting::settings::Value;
use rfd::FileDialog;

use crate::{settings_menu::{SettingsMenu, UpdateRequest}, timer_components::UpdateData};

impl SettingsMenu {
    pub fn show_autosplitters_menu(&mut self, ui: &mut Ui, update_data: &UpdateData) {
        // File options.
        ui.horizontal(|ui| {
            let path_string = match &self.autosplitter_path {
                Some(p) => p.to_str().unwrap_or("None"),
                None => "None",
            };
            ui.label("Active autosplitter: ".to_owned() + path_string);
            if ui.button("Open file...").clicked() {
                self.autosplitter_path = FileDialog::new()
                    // TODO: add filter for every supported file type
                    // would have to check which types are supported.
                    .add_filter("Autosplitters", &["wasm"])
                    .pick_file();
                self.update_requests.push(UpdateRequest::ReloadAutosplitter);
            }
            if ui.button("Clear autosplitter").clicked() {
                self.autosplitter_path = None;
                self.update_requests.push(UpdateRequest::ReloadAutosplitter);
            }
        });

        if let Some(manager) = update_data.autosplitter_manager {
            let mut map = manager.settings_map();

            // Show settings menu.
            for widget in manager.settings_widgets().iter() {
                match widget.kind {
                    livesplit_auto_splitting::settings::WidgetKind::Bool { default_value } => {
                        let mut val = map.get(&widget.key).and_then(|v| v.to_bool()).unwrap_or(default_value);
                        if ui.checkbox(&mut val, widget.key.as_ref()).changed() {
                            map.insert(widget.key.to_owned(), Value::Bool(val));
                        }
                    }

                    // TODO: Support more setting types
                    _ => {
                        ui.label("<unsupported setting type>");
                    },
                }
            }
            manager.set_settings_map(map);
        }
    }
}
