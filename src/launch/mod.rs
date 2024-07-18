use std::env::consts::OS;
use std::error::Error;
use std::path::MAIN_SEPARATOR_STR;
use std::process::Command;

use crate::launch::utils::LaunchBuilder;
use crate::update::updater::Updater;
use crate::update::utils::{Directory, get_relative_local_dir_path};

mod utils;

pub(crate) struct GameLauncher {
    version: String,
    game_dir: String,
    game_args: Vec<String>,
    jvm_args: Vec<String>,
}

impl GameLauncher {
    pub fn new(
        version: String,
        game_dir: String,
        game_args: Vec<String>,
        jvm_args: Vec<String>,
    ) -> Self {
        Self {
            version,
            game_dir: get_relative_local_dir_path(game_dir.as_str()),
            game_args,
            jvm_args,
        }
    }

    //launch the game using the access_token / pseudo
    pub fn launch(&self, access_token: &str, username: &str) -> Result<(), Box<dyn Error>> {
        let mut updater = Updater::new(&self.version);
        if updater.update_files_list().is_err() {
            return Err("Error getting files list".into());
        }

        let extension = if cfg!(windows) { ".exe" } else { "" };

        let mut builder: LaunchBuilder = LaunchBuilder::new(format!(
            "{}{}bin{}java{}",
            self.game_dir.clone(),
            Directory::Runtime.as_str(),
            MAIN_SEPARATOR_STR,
            extension
        ));
        builder.set_libs_to_launch(
            format!("{}{}", &self.game_dir, &Directory::Libraries.as_str()),
            format!("{}client.jar", &self.game_dir),
        );

        println!("{}", builder.libs());
        // Ok(())

        let mut command = Command::new(
            self.game_dir.clone()
                + &Directory::Runtime.as_str()
                + "bin"
                + &*std::path::MAIN_SEPARATOR.to_string()
                + "java"
                + extension,
        );
        command.args(&self.jvm_args);
        command.arg("-cp");
        command.arg(builder.libs());
        command.arg("net.minecraft.client.main.Main");
        command.args(["--accessToken", access_token]);
        command.args(["--version", &*self.version]);
        command.args(["--username", username]);
        command.args(["--gameDir", &*self.game_dir]);
        command.args([
            "--assetIndex",
            updater
                .libs_manifest()
                .as_ref()
                .unwrap()
                .asset_index
                .id
                .as_str(),
        ]);
        command.args([
            "--assetsDir",
            &(self.game_dir.clone() + &*Directory::Assets.as_str()),
        ]);

        println!("Launching game with command: {:?}", command);

        // stream retour à retourner
        let output = command.output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }
}
