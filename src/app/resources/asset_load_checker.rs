use std::marker::PhantomData;

use bevy::{
    asset::{Asset, LoadState},
    prelude::*,
};

/// This checks a trait, to see what has been loaded, and what hasnt
pub struct AssetLoadChecker<T> {
    marker: PhantomData<T>,
}

impl<T> AssetLoadChecker<T>
where
    T: Asset,
{
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
    pub fn update(
        &self,
        loading_source: &mut dyn LoadingSource<T>,
        server: &AssetServer,
    ) {
        {
            let loading = loading_source.loading_mut();

            let newly_loaded: Vec<Handle<T>> = loading
                .iter()
                .filter(|&h| LoadState::Loaded == server.get_load_state(h))
                .map(|h| server.get_handle(h))
                .collect();
            loading_source.loaded_mut().extend(newly_loaded.clone());
        }
        let loading = loading_source.loading_mut();

        let still_loading: Vec<HandleUntyped> = loading
            .iter()
            .filter(|&h| !(LoadState::Loaded == server.get_load_state(h)))
            .cloned()
            .collect();
        loading.clear();
        loading.extend(still_loading);
    }
}

pub trait LoadingSource<T>: Send + Sync
where
    T: Asset,
{
    fn loading_mut(&mut self) -> &mut Vec<HandleUntyped>;
    fn loaded_mut(&mut self) -> &mut Vec<Handle<T>>;
}
