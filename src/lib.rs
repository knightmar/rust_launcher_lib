mod auth;
mod update;

#[cfg(test)]
mod tests {
    use crate::auth::Authenticator;
    use crate::update::Updater;

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
        println!("{}", updater.local_dir_path());

    }
}
