mod files_menu;
mod run_menu;

pub struct SettingsMenu {
    pub shown: bool,
}

impl SettingsMenu {
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.collapsing("Files", |ui| self.show_files_menu(ui));
        ui.collapsing("Edit Run", |ui| self.show_run_menu(ui));
    }

    pub fn new() -> Self {
        Self {
            shown: false,
        }
    }
}
