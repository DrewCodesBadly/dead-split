use std::{fs, path::PathBuf};

use livesplit_core::{layout::parser, run::parser::{composite, parse}, Run};
use rfd::FileDialog;

use crate::timer_components::UpdateData;

use super::SettingsMenu;

impl SettingsMenu {
    pub fn show_run_menu(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        // File options.
        ui.horizontal(|ui| {
            let path_string = match &self.split_file_path {
                Some(p) => p.to_str().unwrap_or("None"),
                None => "None",
            };
            ui.label("Active Splits File: ".to_owned() + path_string);
            if ui.button("Open file...").clicked() {
                self.split_file_path = FileDialog::new()
                    // TODO: add filter for every supported file type
                    // would have to check which types are supported.
                    .pick_file();
                if let Some(p) = &self.split_file_path {
                    match try_load_run(p) {
                        Ok(run) => self.changed_run = Some(run),
                        Err(_) => {
                            self.changed_run = Some(crate::get_default_run());
                            self.split_file_path = None;
                        }
                    }
                } else {
                    self.split_file_path = None;
                    self.changed_run = Some(crate::get_default_run());
                }
            }
            if ui.button("Create new splits").clicked() {
                self.split_file_path = None;
                self.changed_run = Some(crate::get_default_run());
            }
        });
    }
}

fn try_load_run(p: &PathBuf) -> Result<Run, std::io::Error> {
    let file = match fs::read(p) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    match composite::parse(file.as_slice(), Some(p.as_path())) {
        Ok(parsed) => return Ok(parsed.run),
        Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "invalid splits file")),
    }
}
