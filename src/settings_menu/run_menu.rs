use std::{fs, path::PathBuf, str::FromStr};

use egui::DragValue;
use livesplit_core::{layout::parser, run::parser::{composite, parse}, timing::formatter::TimeFormatter, Run, Segment, TimeSpan};
use rfd::FileDialog;

use crate::{settings_menu::UpdateRequest, timer_components::UpdateData};

use super::SettingsMenu;

#[derive(Default)]
pub struct SplitMenuData {
    pub name: String,
    pub pb_igt_str: String,
    pub pb_rta_str: String,
    pub best_igt_str: String,
    pub best_rta_str: String,
}

#[derive(Default)]
pub struct RunMenuData {
    pub generated: bool,
    pub game_name_str: String,
    pub cat_name_str: String,
    pub attempt_val: u32,
    pub offset_str: String,
    pub split_data: Vec<SplitMenuData>,
    pub use_game_time_vals: bool,
}

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
                // TODO: Handle other file types correctly, prompting for a new path
                // if a non-lss file type is used.
                self.split_file_path = FileDialog::new()
                    .add_filter("Split files", &["lss"])
                    .pick_file();
                if let Some(p) = &self.split_file_path {
                    match try_load_run(p) {
                        Ok(run) => self.changed_run = Some(run),
                        Err(_) => {
                            self.changed_run = Some(crate::get_default_run());
                            self.run_menu_data.generated = false;
                            self.split_file_path = None;
                        }
                    }
                } else {
                    self.split_file_path = None;
                    self.run_menu_data.generated = false;
                    self.changed_run = Some(crate::get_default_run());
                }
            }
            if ui.button("Create new splits").clicked() {
                self.split_file_path = None;
                self.changed_run = Some(crate::get_default_run());
                self.run_menu_data.generated = false;
            }
        });

        if ui.button("Save splits to new file...").clicked() {
            if let Some(p) = FileDialog::new()
                .add_filter("Split files", &["lss"])
                .save_file() {
                self.split_file_path = Some(p);
                self.update_requests.push(UpdateRequest::SaveSplits);
            }
        }
        
        if let Some(run) = &mut self.changed_run {
            if !self.run_menu_data.generated {
                self.run_menu_data.generated = true;
                // Populate the run editor with the run data.
                self.run_menu_data.game_name_str = run.game_name().to_owned();
                self.run_menu_data.cat_name_str = run.category_name().to_owned();
                self.run_menu_data.attempt_val = run.attempt_count();
                self.run_menu_data.offset_str = self.time_formatter.format(run.offset())
                    .to_string();

                self.run_menu_data.split_data.clear();
                for split in run.segments() {
                    self.run_menu_data.split_data.push(SplitMenuData {
                        name: split.name().to_owned(),
                        pb_igt_str: self.time_formatter.format(
                            split.personal_best_split_time().game_time).to_string(),
                        pb_rta_str: self.time_formatter.format(
                            split.personal_best_split_time().real_time).to_string(),
                        best_igt_str: self.time_formatter.format(
                            split.best_segment_time().game_time).to_string(),
                        best_rta_str: self.time_formatter.format(
                            split.best_segment_time().real_time).to_string(),

                    });
                }
            }
            // Edit basic run data - game name, category name, etc
            ui.horizontal(|ui| {
                ui.label("Game name: ");
                if ui.text_edit_singleline(&mut self.run_menu_data.game_name_str).changed() {
                    run.set_game_name(self.run_menu_data.game_name_str.to_owned());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Category name: ");
                if ui.text_edit_singleline(&mut self.run_menu_data.cat_name_str).changed() {
                    run.set_category_name(self.run_menu_data.cat_name_str.to_owned());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Attempt count: ");
                if ui.add(DragValue::new(&mut self.run_menu_data.attempt_val)).changed() {
                    run.set_attempt_count(self.run_menu_data.attempt_val);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Timer offset: ");
                if ui.text_edit_singleline(&mut self.run_menu_data.offset_str).lost_focus() {
                    if let Ok(t) = TimeSpan::from_str(&self.run_menu_data.offset_str) {
                        run.set_offset(t);
                    }
                    self.run_menu_data.offset_str = self.time_formatter.format(run.offset())
                        .to_string();
                }
            });

            // Split editor
            ui.checkbox(&mut self.run_menu_data.use_game_time_vals, "Show game time values");
            if (ui.button("Add New Segment")).clicked() {
                run.push_segment(Segment::new("New Segment"));
                self.run_menu_data.split_data.push(SplitMenuData {
                    name: "New Segment".to_owned(),
                    ..Default::default()
                });
            }

            for i in 0..self.run_menu_data.split_data.len() {
                let segment = run.segment_mut(i);
                ui.separator();
                if let Some(data) = self.run_menu_data.split_data.get_mut(i) {
                    if ui.text_edit_singleline(&mut data.name).lost_focus() {
                        segment.set_name(data.name.to_owned());
                    }
                    if self.run_menu_data.use_game_time_vals {
                        ui.horizontal(|ui| {
                            ui.label("PB Split Time (IGT)");
                            if ui.text_edit_singleline(&mut data.pb_igt_str).lost_focus() {
                                if let Ok(t) = TimeSpan::from_str(&data.pb_igt_str) {
                                    segment.personal_best_split_time_mut().game_time = Some(t);
                                }
                                data.pb_igt_str = self.time_formatter.format(segment.personal_best_split_time().game_time)
                                    .to_string();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Best Split Time (IGT)");
                            if ui.text_edit_singleline(&mut data.best_igt_str).lost_focus() {
                                if let Ok(t) = TimeSpan::from_str(&data.best_igt_str) {
                                    segment.best_segment_time_mut().game_time = Some(t);
                                }
                                data.best_igt_str = self.time_formatter.format(segment.best_segment_time().game_time)
                                    .to_string();
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("PB Split Time (RTA)");
                            if ui.text_edit_singleline(&mut data.pb_rta_str).lost_focus() {
                                if let Ok(t) = TimeSpan::from_str(&data.pb_rta_str) {
                                    segment.personal_best_split_time_mut().real_time = Some(t);
                                }
                                data.pb_rta_str = self.time_formatter.format(segment.personal_best_split_time().real_time)
                                    .to_string();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Best Split Time (RTA)");
                            if ui.text_edit_singleline(&mut data.best_rta_str).lost_focus() {
                                if let Ok(t) = TimeSpan::from_str(&data.best_rta_str) {
                                    segment.best_segment_time_mut().real_time = Some(t);
                                }
                                data.best_rta_str = self.time_formatter.format(segment.best_segment_time().real_time)
                                    .to_string();
                            }
                        });
                    }
                    
                    // These need to be delayed, so we're using UpdateRequest
                    if ui.button("Move Segment Up").clicked() {
                        self.update_requests.push(UpdateRequest::MoveSegmentUp(i));
                    }
                    if ui.button("Move Segment Down").clicked() {
                        self.update_requests.push(UpdateRequest::MoveSegmentDown(i));
                    }
                    if ui.button("Remove Segment").clicked() {
                        self.update_requests.push(UpdateRequest::RemoveSegment(i));
                    }
                }
                ui.separator();
            }
            if ui.button("Save changes").clicked() {
                self.update_requests.push(UpdateRequest::SaveSplits);
            }
        } else {
            if ui.button("Edit splits data...").clicked() {
                self.changed_run = Some(update_data.run.clone());
            }

            // TODO: Show all the run data, view-only
        }
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
