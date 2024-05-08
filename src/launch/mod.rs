use std::error::Error;
use std::process::Command;

use crate::launch::utils::get_relative_local_dir_path;
use crate::update::Updater;
use crate::update::utils::Directory;

mod utils;

pub(crate) struct GameLauncher {
    version: String,
    game_dir: String,
    game_jar: String,
    game_args: Vec<String>,
    jvm_args: Vec<String>,
    main_class: String,
}

impl GameLauncher {
    pub fn new(
        version: String,
        game_dir: String,
        game_jar: String,
        game_args: Vec<String>,
        jvm_args: Vec<String>,
        main_class: String,
    ) -> Self {
        Self {
            version,
            game_dir: get_relative_local_dir_path(game_dir.as_str()),
            game_jar,
            game_args,
            jvm_args,
            main_class,
        }
    }

    fn get_libs(&self) -> String {
        let lib_path = self.game_dir.clone() + &*Directory::Libraries.as_str();
        let client_path = self.game_dir.clone() + &self.game_jar;

        let mut lib_str = String::new();
        lib_str.push_str(&(client_path.as_str().to_owned() + ";"));

        for entry in std::fs::read_dir(lib_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                lib_str.push_str(path.to_str().unwrap());
                lib_str.push(';');
            }
        }

        lib_str
    }

    pub fn launch(&self, access_token: &str) -> Result<(), Box<dyn Error>> {
        let mut updater = Updater::new(self.version.as_str());
        if let Err(..) = updater.get_files_list() {
            return Err("Error getting files list".into());
        }
        
        
        let lib_str = self.get_libs();
        let mut command = Command::new(   self.game_dir.clone() + &*Directory::Runtime.as_str() + "bin" + &*std::path::MAIN_SEPARATOR.to_string() + "java.exe");
        command.args(&self.jvm_args);
        command.arg("-cp");
        command.arg(lib_str);
        command.arg(self.main_class.as_str());
        command.args(["--accessToken", access_token]);
        command.args(["--version", &*self.version]);
        command.args(["--username", "Player"]);
        command.args(["--gameDir", &*self.game_dir]);
        command.args([
            "--assetsDir",
            &*(self.game_dir.clone() + &*Directory::Assets.as_str()),
        ]);
        command.args(["--assetIndex", updater.libs_manifest().as_ref().unwrap().asset_index.id.as_str()]);

        println!("Launching game with command: {:?}", command);

        let output = command.output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }
}
