use livesplit_core::SharedTimer;
use sysinfo::System;

use crate::{autosplitter_manager::AutosplitterManager, hotkey_manager::HotkeyManager, ProcessData};

pub struct DeadSplitTimer {
    // TODO: avoid unwraps?
    timer: SharedTimer,
    pub current_time: f64,
    pub current_game_time: f64,
    pub current_split_index: i32,
    pub timer_phase: u8,
    hotkey_mgr: HotkeyManager,
    system: System,
    attached_process: Option<ProcessData>,
    autosplitter_manager: Option<AutosplitterManager>,
}
