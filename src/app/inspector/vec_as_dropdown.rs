use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::fmt::{Debug, Display};

// #[derive(Default)]
pub struct VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    from: Vec<T>,
    selected: usize,
}

impl<T> VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    pub fn new(from: Vec<T>) -> Self {
        Self::new_with_selected(from, 0)
    }

    pub fn new_with_selected(
        from: Vec<T>,
        selected: usize,
    ) -> Self {
        Self { from, selected }
    }

    pub fn selected_value(&self) -> T {
        self.from[self.selected].clone()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }
}

impl<T> Default for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    fn default() -> Self {
        Self {
            from: Vec::new(),
            selected: 0,
        }
    }
}

impl<T> Inspectable for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Sized + Debug + Default,
{
    type Attributes = Vec<T>;

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _: Self::Attributes,
        _: &bevy_inspector_egui::Context,
    ) -> bool {
        let mut display = T::default();
        if self.from.len() > 0 {
            display = self.from[self.selected].clone();
        }

        bevy_inspector_egui::egui::ComboBox::from_id_source(self.selected)
            .selected_text(format!("{}", display))
            .show_ui(ui, |ui| {
                for (index, value) in self.from.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, index, format!("{}", value));
                }
            });
        true
    }

    fn setup(_: &mut AppBuilder) {
        // eprintln!("Running setup code...");

        // app.init_resource::<WhateverYouNeed>();
    }
}
