use std::env::consts::OS;
use std::error::Error;
use std::process::Command;

use crate::update::updater::Updater;
use crate::update::utils::{Directory, get_relative_local_dir_path};


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

    // get the string for the libs to use on the cmdline to launch the game 
    fn get_libs_to_launch(&self) -> String {
        let lib_path = self.game_dir.clone() + &*Directory::Libraries.as_str();
        let client_path = self.game_dir.clone() + "client.jar";
        let separator = if OS == "windows" { ";" } else { ":" };

        let mut lib_str = String::new();
        lib_str.push_str(&(client_path.as_str().to_owned() + separator));

        for entry in std::fs::read_dir(lib_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                lib_str.push_str(path.to_str().unwrap());
                lib_str.push(separator.parse().unwrap());
            }
        }

        lib_str
    }

    //launch the game using the access_token / pseudo
    pub fn launch(&self, access_token: &str, username: &str) -> Result<(), Box<dyn Error>> {
        let mut updater = Updater::new(self.version.as_str());
        if updater.update_files_list().is_err() {
            return Err("Error getting files list".into());
        }

        let extension = if cfg!(windows) {".exe"} else {""};


        let lib_str = self.get_libs_to_launch();
        let mut command = Command::new(self.game_dir.clone() + &*Directory::Runtime.as_str() + "bin" + &*std::path::MAIN_SEPARATOR.to_string() + "java" + extension);
        command.args(&self.jvm_args);
        command.arg("-cp");
        command.arg(lib_str);
        command.arg("net.minecraft.client.main.Main");
        command.args(["--accessToken", access_token]);
        command.args(["--version", &*self.version]);
        command.args(["--username", username]);
        command.args(["--gameDir", &*self.game_dir]);
        command.args(["--assetIndex", updater.libs_manifest().as_ref().unwrap().asset_index.id.as_str()]);
        command.args([
            "--assetsDir",
            &*(self.game_dir.clone() + &*Directory::Assets.as_str()),
        ]);


        println!("Launching game with command: {:?}", command);

        let output = command.output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }
}
