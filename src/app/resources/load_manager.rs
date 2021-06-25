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
        self.loading = self
            .load_iterator
            .clone()
            .map(|fluid_file| {
                (
                    fluid_file.clone(),
                    server.load_untyped(Path::new(&fluid_file).strip_prefix("assets/").unwrap()),
                )
            })
            .collect();
        println!("load_assets: loading len: {}", self.loading.len());
    }

    pub fn update_load_state(
        &mut self,
        server: &AssetServer,
    ) {
        //TODO: this is not working correctly (len of both is always zero)
        let newly_loaded: VecAssetLoaded = self
            .loading
            .iter()
            .filter(|(_, handle)| LoadState::Loaded == server.get_load_state(handle))
            .map(|(file, handle)| (file.clone(), server.get_handle(handle)))
            .collect();
        self.loaded.extend(newly_loaded);

        self.loading = self
            .loading
            .iter()
            .filter(|(_, handle)| !(LoadState::Loaded == server.get_load_state(handle)))
            .cloned()
            .collect();
        println!(
            "update_load_state: len: {}, {}",
            self.loading.len(),
            self.loaded.len()
        );
    }

    pub fn next_lod(&mut self) {
        if let Some(next_lod) = self.load_iterator.next_lod() {
            self.load_iterator = next_lod;
        }
    }

    pub fn next_lod_and_reload(
        &mut self,
        server: &AssetServer,
    ) {
        self.next_lod();

        let loading: Vec<(String, HandleUntyped)> = self
            .load_iterator
            .clone()
            .filter(|to_load| !self.loading.iter().any(|f| f.0 == *to_load))
            .filter(|to_load| !self.loaded.iter().any(|f| f.0 == *to_load))
            .map(|fluid_file| {
                (
                    fluid_file.clone(),
                    server.load_untyped(Path::new(&fluid_file).strip_prefix("assets/").unwrap()),
                )
            })
            .collect();

        self.loading.extend(loading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three() {}
}
