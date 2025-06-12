use std::{cmp::min};

use egui::{hex_color, Color32, Grid};
use livesplit_core::{timing::formatter::{self, Accuracy, TimeFormatter}, Run, TimeSpan, TimerPhase};

use super::{TimerComponent, UpdateData};

pub struct SplitComponent {
    config: SplitComponentConfig,
    subsplit_map: SubsplitMap,
    last_real_segment: usize,
    segment_pos: usize,
    segment_formatter: formatter::Regular,
    delta_formatter: formatter::Delta,
}

pub struct SplitRenderer {
    pub comparison_index: usize,
    name: String,
}

pub struct SubsplitContainer {
    pub start_idx: usize,
    pub splits: Vec<SplitRenderer>,
    pub header: SplitRenderer,
}

pub struct SubsplitMap {
    pub subsplits: Vec<SubsplitContainer>,
}

impl SplitRenderer {
    fn get_delta_color(&self, update_data: &UpdateData, delta: f64, component: &SplitComponent, best_time: Option<TimeSpan>, t_secs: f64) -> Color32 {
        if best_time.filter(|t| t.total_seconds() < t_secs).is_none() {
            component.config.best_segment_color
        } else {
            if let Some(last_delta) = {
                if self.comparison_index > 0 {
                    let seg = update_data.run.segment(self.comparison_index - 1);
                    let comp = seg.comparison(update_data.snapshot.current_comparison());
                    let time = seg.split_time();
                    match update_data.snapshot.current_timing_method() {
                        livesplit_core::TimingMethod::GameTime => {
                            let comp_secs = comp.game_time.map(|t| t.total_seconds());
                            if let Some(t) =  time.game_time.map(|t| t.total_seconds()) {
                                comp_secs.map(|comp_secs_time| t - comp_secs_time)
                            } else {
                                None
                            }
                        },
                        livesplit_core::TimingMethod::RealTime => {
                            let comp_secs = comp.real_time.map(|t| t.total_seconds());
                            if let Some(t) =  time.real_time.map(|t| t.total_seconds()) {
                                comp_secs.map(|comp_secs_time| t - comp_secs_time)
                            } else {
                                None
                            }

                        },
                    }
                } else {
                    None
                }
            } {
                let gaining = last_delta > delta;
                if delta > 0.0 {
                    if gaining {
                        component.config.behind_gaining_color
                    } else {
                        component.config.behind_losing_color
                    }
                } else if gaining {
                    component.config.ahead_gaining_color
                } else {
                    component.config.ahead_losing_color
                }
            } else {
                if delta > 0.0 {
                    component.config.behind_losing_color
                } else {
                    component.config.ahead_gaining_color
                }
            }
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, update_data: &UpdateData, active: bool, component: &SplitComponent) {
        // This may panic but uhhh that's how it's implemented i guess and it *shouldn't*
        let segment = update_data.run.segment(self.comparison_index);
        let (time, comparison_time, best_time, seg_time) = {
            let comp = segment.comparison(update_data.snapshot.current_comparison());
            let t = update_data.snapshot.current_time();
            let best = segment.best_segment_time();
            match update_data.snapshot.current_timing_method() {
                livesplit_core::TimingMethod::GameTime => (
                    t.game_time, 
                    comp.game_time, 
                    best.game_time,
                    segment.split_time().game_time,
                ),
                livesplit_core::TimingMethod::RealTime => (
                    t.real_time, 
                    comp.real_time, 
                    best.real_time,
                    segment.split_time().real_time,
                ),
            }
        };
        // TODO: Show icon

        // Show name
        ui.label(&self.name);

        // Handle segment delta
        match update_data.snapshot.current_phase() {
            TimerPhase::Running | TimerPhase::Paused => {
                match self.comparison_index.cmp(&update_data.snapshot.current_split_index().unwrap_or(0)) {
                    std::cmp::Ordering::Greater => {
                        ui.label("");
                    },
                    std::cmp::Ordering::Equal => {
                        let t_secs = time.map(|t| t.total_seconds()).unwrap_or(0.0);
                        if let Some(t) =  comparison_time.map(|t| t.total_seconds()) {
                            let delta = t_secs - t;
                            // Check if we need to show the split
                            if delta > 0.0 || best_time.filter(|t| t_secs > t.total_seconds()).is_some() {
                                ui.label(component.delta_formatter.format(TimeSpan::from_seconds(delta)).to_string());
                            } else {
                                ui.label("");
                            }
                        } else {
                            ui.label("");
                        }
                    },
                    std::cmp::Ordering::Less => {
                        // Display delta with colored times.
                        let t_secs = seg_time.map(|t| t.total_seconds()).unwrap_or(0.0);
                        if let Some(t) =  comparison_time.map(|t| t.total_seconds()) {
                            let delta = t_secs - t;
                            // Choose correct color, first checking if this is a gold.
                            let color = self.get_delta_color(update_data, delta, component, best_time, t_secs);
                            ui.colored_label(
                                color,
                                component.delta_formatter.format(TimeSpan::from_seconds(delta)).to_string(),
                            );
                        } else {
                            ui.label("--");
                        }
                    },
                }
            },
            TimerPhase::Ended => {
                // Only ever shows the finished split times
                let t_secs = seg_time.map(|t| t.total_seconds()).unwrap_or(0.0);
                if let Some(t) =  comparison_time.map(|t| t.total_seconds()) {
                    let delta = t_secs - t;
                    // Check if we need to show the split
                    if delta > 0.0 || best_time.filter(|t| t_secs > t.total_seconds()).is_some() {
                        let color = self.get_delta_color(update_data, delta, component, best_time, t_secs);
                        ui.colored_label(
                            color, 
                            component.delta_formatter.format(TimeSpan::from_seconds(delta)).to_string(),
                        );
                    }
                } else {
                    // Skips drawing delta when we aren't comparing to a split.
                    ui.label("--");
                }
            },
            TimerPhase::NotRunning => {
                ui.label("");
            },
        }
        // Display segment time(s)
        ui.label(component.segment_formatter.format(comparison_time).to_string());

        ui.end_row();
    }

    pub fn from_run_index(run: &Run, idx: usize) -> Self {
        // Not a header, so it should start with something.
        let mut name = run.segments()[idx].name().to_string();
        if name.starts_with("-") {
            name = name[1..].to_string();
        } else if name.starts_with("{") {
            if let Some(i) = name.find("}") {
                name.drain(0..=i);
            }
        }
        Self { 
            comparison_index: idx, 
            name, 
        }
    }

    pub fn subsplit_header_from_run_index(run: &Run, idx: usize) -> Self {
        // This segment should start with braces and the subsplit name.
        // If not, just take the full name I guess.
        let mut name = run.segments()[idx].name().to_string();
        if name.starts_with("{") {
            if let Some(i) = name.find("}") {
                name = name.drain(1..i).collect();
            }
        }

        Self {
            comparison_index: idx,
            name,
        }
    }
}

impl SubsplitMap {
    pub fn from_run(run: &Run) -> Self {
        let mut subsplits = Vec::new();
        let mut i: usize = 0;
        let segments = run.segments();
        let seg_count = segments.len();
        while i < seg_count {
            if segments[i].name().starts_with("-") {
                // This is the start of a subsplit.
                let start_idx = i;
                let mut subsplit_list = Vec::new();
                subsplit_list.push(SplitRenderer::from_run_index(run, i));
                i += 1;
                while segments[i].name().starts_with("-") {
                    subsplit_list.push(SplitRenderer::from_run_index(run, i));
                    i += 1;
                }
                // Now add the final split
                subsplit_list.push(SplitRenderer::from_run_index(run, i));
                subsplit_list.insert(0, SplitRenderer::subsplit_header_from_run_index(run, i));

                subsplits.push(SubsplitContainer {
                    start_idx,
                    splits: subsplit_list,
                    header: SplitRenderer::subsplit_header_from_run_index(run, i),
                });
            } else {
                // This is a single split. 
                subsplits.push(SubsplitContainer {
                    start_idx: i,
                    splits: Vec::new(),
                    header: SplitRenderer::subsplit_header_from_run_index(run, i),
                });
                i += 1;
            }
        }
    
        Self {
            subsplits,
        }
    }

