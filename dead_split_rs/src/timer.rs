use std::{
    ffi::OsStr,
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use crate::editable_run::EditableRun;
use godot::prelude::*;
use livesplit_core::{
    analysis::total_playtime::TotalPlaytime,
    run::{
        parser::composite,
        saver::livesplit::{self, IoWrite},
    },
};

use read_process_memory::*;

use super::*;

// impl block for basic livesplit-core interface exposed to godot
#[godot_api]
impl DeadSplitTimer {
    // Timer control
    #[func]
    fn new_run(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.replace_run(get_default_run(), true);
    }

    #[func]
    fn start_split(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.split_or_start();
    }

    #[func]
    fn reset(&self) {
        let mut binding = timer_write(&self.timer);
        let current_split_index = binding.current_split_index().unwrap_or_default();
        let _ = binding.reset(true);
        let mut run = binding.run().clone();
        run.update_segment_history(current_split_index);
        run.fix_splits();
        let _ = binding.replace_run(run, true); // WHY WOULD YOU MAKE THIS A RESULT ISTG
    }

    #[func]
    fn pause(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.pause();
    }

    #[func]
    fn resume(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.resume();
    }

    #[func]
    fn toggle_pause(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.toggle_pause();
    }

    #[func]
    fn undo_all_pauses(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.undo_all_pauses();
    }

    #[func]
    fn skip_split(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.skip_split();
    }

    #[func]
    fn undo_split(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.undo_split();
    }

    #[func]
    fn toggle_timing_method(&self) {
        let mut binding = timer_write(&self.timer);
        binding.toggle_timing_method();
    }

    #[func]
    fn try_save_run(&self, file_path: String) -> bool {
        let binding = timer_read(&self.timer);
        let writer = match File::create(file_path) {
            Ok(f) => BufWriter::new(f),
            Err(_) => return false,
        };

        match livesplit::save_run(binding.run(), IoWrite(writer)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[func]
    fn try_load_run(&self, file_path: String) -> bool {
        let mut binding = timer_write(&self.timer);
        let path = Path::new(&file_path);
        let file = match fs::read(path) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let _ = binding.replace_run(
            match composite::parse(&file, Some(path)) {
                Ok(p) => p.run,
                Err(_) => return false,
            },
            true,
        );
        true
    }

    #[func]
    fn init_game_time(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.initialize_game_time();
    }

    #[func]
    fn deinit_game_time(&self) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.deinitialize_game_time();
    }

    #[func]
    fn is_game_time_initialized(&self) -> bool {
        let binding = timer_read(&self.timer);
        binding.is_game_time_initialized()
    }

    #[func]
    fn is_game_time_running(&self) -> bool {
        let binding = timer_read(&self.timer);
        !binding.is_game_time_paused()
    }

    #[func]
    fn regenerate_comparisons(&self) {
        let mut binding = timer_write(&self.timer);
        let mut new_run = binding.run().clone();
        new_run.regenerate_comparisons();
        let _ = binding.set_run(new_run);
    }

    // Get timer data
    #[func]
    fn get_segment_count(&self) -> i32 {
        let binding = timer_read(&self.timer);
        binding.run().len() as i32
    }

    #[func]
    fn get_segment_name(&self, idx: i32) -> String {
        let binding = timer_read(&self.timer);
        binding.run().segment(idx as usize).name().to_owned()
    }

    #[func]
    fn get_segment_comparison(&self, idx: i32, comparing_to: String, rta: bool) -> f64 {
        let binding = timer_read(&self.timer);
        let comp = binding
            .run()
            .segment(idx as usize)
            .comparison(&comparing_to);
        if rta {
            comp.real_time.unwrap_or_default().total_seconds()
        } else {
            comp.game_time.unwrap_or_default().total_seconds()
        }
    }

    #[func]
    fn get_segment_time(&self, idx: i32, rta: bool) -> f64 {
        let binding = timer_read(&self.timer);
        let split_time = binding.run().segment(idx as usize).split_time();
        if rta {
            split_time.real_time.unwrap_or_default().total_seconds()
        } else {
            split_time.game_time.unwrap_or_default().total_seconds()
        }
    }

    #[func]
    fn get_pb_segment_time(&self, idx: i32, rta: bool) -> f64 {
        let binding = timer_read(&self.timer);
        let split_time = binding
            .run()
            .segment(idx as usize)
            .personal_best_split_time();
        if rta {
            split_time.real_time.unwrap_or_default().total_seconds()
        } else {
            split_time.game_time.unwrap_or_default().total_seconds()
        }
    }

    #[func]
    fn get_segment_best(&self, idx: i32, rta: bool) -> f64 {
        let binding = timer_read(&self.timer);
        let best_time = binding.run().segment(idx as usize).best_segment_time();
        if rta {
            best_time.real_time.unwrap_or_default().total_seconds()
        } else {
            best_time.game_time.unwrap_or_default().total_seconds()
        }
    }

    #[func]
    fn get_total_playtime(&self) -> f64 {
        let binding = timer_read(&self.timer);
        binding.run().total_playtime().total_seconds()
    }

    #[func]
    fn get_game_name(&self) -> String {
        let binding = timer_read(&self.timer);
        binding.run().game_name().to_owned()
    }

    #[func]
    fn get_category_name(&self) -> String {
        let binding = timer_read(&self.timer);
        binding.run().category_name().to_owned()
    }

    #[func]
    fn get_attempt_count(&self) -> i32 {
        let binding = timer_read(&self.timer);
        binding.run().attempt_count() as i32
    }

    #[func]
    fn get_finished_run_count(&self) -> i32 {
        let binding = timer_read(&self.timer);
        let mut count: i32 = 0;
        for attempt in binding.run().attempt_history() {
            if let Some(_) = attempt.time().real_time {
                count += 1;
            }
        }
        count
    }

    // run interfacing
    #[func]
    fn update_run(&mut self, editable_run: Gd<EditableRun>) {
        let mut binding = timer_write(&self.timer);
        let _ = binding.replace_run(editable_run.bind().get_run(), true);
    }

    #[func]
    fn get_editable_run(&self) -> Gd<EditableRun> {
        let binding = timer_read(&self.timer);
        EditableRun::from_run(binding.run())
    }

    #[func]
    fn get_comparisons(&self) -> Array<GString> {
        let binding = timer_read(&self.timer);
        Array::from_iter(binding.run().comparisons().map(|s| GString::from(s)))
    }

    // hotkeys
    #[func]
    fn add_hotkey(&mut self, key_string: String, hotkey_id: i32) -> bool {
        self.hotkey_mgr.bind_key(key_string, hotkey_id).is_ok()
    }

    #[func]
    fn remove_hotkey(&mut self, hotkey_id: i32) -> bool {
        self.hotkey_mgr.remove_key(hotkey_id).is_ok()
    }

    #[func]
    fn reset_hotkey_manager(&mut self, use_wayland: bool) {
        if use_wayland {
            self.hotkey_mgr = HotkeyManager::new_wayland(Hook::new()
                .expect("Failed to create hotkey hook"));
        } else {
            self.hotkey_mgr = HotkeyManager::new_x11(GlobalHotKeyManager::new()
                .expect("Failed to create global hotkey manager"));
        }
    }

    #[func]
    fn get_hotkey_string(&self, hotkey_id: i32) -> String {
        self.hotkey_mgr.get_hotkey_string(hotkey_id).unwrap_or(String::from("None"))
    }

    #[func]
    fn get_hotkeys_dict(&self) -> Dictionary {
        self.hotkey_mgr.get_hotkeys_dict()
    }

    #[signal]
    pub fn hotkey_pressed(&mut self, hotkey_id: i32);

    // Autosplitter API
    #[func]
    pub fn try_attach_process(&mut self, process_name: String) {
        // Don't use an empty string, since that's clearly a user error.
        if process_name.len() < 1 {
            return;
        };
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
        );
        if let Some(p) = self
            .system
            .processes_by_name(OsStr::new(&process_name))
            .next()
        {
            // bunch of garbage to get from sysinfo::Process to read_process_memory::ProcessHandle
            // should work like 99% of the time, the results aren't a major concern but if it fails it just fails anyway
            let pid = p.pid();
            self.attached_process =
                // Uncomment the top line for linux builds, bottom for windows. Jank but it works.
                ProcessHandle::try_from(read_process_memory::Pid::from(pid.as_u32() as i32))
                // ProcessHandle::try_from(read_process_memory::Pid::from(pid.as_u32()))
                    .ok()
                    .map(|h| ProcessData {
                        handle: h,
                        pid: pid,
                    });
        }
    }

    #[func]
    pub fn has_valid_process(&self) -> bool {
        if let Some(d) = &self.attached_process {
            self.system.process(d.pid).is_some()
        } else {
            false
        }
    }

    #[func]
    pub fn detach_process(&mut self) {
        self.attached_process = None;
    }

    #[func]
    pub fn read_pointer_path(
        &self,
        offsets: PackedInt64Array,
        pointer_size_32: bool,
        data_type: i32,
    ) -> Variant {
        if let Some(p) = &self.attached_process {
            let mut iter = offsets.as_slice().iter();
            if let Some(v) = iter.next() {
                let mut ptr: usize = *v as usize;
                if pointer_size_32 {
                    let mut buf = [0 as u8; 4];
                    // Read and add each offset
                    for offset in iter {
                        if p.handle.copy_address(ptr, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        ptr = i32::from_le_bytes(buf) as usize;
                        ptr += *offset as usize;
                    }
                } else {
                    let mut buf = [0 as u8; 8];
                    // Read and add each offset
                    for offset in iter {
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        ptr = i64::from_le_bytes(buf) as usize;
                        ptr += *offset as usize
                    }
                }
                // Read the final value - data_type matches an enum declared in godot
                match data_type {
                    0 => {
                        // i32
                        let mut buf = [0 as u8; 4];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(i32::from_le_bytes(buf));
                    }
                    1 => {
                        // i64
                        let mut buf = [0 as u8; 8];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(i64::from_le_bytes(buf));
                    }
                    2 => {
                        // u32
                        let mut buf = [0 as u8; 4];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(u32::from_le_bytes(buf));
                    }
                    3 => {
                        // u64
                        let mut buf = [0 as u8; 8];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(u64::from_le_bytes(buf));
                    }
                    4 => {
                        // f32
                        let mut buf = [0 as u8; 4];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(f32::from_le_bytes(buf));
                    }
                    5 => {
                        // f64
                        let mut buf = [0 as u8; 8];
                        if p.handle.copy_address(ptr as usize, &mut buf).is_err() {
                            return Variant::nil();
                        }
                        return Variant::from(f64::from_le_bytes(buf));
                    }

                    // invalid input
                    _ => return Variant::nil(),
                }
            } else {
                return Variant::nil();
            }
        }

        todo!()
    }

    #[func]
    fn pause_game_time(&self) {
        let mut binding = timer_write(&self.timer);
        binding.pause_game_time();
    }

    #[func]
    fn resume_game_time(&self) {
        let mut binding = timer_write(&self.timer);
        binding.resume_game_time();
    }

    #[func]
    fn load_wasm_autosplitter(&mut self, path: String) -> bool {
        self.autosplitter_manager = AutosplitterManager::new(self.timer.clone(), path).ok();
        self.autosplitter_manager.is_some()
    }

    // AutosplitterManager.drop() stops the running thread.
    #[func]
    fn unload_wasm_autosplitter(&mut self) {
        self.autosplitter_manager = None;
    }

    #[func]
    fn get_wasm_settings_dict(&self) -> Dictionary {
        if let Some(m) = &self.autosplitter_manager {
            m.get_settings_dict()
        } else {
            godot::global::print(&[Variant::from("failed to get settings")]);
            Dictionary::new()
        }
    }

    #[func]
    fn set_wasm_settings_dict(&mut self, dict: Dictionary) {
        if let Some(m) = &mut self.autosplitter_manager {
            m.set_settings_dict(dict);
        }
    }
}
