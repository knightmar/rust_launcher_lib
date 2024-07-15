#![allow(dead_code)]
#![allow(non_snake_case)]
mod auth;
mod launch;
pub mod update;

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use crate::auth::Authenticator;
    use crate::launch;
    use crate::update::java::get_java_zulu_dl_link;
    use crate::update::structs::mc_libs::Library;
    use crate::update::updater::Updater;

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
    fn parsing() {
        let libraries_json = r#"
{
    "libraries": [
        {
            "downloads": {
                "artifact": {
                    "path": "com/mo jang/netty/1.8.8/netty-1.8.8.jar",
                    "sha1": "0a796914d1c8a55b4da9f4a8856dd9623375d8bb",
                    "size": 15966,
                    "url": "https://libraries.minecraft.net/com/mojang/netty/1.8.8/netty-1.8.8.jar"
                }
            },
            "name": "com.mojang:netty:1.8.8"
        }
    ]
}
"#;
        let json_object: Value = serde_json::from_str(libraries_json).unwrap();

        let libraries: Vec<Library> =
            serde_json::from_value(json_object["libraries"].clone()).unwrap();
    }

    #[test]
    fn check_files() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            println!(
                "{}",
                get_java_zulu_dl_link("11.0.11".to_string()).await.unwrap()
            );
        });
    }

    #[test]
    fn launch_test() {
        let mut updater = Updater::new("1.8.9");
        updater.set_relative_local_dir_path(".banane");
        updater.install_files();

        let launcher =
            launch::GameLauncher::new("1.8.9".to_string(), ".banane".to_string(), vec![], vec![]);
        if let Err(error) = launcher.launch("", "knightmar67") {
            println!("{}", error);
        };
    }
}
