use core::fmt;
use std::{collections::HashMap, fs::{self, File}, io::{BufWriter, Cursor, Read, Write}, path::PathBuf, str::FromStr, sync::{RwLockReadGuard, RwLockWriteGuard}};

use autosplitter_manager::AutosplitterManager;
use directories::ProjectDirs;
use eframe::{run_native, App, NativeOptions};
use egui::{CentralPanel, Vec2, ViewportBuilder, Window, WindowLevel};
use hotkey_manager::HotkeyManager;
use livesplit_core::{hotkey::Hook, run::saver::livesplit::{self, IoWrite}, Run, Segment, SharedTimer, Timer, TimerPhase};
use read_process_memory::ProcessHandle;
use serde::{Deserialize, Serialize};
use settings_menu::SettingsMenu;
use sysinfo::Pid;
use timer_components::{split_component::{SplitComponent, SplitComponentConfig}, RunTimerComponent, TimerComponent, TitleComponent, UpdateData};
use zip::{result::ZipError, write::SimpleFileOptions, ZipWriter};

use crate::{autosplitter_manager::{AutosplitterConfig, SerializableSettingValue}, hotkey_manager::HotkeyAction, timer_components::{TimerComponentConfig, TimerComponentType, TitleComponentConfig}};

mod hotkey_manager;
mod autosplitter_manager;
mod timer_components;
mod settings_menu;

#[derive(Debug)]
pub enum ProfileSaveError {
    NoPath,
    CannotCreateFile(std::io::Error),
    ZipWriterError(ZipError),
    TomlSerializeError(toml::ser::Error),
    LivesplitSaveError(fmt::Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct GlobalConfig {
    pub active_profile: Option<PathBuf>,
    pub known_directories: Vec<PathBuf>,
    pub autosave_splits: bool,
}

#[derive(Serialize, Deserialize)]
struct TimerConfig {
    pub components: Vec<TimerComponentType>,
}


struct ConfigReferences<'a> {
    pub timer_config: &'a mut TimerConfig,
    pub global_config: &'a mut GlobalConfig,
    pub autosplitter_config: &'a mut AutosplitterConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            active_profile: None,
            known_directories: Vec::new(),
            autosave_splits: true,
        }
    }
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            components: vec![
                TimerComponentType::TitleComponent(TitleComponentConfig::default()),
                TimerComponentType::SplitComponent(SplitComponentConfig::default()),
                TimerComponentType::TimerComponent(TimerComponentConfig::default()),
            ]
        }
    }
}

struct DeadSplit {
    // TODO: avoid unwraps?
    timer: SharedTimer,
    hotkey_mgr: HotkeyManager,
    autosplitter_manager: Option<AutosplitterManager>,

    components: Vec<Box<dyn TimerComponent>>,
    settings_menu: SettingsMenu,

    notification_text: String,
    notification_active: f32,

    global_config: GlobalConfig,
    timer_config: TimerConfig,
    autosplitter_config: AutosplitterConfig,
}

impl Default for DeadSplit {
    fn default() -> Self {
        let timer = Timer::new(get_default_run())
            .expect("failed to create new splits")
            .into_shared();
        let timer_config = TimerConfig::default();
        let mut components: Vec<Box<dyn TimerComponent>> = Vec::new();
        for comp_type in &timer_config.components {
            components.push(
                match comp_type {
                    TimerComponentType::TitleComponent(cfg) => {
                        Box::new(TitleComponent::new(timer_read(&timer).run(), cfg))
                    }
                    TimerComponentType::SplitComponent(cfg) => {
                        Box::new(SplitComponent::new(timer_read(&timer).run(), cfg))
                    }
                    TimerComponentType::TimerComponent(cfg) => {
                        Box::new(RunTimerComponent::new(cfg))
                    }
                }
            );
        }
        Self {
            timer: timer,
            hotkey_mgr: HotkeyManager::new_wayland(Hook::new().expect("Failed to create hotkey hook")),
            autosplitter_manager: None,
            // TODO: Load these instead of hardcoding them
            // components: Vec::new(),
            components,
            settings_menu: SettingsMenu::new(),
            notification_text: String::new(),
            notification_active: 0.0,
            global_config: GlobalConfig::default(),
            timer_config,
            autosplitter_config: AutosplitterConfig::default(),
        } 
    }
}

