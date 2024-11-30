use eframe::egui;
use std::any::Any;

pub mod connection;
pub mod editor;
pub mod message;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ToggledViewManager {
    pub view: Box<dyn View>,
    pub enabled: bool,
}

impl ToggledViewManager {
    pub fn new(view: Box<dyn View>) -> Self {
        Self {
            view,
            enabled: false,
        }
    }

    pub fn as_original<T: 'static>(&self) -> Option<&T> {
        self.view.as_any().downcast_ref::<T>()
    }

    pub fn as_original_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.view.as_any_mut().downcast_mut::<T>()
    }
}
