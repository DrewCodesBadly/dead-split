mod split_component;

pub trait TimerComponent {
    fn show(&self, ui: &mut egui::Ui);
}

pub struct TitleComponent {

}

impl TimerComponent for TitleComponent {
    fn show(&self, ui: &mut egui::Ui) {
        todo!()
    }
}
