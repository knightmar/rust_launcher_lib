use std::{fs, io};

use sha2::Digest;

use crate::update::structs::mc_assets::AssetsRoot;
use crate::update::structs::mc_libs::LibsRoot;
use crate::update::Updater;

pub(crate) fn get_file_name_from_url(url: &str) -> String {
    url.split('/').last().unwrap().to_string()
}

pub(crate) enum Directory {
    Libraries,
    Assets,
    Indexes,
    Runtime,
}

impl Directory {
    pub fn as_str(&self) -> String {
        match self {
            Directory::Libraries => "libs".to_string() + std::path::MAIN_SEPARATOR_STR,
            Directory::Assets => "assets".to_string() + std::path::MAIN_SEPARATOR_STR,
            Directory::Indexes => {
                Directory::Assets.as_str() + &*"indexes".to_string() + std::path::MAIN_SEPARATOR_STR
            }
            Directory::Runtime => "runtime".to_string() + std::path::MAIN_SEPARATOR_STR,
        }
    }
}

pub fn check_all_directories(base_dir: String) -> bool {
    if base_dir.is_empty() {
        eprintln!("Please set the local directory path before installing files");
        return false;
    }

    //check if the local directory exists
    if !std::path::Path::new(&base_dir).exists() {
        println!("Path not found, creating directory: {}", &base_dir);
        fs::create_dir_all(&base_dir).unwrap();
    }

    //check if lib directory exists
    if !std::path::Path::new(&(base_dir.to_string() + &Directory::Libraries.as_str())).exists() {
        println!(
            "Path not found, creating directory: {}",
            base_dir.to_string() + &Directory::Libraries.as_str()
        );
        fs::create_dir_all(base_dir.to_string() + &Directory::Libraries.as_str()).unwrap();
    };

    if !std::path::Path::new(&(base_dir.to_string() + &Directory::Runtime.as_str())).exists() {
        println!(
            "Path not found, creating directory: {}",
            base_dir.to_string() + &Directory::Runtime.as_str()
        );
        fs::create_dir_all(base_dir.to_string() + &Directory::Runtime.as_str()).unwrap();
    };

    // NOTE: the assets directories are created in the download_assets function (src/update/downloads.rs)

    true
}

pub fn check_file_hash(file_path: &str, hash: &str) -> bool {
    let file = fs::File::open(file_path);
    if let Ok(mut file) = file {
        let mut hasher = sha1::Sha1::new();
        io::copy(&mut file, &mut hasher).unwrap();
        let file_hash = hasher.finalize();
        let computed_hash = hex::encode(file_hash);
        println!("Computed hash: {} for file : {}", computed_hash, file_path);
        hash == computed_hash
    } else if let Err(e) = file {
        eprintln!("Error opening file: {}", e);
        false
    } else {
        false
    }
}

impl Updater {
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
        let app_Root = std::env::var("APPDATA").expect("No APP_Root directory");
        self.local_dir_path = app_Root
            + std::path::MAIN_SEPARATOR_STR
            + local_dir_path
            + std::path::MAIN_SEPARATOR_STR;
    }
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }
    pub fn new(version: &str) -> Self {
        Self {
            local_dir_path: String::new(),
            version: String::from(version),
            libs_manifest: None,
            assets_manifest: None,
        }
    }
    pub fn libs_manifest(&self) -> &Option<LibsRoot> {
        &self.libs_manifest
    }

    pub fn assets_manifest(&self) -> &Option<AssetsRoot> {
        &self.assets_manifest
    }
}
