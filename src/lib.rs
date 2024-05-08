#![allow(dead_code)]
#![allow(non_snake_case)]
mod auth;
mod launch;
mod update;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::auth::Authenticator;
    use crate::{launch, update};
    use crate::update::downloads::DownloadManager;
    use crate::update::structs::mc_libs::LibsRoot;
    use crate::update::Updater;

    #[test]
    fn test_auth() {
        let auth = Authenticator::new();
        match auth.authenticate_ms() {
            Ok(auth) => {
                if let Err(e) = auth.get_profile() {
                    eprintln!("Error getting profile: {}", e);
                }
            }
            Err(e) => eprintln!("Error during authentication: {}", e),
        }
    }

    #[test]
    fn test_updater() {
        let mut updater = Updater::new("1.20.2");
        updater.set_relative_local_dir_path(".banane");
        updater.install_files().unwrap();
    }

    #[test]
    fn test_downloader() {
        let mut updater = Updater::new("1.20.2");
        updater.set_relative_local_dir_path(".banane");
        updater.get_files_list().unwrap();
        // println!("Before calling download_assets");
        // let asset_root = updater.assets_manifest().as_ref().unwrap();
        // download_assets(asset_root, updater.local_dir_path().to_string());
        // println!("download_assets completed");

        // let runtime = tokio::runtime::Runtime::new().unwrap();
        // runtime.block_on(async {
        //     download_file( reqwest::Client::new() , "https://resources.download.minecraft.net/2e/2e630d9df93d600d90dca070c96e3aa89237afa9", "C:\\Users\\arthu\\AppData\\Roaming\\.banane\\break4.ogg").await.expect("TODO: panic message");
        // });
    }

    #[test]
    fn random_test() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            DownloadManager::download_file(Arc::new( reqwest::Client::new()), "https://resources.download.minecraft.net/2e/2e630d9df93d600d90dca070c96e3aa89237afa9", "C:\\Users\\arthu\\AppData\\Roaming\\.banane\\assets\\objects\\2e\\2e630d9df93d600d90dca070c96e3aa89237afa9", &Some("2e630d9df93d600d90dca070c96e3aa89237afa9".to_string())).await.expect("TODO: panic message");
        });
    }

    #[test]
    fn launch_test() {
        let mut updater = Updater::new("1.20.6");
        updater.set_relative_local_dir_path(".trucmuche");
        updater.install_files().unwrap();
        
        let launcher = launch::GameLauncher::new(
            "1.20.6".to_string(),
            ".trucmuche".to_string(),
            "client.jar".to_string(),
            vec![],
            vec![],
            "net.minecraft.client.main.Main".to_string(),
        );
        launcher.launch("").unwrap();
    }
    
    #[test]
    fn test_get_java_dl_link() {
        let mut manager = DownloadManager::new();
        let mut updater = Updater::new("1.20.6");
        updater.set_relative_local_dir_path(".trucmuche");
        updater.get_files_list().expect("TODO: panic message");
    }
}
