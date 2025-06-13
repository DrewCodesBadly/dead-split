use egui::Ui;

use crate::{settings_menu::{SettingsMenu, UpdateRequest}, timer_components::UpdateData, ConfigReferences};

impl SettingsMenu {
    pub fn show_global_settings_menu(&mut self, ui: &mut Ui, update_data: &UpdateData, configs: &mut ConfigReferences) {
        if ui.checkbox(&mut configs.global_config.autosave_splits, "Autosave splits on run reset").changed() {
            self.update_requests.push(UpdateRequest::SaveGlobalConfig);
        }
    }
}