impl DeadSplit {
    pub fn reload_components(&mut self) {
        let mut new_comps: Vec<Box<dyn TimerComponent>> = Vec::new();

        for comp_type in &self.timer_config.components {
            new_comps.push(
                match comp_type {
                    TimerComponentType::TitleComponent(cfg) => {
                        Box::new(TitleComponent::new(timer_read(&self.timer).run(), cfg))
                    }
                    TimerComponentType::SplitComponent(cfg) => {
                        Box::new(SplitComponent::new(timer_read(&self.timer).run(), cfg))
                    }
                    TimerComponentType::TimerComponent(cfg) => {
                        Box::new(RunTimerComponent::new(cfg))
                    }
                }
            );
        }

        self.components = new_comps;
    }

    pub fn try_save_profile_zip(&self, timer: &Timer) -> Result<(), ProfileSaveError> {
        let path = self.global_config.active_profile.as_ref()
            .ok_or(ProfileSaveError::NoPath)?;
        let file = File::create(path).map_err(|e| ProfileSaveError::CannotCreateFile(e))?;
        let options = SimpleFileOptions::default();
        let mut zip = ZipWriter::new(file);

        zip.start_file("timer_config.toml", options)
            .map_err(|e| ProfileSaveError::ZipWriterError(e))?;
        zip.write(toml::to_string(&self.timer_config)
            .map_err(|e| ProfileSaveError::TomlSerializeError(e))?
            .as_bytes())
            .map_err(|e| ProfileSaveError::ZipWriterError(ZipError::Io(e)))?;

        zip.start_file("splits.lss", options)
            .map_err(|e| ProfileSaveError::ZipWriterError(e))?;
        livesplit::save_run(timer.run(), IoWrite(&mut zip))
            .map_err(|e| ProfileSaveError::LivesplitSaveError(e))?;

        let hotkey_cfg = self.hotkey_mgr.get_hotkey_config();
        zip.start_file("hotkeys.toml", options)
            .map_err(|e| ProfileSaveError::ZipWriterError(e))?;
        zip.write(toml::to_string(&hotkey_cfg)
            .map_err(|e| ProfileSaveError::TomlSerializeError(e))?
            .as_bytes())
            .map_err(|e| ProfileSaveError::ZipWriterError(ZipError::Io(e)))?;

        // Update the autosplitter config if needed.
        zip.start_file("autosplitters.toml", options)
            .map_err(|e| ProfileSaveError::ZipWriterError(e))?;
        zip.write(toml::to_string(&self.autosplitter_config)
            .map_err(|e| ProfileSaveError::TomlSerializeError(e))?
            .as_bytes())
            .map_err(|e| ProfileSaveError::ZipWriterError(ZipError::Io(e)))?;

        zip.finish().map_err(|e| ProfileSaveError::ZipWriterError(e))?;

        Ok(())
    }

    fn update_autosplitter_config_settings(&mut self) {
        if let Some(mgr) = &self.autosplitter_manager {
            let settings_map = mgr.settings_map();
            let mut new_map: HashMap<String, SerializableSettingValue> = HashMap::new();
            for (k, v) in settings_map.iter() {
                match v {
                    // TODO: Support nested settings
                    livesplit_auto_splitting::settings::Value::Map(_) => {},
                    livesplit_auto_splitting::settings::Value::List(_) => {},
                    livesplit_auto_splitting::settings::Value::Bool(b) => {
                        new_map.insert(k.to_owned(), SerializableSettingValue::Bool(*b));
                    }
                    livesplit_auto_splitting::settings::Value::I64(i) => {
                        new_map.insert(k.to_owned(), SerializableSettingValue::I64(*i));
                    }
                    livesplit_auto_splitting::settings::Value::F64(f) => {
                        new_map.insert(k.to_owned(), SerializableSettingValue::F64(*f));
                    }
                    livesplit_auto_splitting::settings::Value::String(s) => {
                        new_map.insert(k.to_owned(), SerializableSettingValue::StringValue((*s).to_string()));
                    }
                    _ => {},
                }
            }
            self.autosplitter_config.settings = new_map;
        }
    }
}

