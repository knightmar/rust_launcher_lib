mod mc_version;

pub struct Updater {
    local_dir_path: String,
    version: String,
}


impl Updater {
    pub fn install_files(&self) {}


    fn get_files_list(&self) {}

    pub fn local_dir_path(&self) -> &str {
        &self.local_dir_path
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn set_full_local_dir_path(&mut self, local_dir_path: String) {
        self.local_dir_path = local_dir_path;
    }
    pub fn set_relative_local_dir_path(&mut self, local_dir_path: &str) {
        #[cfg(unix)]{
            let app_data = std::env::var("HOME").expect("No HOME directory");
            self.local_dir_path = app_data + "/" + local_dir_path;
        }

        #[cfg(windows)] {
            let app_data = std::env::var("APPDATA").expect("No APP_DATA directory");
            self.local_dir_path = app_data + "\\" + local_dir_path;
        }
    }
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }
    pub fn new(version: &str) -> Self {
        Self { local_dir_path: String::new(), version: String::from(version) }
    }
}