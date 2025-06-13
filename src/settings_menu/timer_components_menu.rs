use egui::{popup, popup_below_widget, Ui};

use crate::{settings_menu::{SettingsMenu, UpdateRequest}, timer_components::{TimerComponentType, TitleComponentConfig}, ConfigReferences};

impl SettingsMenu {
    pub fn show_timer_components_menu(&mut self, ui: &mut Ui, configs: &mut ConfigReferences) {
        // These settings do not reload immediately, you have to close settings first.
        // It would be too much of a pain to reaload every time one thing is tweaked a bit.
        ui.label("These settings will not update until the settings menu is closed.");

        for (idx, comp) in &mut configs.timer_config.components.iter().enumerate() {
            ui.collapsing((idx + 1).to_string() + ": " + &comp.to_string(), |ui| {
                match comp {
                    TimerComponentType::TitleComponent(cfg) => {

                    }
                    TimerComponentType::TimerComponent(cfg) => {

                    }
                    TimerComponentType::SplitComponent(cfg) => {

                    }
                }
                
                // These are delayed using requests since we're in a for loop.
                if ui.button("Remove Component").clicked() {
                    self.update_requests.push(UpdateRequest::RemoveComponent(idx));
                }
                if ui.button("Move Component Up").clicked() {
                    self.update_requests.push(UpdateRequest::MoveComponentUp(idx));
                }
                if ui.button("Move Component Down").clicked() {
                    self.update_requests.push(UpdateRequest::MoveComponentDown(idx));
                }
            });
        }

        let response = ui.button("Add Component");
        let popup_id = ui.make_persistent_id("component_choice");
        if response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        popup_below_widget(ui, 
            popup_id,
            &response,
            popup::PopupCloseBehavior::CloseOnClickOutside, 
            |ui| {
                if ui.button("Title").clicked() {
                    configs.timer_config.components.push(
                        TimerComponentType::TitleComponent(TitleComponentConfig::default())
                    );
                }
            });
    }
}