// Wrapper for sysinfo::Process and read_process_memory::ProcessHandle since we really need both or none
pub struct ProcessData {
    pub handle: ProcessHandle,
    pub pid: Pid,
}

pub fn timer_read(t: &SharedTimer) -> RwLockReadGuard<'_, Timer> {
    t.read().unwrap()
}

pub fn timer_write(t: &SharedTimer) -> RwLockWriteGuard<'_, Timer> {
    t.write().unwrap()
}

fn get_default_run() -> Run {
    let mut run = Run::new();
    run.push_segment(Segment::new("Time"));
    run.set_game_name("Game");
    run.set_category_name("Any%");
    run
}

impl App for DeadSplit {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut reload_components_flag = false;
        let mut save_global_cfg_flag = false;
        let mut save_profile_flag = false;
        // Scope during which the timer binding lives
        // This can block attempts to change other stuff, like modifying the timer setup.
        {
            let mut binding = timer_write(&self.timer);
            if !binding.is_game_time_initialized() {
                binding.initialize_game_time();
            }
            let update_data = UpdateData {
                snapshot: binding.snapshot(),
                run: binding.run(),
                hotkey_manager: &self.hotkey_mgr,
                autosplitter_manager: &self.autosplitter_manager,
            };

            CentralPanel::default().show(ctx, |ui| {
                for component in &mut self.components {
                    component.show(ui, &update_data);
                    ui.separator();
                }
                if ui.input(|i| i.pointer.secondary_clicked()) {
                    self.settings_menu.shown = true;
                }
            });

            if self.settings_menu.shown {
                let mut configs = ConfigReferences {
                    global_config: &mut self.global_config,
                    timer_config: &mut self.timer_config,
                    autosplitter_config: &mut self.autosplitter_config,
                };

                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("DeadSplit_settings"),
                    egui::ViewportBuilder::default()
                        .with_title("DeadSplit Settings")
                        .with_inner_size([500.0, 500.0]), 
                    |ctx, class| {
                        assert!(class == egui::ViewportClass::Immediate, "multiple viewports not supported");
                        egui::CentralPanel::default().show(ctx, |ui| {
                            self.settings_menu.show(ctx, ui, &update_data, &mut configs);
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            self.settings_menu.shown = false;
                        }
                    },
                );

                // Read any changes that were made and need to be fixed here.
                for request in self.settings_menu.update_requests.drain(0..) {
                    match request {
                        settings_menu::UpdateRequest::ReloadHotkeyManager(x11_hotkeys) => {
                                                self.hotkey_mgr = self.hotkey_mgr.new_with_type(x11_hotkeys);
                                                self.hotkey_mgr.setup_old_keys();
                                            }
                        settings_menu::UpdateRequest::ClearHotkey(hotkey_action) => {
                                                let _ = self.hotkey_mgr.remove_key(hotkey_action);
                                            }
                        settings_menu::UpdateRequest::NewHotkeyBind(string, hotkey_action) => {
                                                let _ = self.hotkey_mgr.bind_key(string, hotkey_action);
                                            }
                        settings_menu::UpdateRequest::LoadProfile(path_buf) => {
                            self.global_config.active_profile = Some(path_buf);
                            save_global_cfg_flag = true;
                            // TODO
                        }
                        settings_menu::UpdateRequest::RemoveKnownDirectory(idx) => {
                            self.global_config.known_directories.remove(idx);
                            save_global_cfg_flag = true;
                        }
                        settings_menu::UpdateRequest::AddKnownDirectory(path_buf) => {
                            if !self.global_config.known_directories.contains(&path_buf) {
                                self.global_config.known_directories.push(path_buf);
                                save_global_cfg_flag = true;
                            }
                        }
                        settings_menu::UpdateRequest::SaveGlobalConfig => {
                            save_global_cfg_flag = true;
                        }
                        settings_menu::UpdateRequest::RemoveComponent(idx) => {
                            let _ = self.timer_config.components.remove(idx);
                        }
                        settings_menu::UpdateRequest::MoveComponentDown(idx) => {
                            if idx < self.timer_config.components.len() - 1 {
                                self.timer_config.components.swap(idx, idx + 1);
                            }
                        }
                        settings_menu::UpdateRequest::MoveComponentUp(idx) => {
                            if idx > 0 && self.timer_config.components.len() > 1 {
                                self.timer_config.components.swap(idx, idx - 1);
                            }
                        }
                        settings_menu::UpdateRequest::RemoveSegment(idx) => {
                            if let Some(run) = &mut self.settings_menu.changed_run {
                                if run.len() > 1 {
                                    let _ = run.segments_mut().remove(idx);
                                }
                            }
                        }
                        settings_menu::UpdateRequest::MoveSegmentDown(idx) => {
                            if let Some(run) = &mut self.settings_menu.changed_run {
                                if idx < run.segments().len() - 1 {
                                    run.segments_mut().swap(idx, idx + 1);
                                }
                            }
                        }
                        settings_menu::UpdateRequest::MoveSegmentUp(idx) => {
                            if let Some(run) = &mut self.settings_menu.changed_run {
                                if idx > 0 && run.segments().len() > 1 {
                                    run.segments_mut().swap(idx, idx - 1);
                                }
                            }
                        }
                        settings_menu::UpdateRequest::CheckGameForAutosplitter(name) =>  {
                            if let Some(dirs) = ProjectDirs::from("com", "DrewCodesBadly", "DeadSplit") {
                                let game_name = to_snake_case(&name);
                                let mut path = dirs.preference_dir().to_path_buf();
                                path.push(game_name + ".wasm");
                                if path.exists() {
                                    self.settings_menu.autosplitter_path = Some(path);
                                }
                            }
                        }
                        settings_menu::UpdateRequest::TryImportAutosplitter(path) => {
                            let mgr = AutosplitterManager::new(self.timer.clone(), &path).ok();
                            // If it loaded successfully, save it as a known autosplitter.
                            if mgr.is_some() {
                                let game_name = to_snake_case(binding.run().game_name());
                                if let Some(dirs) = ProjectDirs::from("com", "DrewCodesBadly", "DeadSplit") {
                                    let mut new_path = dirs.preference_dir().to_path_buf();
                                    new_path.push(game_name + ".wasm");
                                    let _ = fs::copy(path, &new_path);
                                    self.settings_menu.autosplitter_path = Some(new_path);
                                }

                                if self.autosplitter_config.enabled {
                                    self.autosplitter_manager = mgr;
                                }
                            }
                        }
                        settings_menu::UpdateRequest::ToggleAutosplitterEnabled(enabled) => {
                            if enabled {
                                if let Some(p) = &self.settings_menu.autosplitter_path {
                                    self.autosplitter_manager = AutosplitterManager::new(
                                        self.timer.clone(),
                                        p
                                    ).ok();
                                }
                            } else {
                                self.autosplitter_manager = None;
                            }
                        }
                    }
                }

                // Reloads the timer once the settings menu closes.
                if !self.settings_menu.shown {
                    if let Some(run) = &self.settings_menu.changed_run {
                        let _ = binding.replace_run(run.clone(), true);
                        self.settings_menu.changed_run = None;
                    }
                    save_profile_flag = true;
                    reload_components_flag = true;
                }
            }
        }

