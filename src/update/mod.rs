use std::clone;
use std::error::Error;

use crate::update::downloads::DownloadManager;
use crate::update::structs::mc_assets::AssetsRoot;
use crate::update::structs::mc_libs::LibsRoot;
use crate::update::structs::mc_versions::{Version, Versions};
use crate::update::utils::check_all_directories;

pub mod downloads;
pub mod utils;
mod structs;

pub struct Updater {
    local_dir_path: String,
    version: String,
    libs_manifest: Option<LibsRoot>,
    assets_manifest: Option<AssetsRoot>,
}

impl Updater {
    pub fn install_files(&mut self) -> Result<(), Box<dyn Error>> {
        self.get_files_list().unwrap();
        check_all_directories(self.local_dir_path.clone());

        if self.libs_manifest.is_none() {
            eprintln!("Please get the files list before installing files");
            return Err("Files list not available".into());
        }

        let libs_manifest = match &self.libs_manifest {
            Some(manifest) => manifest,
            None => return Err::<_, Box<dyn Error>>("Libs manifest not available".into()),
        };

        let assets_manifest = match &self.assets_manifest {
            Some(manifest) => manifest,
            None => return Err::<_, Box<dyn Error>>("Assets manifest not available".into()),
        };

        let mut download_manager = DownloadManager::new();
        download_manager.populate_libs(libs_manifest.clone(), self.local_dir_path.clone());
        download_manager.populate_assets(assets_manifest, self.local_dir_path.clone());
        download_manager.populate_game(libs_manifest.clone(), self.local_dir_path.clone());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            download_manager.download_all().await;
        });

        Ok(())
    }

    pub fn get_versions_list(&self) -> Option<Versions> {
        let client = reqwest::Client::new();
        let mut versions = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            if let Ok(res) = client
                .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await
            {
                if let Ok(text) = res.text().await {
                    if let Ok(parse) = serde_json::from_str(&*text) {
                        versions = Some(parse)
                    }
                }
            }
        });

        versions
    }
    pub fn get_files_list(&mut self) -> Result<(), Box<dyn Error>> {
        let mut is_version_correct = false;
        let mut version: Option<Version> = None;
        match self.get_versions_list() {
            None => {
                return Err("No versions found".into());
            }
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
        let mut libs_manifest: Option<LibsRoot> = None;
        let mut assets_manifest: Option<AssetsRoot> = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match version {
                None => {}
                Some(version) => {
                    if let Ok(res) = client.get(version.url()).send().await {
                        if let Ok(text) = res.text().await {
                            match serde_json::from_str::<LibsRoot>(&text) {
                                Ok(parse) => {
                                    libs_manifest = Some(parse);

                                    if let Ok(res) = client
                                        .get(libs_manifest.clone().unwrap().asset_index.url)
                                        .send()
                                        .await
                                    {
                                        if let Ok(text) = res.text().await {
                                            match serde_json::from_str::<AssetsRoot>(&text) {
                                                Ok(parse) => {
                                                    assets_manifest = Some(parse);
                                                }
                                                Err(e) => {
                                                    println!(
                                                        "Error parsing assets manifest: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        } else {
                                            println!("Error getting files list");
                                        }
                                    } else {
                                        println!("Error getting files list");
                                    }
                                }
                                Err(e) => {
                                    println!("Error parsing lib manifest: {}", e);
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

        self.libs_manifest = libs_manifest;
        self.assets_manifest = assets_manifest;
        Ok(())
    }

    // All the other basic methods are located in the utils.rs file for organization purposes
}
