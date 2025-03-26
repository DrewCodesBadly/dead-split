use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use global_hotkey::GlobalHotKeyManager;
use godot::prelude::*;
use hotkey_manager::HotkeyManager;
use livesplit_core::{Run, Segment, SharedTimer, Timer, hotkey::Hook};
use read_process_memory::ProcessHandle;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

mod editable_run;
mod timer;
mod hotkey_manager;

struct DeadSplitRust;

#[gdextension]
unsafe impl ExtensionLibrary for DeadSplitRust {}

// Wrapper for sysinfo::Process and read_process_memory::ProcessHandle since we really need both or none
pub struct ProcessData {
    pub handle: ProcessHandle,
    pub pid: Pid,
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct DeadSplitTimer {
    // alright look idk how on earth this lock is getting poisoned
    // so i'm just going to unwrap all of these and you're going to pretend you saw nothing
    timer: SharedTimer,
    #[var]
    pub current_time: f64,
    #[var]
    pub current_game_time: f64,
    #[var]
    pub current_split_index: i32,
    #[var]
    pub timer_phase: u8,
    hotkey_mgr: HotkeyManager,
    system: System,
    attached_process: Option<ProcessData>,

    base: Base<Node>,
}


pub fn timer_read(t: &SharedTimer) -> RwLockReadGuard<'_, Timer> {
    t.read().unwrap()
}

pub fn timer_write(t: &SharedTimer) -> RwLockWriteGuard<'_, Timer> {
    t.write().unwrap()
}

#[godot_api]
impl INode for DeadSplitTimer {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        let timer_shared = Timer::new(get_default_run())
            .expect("default run should have 1 segment")
            .into_shared();
        Self {
            timer: timer_shared.clone(),
            current_time: 0.0,
            current_game_time: 0.0,
            current_split_index: -1,
            timer_phase: 0,
            // Starts by default with a wayland hook.
            // This should be reloaded when the timer's settings are loaded.
            hotkey_mgr: HotkeyManager::new_wayland(Hook::new().expect("Failed to create hotkey hook")),
            system: System::new_with_specifics(RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            )),
            attached_process: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        // Updates displayed properties from a snapshot every frame
        // Make sure binding is dropped before we need to access self's hotkey data
        {
            let binding = timer_read(&self.timer);
            let snapshot = binding.snapshot();
            let t = snapshot.current_time();
            self.current_time = t.real_time.unwrap_or_default().total_seconds();
            self.current_game_time = t.game_time.unwrap_or_default().total_seconds();
            self.current_split_index = match snapshot.current_split_index() {
                Some(i) => i as i32,
                None => -1,
            };
            self.timer_phase = snapshot.current_phase() as u8;
        }

        // Check for hotkey presses
        if let Some(idx) = self.hotkey_mgr.poll_keypress() {
            self.base_mut()
                .clone()
                .upcast::<Object>()
                .emit_signal("hotkey_pressed", &[Variant::from(idx)]);
                // self.hotkey_pressed(idx);
        }
    }
}

fn get_default_run() -> Run {
    let mut run = Run::new();
    run.push_segment(Segment::new("Time"));
    run.set_game_name("Game");
    run.set_category_name("Any%");
    run
}
