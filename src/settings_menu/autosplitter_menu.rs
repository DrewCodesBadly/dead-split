use egui::Ui;
use livesplit_auto_splitting::settings::Value;
use rfd::FileDialog;

use crate::{settings_menu::{SettingsMenu, UpdateRequest}, timer_components::UpdateData, ConfigReferences};

impl SettingsMenu {
    pub fn show_autosplitters_menu(&mut self, ui: &mut Ui, update_data: &UpdateData, configs: &mut ConfigReferences) {
        ui.horizontal(|ui| {
            match self.autosplitter_path {
                Some(_) => ui.label("Found an autosplitter for this game."),
                None => ui.label("No autosplitter found for this game."),
            };
            if ui.button("Import an autosplitter for this game...").clicked() {
                if let Some(p) = FileDialog::new()
                    .add_filter("Autosplitters", &["wasm"])
                    .pick_file() {
                    self.update_requests.push(UpdateRequest::TryImportAutosplitter(p));
                }
            }
        });

        if ui.checkbox(&mut configs.autosplitter_config.enabled, "Enabled").changed() {
            self.update_requests.push(UpdateRequest::ToggleAutosplitterEnabled(configs.autosplitter_config.enabled));
        }

        if let Some(manager) = update_data.autosplitter_manager {
            let mut map = manager.settings_map();

            // Show settings menu.
            ui.label("Autosplitter loaded successfully, its settings are below.");
            ui.separator();
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
