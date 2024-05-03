#![allow(dead_code)]
#![allow(non_snake_case)]
mod auth;
mod launch;
mod update;

#[cfg(test)]
mod tests {
    use crate::auth::Authenticator;
    use crate::launch;
    use crate::update::Updater;
    use crate::update::utils::Directory;

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
        let mut up = Updater::new("1.20.2");
        up.set_relative_local_dir_path(".banane");
        let local_dir_path = up.local_dir_path().to_string();
        let asset_path = "minecraft/sounds/block/powder_snow/break4.ogg";

        let path_builder = local_dir_path.to_string()
            + &*Directory::Assets.as_str()
            + &asset_path.replace('/', "\\");

        println!("{}", path_builder);

        let mut parts: Vec<&str> = path_builder.split('\\').collect();
        println!("{:?}", parts);
        parts.pop();
        let path = parts.join("\\");
        println!("{}", path);
    }

    #[test]
    fn launch_test() {
        let launcher = launch::GameLauncher::new(
            "1.20.2".to_string(),
            ".banane".to_string(),
            "client.jar".to_string(),
            vec![],
            vec![],
            "net.minecraft.client.main.Main".to_string(),
        );
        launcher.launch("").unwrap();
    }
}
