use super::lod_midpoint_iterator::MidpointIterator;
use bevy::{
    asset::LoadState,
    prelude::{AssetServer, Handle, HandleUntyped, Mesh},
};
use std::path::Path;

type VecAssetLoading = Vec<(String, HandleUntyped)>;
type VecAssetLoaded = Vec<(String, Handle<Mesh>)>;

#[derive(Clone)]
pub struct LoadManager {
    //FIXME: make this an LodIterator trait
    pub load_iterator: MidpointIterator<String>,
    pub loaded: VecAssetLoaded,
    pub loading: VecAssetLoading,
}

impl LoadManager {
    pub fn new(load_iterator: MidpointIterator<String>) -> Self {
        Self {
            load_iterator,
            loaded: Vec::new(),
            loading: Vec::new(),
        }
    }

    pub fn load_assets(
        &mut self,
        server: &AssetServer,
    ) {
        let to_load: VecAssetLoading = self
            .load_iterator
            .clone()
            .map(|fluid_file| {
                (
                    fluid_file.clone(),
                    server.load_untyped(Path::new(&fluid_file).strip_prefix("assets/").unwrap()),
                )
            })
            .collect();
        self.loading.extend(to_load);
        // println!("load_assets: loading len: {}", self.loading.len());
    }

    pub fn update_load_state(
        &mut self,
        server: &AssetServer,
    ) {
        let newly_loaded: VecAssetLoaded = self
            .loading
            .iter()
            .filter(|(_, h)| LoadState::Loaded == server.get_load_state(h))
            .map(|(f, h)| (f.clone(), server.get_handle(h)))
            .collect();
        self.loaded.extend(newly_loaded);

        self.loading = self
            .loading
            .iter()
            .filter(|(_, h)| !(LoadState::Loaded == server.get_load_state(h)))
            .cloned()
            .collect();
    }

    pub fn next_lod_and_reload(
        &mut self,
        server: &AssetServer,
    ) {
        self.next_lod();

        let loading: Vec<(String, HandleUntyped)> = self
            .load_iterator
            .clone()
            .filter(|f| !self.in_loaded_or_loading(f.clone()))
            .map(|f| {
                (
                    f.clone(),
                    server.load_untyped(Path::new(&f).strip_prefix("assets/").unwrap()),
                )
            })
            .collect();

        self.loading.extend(loading)
    }

    pub fn next_lod(&mut self) {
        if let Some(next_lod) = self.load_iterator.next_lod() {
            self.load_iterator = next_lod;
        }
    }

    fn in_loaded_or_loading(
        &self,
        path: String,
    ) -> bool {
        self.loading.iter().any(|f| f.0 == *path) || self.loaded.iter().any(|f| f.0 == *path)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_three() {}
}
