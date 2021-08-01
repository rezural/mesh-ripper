use std::{fs::read_dir, path::Path};

use bevy::prelude::AssetServer;
use walkdir::WalkDir;

use super::load_manager::LoadManager;

#[derive(Clone)]
pub struct GlobOrDirLoader {
    load_manager: LoadManager,
    glob: Option<String>,
    load_dirs: String,
}

const FILE_EXTENSIONS: [&str; 3] = ["obj", "ply", "stl"];

impl GlobOrDirLoader {
    pub fn new(
        load_manager: LoadManager,
        glob: Option<String>,
        load_dirs: String,
    ) -> Self {
        Self {
            load_manager,
            glob,
            load_dirs,
        }
    }

    pub fn update(
        &mut self,
        glob: Option<String>,
        load_dir_chosen: Option<String>,
        server: &AssetServer,
    ) {
        let mut files = Vec::new();
        // load_dir_chosen takes precedence over glob passed via command line
        if let Some(dir_chosen) = load_dir_chosen {
            if let Some(new_files) = self.get_files_from_load_dir(dir_chosen) {
                files.extend(new_files);
            }
        } else if let Some(glob) = glob {
            files.extend(Self::files_from_glob(glob))
        }
        // self.load_manager.clear();
        self.load_manager.add_new_assets(files);
        self.load_manager.load_assets(server)
    }

    pub fn dirs_from_load_dir(&self) -> Option<Vec<String>> {
        let walker = WalkDir::new(self.load_dirs.clone()).follow_links(true);
        let dirs = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();
        return Some(dirs);
    }

    pub fn load_manager_mut(&mut self) -> &mut LoadManager {
        &mut self.load_manager
    }

    pub fn load_manager(&self) -> &LoadManager {
        &self.load_manager
    }

    fn files_from_glob(glob: String) -> Vec<String> {
        glob::glob(glob.as_str())
            .expect("Loading fluid from assets failed in glob")
            .map(|entry| {
                entry
                    .unwrap()
                    .strip_prefix("assets/")
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }

    fn get_files_from_load_dir(
        &self,
        chosen: String,
    ) -> Option<Vec<String>> {
        if let Some(dirs) = self.dirs_from_load_dir() {
            if let Some(loading_from) = dirs.iter().find(|&d| *d == chosen) {
                if let Ok(entries) = read_dir(Path::new(loading_from)) {
                    let files: Vec<String> = entries
                        .filter(|e| e.is_ok())
                        .map(|e| e.unwrap().path())
                        .filter(|f| f.is_file())
                        .filter(|f| {
                            FILE_EXTENSIONS
                                .iter()
                                .find(|ext| {
                                    let r = &*(f.extension().unwrap_or_default().to_string_lossy());
                                    &r == *ext
                                })
                                .is_some()
                        })
                        .map(|e| {
                            e.strip_prefix("assets/")
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect();
                    return Some(files);
                }
            }
        }
        None
    }
}
