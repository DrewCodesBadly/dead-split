use std::{sync::Arc, thread::{self}, time::Instant};

use godot::builtin::{Dictionary, Variant, VariantType};
use livesplit_auto_splitting::{settings, AutoSplitter, Runtime};
use livesplit_core::SharedTimer;

use crate::{timer_read, timer_write};

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

    // TODO: Find way to print in godot from another thread without causing a crash.
    fn log_auto_splitter(&mut self, _: std::fmt::Arguments<'_>) {
        // godot::global::print(&[Variant::from(message.as_str().unwrap_or_default())]);
    }

    fn log_runtime(&mut self, _: std::fmt::Arguments<'_>, _log_level: livesplit_auto_splitting::LogLevel) {
        // godot::global::print(&[Variant::from(message.as_str().unwrap_or_default())]);
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
    pub fn new(timer: SharedTimer, wasm_file_path: String) -> Result<Self, ()> {
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

    pub fn get_settings_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        godot::global::print(&[Variant::from(self.auto_splitter.lock().memory().len() as i64)]);

        for widget in self.auto_splitter.settings_widgets().iter() {
            match &widget.kind {
                settings::WidgetKind::Bool { default_value } => {
                    let _ = dict.insert(&*widget.key.to_string(), *default_value);
                },
                settings::WidgetKind::Choice { default_option_key: _, options: _ } => todo!(),
                settings::WidgetKind::FileSelect { filters: _ } => todo!(),
                settings::WidgetKind::Title { heading_level: _ } => todo!(),
            }

        }

        for (k, v) in self.auto_splitter.settings_map().iter() {
            match v {
                settings::Value::Bool(b) => { let _ = dict.insert(k.to_string(), *b); },
                settings::Value::I64(_) => todo!(),
                settings::Value::F64(_) => todo!(),
                settings::Value::String(_) => todo!(),
                settings::Value::Map(_) => todo!(),
                settings::Value::List(_) => todo!(),
                _ => todo!(),
            }
        }

        dict
    }

    pub fn set_settings_dict(&mut self, dict: Dictionary) {
        let mut map = self.auto_splitter.settings_map().clone();
        for (k, v) in dict.iter_shared() {
            match v.get_type() {
                VariantType::BOOL => map.insert(k.to_string().into(), settings::Value::Bool(v.booleanize())),
                _ => todo!(),
            }
        }

        self.auto_splitter.set_settings_map(map);
    }
}

// Tells the autosplitter thread to stop running when the manager is dropped.
impl Drop for AutosplitterManager {
    fn drop(&mut self) {
        self.auto_splitter.interrupt_handle().interrupt();
    }
}
