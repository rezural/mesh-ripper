use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use rand::Rng;
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{Debug, Display},
    hash::Hasher,
    ops::BitXor,
};

#[derive(Debug)]
pub struct VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Debug,
{
    from: Vec<T>,
    selected: usize,
    changed: bool,
}

impl<T> VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Default + Debug,
{
    pub fn new(from: Vec<T>) -> Self {
        Self::new_with_selected(from, 0)
    }

    pub fn new_with_selected(
        from: Vec<T>,
        selected: usize,
    ) -> Self {
        Self {
            from,
            selected,
            changed: false,
        }
    }

    pub fn selected_value(&self) -> Option<T> {
        if self.from.len() > 0 {
            Some(self.from[self.selected].clone())
        } else {
            None
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
}

impl<T> Default for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Debug,
{
    fn default() -> Self {
        Self {
            from: Vec::new(),
            selected: 0,
            changed: false,
        }
    }
}

impl<T> Inspectable for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Sized + Debug + Default + std::hash::Hash,
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
        };
        let mut rng = rand::thread_rng();
        let mut hash: u64 = rng.gen();
        if let Some(try_hash) = self
            .from
            .iter()
            .map(|a| {
                let mut hasher = DefaultHasher::new();
                a.hash(&mut hasher);
                hasher.finish()
            })
            .reduce(|a, b| a.bitxor(b))
        {
            hash = try_hash;
        }

        let mut changed = false;
        bevy_inspector_egui::egui::ComboBox::from_id_source(hash)
            .selected_text(format!("{}", display))
            .show_ui(ui, |ui| {
                for (index, value) in self.from.iter().enumerate() {
                    changed = ui
                        .selectable_value(&mut self.selected, index, format!("{}", value))
                        .changed()
                        || changed
                }
            })
            .changed();
        self.changed = changed;
        self.changed
    }

    fn setup(_: &mut AppBuilder) {
        // app.init_resource::<WhateverYouNeed>();
    }
}
