use std::fmt::Display;

use livesplit_core::{timing::{formatter::{Accuracy, SegmentTime, TimeFormatter}, Snapshot}, Run};
use serde::{Deserialize, Serialize};

use crate::{autosplitter_manager::AutosplitterManager, hotkey_manager::HotkeyManager, timer_components::split_component::SplitComponentConfig};

pub mod split_component;

#[derive(Serialize, Deserialize)]
pub enum TimerComponentType {
    TitleComponent(TitleComponentConfig),
    TimerComponent(TimerComponentConfig),
    SplitComponent(SplitComponentConfig),
}

impl Display for TimerComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerComponentType::TitleComponent(_) => {
                f.write_str("Title")
            },
            TimerComponentType::TimerComponent(_) => {
                f.write_str("Timer")
            },
            TimerComponentType::SplitComponent(_) => {
                f.write_str("Splits")
            },
        }
    }
}

pub struct UpdateData<'a> {
    pub snapshot: Snapshot<'a>,
    pub run: &'a Run,
    pub hotkey_manager: &'a HotkeyManager,
    pub autosplitter_manager: &'a Option<AutosplitterManager>,
}

pub trait TimerComponent {
    fn show(&mut self, ui: &mut egui::Ui, update_data: &UpdateData);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TitleComponentConfig {
    pub show_game_name: bool,
    pub show_cat_name: bool,
    pub show_attempt_count: bool,
    pub show_finished_run_count: bool,
}

impl Default for TitleComponentConfig {
    fn default() -> Self {
        Self {
            show_game_name: true,
            show_cat_name: true,
            show_attempt_count: true,
            show_finished_run_count: true,
        }
    }
}

pub struct TitleComponent {
    game_name: String,
    cat_name: String,
    attempt_string: String,
    config: TitleComponentConfig,
}

impl TimerComponent for TitleComponent {
    fn show(&mut self, ui: &mut egui::Ui, _update_data: &UpdateData) {
        ui.label(&self.game_name);
        ui.label(&self.cat_name);
        ui.label(&self.attempt_string);
    }
}

impl TitleComponent {
    pub fn new(run: &Run, config: &TitleComponentConfig) -> Self {
        let mut finished_count = 0;
        for attempt in run.attempt_history() {
            if let Some(_) = attempt.time().real_time {
                finished_count += 1;
            }
        }
        Self {
            game_name: run.game_name().to_owned(),
            cat_name: run.category_name().to_owned(),
            attempt_string: finished_count.to_string() + "/" + &run.attempt_count().to_string(),
            config: config.clone(),
        }
    }
}

pub struct RunTimerComponent {
    formatter: SegmentTime,
}

#[derive(Serialize, Deserialize)]
pub struct TimerComponentConfig {
    timer_accuracy: Accuracy,
}

impl Default for TimerComponentConfig {
    fn default() -> Self {
        Self { timer_accuracy: Accuracy::Hundredths }
    }
}

impl TimerComponent for RunTimerComponent {
    fn show(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        let time = match update_data.snapshot.current_timing_method() {
            livesplit_core::TimingMethod::GameTime => update_data.snapshot.current_time().game_time,
            livesplit_core::TimingMethod::RealTime => update_data.snapshot.current_time().real_time,
        };
        ui.label(self.formatter.format(time).to_string());
    }
}

impl RunTimerComponent {
    pub fn new(config: &TimerComponentConfig) -> Self {
        Self {
            formatter: SegmentTime::with_accuracy(config.timer_accuracy),
        }
    }
}