        // Check for hotkey presses
        // Must happen AFTER visual updates so it does not interfere with the immutable snapshot.
        if let Some(action) = self.hotkey_mgr.poll_keypress() {
            let mut binding = timer_write(&self.timer);
            match action {
                hotkey_manager::HotkeyAction::StartSplit => {
                    binding.split_or_start();
                    if binding.current_phase() == TimerPhase::Ended {
                        reload_components_flag = true;
                    }
                }
                hotkey_manager::HotkeyAction::Pause => binding.pause(),
                hotkey_manager::HotkeyAction::Unpause => binding.resume(),
                hotkey_manager::HotkeyAction::TogglePause => binding.toggle_pause(),
                // who in their right mind wants to reset without updating splits
                // just go fix it after if it's wrong??
                hotkey_manager::HotkeyAction::Reset => {
                    binding.reset(true);
                    reload_components_flag = true;
                    if self.global_config.autosave_splits {
                        let _ = self.try_save_profile_zip(&binding);
                    }
                },
                hotkey_manager::HotkeyAction::OpenSettings => self.settings_menu.shown = true,
                hotkey_manager::HotkeyAction::ToggleTimingMethod => {
                    binding.toggle_timing_method();
                    self.notification_text = match binding.current_timing_method() {
                        livesplit_core::TimingMethod::RealTime => "Timing Method: Real Time".to_string(),
                        livesplit_core::TimingMethod::GameTime => "Timing Method: Game Time".to_string(),
                    };
                    self.notification_active = 3.0;
                }
            }
        }

