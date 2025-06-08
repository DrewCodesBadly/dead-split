use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use autosplitter_manager::AutosplitterManager;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, ViewportBuilder, WindowLevel};
use global_hotkey::GlobalHotKeyManager;
use hotkey_manager::HotkeyManager;
use livesplit_core::{hotkey::Hook, Run, Segment, SharedTimer, Timer, TimerPhase};
use read_process_memory::ProcessHandle;
use settings_menu::SettingsMenu;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
use timer_components::{split_component::{SplitComponent, SplitComponentConfig}, RunTimerComponent, RunTimerConfig, TimerComponent, TitleComponent, UpdateData};

mod editable_run;
mod hotkey_manager;
mod autosplitter_manager;
mod timer_components;
mod settings_menu;

struct DeadSplit {
    // TODO: avoid unwraps?
    timer: SharedTimer,
    current_time: f64,
    current_game_time: f64,
    current_split_index: i32,
    hotkey_mgr: HotkeyManager,
    system: System,
    attached_process: Option<ProcessData>,
    autosplitter_manager: Option<AutosplitterManager>,

    components: Vec<Box<dyn TimerComponent>>,
    settings_menu: SettingsMenu,
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
            current_time: 0.0,
            current_game_time: 0.0,
            current_split_index: -1,
            hotkey_mgr: HotkeyManager::new_wayland(Hook::new().expect("Failed to create hotkey hook")),
            system: System::new_with_specifics(RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            )),
            attached_process: None,
            autosplitter_manager: None,
            // TODO: Load these instead of hardcoding them
            // components: Vec::new(),
            components: vec![
                Box::new(title_comp),
                Box::new(split_comp), 
                Box::new(RunTimerComponent::new(RunTimerConfig::default()))
            ],
            settings_menu: SettingsMenu::new(),
        }
    }

    pub fn reload_layout() {
        todo!()
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
        let binding = timer_read(&self.timer);
        let update_data = UpdateData {
            snapshot: binding.snapshot(),
            run: binding.run(),
        };
        CentralPanel::default().show(ctx, |ui| {
            for component in &self.components {
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
                        self.settings_menu.show(ctx, ui);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.settings_menu.shown = false;
                    }
                });
        }
    }
}

fn main() {
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
            Ok(Box::new(DeadSplit::new(cc)))
        }),
    ).expect("failed to open window");
}
