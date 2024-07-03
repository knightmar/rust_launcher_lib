#![allow(dead_code)]
#![allow(non_snake_case)]
mod auth;
mod launch;
pub mod update;


#[cfg(test)]
mod tests {
    use crate::auth::Authenticator;
    use crate::launch;
    use crate::update::update::Updater;

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
        let mut updater = Updater::new("1.19.4");
        updater.set_relative_local_dir_path(".banane");
        
        updater.update_files_list().expect("TODO: panic message");
        updater.install_files();
    }

    #[test]
    fn launch_test() {
        let mut updater = Updater::new("1.19.4");
        updater.set_relative_local_dir_path(".banane");
        updater.install_files();
        
        let launcher = launch::GameLauncher::new(
            "1.19.4".to_string(),
            ".banane".to_string(),
            vec![],
            vec![],
        );
        launcher.launch("", "knightmar67").unwrap();
    }
}
