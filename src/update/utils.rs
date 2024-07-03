use std::{fs, io};

use sha2::Digest;

// ----------------------------------------- //
// Utils files that contains utils functions //
// ----------------------------------------- //


pub(crate) fn get_file_name_from_url(url: &str) -> String {
    url.split('/').last().unwrap().to_string()
}

pub(crate) fn get_lib_path_from_url(local_dir_path: String, url: &str) -> String {
    local_dir_path.to_string()
        + &Directory::Libraries.as_str()
        + get_file_name_from_url(url).as_str()
}

pub(crate) fn get_asset_path_from_hash(local_dir_path: String, hash: &str) -> (String, String) {
    // Construct the path where the asset will be stored
    let file_path = local_dir_path.to_string()
        + &*Directory::Assets.as_str()
        + "objects"
        + std::path::MAIN_SEPARATOR_STR
        + hash[0..2].to_string().as_str()
        + std::path::MAIN_SEPARATOR_STR
        + hash;

    let mut parts: Vec<&str> = file_path.split(std::path::MAIN_SEPARATOR_STR).collect();
    parts.pop();
    parts.pop();
    let directory_path = parts.join(std::path::MAIN_SEPARATOR_STR);

    (directory_path, file_path)
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

// creating all the dirs of the file tree
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

// check hash of a file
pub fn check_file_hash(file_path: &str, hash: &str) -> bool {
    let file = fs::File::open(file_path);
    if let Ok(mut file) = file {
        let mut hasher = sha1::Sha1::new();
        io::copy(&mut file, &mut hasher).unwrap();
        let file_hash = hasher.finalize();
        let computed_hash = hex::encode(file_hash);
        hash == computed_hash
    } else if let Err(_e) = file {
        false
    } else {
        false
    }
}