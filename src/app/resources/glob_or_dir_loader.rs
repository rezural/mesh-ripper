use std::{fs::read_dir, path::Path};

use bevy::prelude::AssetServer;

use super::load_manager::LoadManager;

#[derive(Clone)]
pub struct GlobOrDirLoader {
    load_manager: LoadManager,
    glob: Option<String>,
    load_dirs: String,
}

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
            println!("dir_chosen {:?}", dir_chosen);

            if let Some(new_files) = self.get_files_from_load_dir(dir_chosen) {
                files.extend(new_files);
            }
        } else if let Some(glob) = glob {
            files.extend(Self::files_from_glob(glob))
        }
        self.load_manager.clear();
        self.load_manager.add_new_assets(files);
        self.load_manager.load_assets(server)
    }

    pub fn dirs_from_load_dir(&self) -> Option<Vec<String>> {
        if let Ok(entries) = read_dir(self.load_dirs.clone()) {
            let dirs: Vec<String> = entries
                .filter(|a| a.is_ok())
                .map(|a| a.unwrap())
                .filter(|a| a.path().is_dir())
                .map(|a| a.path().file_name().unwrap().to_string_lossy().to_string())
                .collect();
            if dirs.len() > 0 {
                return Some(dirs);
            }
        }
        None
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
            .map(|entry| entry.unwrap().to_string_lossy().to_string())
            .collect()
    }

    fn get_files_from_load_dir(
        &self,
        chosen: String,
    ) -> Option<Vec<String>> {
        println!("get_files_from_load_dir");
        if let Some(dirs) = self.dirs_from_load_dir() {
            println!("get_files_from_load_dir got Some from dirs_from_load_dir");
            if let Some(loading_from) = dirs.iter().find(|&d| *d == chosen) {
                println!("get_files_from_load_dir got Some find");
                println!("{}", loading_from);
                if let Ok(entries) =
                    read_dir(Path::new(self.load_manager().data_path().as_str()).join(loading_from))
                {
                    println!("get_files_from_load_dir got entries from read_dir");

                    let files: Vec<String> = entries
                        .filter(|e| e.is_ok())
                        .map(|e| e.unwrap().path())
                        .filter(|e| e.is_file())
                        .map(|e| e.to_string_lossy().to_string())
                        .collect();
                    return Some(files);
                }
            }
        }
        None
    }
}