    pub fn get_subsplit_index(&self, idx: usize) -> usize {
        let mut i = 0;
            for subsplit in &self.subsplits {
            if subsplit.start_idx >= idx {
                return i;
            }
            i += 1;
        }
        0 // fail! idx was probably larger than the actual list of segments.
    }
}

impl TimerComponent for SplitComponent {
    fn show(&mut self, ui: &mut egui::Ui, update_data: &UpdateData) {
        // Figure out the current index, accounting for scrolling and timer changes.
        let current_timer_idx = update_data.snapshot.current_split_index().unwrap_or(0);
        if current_timer_idx != self.last_real_segment {
            self.last_real_segment = current_timer_idx;
            self.segment_pos = current_timer_idx;
        }
        let scroll_dist = ui.input(|i| i.raw_scroll_delta.y);
        match scroll_dist.partial_cmp(&0.0) {
            Some(ord) => match ord {
                std::cmp::Ordering::Equal => {},
                std::cmp::Ordering::Less => self.segment_pos = min(self.segment_pos + 1, update_data.run.segments().len() - 1),
                std::cmp::Ordering::Greater => self.segment_pos = self.segment_pos.checked_sub(1).unwrap_or(0),
            }
            None => {},
        }

        // Make the full list of splits
        let mut splits: Vec<&SplitRenderer> = Vec::new();
        let current_subsplit = self.subsplit_map.get_subsplit_index(self.segment_pos);
        for subsplit in &self.subsplit_map.subsplits[0..current_subsplit] {
            splits.push(&subsplit.header);
        }
        let current_subsplit_container = &self.subsplit_map.subsplits[current_subsplit];
        splits.push(&current_subsplit_container.header);
        let mut relative_current_idx = splits.len() - 1;
        for split in &current_subsplit_container.splits {
            splits.push(split);
            if split.comparison_index == self.segment_pos {
                relative_current_idx = splits.len() - 1;
            }
        }

        for subsplit in &self.subsplit_map.subsplits[(current_subsplit + 1)..] {
            splits.push(&subsplit.header);
        }

        // Render the slice that is currently visible
        let mut show_last_split = self.config.always_show_last_split;
        let real_shown_splits_num: usize;
        let end_idx = {
            let mut n = relative_current_idx  + self.config.shown_ahead_splits;
            if n >= splits.len() - 1 {
                n = splits.len() - 1;
                show_last_split = false;
            }
            real_shown_splits_num = self.config.num_splits_shown - if show_last_split { 1 } else { 0 };
            if n < real_shown_splits_num - 1 {
                if real_shown_splits_num - 1 < splits.len() - 1 {
                    n = real_shown_splits_num - 1;
                } else {
                    n = splits.len() - 1;
                    show_last_split = false;
                }
            }
            n
        };
        let start_idx = end_idx.checked_sub(real_shown_splits_num - 1).unwrap_or(0);

        Grid::new("SplitsGrid").show(ui, |ui| {
            for split in &splits[start_idx..relative_current_idx] {
                // TODO: Active vs inactive splits
                split.show(ui, update_data, false, self);
            }
            splits[relative_current_idx].show(ui, update_data, true, self);
            for split in &splits[(relative_current_idx + 1)..=end_idx] {
                split.show(ui, update_data, false, self);
            }

            // Render the last split, if we have to.
            if show_last_split {
                if let Some(split) = splits.last() {
                    if self.config.show_last_split_separator {
                        ui.separator();
                        ui.end_row();
                    }
                    split.show(ui, update_data, false, self);
                }
            }
        });
    }
}

impl SplitComponent {
    pub fn new(config: SplitComponentConfig, run: &Run) -> Self {
        Self {
            subsplit_map: SubsplitMap::from_run(run),
            last_real_segment: 0,
            segment_pos: 0,
            segment_formatter: formatter::Regular::with_accuracy(config.segment_time_accuracy),
            delta_formatter: formatter::Delta::custom(true, config.delta_accuracy),
            config,
        }
    }
}

pub struct SplitComponentConfig {
    pub num_splits_shown: usize,
    pub always_show_last_split: bool,
    pub show_last_split_separator: bool,
    pub shown_ahead_splits: usize,
    pub segment_time_accuracy: Accuracy,
    pub delta_accuracy: Accuracy,
    pub best_segment_color: Color32,
    pub behind_losing_color: Color32,
    pub behind_gaining_color: Color32,
    pub ahead_losing_color: Color32,
    pub ahead_gaining_color: Color32,
}

impl Default for SplitComponentConfig {
    fn default() -> Self {
        Self {
            num_splits_shown: 10,
            always_show_last_split: true,
            show_last_split_separator: true,
            shown_ahead_splits: 1,
            segment_time_accuracy: Accuracy::Seconds,
            delta_accuracy: Accuracy::Hundredths,
            best_segment_color: hex_color!("#ffb340"),
            behind_losing_color: hex_color!("#690000"),
            behind_gaining_color: hex_color!("#ff5454"),
            ahead_losing_color: hex_color!("#80ff80"),
            ahead_gaining_color: hex_color!("#00c900"),
        }
    }
}
