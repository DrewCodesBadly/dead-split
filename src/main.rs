use std::{fs::{self, File}, io::Write, path::PathBuf, str::FromStr, sync::{RwLockReadGuard, RwLockWriteGuard}};

use autosplitter_manager::AutosplitterManager;
use directories::ProjectDirs;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Vec2, ViewportBuilder, Window, WindowLevel};
use hotkey_manager::HotkeyManager;
use livesplit_core::{hotkey::Hook, Run, Segment, SharedTimer, Timer, TimerPhase};
use read_process_memory::ProcessHandle;
use serde::{Deserialize, Serialize};
use settings_menu::SettingsMenu;
use sysinfo::Pid;
use timer_components::{split_component::{SplitComponent, SplitComponentConfig}, RunTimerComponent, RunTimerConfig, TimerComponent, TitleComponent, UpdateData};

mod hotkey_manager;
mod autosplitter_manager;
mod timer_components;
mod settings_menu;

#[derive(Serialize, Deserialize, Default, Debug)]
struct DirectoryConfig {
    pub active_profile: Option<PathBuf>,
    pub known_directories: Vec<PathBuf>,
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

    directory_config: DirectoryConfig,
}

impl DeadSplit {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let timer = Timer::new(get_default_run())
            .expect("failed to create new splits")
            .into_shared();
        let title_comp = TitleComponent::new(timer_read(&timer).run());
        let split_comp = SplitComponent::new(SplitComponentConfig::default(), timer_read(&timer).run());
        Self {
            timer: timer,
            hotkey_mgr: HotkeyManager::new_wayland(Hook::new().expect("Failed to create hotkey hook")),
            autosplitter_manager: None,
            // TODO: Load these instead of hardcoding them
            // components: Vec::new(),
            components: vec![
                Box::new(title_comp),
                Box::new(split_comp), 
                Box::new(RunTimerComponent::new(RunTimerConfig::default()))
            ],
            settings_menu: SettingsMenu::new(),
            notification_text: String::new(),
            notification_active: 0.0,
            directory_config: DirectoryConfig::default(),
        }
    }

    pub fn reload_components(&mut self) {
        let title_comp = TitleComponent::new(timer_read(&self.timer).run());
        let timer_comp = RunTimerComponent::new(RunTimerConfig::default());
        let split_comp = SplitComponent::new(SplitComponentConfig::default(), timer_read(&self.timer).run());
        self.components = vec![
            Box::new(title_comp), Box::new(split_comp), Box::new(timer_comp),
        ];
    }

    pub fn set_directory_config(&mut self, cfg: DirectoryConfig) {
        self.directory_config = cfg;
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
        let mut save_dirs_flag = false;
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
                directory_config: &self.directory_config,
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
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("DeadSplit_settings"),
                    egui::ViewportBuilder::default()
                        .with_title("DeadSplit Settings")
                        .with_inner_size([500.0, 500.0]), 
                    |ctx, class| {
                        assert!(class == egui::ViewportClass::Immediate, "multiple viewports not supported");
                        egui::CentralPanel::default().show(ctx, |ui| {
                            self.settings_menu.show(ctx, ui, &update_data);
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
                        settings_menu::UpdateRequest::ReloadAutosplitter => {
                                                if let Some(p) = &self.settings_menu.autosplitter_path {
                                                    self.autosplitter_manager = AutosplitterManager::new(self.timer.clone(), p).ok();
                                                    if self.autosplitter_manager.is_none() {
                                                        self.settings_menu.autosplitter_path = None;
                                                        // TODO: Visibly show an error?
                                                    }
                                                } else {
                                                    self.autosplitter_manager = None;
                                                }
                                            }
                        settings_menu::UpdateRequest::LoadProfile(path_buf) => {
                            self.directory_config.active_profile = Some(path_buf);
                            // TODO
                        }
                        settings_menu::UpdateRequest::RemoveKnownDirectory(idx) => {
                            self.directory_config.known_directories.remove(idx);
                            save_dirs_flag = true;
                        }
                        settings_menu::UpdateRequest::AddKnownDirectory(path_buf) => {
                            if !self.directory_config.known_directories.contains(&path_buf) {
                                self.directory_config.known_directories.push(path_buf);
                                save_dirs_flag = true;
                            }
                        }
                    }
                }

                // Check if we need to reload settings
                if !self.settings_menu.shown {
                    if let Some(run) = &self.settings_menu.changed_run {
                        let _ = binding.replace_run(run.clone(), true);
                        self.settings_menu.changed_run = None;
                    }
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
        if save_dirs_flag {
            if let Ok(data) = toml::to_string(&self.directory_config) {
                if let Some(path) = get_project_save_dir() {
                    if !path.exists() {
                        let _ = fs::create_dir_all(path.parent().unwrap());
                    }
                    let file = File::create(path);
                   let _ = file.inspect(|mut f| { let _ = f.write_all(data.as_bytes()); });
                }
            }
        }

        // Save needed resources to file.

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
    let dirs_cfg: DirectoryConfig = {
        if let Some(path) = get_project_save_dir() {
            if let Ok(s) = fs::read_to_string(path) {
                toml::from_str(&s).unwrap_or(DirectoryConfig::default())
            } else {
                DirectoryConfig::default()
            }
        } else {
            DirectoryConfig::default()
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
            let mut app = DeadSplit::new(cc);
            app.set_directory_config(dirs_cfg);
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
