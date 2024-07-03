use std::error::Error;
use std::sync::Arc;

use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::update::downloads::DownloadManager;
use crate::update::structs::mc_assets::AssetsRoot;
use crate::update::structs::mc_libs::LibsRoot;
use crate::update::structs::mc_versions::{Version, Versions};
use crate::update::updater::Updater;
use crate::update::utils::check_file_hash;

pub mod downloads;
pub(crate) mod java;
pub mod structs;
pub mod utils;
pub(crate) mod updater;


impl Updater {
    // get the version list from the json
    pub fn get_versions_list(&self) -> Option<Versions> {
        let client = Client::new();
        let mut versions = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            if let Ok(res) = client
                .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await
            {
                if let Ok(text) = res.text().await {
                    if let Ok(parse) = serde_json::from_str(&text) {
                        versions = Some(parse)
                    }
                }
            }
        });

        versions
    }

    // load the Updater instance fields libs_manifest and assets_manifest with the matching files for the good version of the game
    pub fn update_files_list(&mut self) -> Result<(), Box<dyn Error>> {
        let mut is_version_correct = false;
        let mut version: Option<Version> = None;
        match self.get_versions_list() {
            None => {
                return Err("No versions found".into());
            }
            Some(mc_versions) => {
                for version_item in mc_versions.versions() {
                    if version_item.id() == self.version() {
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

        let client = Client::new();
        let mut libs_manifest: Option<LibsRoot> = None;
        let mut assets_manifest: Option<AssetsRoot> = None;

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match version {
                None => {}
                Some(version) => {
                    if let Ok(res) = client.get(version.url()).send().await {
                        if let Ok(text) = res.text().await {
                            match LibsRoot::parse_json(text) {
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

        self.set_libs_manifest(libs_manifest);
        self.set_assets_manifest(assets_manifest);
        Ok(())
    }
}

impl DownloadManager {
    // function that download a file, if it is not existing, and then check its hash
    pub(crate) async fn download_file(
        client: Arc<Client>,
        url: &str,
        path: String,
        hash: &Option<String>,
    ) -> Result<(), String> {
        if std::path::Path::new(&path).exists() {
            return Ok(());
        }

        // Send a GET request
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|_| format!("Failed to send GET request to {}", url))?;

        //println!("Downloading: {}", url);

        // Get the bytes of the file
        let bytes = response
            .bytes()
            .await
            .map_err(|_| format!("Failed to get bytes from {}", url))?;

        //println!("Downloaded {} bytes", bytes.len());

        // Create the directories leading up to the file
        let parent_dir = std::path::Path::new(path.as_str()).parent().unwrap();
        tokio::fs::create_dir_all(parent_dir)
            .await
            .map_err(|_| format!("Failed to create directories for file at {}", path))?;

        // Create a new async file and write the bytes into it
        let mut file = File::create(path.as_str())
            .await
            .map_err(|_| format!("Failed to create file at {}", path))?;
        file.write_all(&bytes)
            .await
            .map_err(|_| format!("Failed to write to file at {}", path))?;

        if let Some(hash) = hash {
            if !check_file_hash(path.as_str(), hash.as_str()) {
                return Err(format!("Hash mismatch for file at {}", path));
            }
        }

        Ok(())
    }
}