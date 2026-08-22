use crate::update::downloader::Downloader;
use crate::update::errors::UpdateErrors;
use crate::update::errors::UpdateErrors::{Download, Fetch, Install, Verification};
use crate::update::json_structs::{
    AssetIndex, AssetManifest, Library, Rule, UniversalVersionJson, VersionEntry, VersionManifest,
};
use crate::update::structs::UpdateFile;
use futures_util::TryStreamExt;
use futures_util::future::join_all;
use reqwest::Response;
use serde_json::to_string;
use sha1::{Digest, Sha1};
use std::env::args_os;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use walkdir::WalkDir;
use zip::ZipArchive;

pub(crate) mod downloader;
mod errors;
mod json_structs;
pub(crate) mod structs;

pub struct Updater {
    version: String,
    game_files_location: String,
    all_downloaded_files: Vec<UpdateFile>,
    all_failed_files: Vec<UpdateFile>,
}

impl Updater {
    pub fn new(version: String, game_files_location: String) -> Self {
        Self {
            version,
            game_files_location,
            all_downloaded_files: Vec::new(),
            all_failed_files: Vec::new(),
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
            self.all_failed_files
                .extend(downloader.failed_files().clone());
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
            self.all_failed_files
                .extend(downloader.failed_files().clone());
        }

        self.all_downloaded_files.extend(files);

        Ok(())
    }

