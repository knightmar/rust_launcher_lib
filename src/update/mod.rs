use std::error::Error;

use crate::update::mc_files::Root;
use crate::update::mc_versions::{Version, Versions};

mod mc_files;
mod mc_versions;

pub struct Updater {
    local_dir_path: String,
    version: String,
    manifest: Option<Root>,
}

impl Updater {
    pub fn install_files(&mut self) {
        self.get_versions_list();
        self.get_files_list().unwrap();
    }

    fn get_versions_list(&self) -> Option<Versions> {
        let client = reqwest::Client::new();
        let mut versions = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            if let Ok(res) = client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .send().await {
                if let Ok(text) = res.text().await {
                    if let Ok(parse) = serde_json::from_str(&*text) {
                        versions = Some(parse)
                    }
                }
            }
        });

        versions
    }
    fn get_files_list(&mut self) -> Result<(), Box<dyn Error>> {
        let mut is_version_correct = false;
        let mut version: Option<Version> = None;
        match self.get_versions_list() {
            None => { return Err("No versions found".into()); }
            Some(mc_versions) => {
                for version_item in mc_versions.versions() {
                    if version_item.id() == self.version {
                        is_version_correct = true;
                        version = Some((*version_item).clone());
                        break;
                    }
                }
            }
        }

        if !is_version_correct {
            println!("Version not found");
            return Err("Version not found".into());
        }

        let client = reqwest::Client::new();
        let mut manifest: Option<Root> = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match version {
                None => { return; }
                Some(version) => {
                    if let Ok(res) = client.get("https://piston-meta.mojang.com/v1/packages/1faf090ed4b4fd9d49e5f229a2315e3b6941dc9e/1.20.2.json")
                        .send().await {
                        if let Ok(text) = res.text().await {
                            println!("{}", text);
                            match serde_json::from_str::<Root>(&text) {
                                Ok(parse) => {
                                    manifest = Some(parse);
                                }
                                Err(e) => {
                                    println!("Error parsing manifest: {}", e);
                                }
                            }
                        } else {
                            println!("Error getting files list");
                        }
                    } else {
                        println!("Error getting files list");
                    }
                }
            }
        });

        self.manifest = manifest;
        Ok(())
    }

    pub fn local_dir_path(&self) -> &str {
        &self.local_dir_path
    }
    pub fn version(&self) -> &String {
        &self.version
    }
    pub fn set_full_local_dir_path(&mut self, local_dir_path: String) {
        self.local_dir_path = local_dir_path;
    }
    pub fn set_relative_local_dir_path(&mut self, local_dir_path: &str) {
        #[cfg(unix)]{
            let app_Root = std::env::var("HOME").expect("No HOME directory");
            self.local_dir_path = app_Root + "/" + local_dir_path;
        }

        #[cfg(windows)] {
            let app_Root = std::env::var("APPDATA").expect("No APP_Root directory");
            self.local_dir_path = app_Root + "\\" + local_dir_path;
        }
    }
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }
    pub fn new(version: &str) -> Self {
        Self { local_dir_path: String::new(), version: String::from(version), manifest: None }
    }
    pub fn manifest(&self) -> &Option<Root> {
        &self.manifest
    }
}