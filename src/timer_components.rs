use livesplit_core::{timing::{formatter::{Accuracy, SegmentTime, TimeFormatter}, Snapshot}, Run};

use crate::{autosplitter_manager::AutosplitterManager, hotkey_manager::HotkeyManager};

pub mod split_component;

pub struct UpdateData<'a> {
    pub snapshot: Snapshot<'a>,
    pub run: &'a Run,
    pub hotkey_manager: &'a HotkeyManager,
    pub autosplitter_manager: &'a Option<AutosplitterManager>,
}

pub trait TimerComponent {
    fn show(&mut self, ui: &mut egui::Ui, update_data: &UpdateData);
}

pub struct TitleComponent {
    game_name: String,
    cat_name: String,
    attempt_string: String,
}

impl TimerComponent for TitleComponent {
    fn show(&mut self, ui: &mut egui::Ui, _update_data: &UpdateData) {
        ui.label(&self.game_name);
        ui.label(&self.cat_name);
        ui.label(&self.attempt_string);
    }
}

impl TitleComponent {
    pub fn new(run: &Run) -> Self {
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
        }
    }
}

pub struct RunTimerComponent {
    formatter: SegmentTime,
}

pub struct RunTimerConfig {
    timer_accuracy: Accuracy,
}

impl Default for RunTimerConfig {
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
    pub fn new(config: RunTimerConfig) -> Self {
        Self {
            formatter: SegmentTime::with_accuracy(config.timer_accuracy),
        }
    }
}
