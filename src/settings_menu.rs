use std::{any::Any, path::PathBuf};

use egui::ScrollArea;
use livesplit_core::{timing::formatter::{self, Accuracy}, Run};

use crate::{hotkey_manager::HotkeyAction, settings_menu::run_menu::RunMenuData, timer_components::UpdateData};

mod profiles_menu;
mod run_menu;
mod hotkey_menu;
mod autosplitter_menu;

pub enum UpdateRequest {
    ReloadHotkeyManager(bool),
    ClearHotkey(HotkeyAction),
    NewHotkeyBind(String, HotkeyAction),
    ReloadAutosplitter,
    LoadProfile(PathBuf),
    RemoveKnownDirectory(usize),
    AddKnownDirectory(PathBuf),
}

#[derive(Default)]
pub struct SettingsMenu {
    pub run_menu_data: RunMenuData,
    pub shown: bool,
    pub action_awaiting_bind: Option<HotkeyAction>,
    pub changed_run: Option<Run>,
    pub split_file_path: Option<PathBuf>,
    pub time_formatter: formatter::Regular,
    pub autosplitter_path: Option<PathBuf>,

    pub update_requests: Vec<UpdateRequest>,
}

impl SettingsMenu {
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, update_data: &UpdateData) {
        ScrollArea::both().show(ui, |ui| {
            ui.collapsing("Profiles", |ui| self.show_profiles_menu(ui, update_data));
            ui.collapsing("Edit Run", |ui| self.show_run_menu(ui, update_data));
            ui.collapsing("Hotkeys", |ui| self.show_hotkey_menu(ui, update_data));
            ui.collapsing("Autosplitters", |ui| self.show_autosplitters_menu(ui, update_data));
        });
    }

    pub fn new() -> Self {
        Self {
            time_formatter: formatter::Regular::with_accuracy(Accuracy::Milliseconds),
            ..Default::default()
        }
    }
}
