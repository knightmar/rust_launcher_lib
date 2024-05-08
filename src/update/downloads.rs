use std::fs;
use std::fs::create_dir_all;
use std::hash::Hash;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use futures::future::join_all;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime;
use tokio::task::JoinHandle;

use crate::update::java;
use crate::update::structs::mc_assets::AssetsRoot;
use crate::update::structs::mc_libs::LibsRoot;
use crate::update::utils::{check_file_hash, Directory, get_file_name_from_url};

#[derive(Clone, PartialEq)]
struct DownloadElement {
    url: String,
    path: String,
    dl_tries: u8,
    hash: Option<String>,
}

#[derive(Clone)]
pub struct DownloadManager {
    client: Arc<reqwest::Client>,
    elements: Vec<DownloadElement>,
    failed: Arc<Mutex<Vec<DownloadElement>>>,
}
impl DownloadManager {
    pub(crate) async fn download_file(
        client: Arc<reqwest::Client>,
        url: &str,
        path: &str,
        hash: &Option<String>,
    ) -> Result<(), String> {
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
        let parent_dir = std::path::Path::new(path).parent().unwrap();
        tokio::fs::create_dir_all(parent_dir)
            .await
            .map_err(|_| format!("Failed to create directories for file at {}", path))?;

        // Create a new async file and write the bytes into it
        let mut file = File::create(path)
            .await
            .map_err(|_| format!("Failed to create file at {}", path))?;
        file.write_all(&bytes)
            .await
            .map_err(|_| format!("Failed to write to file at {}", path))?;

        if let Some(hash) = hash {
            if !check_file_hash(path, hash.as_str()) {
                return Err(format!("Hash mismatch for file at {}", path));
            }
        }

        Ok(())
    }

    #[async_recursion(?Send)]
    pub(crate) async fn download_all(&mut self) {
        println!("Downloading all files: {}", self.elements.len());
        let mut tasks: Vec<JoinHandle<()>> = vec![];

        self.failed.lock().unwrap().clear();

        for element in self.elements.iter() {
            let url = element.url.clone();
            let path = element.path.clone();
            let dl_tries = element.dl_tries;
            let hash = element.hash.clone();

            // Clone client and failed here
            let client = Arc::clone(&self.client);
            let failed = Arc::clone(&self.failed);

            tasks.push(tokio::spawn(async move {
                match DownloadManager::download_file(client, &url, &path, &hash).await {
                    Ok(_) => {
                        //println!("Downloaded file: {}", path);
                    }
                    Err(e) => {
                        {
                            let mut failed = failed.lock().unwrap();
                            failed.push(DownloadElement {
                                url,
                                path,
                                dl_tries: dl_tries + 1,
                                hash,
                            });
                        }
                        println!("Error downloading file: {}", e);
                    }
                }
            }));
        }

        join_all(tasks).await;

        println!(
            "Downloaded all files, {} errors appends.",
            self.failed.lock().unwrap().len()
        );

        self.elements.clear();

        if self.failed.lock().unwrap().len() > 0 {
            self.download_failed_files().await;
        }
    }

    #[async_recursion(?Send)]
    async fn download_failed_files(&mut self) {
        println!(
            "Downloading failed files : {}",
            self.failed.lock().unwrap().len()
        );
        {
            let failed = self.failed.lock().unwrap();
            self.elements = failed.clone();

            for failed_element in failed.iter() {
                let _ = fs::remove_file(&failed_element.path).is_err();

                if failed_element.dl_tries > 3 {
                    println!(
                        "Failed to download file after 3 tries: {}",
                        failed_element.path
                    );
                    self.failed.lock().unwrap().retain(|x| x != failed_element);
                    continue;
                }
            }
        }

        let mut cloned_self = self.clone();
        cloned_self.download_all().await;
    }

