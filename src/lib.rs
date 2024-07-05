#![allow(dead_code)]
#![allow(non_snake_case)]
mod auth;
mod launch;
pub mod update;


#[cfg(test)]
mod tests {
    use crate::auth::Authenticator;
    use crate::launch;
    use crate::update::java::get_java_zulu_dl_link;
    use crate::update::updater::Updater;
    use crate::update::utils::get_relative_local_dir_path;

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
    fn check_files() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            println!("{}", get_java_zulu_dl_link("11.0.11".to_string()).await.unwrap());
        });
    }

    #[test]
    fn launch_test() {
        let mut updater = Updater::new("1.21");
        updater.set_relative_local_dir_path(".banane");
        updater.install_files();

        let launcher = launch::GameLauncher::new(
            "1.21".to_string(),
            ".banane".to_string(),
            vec![],
            vec![],
        );
        if let Err(error) = launcher.launch("", "knightmar67"){
            println!("{}", error);
        };
    }
}
