use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;

use reqwest::Client;

use crate::update::java;
use crate::update::structs::mc_assets::Object;
use crate::update::structs::mc_libs::{Library, LibsRoot};
use crate::update::utils::{
    Directory, get_asset_path_from_hash, get_file_name_from_url, get_lib_path_from_url,
};

// struct that describe an element to download
#[derive(Clone, PartialEq)]
pub struct DownloadElement {
    pub url: String,
    pub path: String,
    pub dl_tries: u8,
    pub hash: Option<String>,
}

// base struct that is responsible to manage the downloads
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
    // download of the libs
    pub async fn download_libs(&mut self, libs: Vec<Library>) {
        println!("Downloading libs");
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

    // download of the assets
    pub async fn download_assets(&mut self, assets: &HashMap<String, Object>) {
        println!("Downloading assets");
        for asset in assets {
            let hash = asset.1.hash();
            let url = &("https://resources.download.minecraft.net/".to_string()
                + hash[0..2].to_string().as_str()
                + std::path::MAIN_SEPARATOR_STR
                + hash);

            let download_path = get_asset_path_from_hash(self.local_dir_path.clone(), hash);

            if let Err(_result) = Self::download_file(
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

    // download + unzip of the java runtime in the correct dir
    pub async fn download_java(&self, java_version: String) {
        println!("Downloading java");
        let java_path =
            self.local_dir_path.to_string() + &*Directory::Runtime.as_str() + "java.zip";
        let java_url = java::get_java_zulu_dl_link(java_version).await.unwrap();
        if std::path::Path::new(
            &(self.local_dir_path.to_string() + &(Directory::Runtime.as_str() + "bin")),
        )
            .exists()
        {
            println!("File already exists: {}", java_path);
            return;
        } else {
            println!("Downloading: {}", java_path);
        }

        DownloadManager::download_file(
            Arc::clone(&self.client),
            java_url.as_str(),
            java_path.clone(),
            &None,
        )
            .await
            .expect("TODO: panic message");

        if zip_extract::extract(
            Cursor::new(fs::read(&java_path).unwrap()),
            (self.local_dir_path.clone() + &*Directory::Runtime.as_str()).as_ref(),
            true,
        )
            .is_err()
        {
            return;
        }

        if fs::remove_file(java_path.clone()).is_err() {}
    }

    // download of the client.jar + asset index
    pub async fn download_game_files(&mut self, root: LibsRoot) {
        let mut files_to_dl: Vec<DownloadElement> = vec![];

        println!("Downloading game files");
        let client_path = self.local_dir_path.to_string() + "client.jar";
        let client_url = root.client.url.clone();

        files_to_dl.push(DownloadElement {
            url: client_url,
            path: client_path,
            dl_tries: 0,
            hash: Some(root.client.sha1),
        });

        files_to_dl.push(DownloadElement {
            url: root.asset_index.url.clone(),
            path: self.local_dir_path.clone()
                + &Directory::Indexes.as_str()
                + &get_file_name_from_url(root.asset_index.url.as_str()),
            dl_tries: 0,
            hash: Some(root.asset_index.sha1),
        });

        for file in files_to_dl {
            if let Err(_result) = Self::download_file(
                self.client.clone(),
                file.url.as_str(),
                file.path.clone(),
                &file.hash,
            )
                .await
            {
                self.fails.push(file);
            };
        }
    }

    // function that takes the failed downloads of the other download functions, and re-dl the fills that had errors
    pub async fn download_fails(&mut self) {
        while !self.fails.is_empty() {
            let current_fails = std::mem::take(&mut self.fails);
            for fail in current_fails {
                if Self::download_file(
                    self.client.clone(),
                    &fail.url,
                    fail.path.clone(),
                    &fail.hash,
                )
                    .await.is_err()
                {
                    self.fails.push(DownloadElement {
                        url: fail.url,
                        path: fail.path,
                        dl_tries: fail.dl_tries + 1,
                        hash: fail.hash,
                    });
                }
            }
        }
    }

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