    pub(crate) async fn install_libs(
        &mut self,
        version: &UniversalVersionJson,
    ) -> Result<(), UpdateErrors> {
        let os = Self::get_os();
        let arch = Self::get_arch();
        let mut files = Vec::new();

        for lib in version.libraries.clone() {
            let should_include = match &lib.rules {
                Some(rules) => Self::check_rules(rules, &os, &arch),
                None => true,
            };

            if !should_include {
                continue;
            }

            let is_native = Self::is_native_library(&lib);

            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if let Some(artifact_path) = &artifact.path {
                        let local_path = Path::new(&self.game_files_location)
                            .join("libraries")
                            .join(artifact_path)
                            .to_str()
                            .ok_or_else(|| UpdateErrors::Install("Path malformed".to_string()))?
                            .to_string();

                        files.push(UpdateFile {
                            url: artifact.url.clone(),
                            hash: artifact.sha1.clone(),
                            size: artifact.size as u64,
                            local_path,
                        });
                    }
                }

                if let Some(classifiers) = &downloads.classifiers {
                    if let Some(natives) = &lib.natives {
                        if let Some(native_key) = natives.get(&os) {
                            let native_key = native_key.replace("${arch}", &arch);
                            if let Some(native_artifact) = classifiers.get(&native_key) {
                                if let Some(native_path) = &native_artifact.path {
                                    let local_path = Path::new(&self.game_files_location)
                                        .join("libraries")
                                        .join(native_path)
                                        .to_str()
                                        .ok_or_else(|| {
                                            UpdateErrors::Install("Path malformed".to_string())
                                        })?
                                        .to_string();

                                    files.push(UpdateFile {
                                        url: native_artifact.url.clone(),
                                        hash: native_artifact.sha1.clone(),
                                        size: native_artifact.size as u64,
                                        local_path,
                                    });
                                }
                            }
                        }
                    }

                    if downloads.artifact.is_none() {
                        for (key, classifier) in classifiers {
                            if key.contains(&os) || key.contains(&arch) {
                                if let Some(classifier_path) = &classifier.path {
                                    let local_path = Path::new(&self.game_files_location)
                                        .join("libraries")
                                        .join(classifier_path)
                                        .to_str()
                                        .ok_or_else(|| {
                                            UpdateErrors::Install("Path malformed".to_string())
                                        })?
                                        .to_string();

                                    files.push(UpdateFile {
                                        url: classifier.url.clone(),
                                        hash: classifier.sha1.clone(),
                                        size: classifier.size as u64,
                                        local_path,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                if let Some((url, path)) = Self::construct_legacy_library(&lib.name) {
                    let local_path = Path::new(&self.game_files_location)
                        .join(&path)
                        .to_str()
                        .ok_or_else(|| UpdateErrors::Install("Path malformed".to_string()))?
                        .to_string();

                    files.push(UpdateFile {
                        url,
                        hash: String::new(),
                        size: 0,
                        local_path,
                    });
                }
            }
        }

        if !files.is_empty() {
            files.sort_by(|a, b| a.local_path.cmp(&b.local_path));
            files.dedup_by(|a, b| a.local_path == b.local_path);

            let mut downloader = Downloader::new(10).with_files(files.clone());
            downloader.download_all_files().await?;

            if !downloader.failed_files().is_empty() {
                self.all_failed_files
                    .extend(downloader.failed_files().clone());
            }

            self.all_downloaded_files.extend(files);
        }

        Ok(())
    }

    fn construct_legacy_library(name: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        let package = parts[0].replace('.', "/");
        let name_part = parts[1];
        let version = parts[2];

        let path = format!(
            "libraries/{}/{}/{}/{}-{}.jar",
            package, name_part, version, name_part, version
        );
        let url = format!("https://libraries.minecraft.net/{}", path);

        Some((url, path))
    }

    fn is_native_library(lib: &Library) -> bool {
        if lib.natives.is_some() {
            return true;
        }

        if lib.name.contains(":natives-") {
            return true;
        }

        if lib.name.contains(":linux-")
            || lib.name.contains(":windows-")
            || lib.name.contains(":osx-")
        {
            return true;
        }

        if let Some(rules) = &lib.rules {
            let has_os_rule = rules.iter().any(|r| r.os.is_some());
            if has_os_rule {
                if let Some(downloads) = &lib.downloads {
                    if downloads.classifiers.is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn get_os() -> String {
        match std::env::consts::OS {
            "windows" => "windows",
            "linux" => "linux",
            "macos" => "osx",
            _ => "unknown",
        }
        .to_string()
    }

    fn get_arch() -> String {
        if std::env::consts::ARCH == "x86_64" || std::env::consts::ARCH == "aarch64" {
            "64".to_string()
        } else {
            "32".to_string()
        }
    }

    fn check_rules(rules: &[Rule], os: &str, arch: &str) -> bool {
        let mut allowed = false;

        for rule in rules {
            let mut matches = true;

            if let Some(os_rule) = &rule.os {
                if let Some(name) = &os_rule.name {
                    if name != os {
                        matches = false;
                    }
                }

                if let Some(arch_rule) = &os_rule.arch {
                    if arch_rule != arch && arch_rule != "any" {
                        matches = false;
                    }
                }
            }

            if matches {
                allowed = rule.action == "allow";
            }
        }

        allowed
    }
    pub(crate) async fn extract_natives(&self) -> Result<(), UpdateErrors> {
        let natives_dir = Path::new(&self.game_files_location).join("natives");
        fs::create_dir_all(&natives_dir)
            .await
            .map_err(|e| Install(e.to_string()))?;

        let libraries_dir = Path::new(&self.game_files_location).join("libraries");
        let native_jars = Self::find_native_jars(&libraries_dir).await?;

        println!("Found {} native JARs to extract", native_jars.len());

        for jar_path in native_jars {
            println!("Extracting: {:?}", jar_path.file_name().unwrap());

            let jar_data = fs::read(&jar_path)
                .await
                .map_err(|e| Install(e.to_string()))?;

            let cursor = std::io::Cursor::new(jar_data);
            let mut archive = ZipArchive::new(cursor).map_err(|e| Install(e.to_string()))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| Install(e.to_string()))?;

                let safe_name = match file.enclosed_name() {
                    Some(name) => name.to_path_buf(),
                    None => {
                        eprintln!("⚠️ Skipping unsafe path: {:?}", file.name());
                        continue;
                    }
                };

                if !file.is_file() {
                    continue;
                }

                let file_name_str = safe_name.to_str().unwrap_or("");
                if !Self::is_native_library_file(file_name_str) {
                    continue;
                }

                let dest_path = natives_dir.join(safe_name.file_name().unwrap());

                let mut dest_file = fs::File::create(&dest_path)
                    .await
                    .map_err(|e| Install(e.to_string()))?;

                let mut buffer = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut buffer)
                    .map_err(|e| Install(e.to_string()))?;

                dest_file
                    .write_all(&buffer)
                    .await
                    .map_err(|e| Install(e.to_string()))?;
                dest_file
                    .flush()
                    .await
                    .map_err(|e| Install(e.to_string()))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = dest_path
                        .metadata()
                        .map_err(|e| Install(e.to_string()))?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest_path, perms)
                        .await
                        .map_err(|e| Install(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    async fn find_native_jars(libraries_dir: &Path) -> Result<Vec<PathBuf>, UpdateErrors> {
        let mut jars = Vec::new();

        let libraries_dir = libraries_dir.to_path_buf();

        let entries = tokio::task::spawn_blocking(move || {
            let mut result = Vec::new();
            for entry in WalkDir::new(libraries_dir) {
                if let Ok(entry) = entry {
                    let path = entry.path().to_path_buf();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.contains("natives-") && name.ends_with(".jar") {
                                result.push(path);
                            }
                        }
                    }
                }
            }
            result
        })
        .await
        .map_err(|e| Install(e.to_string()))?;

        jars.extend(entries);
        Ok(jars)
    }

    fn is_native_library_file(file_name: &str) -> bool {
        file_name.ends_with(".so")
            || file_name.ends_with(".dylib")
            || file_name.ends_with(".jnilib")
            || file_name.ends_with(".dll")
    }

    pub async fn install_version(&mut self) -> Result<(), UpdateErrors> {
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
        self.install_libs(&version).await?;
        println!("Install done, verifying");
        self.extract_natives().await?;

        let mut downloader = Downloader::new(10).with_files(self.all_failed_files.clone());
        downloader.download_all_files().await?;

        let failed = verify_files(self.all_downloaded_files.clone(), 10).await;
        if let Some(files) = failed.err() {
            self.all_failed_files.clear();
            let mut downloader = Downloader::new(10).with_files(files);
            downloader.download_all_files().await?;
        }

        println!(
            "Verifying complete with {} errors",
            if self.all_failed_files.is_empty() {
                "0".to_string()
            } else {
                self.all_failed_files.len().to_string()
            }
        );

        Ok(())
    }

    pub fn game_files_location(&self) -> &str {
        &self.game_files_location
    }

    pub fn version(&self) -> &str {
        &self.version
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
