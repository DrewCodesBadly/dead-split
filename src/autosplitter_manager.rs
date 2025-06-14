use std::{collections::HashMap, path::PathBuf, sync::Arc, thread::{self}, time::Instant};

use livesplit_auto_splitting::{settings, AutoSplitter, Runtime};
use livesplit_core::SharedTimer;
use serde::{Deserialize, Serialize};

use crate::{timer_read, timer_write};

#[derive(Serialize, Deserialize, Debug)]
pub enum SerializableSettingValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    StringValue(String),
}

#[derive(Serialize, Deserialize, Default)]
pub struct AutosplitterConfig {
    pub enabled: bool,
    pub settings: HashMap<String, SerializableSettingValue>,
}

impl AutosplitterConfig {
    pub fn get_settings_map(&self) -> settings::Map {
        let mut map = settings::Map::new();
        for (k, v) in &self.settings {
            let arc: Arc<str> = Arc::from(k.as_str());
            match v {
                SerializableSettingValue::Bool(b) => {
                    map.insert(arc, settings::Value::Bool(*b));
                }
                SerializableSettingValue::I64(i) => {
                    map.insert(arc, settings::Value::I64(*i))
                }
                SerializableSettingValue::F64(f) => {
                    map.insert(arc, settings::Value::F64(*f));
                }
                SerializableSettingValue::StringValue(s) => {
                    map.insert(arc, settings::Value::String(Arc::from(s.as_str())));
                }
            }
        }

        map
    }
}

// deplorable.
struct TimerBox(SharedTimer);

// more deplorable.
impl livesplit_auto_splitting::Timer for TimerBox {
    fn state(&self) -> livesplit_auto_splitting::TimerState {
        match timer_read(&self.0).current_phase() {
            livesplit_core::TimerPhase::NotRunning => livesplit_auto_splitting::TimerState::NotRunning,
            livesplit_core::TimerPhase::Running => livesplit_auto_splitting::TimerState::Running,
            livesplit_core::TimerPhase::Ended => livesplit_auto_splitting::TimerState::Ended,
            livesplit_core::TimerPhase::Paused => livesplit_auto_splitting::TimerState::Paused,
        }
    }

    fn start(&mut self) {
        timer_write(&mut self.0).start();
    }

    fn split(&mut self) {
        timer_write(&mut self.0).split();
    }

    fn skip_split(&mut self) {
        timer_write(&mut self.0).skip_split();
    }

    fn undo_split(&mut self) {
        timer_write(&mut self.0).undo_split();
    }

    fn reset(&mut self) {
        timer_write(&mut self.0).reset(true);
    }

    fn set_game_time(&mut self, time: livesplit_auto_splitting::time::Duration) {
        // kekw ass line
        timer_write(&mut self.0).set_game_time(livesplit_core::TimeSpan::from_seconds(time.as_seconds_f64()));
    }

    fn pause_game_time(&mut self) {
        timer_write(&mut self.0).pause_game_time();
    }

    fn resume_game_time(&mut self) {
        timer_write(&mut self.0).resume_game_time();
    }

    fn set_variable(&mut self, key: &str, value: &str) {
        timer_write(&mut self.0).set_custom_variable(key, value);
    }

    // TODO: Logging system (or just like, use the asr debugger when debugging.)
    fn log_auto_splitter(&mut self, _: std::fmt::Arguments<'_>) {
    }

    fn log_runtime(&mut self, _: std::fmt::Arguments<'_>, _log_level: livesplit_auto_splitting::LogLevel) {
    }
}

pub struct AutosplitterManager {
    _runtime: Runtime,
    auto_splitter: Arc<AutoSplitter<TimerBox>>,
}

// Adapted from the asr-debugger @ https://github.com/LiveSplit/asr-debugger/blob/master/src/main.rs
fn autosplitter_thread(auto_splitter: Arc<AutoSplitter<TimerBox>>) {
    let mut next_tick = Instant::now();
    loop {
        let mut lock = auto_splitter.lock();
        let _ = lock.update(); // TODO: Find way to print errors successfully in godot
        drop(lock);
        let tick_rate = auto_splitter.tick_rate();
        next_tick += tick_rate;
        let now = Instant::now();
        if let Some(sleep_time) = next_tick.checked_duration_since(now) {
            thread::sleep(sleep_time);
        } else {
            // In this case we missed the next tick already. This likely comes
            // up when the operating system was suspended for a while. Instead
            // of trying to catch up, we just reset the next tick to start from
            // now.
            next_tick = now;
        }
        // thread::sleep(std::time::Duration::from_secs_f64(3.5));
    }
}
    
impl AutosplitterManager {
    pub fn new(timer: SharedTimer, wasm_file_path: &PathBuf) -> Result<Self, ()> {
        let module = std::fs::read(wasm_file_path).map_err(|_| ())?;
        let mut config = livesplit_auto_splitting::Config::default();
        config.optimize = true;
        config.backtrace_details = false;
        config.debug_info = false;
        let runtime = Runtime::new(config).expect("Failed to create autosplitter runtime");
        let auto_splitter = runtime.compile(module.as_slice()).map_err(|_| ())?
            .instantiate(TimerBox(timer), None, None).map_err(|_| ())?;

        let auto_splitter_arc: Arc<AutoSplitter<TimerBox>> = auto_splitter.into();

        let _ = thread::Builder::new()
        .name("autosplitter-thread".into())
        .spawn({
                let arc_clone = auto_splitter_arc.clone();
                move || autosplitter_thread(arc_clone)
        })
        .expect("Failed to start autosplitter thread");

        Ok(Self {
            _runtime: runtime,
            auto_splitter: auto_splitter_arc,
        })
    }

    pub fn settings_widgets(&self) -> Arc<Vec<settings::Widget>> {
        self.auto_splitter.settings_widgets()
    }

    pub fn settings_map(&self) -> settings::Map {
        self.auto_splitter.settings_map()
    }

    pub fn set_settings_map(&self, map: settings::Map) {
        self.auto_splitter.set_settings_map(map);
    }
}

// Tells the autosplitter thread to stop running when the manager is dropped.
impl Drop for AutosplitterManager {
    fn drop(&mut self) {
        self.auto_splitter.interrupt_handle().interrupt();
    }
}
