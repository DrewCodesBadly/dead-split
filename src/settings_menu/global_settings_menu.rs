use egui::Ui;

use crate::{settings_menu::{SettingsMenu, UpdateRequest}, timer_components::UpdateData};

impl SettingsMenu {
    pub fn show_global_settings_menu(&mut self, ui: &mut Ui, update_data: &UpdateData) {
        let mut autosave = update_data.global_config.autosave_splits;
        if ui.checkbox(&mut autosave, "Autosave splits on run reset").changed() {
            self.update_requests.push(UpdateRequest::SetAutosaveSplits(autosave));
        }
    }
}
