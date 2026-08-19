pub mod auth;
mod update;

#[cfg(test)]
mod test {
    use crate::auth::Authenticator;
    use crate::update::Updater;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_authenticator() {
        dotenv::dotenv().ok();

        let client_id = env::var("CLIENT_ID").unwrap();
        let oauth = Authenticator::auth_oauth2(&client_id);

        assert!(oauth.is_ok(), "{:?}", oauth.err().unwrap());
        let oauth2_code = oauth.unwrap();
        println!("[capture_code] : {}", oauth2_code);

        let oauth2_token_response =
            Runtime::new()
                .unwrap()
                .block_on(Authenticator::exchange_code_for_token(
                    &client_id,
                    &oauth2_code,
                ));

        assert!(
            oauth2_token_response.is_ok(),
            "{}",
            oauth2_token_response.err().unwrap()
        );
        let oauth2_token = oauth2_token_response.unwrap();
        println!("[access_token] : {}", oauth2_token.access_token);

        let xbox_live_response = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth_xbox_live(&oauth2_token.access_token));

        assert!(
            xbox_live_response.is_ok(),
            "{:?}",
            xbox_live_response.err().unwrap()
        );
        let xbox_live_token = xbox_live_response.unwrap().token;
        println!("[xbl token] : {}", xbox_live_token);

        let xsts_response = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth_xsts(&xbox_live_token));

        assert!(xsts_response.is_ok(), "{:?}", xsts_response.err().unwrap());
        let xsts_struct = xsts_response.unwrap();
        println!("[xsts token] : {}", xsts_struct.token);

        let minecraft_response = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth_minecraft(
                &xsts_struct.display_claims.xui[0].uhs,
                &xsts_struct.token,
            ));

        assert!(
            minecraft_response.is_ok(),
            "{:?}",
            minecraft_response.err().unwrap()
        );
        let minecraft_response = minecraft_response.unwrap();
        println!(
            "[minecraft_access_token] : {}",
            minecraft_response.access_token
        );

        let check_ownership_response =
            Runtime::new()
                .unwrap()
                .block_on(Authenticator::check_game_ownership(
                    minecraft_response.access_token.as_str(),
                ));
        assert!(
            check_ownership_response.is_ok(),
            "{:?}",
            check_ownership_response.err().unwrap()
        );
        println!("Game owned : {}", check_ownership_response.ok().unwrap());

        let minecraft_profile =
            Runtime::new()
                .unwrap()
                .block_on(Authenticator::get_minecraft_profile(
                    minecraft_response.access_token.as_str(),
                ));
        assert!(
            minecraft_profile.is_ok(),
            "{:?}",
            minecraft_profile.err().unwrap()
        );

        let minecraft_profile = minecraft_profile.unwrap();
        println!("Welcome {}", minecraft_profile.name);
    }

    #[test]
    fn test_auth_abstract() {
        dotenv::dotenv().ok();

        let client_id = env::var("CLIENT_ID").unwrap();

        let profile = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth(client_id.as_str()));
        assert!(profile.is_ok());
        println!("Welcome {}", profile.unwrap().name);
    }

    #[test]
    fn test_updater() {
        let mut updater = Updater::new(
            "26.2".to_string(),
            "/home/knightmar/.knightlauncher".to_string(),
        );

        println!(
            "{:?}",
            Runtime::new().unwrap().block_on(updater.install_version())
        );
    }
}