    pub fn populate_libs(&mut self, manifest: LibsRoot, local_dir_path: String) {
        let libraries = manifest.libraries.clone();

        // Iterate over all libraries and download them
        for lib in libraries {
            let lib_path = local_dir_path.to_string()
                + &Directory::Libraries.as_str()
                + get_file_name_from_url(&lib.downloads.artifact.url).as_str();

            //check if the file already exists
            if std::path::Path::new(&lib_path).exists() {
                continue;
            }

            self.elements.push(DownloadElement {
                url: lib.downloads.artifact.url,
                path: lib_path,
                dl_tries: 0,
                hash: Some(lib.downloads.artifact.sha1),
            });
        }

        // Download the assets index (same function because of the url is in the LibsRoot struct)
        let assets_index_url = manifest.asset_index.url;
        let assets_index_file_path = local_dir_path.to_string()
            + &Directory::Indexes.as_str()
            + get_file_name_from_url(&assets_index_url).as_str();

        if std::path::Path::new(&assets_index_file_path).exists() {
            //println!("File already exists: {}", path_builder);
        } else {
            //println!("Downloading: {}", path_builder);

            self.elements.push(DownloadElement {
                url: assets_index_url,
                path: assets_index_file_path,
                dl_tries: 0,
                hash: Some(manifest.asset_index.sha1),
            });
        }
    }

    pub fn populate_assets(&mut self, manifest: &AssetsRoot, local_dir_path: String) {
        // Iterate over all assets
        let assets = manifest.objects();
        for asset in assets.values() {
            // Construct the URL for the asset
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &asset.hash()[0..2],
                &asset.hash()
            );

            // Construct the path where the asset will be stored
            let file_path = local_dir_path.to_string()
                + &*Directory::Assets.as_str()
                + "objects"
                + std::path::MAIN_SEPARATOR_STR
                + asset.hash()[0..2].to_string().as_str()
                + std::path::MAIN_SEPARATOR_STR
                + asset.hash();

            let mut parts: Vec<&str> = file_path.split(std::path::MAIN_SEPARATOR_STR).collect();
            parts.pop();
            parts.pop();
            let directory_path = parts.join(std::path::MAIN_SEPARATOR_STR);

            // Check if the directory already exists
            if !std::path::Path::new(&directory_path).exists() {
                create_dir_all(&directory_path).unwrap();
            }

            if std::path::Path::new(&file_path).exists() {
                continue;
            } else {
                //println!("Downloading: {}", url);

                self.elements.push(DownloadElement {
                    url,
                    path: file_path,
                    dl_tries: 0,
                    hash: Some(asset.hash().to_string()),
                });
            }
        }
    }

    pub fn populate_game(&mut self, manifest: LibsRoot, local_dir_path: String) {
        let client_path = local_dir_path.to_string() + "client.jar";
        let client_url = manifest.downloads.client.url.clone();

        if std::path::Path::new(&client_path).exists() {
            println!("File already exists: {}", client_path);
        } else {
            println!("Downloading: {}", client_path);
        }
        self.elements.push(DownloadElement {
            url: client_url,
            path: client_path,
            dl_tries: 0,
            hash: Some(manifest.downloads.client.sha1.clone()),
        });
    }

    pub(crate) fn install_java(
        &mut self,
        version: String,
        local_dir_path: String,
    ) -> Result<(), String> {
        let java_path = local_dir_path.to_string() + &*Directory::Runtime.as_str() + "java.zip";
        let java_url = java::get_java_zulu_dl_link(version)
            .unwrap_or_else(|e| {
                println!("Error getting Java download link: {}", e);
                e.to_string()
            });
        if std::path::Path::new(&(local_dir_path.to_string() + &(Directory::Runtime.as_str() + "bin"))).exists() {
            println!("File already exists: {}", java_path);
            return Ok(());
        } else {
            println!("Downloading: {}", java_path);
        }

        let runtime = runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            DownloadManager::download_file(
                Arc::clone(&self.client),
                java_url.as_str(),
                java_path.as_str(),
                &None,
            )
            .await
            .expect("TODO: panic message");
        });

        if zip_extract::extract(
            Cursor::new(fs::read(&java_path).unwrap()),
            (local_dir_path + &*Directory::Runtime.as_str()).as_ref(),
            true,
        )
        .is_err()
        {
            return Err("Failed to extract the file".parse().unwrap());
        }

        if fs::remove_file(java_path.clone()).is_err() {
            return Err("Failed to remove the zip file".parse().unwrap());
        }

        Ok(())
    }
    pub fn new() -> Self {
        Self {
            client: Arc::new(reqwest::Client::new()),
            elements: vec![],
            failed: Arc::new(Mutex::new(vec![])),
        }
    }
}
