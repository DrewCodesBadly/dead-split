use rfd::FileDialog;

use crate::{settings_menu::UpdateRequest, timer_components::UpdateData, ConfigReferences};

use super::SettingsMenu;

impl SettingsMenu {
    pub fn show_profiles_menu(&mut self, ui: &mut egui::Ui, configs: &mut ConfigReferences) {
        ui.horizontal(|ui| {
            let path_string = match &configs.global_config.active_profile {
                Some(p) => p.to_str().unwrap_or("None"),
                None => "None",
            };
            ui.label("Active Profile: ".to_string() + path_string);
            if ui.button("Open file...").clicked() {
                if let Some(p) = FileDialog::new()
                    .add_filter("Profiles", &["zip"])
                    .pick_file() {
                    self.update_requests.push(UpdateRequest::LoadProfile(p));
                }
            }
            if ui.button("Save to new file...").clicked() {
                configs.global_config.active_profile = FileDialog::new()
                    .add_filter("ZIP Profiles", &["zip"])
                    .save_file();
                self.update_requests.push(UpdateRequest::SaveGlobalConfig);
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Load profile from file...").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("ZIP Profiles", &["zip"])
                    .pick_file() {
                    configs.global_config.active_profile = Some(path);
                    self.update_requests.push(UpdateRequest::RestartTimer);
                }
            }
            ui.weak("The timer will restart.");
        });

        ui.label("Directories to search for profiles:");
        for (idx, dir) in configs.global_config.known_directories.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(dir.to_str().unwrap_or("<invalid path>"));
                if ui.button("Remove").clicked() {
                    self.update_requests.push(UpdateRequest::RemoveKnownDirectory(idx));
                }
            });
        }
        
        if ui.button("Add directory...").clicked() {
            if let Some(p) = FileDialog::new()
                .pick_folder() {
                self.update_requests.push(UpdateRequest::AddKnownDirectory(p));
            }
        }
    }
}
