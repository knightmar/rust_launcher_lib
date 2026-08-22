use reqwest::Version;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitStatus;
use walkdir::WalkDir;

pub struct Launcher {
    game_dir: PathBuf,
    assets_dir: PathBuf,
    asset_index: String,
    uuid: String,
    access_token: String,
    username: String,
    version: String,
}

impl Launcher {
    pub fn get_classpath(&self) -> Result<String, String> {
        let mut classpath_entries = Vec::new();
        let separator = if cfg!(windows) { ";" } else { ":" };

        let client_jar = self.game_dir.join("client.jar");
        if client_jar.exists() {
            classpath_entries.push(
                client_jar
                    .to_str()
                    .ok_or_else(|| "Invalid client.jar path".to_string())?
                    .to_string(),
            );
        } else {
            return Err("client.jar not found".to_string());
        }

        let libraries_dir = self.game_dir.join("libraries");
        if !libraries_dir.exists() {
            return Err("Libraries directory not found".to_string());
        }

        for entry in WalkDir::new(&libraries_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("jar"))
        {
            let path = entry.path();
            let path_str = path
                .to_str()
                .ok_or_else(|| format!("Invalid path: {:?}", path))?;
            classpath_entries.push(path_str.to_string());
        }

        Ok(classpath_entries.join(separator))
    }

    pub fn launch(self) -> Result<i32, String> {
        let classpath = self.get_classpath()?;

        let mut cmd = std::process::Command::new("java");
        cmd.arg("-Djava.library.path=natives")
            .arg("-cp")
            .arg(classpath)
            .arg("net.minecraft.client.main.Main")
            .arg("--username")
            .arg(self.username.clone())
            .arg("--gameDir")
            .arg(self.game_dir.clone())
            .arg("--assetsDir")
            .arg(self.assets_dir.clone())
            .arg("--assetIndex")
            .arg(self.asset_index.clone())
            .arg("--uuid")
            .arg(self.uuid.clone())
            .arg("--accessToken")
            .arg(self.access_token.clone())
            .arg("--version")
            .arg(self.version.clone());

        #[cfg(target_os = "linux")]
        {
            let natives_dir = self.game_dir.join("natives");
            let ld_path = format!(
                "{}:{}",
                natives_dir.to_str().unwrap_or(""),
                std::env::var("LD_LIBRARY_PATH").unwrap_or_default()
            );
            cmd.env("LD_LIBRARY_PATH", ld_path);
        }

        let mut child = cmd.spawn().map_err(|e| e.to_string())?;

        let status = child.wait().map_err(|e| e.to_string())?;

        self.save_logs(&status)?;

        status.code().ok_or("No exit code provided".to_string())
    }

    pub fn new(
        game_dir: PathBuf,
        assets_dir: PathBuf,
        asset_index: String,
        uuid: String,
        access_token: String,
        username: String,
        version: String,
    ) -> Self {
        Self {
            game_dir,
            assets_dir,
            asset_index,
            uuid,
            access_token,
            username,
            version,
        }
    }

    fn save_logs(&self, status: &ExitStatus) -> Result<(), String> {
        let log_path = self.game_dir.join("launcher_log.txt");
        let mut file =
            File::create(log_path).map_err(|e| format!("Failed to create log file: {}", e))?;

        writeln!(file, "Game exited with status: {}", status)
            .map_err(|e| format!("Failed to write to log: {}", e))?;

        Ok(())
    }
}
