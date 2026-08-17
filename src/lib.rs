pub mod auth;

#[cfg(test)]
mod test {
    use crate::auth::Authenticator;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_authenticator() {
        dotenv::dotenv().ok();

        let client_id = env::var("CLIENT_ID").unwrap();
        let oauth = Authenticator::request_auth_code(&client_id);

        assert!(oauth.is_ok(), "{}", oauth.err().unwrap());
        let oauth = oauth.unwrap();
        println!("[capture_code] : {}", oauth);

        let token = Runtime::new()
            .unwrap()
            .block_on(Authenticator::exchange_code_for_token(&client_id, &oauth));

        assert!(token.is_ok(), "{}", token.err().unwrap());
        let token = token.unwrap();
        println!("[access_token] : {}", token.access_token);

        let live = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth_xbox_live(&token.access_token));

        assert!(live.is_ok(), "{:?}", live.err().unwrap());
        print!("[xbl token] : {}", live.ok().unwrap().token);

        let live = Runtime::new()
            .unwrap()
            .block_on(Authenticator::auth_xsts(&token.access_token));

        assert!(live.is_ok(), "{:?}", live.err().unwrap());
        print!("[xsts token] : {}", live.ok().unwrap().token);
    }
}
