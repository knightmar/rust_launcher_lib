use crate::update::downloader::Downloader;
use crate::update::errors::UpdateErrors;
use crate::update::errors::UpdateErrors::{Download, Fetch, Install, Verification};
use crate::update::json_structs::{
    AssetIndex, AssetManifest, UniversalVersionJson, VersionEntry, VersionManifest,
};
use crate::update::structs::UpdateFile;
use futures_util::future::join_all;
use futures_util::TryStreamExt;
use reqwest::Response;
use serde_json::to_string;
use sha1::{Digest, Sha1};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;

pub(crate) mod downloader;
mod errors;
mod json_structs;
pub(crate) mod structs;

pub struct Updater {
    pub version: String,
    pub game_files_location: String,
    all_downloaded_files: Vec<UpdateFile>,
}

impl Updater {
    pub fn new(version: String, game_files_location: String) -> Self {
        Self {
            version,
            game_files_location,
            all_downloaded_files: Vec::new(),
        }
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    }
    pub(crate) async fn get_version_manifest() -> Result<VersionManifest, UpdateErrors> {
        Self::client()
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await
            .map_err(|e| Fetch(e.to_string()))?
            .json::<VersionManifest>()
            .await
            .map_err(|e| Fetch(e.to_string()))
    }

    pub(crate) async fn get_version(
        version: &VersionEntry,
    ) -> Result<UniversalVersionJson, UpdateErrors> {
        Self::client()
            .get(&version.url)
            .send()
            .await
            .map_err(|e| Fetch(e.to_string()))?
            .json()
            .await
            .map_err(|e1| Download(e1.to_string()))
    }

    pub(crate) async fn install_special_files(
        &mut self,
        version: &UniversalVersionJson,
    ) -> Result<(), UpdateErrors> {
        let index = version
            .clone()
            .asset_index
            .ok_or_else(|| Install("Asset index not found".to_string()))?;
        let client = version
            .clone()
            .downloads
            .ok_or_else(|| Download("Asset index not found".to_string()))?
            .client
            .ok_or_else(|| Download("Client exe not found".to_string()))?;

        let files = vec![
            UpdateFile {
                url: index.clone().url,
                name: index.clone().id,
                hash: index.clone().sha1,
                size: index.size as u64,
                local_path: Path::new(&self.game_files_location.clone())
                    .join(Path::new(
                        format!("assets/indexes/{}.json", index.clone().id).as_str(),
                    ))
                    .to_str()
                    .ok_or_else(|| Install("Path malformed".to_string()))?
                    .to_string(),
            },
            UpdateFile {
                url: client.url,
                name: "client.jar".to_string(),
                hash: client.sha1,
                size: client.size as u64,
                local_path: Path::new(&self.game_files_location.clone())
                    .join(Path::new("client.jar"))
                    .to_str()
                    .ok_or_else(|| Install("Path malformed".to_string()))?
                    .to_string(),
            },
        ];

        let mut downloader = Downloader::new(5).with_files(files.clone());
        downloader.download_all_files().await?;

        if !downloader.failed_files().is_empty() {
            return Err(Download("Failed to download some files".to_string()));
        }

        self.all_downloaded_files.extend(files);

        Ok(())
    }

    pub(crate) async fn install_assets(
        &mut self,
        version: &UniversalVersionJson,
    ) -> Result<(), UpdateErrors> {
        let index = version
            .clone()
            .asset_index
            .ok_or_else(|| Install("Asset index not found".to_string()))?;

        let assets_manifest = Self::client()
            .get(index.url.clone())
            .send()
            .await
            .map_err(|e| Download(e.to_string()))?
            .json::<AssetManifest>()
            .await
            .map_err(|e1| Download(e1.to_string()))?;

        let mut files = Vec::new();
        for (field, entry) in assets_manifest.objects {
            let two_char_hash = &entry.hash[0..2];
            let file_url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                two_char_hash, entry.hash
            );
            let file_path = Path::new(&self.game_files_location)
                .join(format!("assets/objects/{}/{}", two_char_hash, entry.hash));
            files.push(UpdateFile {
                url: file_url,
                name: field,
                hash: entry.hash,
                size: entry.size,
                local_path: file_path
                    .to_str()
                    .ok_or_else(|| Install("Path malformed".to_string()))?
                    .to_string(),
            })
        }

        let mut downloader = Downloader::new(10).with_files(files.clone());
        downloader.download_all_files().await?;

        if !downloader.failed_files().is_empty() {
            return Err(Download("Failed to download some files".to_string()));
        }

        self.all_downloaded_files.extend(files);

        Ok(())
    }

    pub(crate) async fn install_version(&mut self) -> Result<(), UpdateErrors> {
        println!("Installing version {}", self.version);

        let manifest = Self::get_version_manifest().await?;
        println!("Manifest found for requested version");

        let requested_version = self.version.clone();

        let version_details = manifest
            .versions
            .iter()
            .find(|v| v.id == requested_version)
            .ok_or_else(|| {
                UpdateErrors::Install(format!("Version : {requested_version} not found"))
            })?;

        let version = Self::get_version(version_details).await?;
        println!("Version details found");

        println!("Starting install");
        self.install_special_files(&version).await?;
        self.install_assets(&version).await?;
        println!("Install done, verifying");

        let failed = verify_files(self.all_downloaded_files.clone(), 10).await;

        println!(
            "Verifying complete with {} errors",
            if failed.is_ok() {
                "0".to_string()
            } else {
                failed.err().unwrap().len().to_string()
            }
        );

        Ok(())
    }
}

pub(crate) async fn verify_files(
    files: Vec<UpdateFile>,
    max_concurrent_verify: usize,
) -> Result<(), Vec<UpdateFile>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_verify));

    let tasks = files.iter().map(|file| {
        let file = file.clone();
        let semaphore = semaphore.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            verify_file(file).await
        })
    });

    let results = join_all(tasks).await;
    let mut failed = Vec::new();

    for (file, result) in files.into_iter().zip(results) {
        match result {
            Ok(Ok(true)) => {}
            _ => {
                failed.push(file);
            }
        }
    }

    if failed.is_empty() {
        Ok(())
    } else {
        Err(failed)
    }
}

pub(crate) async fn verify_file(file: UpdateFile) -> Result<bool, UpdateErrors> {
    let mut hasher = Sha1::new();
    let mut local_file = File::open(&file.local_path)
        .await
        .map_err(|_| UpdateErrors::Download("Failed to open local file".to_string()))?;

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = local_file
            .read(&mut buffer)
            .await
            .map_err(|_| UpdateErrors::Download("Failed to read file".to_string()))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hash_hex = hex::encode(hasher.finalize());

    let is_valid = hash_hex.eq_ignore_ascii_case(&file.hash);

    if !is_valid {
        eprintln!(
            "[FAILED] file {}: local={}, expected={}",
            file.local_path, hash_hex, file.hash
        );
    }

    Ok(is_valid)
}
