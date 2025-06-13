use rfd::FileDialog;

use crate::{settings_menu::UpdateRequest, timer_components::UpdateData};

use super::SettingsMenu;

impl SettingsMenu {
    pub fn show_profiles_menu(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        ui.horizontal(|ui| {
            let path_string = match &update_data.directory_config.active_profile {
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
            }
        });

        ui.label("Directories to search for profiles:");
        for (idx, dir) in update_data.directory_config.known_directories.iter().enumerate() {
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