        // Extra reloading - done after the timer binding is no longer needed.
        if reload_components_flag {
            self.reload_components();
        }

        // kinda jank error ignoring.
        if save_global_cfg_flag {
            if let Ok(data) = toml::to_string(&self.global_config) {
                if let Some(path) = get_project_save_dir() {
                    if !path.exists() {
                        let _ = fs::create_dir_all(path.parent().unwrap());
                    }
                    let file = File::create(path);
                   let _ = file.inspect(|mut f| { let _ = f.write_all(data.as_bytes()); });
                }
            }
        }

        if save_profile_flag {
            self.update_autosplitter_config_settings();
            let _ = self.try_save_profile_zip(&timer_read(&self.timer));
        }

        // Notifications
        if self.notification_active > 0.0 {
            Window::new("notification")
                .order(egui::Order::Foreground)
                .resizable(false)
                .interactable(false)
                .fade_in(true)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2 { x: 0.0, y: 0.0 })
                .show(ctx, |ui| {
                    self.notification_active -= ui.input(|i| i.stable_dt);
                    ui.label(&self.notification_text);
                });
        }


        // Constantly redraw - might be able to time this a little better,
        // but the timer needs to repaint 24/7 anyway.
        ctx.request_repaint();
    }
}

fn main() {
    // Detect saved data.
    let global_config: GlobalConfig = {
        if let Some(path) = get_project_save_dir() {
            if let Ok(s) = fs::read_to_string(path) {
                toml::from_str(&s).unwrap_or(GlobalConfig::default())
            } else {
                GlobalConfig::default()
            }
        } else {
            GlobalConfig::default()
        }
    };
    
    // TODO: Different types of windows
    let window_options = NativeOptions {
        viewport: ViewportBuilder {
            transparent: Some(true),
            window_level: Some(WindowLevel::AlwaysOnTop),
            ..Default::default()
        },
        ..Default::default()
    };

    run_native(
        "DeadSplit", 
        window_options, 
        Box::new(|cc| {
            let app = DeadSplit {
                global_config,
                ..Default::default()
            };
            Ok(Box::new(app))
        }),
    ).expect("failed to open window");
}

fn get_project_save_dir() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "DrewCodesBadly", "DeadSplit");
    if let Some(dirs) = project_dirs {
        let mut path = dirs.preference_dir().to_path_buf();
        path.push("paths.toml");
        Some(path)
    } else {
        None
    }
}

fn to_snake_case(s: &str) -> String {
    s.to_lowercase().replace(" ", "_")
}
