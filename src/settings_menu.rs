use crate::{hotkey_manager::HotkeyAction, timer_components::UpdateData, DeadSplit};

mod profiles_menu;
mod run_menu;
mod hotkey_menu;

#[derive(Default)]
pub struct HotkeyReloadData {
    pub clear: Option<HotkeyAction>,
    pub new_bind: Option<(String, HotkeyAction)>,
}

#[derive(Default)]
pub struct SettingsMenu {
    pub shown: bool,
    pub needs_layout_reload: bool,
    pub hotkey_reload_data: Option<HotkeyReloadData>,
    pub action_awaiting_bind: Option<HotkeyAction>,
    pub needs_run_reload: bool,
}

impl SettingsMenu {
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, update_data: &UpdateData) {
        ui.collapsing("Profiles", |ui| self.show_profiles_menu(ui));
        ui.collapsing("Edit Run", |ui| self.show_run_menu(ui));
        ui.collapsing("Hotkeys", |ui| self.show_hotkey_menu(ui, update_data));
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
