use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use reqwest::Client;

use crate::update::structs::mc_assets::Object;
use crate::update::structs::mc_libs::{Library, LibsRoot};
use crate::update::utils::{get_asset_path_from_hash, get_lib_path_from_url};

#[derive(Clone, PartialEq)]
pub struct DownloadElement {
    pub url: String,
    pub path: String,
    pub dl_tries: u8,
    pub hash: Option<String>,
}

#[derive(Clone)]
pub struct DownloadManager {
    client: Arc<Client>,
    fails: Vec<DownloadElement>,
    local_dir_path: String,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            client: Arc::new(Client::new()),
            fails: vec![],
            local_dir_path: String::new(),
        }
    }
}

impl DownloadManager {
    pub async fn download_libs(&mut self, libs: Vec<Library>) {
        for lib in libs {
            let download_path =
                get_lib_path_from_url(self.local_dir_path.clone(), &lib.downloads.artifact.path);

            if let Err(_result) = DownloadManager::download_file(
                self.client.clone(),
                &lib.downloads.artifact.url,
                download_path.clone(),
                &Some(lib.downloads.artifact.sha1.clone()),
            )
            .await
            {
                self.fails.push(DownloadElement {
                    url: lib.downloads.artifact.url,
                    path: download_path,
                    dl_tries: 0,
                    hash: Some(lib.downloads.artifact.sha1),
                })
            }
        }
    }

    pub async fn download_assets(&mut self, assets: &HashMap<String, Object>) {
        for asset in assets {
            let hash = asset.1.hash();
            let url = &("https://resources.download.minecraft.net/".to_string()
                + hash[0..2].to_string().as_str()
                + std::path::MAIN_SEPARATOR_STR
                + hash);

            let download_path = get_asset_path_from_hash(self.local_dir_path.clone(), hash);

            if let Err(result) = Self::download_file(
                self.client.clone(),
                url,
                download_path.1.to_string(),
                &Some(hash.to_string()),
            )
            .await
            {
                self.fails.push(DownloadElement {
                    url: url.to_string(),
                    path: download_path.1,
                    dl_tries: 0,
                    hash: Some(hash.to_string()),
                })
            }
        }
    }

    pub async fn download_java(&self, java_version: u8) {}

    pub async fn download_game_files(&self, root: LibsRoot) {}

    pub async fn download_fails(&self) {}

    pub fn new(path: String) -> Self {
        Self {
            local_dir_path: path,
            ..Self::default()
        }
    }

    pub fn fails(&self) -> &Vec<DownloadElement> {
        &self.fails
    }
}
