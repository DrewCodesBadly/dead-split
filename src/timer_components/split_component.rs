use std::collections::VecDeque;

use livesplit_core::Run;

use super::{TimerComponent, UpdateData};

pub struct SplitComponent {
    config: SplitComponentConfig,
    subsplit_map: SubsplitMap,
}

pub struct SplitRenderer {
    pub comparison_index: usize,
    name: String,
}

pub struct SubsplitContainer {
    pub start_idx: usize,
    pub end_idx: usize,
    pub splits: Vec<SplitRenderer>,
    pub header: SplitRenderer,
}

pub struct SubsplitMap {
    pub subsplits: Vec<SubsplitContainer>,
}

impl SplitRenderer {
    pub fn show(&self, ui: &mut egui::Ui, update_data: &UpdateData, active: bool) {
        ui.label(&self.name);
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
                    end_idx: i,
                    header: SplitRenderer::subsplit_header_from_run_index(run, i),
                });
            } else {
                // This is a single split. 
                subsplits.push(SubsplitContainer {
                    start_idx: i,
                    splits: Vec::new(),
                    end_idx: i,
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

    pub fn last_split(&self) -> &SplitRenderer {
        todo!();
    }
}

impl TimerComponent for SplitComponent {
    fn show(&self, ui: &mut egui::Ui, update_data: &UpdateData) {
        let shown_splits = self.config.num_splits_shown - if self.config.always_show_last_split { 1 } else { 0 }; 
        let mut ahead_split_count: usize = 0;
        let current_idx = update_data.snapshot.current_split_index().unwrap_or(0);
        
        // Make the full list of splits
        let mut splits: Vec<&SplitRenderer> = Vec::new();
        let current_subsplit = self.subsplit_map.get_subsplit_index(current_idx);
        for subsplit in &self.subsplit_map.subsplits[0..current_subsplit] {
            splits.push(&subsplit.header);
        }
        let current_subsplit_container = &self.subsplit_map.subsplits[current_subsplit];
        splits.push(&current_subsplit_container.header);
        let mut relative_current_idx = splits.len() - 1;
        for split in &current_subsplit_container.splits {
            splits.push(split);
            if split.comparison_index == current_idx {
                relative_current_idx = splits.len() - 1;
            }
        }

        for subsplit in &self.subsplit_map.subsplits[(current_subsplit + 1)..] {
            splits.push(&subsplit.header);
        }

        // Render the slice that is currently visible
        let mut show_last_split = self.config.always_show_last_split;
        let end_idx = {
            let mut n = relative_current_idx  + self.config.shown_ahead_splits;
            if n >= splits.len() {
                n = splits.len() - 1;
                show_last_split = false;
            }
            n
        };
        let start_idx = end_idx.checked_sub(
            self.config.num_splits_shown - if show_last_split { 1 } else { 0 }
        ).unwrap_or(0);
        for split in &splits[start_idx..relative_current_idx] {
            // TODO: Active vs inactive splits
            split.show(ui, update_data, false);
        }
        splits[relative_current_idx].show(ui, update_data, true);
        for split in &splits[(relative_current_idx + 1)..=end_idx] {
            split.show(ui, update_data, false);
        }

        // Render the last split, if we have to.
        if show_last_split {
            if let Some(split) = splits.last() {
                if self.config.show_last_split_separator {
                    ui.separator();
                }
                split.show(ui, update_data, false);
            }
        }
    }
}

impl SplitComponent {
    pub fn new(config: SplitComponentConfig, run: &Run) -> Self {
        Self {
            config,
            subsplit_map: SubsplitMap::from_run(run),
        }
    }
}

pub struct SplitComponentConfig {
    pub num_splits_shown: usize,
    pub always_show_last_split: bool,
    pub show_last_split_separator: bool,
    pub shown_ahead_splits: usize,
}

impl Default for SplitComponentConfig {
    fn default() -> Self {
        Self {
            num_splits_shown: 10,
            always_show_last_split: true,
            show_last_split_separator: true,
            shown_ahead_splits: 1,
        }
    }
}
