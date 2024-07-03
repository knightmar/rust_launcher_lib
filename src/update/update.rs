use crate::update::downloads::DownloadManager;
use crate::update::structs::mc_assets::AssetsRoot;
use crate::update::structs::mc_libs::LibsRoot;

pub struct Updater {
    local_dir_path: String,
    version: String,
    libs_manifest: Option<LibsRoot>,
    assets_manifest: Option<AssetsRoot>,
}

impl Updater {
    pub fn install_files(&mut self) {
        println!("---- Installing files -----");

        let mut download_manager = DownloadManager::new(self.local_dir_path.clone());
        // download assets + libs + java + game files
        if let Ok(_result) = &self.update_files_list() {
            let runtime = tokio::runtime::Runtime::new();

            runtime.unwrap().block_on(async {
                download_manager
                    .download_libs(self.libs_manifest.clone().unwrap().libraries)
                    .await;
                download_manager
                    .download_assets(self.assets_manifest.clone().unwrap().objects())
                    .await;
                download_manager
                    .download_java(self.libs_manifest.clone().unwrap().java_version)
                    .await;
                download_manager
                    .download_game_files(self.libs_manifest.clone().unwrap())
                    .await;
            });

            println!("{}", download_manager.fails().len());
        }

        // validate install

        println!("---- End installing files -----");
    }

    pub fn local_dir_path(&self) -> &str {
        &self.local_dir_path
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn libs_manifest(&self) -> &Option<LibsRoot> {
        &self.libs_manifest
    }

    pub fn assets_manifest(&self) -> &Option<AssetsRoot> {
        &self.assets_manifest
    }

    pub fn set_local_dir_path(&mut self, local_dir_path: String) {
        self.local_dir_path = local_dir_path;
    }

    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    pub fn set_libs_manifest(&mut self, libs_manifest: Option<LibsRoot>) {
        self.libs_manifest = libs_manifest;
    }

    pub fn set_assets_manifest(&mut self, assets_manifest: Option<AssetsRoot>) {
        self.assets_manifest = assets_manifest;
    }

    pub fn set_relative_local_dir_path(&mut self, local_dir_path: &str) {
        let app_Root = std::env::var("APPDATA").expect("No APP_Root directory");
        self.set_local_dir_path(
            app_Root
                + std::path::MAIN_SEPARATOR_STR
                + local_dir_path
                + std::path::MAIN_SEPARATOR_STR,
        );
    }

    pub fn new(version: &str) -> Self {
        Self {
            local_dir_path: "".to_string(),
            version: version.to_string(),
            libs_manifest: None,
            assets_manifest: None,
        }
    }
}
